//! The window and the page rules are fixed: file text can never name the window, and the page can load nothing
//! from outside the app (rows 28 and 30, the milestone plan's fixed copy).

use serde_json::Value;

fn read(name: &str) -> Value {
    let path = format!("{}/{name}", env!("CARGO_MANIFEST_DIR"));
    let text = std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{path}: {e}"));
    serde_json::from_str(&text).unwrap_or_else(|e| panic!("{path} is not JSON: {e}"))
}

/// The page rules (content security policy) of the app's window, exactly.
const CSP: &str = "default-src 'self'; script-src 'self'; style-src 'self'; img-src 'self' data:; font-src 'self'; \
                   connect-src ipc: http://ipc.localhost; object-src 'none'; base-uri 'none'; form-action 'none'; \
                   frame-ancestors 'none'";

#[test]
fn the_window_and_security_settings_are_fixed() {
    let conf = read("tauri.conf.json");
    assert_eq!(conf["productName"], "Steplet");

    let windows = conf["app"]["windows"].as_array().expect("app.windows is a list");
    assert_eq!(windows.len(), 1, "one window entry");
    let w = &windows[0];
    assert_eq!(w["label"], "main");
    assert_eq!(w["title"], "Steplet");
    assert_eq!(w["width"], 1100);
    assert_eq!(w["height"], 800);
    assert_eq!(w["minWidth"], 560);
    assert_eq!(w["minHeight"], 720);
    assert_eq!(w["dragDropEnabled"], false, "so a dropped file reaches the page as a file");
    assert_eq!(w["create"], false, "the app makes the window itself, with its navigation rules");
    assert!(w.get("additionalBrowserArgs").is_none(), "browser arguments come only from steplet_app::browser_args");
    assert!(w.get("url").is_none(), "the window shows only the app's own page");

    assert_eq!(conf["app"]["withGlobalTauri"], false);
    assert_eq!(conf["app"]["security"]["csp"], CSP);
    assert!(conf["app"]["security"].get("devCsp").is_none(), "no looser rules for development builds");
    assert!(
        conf["app"]["security"].get("dangerousDisableAssetCspModification").is_none(),
        "Tauri keeps adding its own protections to the rules"
    );
    assert_eq!(conf["app"]["security"]["capabilities"], serde_json::json!(["main"]));
    assert!(conf["build"].get("devUrl").is_none(), "the window never loads a page from a server");

    let plugins = conf.get("plugins").and_then(Value::as_object);
    assert!(plugins.is_none_or(|p| p.is_empty()), "no Tauri plugins: {plugins:?}");

    let cap = read("capabilities/main.json");
    assert_eq!(cap["identifier"], "main");
    assert_eq!(cap["windows"], serde_json::json!(["main"]));
    assert!(cap.get("remote").is_none(), "no web page outside the app gets any permission");
    let granted: Vec<&str> = cap["permissions"]
        .as_array()
        .expect("permissions is a list")
        .iter()
        .map(|p| p.as_str().expect("each permission is a plain name, without extra scope"))
        .collect();
    for p in &granted {
        let own_command = p.strip_prefix("allow-").map(|c| c.replace('-', "_"));
        assert!(
            ["core:event:allow-listen", "core:event:allow-unlisten"].contains(p)
                || own_command.is_some_and(|c| steplet_app::COMMANDS.contains(&c.as_str())),
            "{p} is neither listening for events nor one of the app's own commands"
        );
    }
    for c in steplet_app::COMMANDS {
        assert!(granted.contains(&format!("allow-{}", c.replace('_', "-")).as_str()), "{c} is not granted");
    }
}

#[test]
#[cfg(not(feature = "e2e"))]
fn no_debug_port_without_the_e2e_feature() {
    let args = steplet_app::browser_args();
    assert!(!args.contains("remote-debugging"), "{args}");
    assert!(!args.contains("remote-allow-origins"), "{args}");
}

#[test]
#[cfg(feature = "e2e")]
fn the_e2e_build_opens_its_debug_port_on_this_computer_only() {
    let args = steplet_app::browser_args();
    assert!(args.contains("--remote-debugging-port=9333"), "{args}");
    assert!(!args.contains("remote-debugging-address"), "Chromium's default address is 127.0.0.1: {args}");
}

#[test]
fn the_browser_keeps_wrys_default_arguments() {
    // Setting browser arguments replaces wry's defaults, so they must be kept by hand (wry's own warning).
    assert!(
        steplet_app::browser_args().starts_with("--disable-features=msWebOOUI,msPdfOOUI,msSmartScreenProtection"),
        "{}",
        steplet_app::browser_args()
    );
}

/// The names in the app's one `generate_handler![…]` list, read from its source. Tauri checks a call of an app
/// command against capabilities/main.json only when `COMMANDS` lists that command (tauri-build writes the app's
/// permission list from it, and only when it is not empty), so a command registered but not listed would be open to
/// the page with no check at all.
fn registered_commands() -> Vec<String> {
    let src = format!("{}/src", env!("CARGO_MANIFEST_DIR"));
    let mut lists = Vec::new();
    for entry in std::fs::read_dir(&src).unwrap() {
        let path = entry.unwrap().path();
        let text = std::fs::read_to_string(&path).unwrap();
        for (at, _) in text.match_indices("generate_handler!") {
            let rest = &text[at..];
            let open = rest.find('[').expect("generate_handler! takes a [list]");
            let close = rest.find(']').expect("the list ends");
            lists.push((path.clone(), rest[open + 1..close].to_owned()));
        }
    }
    assert!(lists.len() <= 1, "the app registers its commands in one place: {lists:?}");
    let mut names: Vec<String> = lists
        .iter()
        .flat_map(|(_, list)| list.split(',').map(|n| n.trim().rsplit("::").next().unwrap().to_owned()))
        .filter(|n| !n.is_empty())
        .collect();
    names.sort();
    names
}

#[test]
fn every_command_the_app_registers_is_listed_for_tauri_to_check() {
    let mut listed: Vec<String> = steplet_app::COMMANDS.iter().map(|c| c.to_string()).collect();
    listed.sort();
    assert_eq!(registered_commands(), listed, "src/commands.rs must list exactly the commands the app registers");
}

#[test]
fn the_web_view_sends_nothing_to_the_network() {
    // Every request the web view would send goes to a closed port on this computer instead, so nothing can leave
    // even where the page rules and the navigation check come too late (measured in Task 8: WebView2 sends a
    // navigation's request before the app can cancel it).
    let args = steplet_app::browser_args();
    assert!(args.contains(" --proxy-server=http://127.0.0.1:47 "), "{args}");
    assert!(
        args.ends_with(" --proxy-bypass-list=<-loopback>") || args.contains(" --proxy-bypass-list=<-loopback> "),
        "{args}"
    );
}

#[test]
fn the_window_goes_nowhere_but_the_apps_own_page() {
    let own = |u: &str| steplet_app::is_own_page(&tauri::Url::parse(u).unwrap());
    assert!(own("http://tauri.localhost/"));
    assert!(own("http://tauri.localhost/index.html?scenario=x#top"));
    for elsewhere in [
        "https://tauri.localhost/",
        "http://tauri.localhost:8080/",
        "http://tauri.localhost.example.com/",
        "http://user@example.com/",
        "http://user@tauri.localhost/",
        "http://user:pw@tauri.localhost/",
        "http://:pw@tauri.localhost/",
        "https://github.com/",
        "file:///C:/Users/",
        "data:text/html,hi",
        "javascript:alert(1)",
        "ipc://localhost/",
        "steplet://open",
    ] {
        assert!(!own(elsewhere), "{elsewhere} is not the app's own page");
    }
}

#[test]
fn the_app_build_holds_no_stand_in_backend() {
    // The stand-in backend and its saved scenarios are for screen tests only (Task 9); the app embeds web/dist.
    let dist = std::path::PathBuf::from(concat!(env!("CARGO_MANIFEST_DIR"), "/../../web/dist"));
    let mut scripts = 0;
    let mut stack = vec![dist.clone()];
    while let Some(dir) = stack.pop() {
        for e in std::fs::read_dir(&dir).unwrap_or_else(|e| panic!("{}: {e} (build the page first)", dir.display())) {
            let p = e.unwrap().path();
            if p.is_dir() {
                stack.push(p);
                continue;
            }
            assert!(!p.to_string_lossy().contains("fixtures"), "{} is a saved scenario", p.display());
            let text = String::from_utf8_lossy(&std::fs::read(&p).unwrap()).into_owned();
            scripts += p.extension().is_some_and(|x| x == "js") as u32;
            assert!(!text.contains("steplet-stand-in-backend"), "{} holds the stand-in backend", p.display());
            assert!(!text.contains("open-crash-cleared"), "{} holds the saved scenarios", p.display());
        }
    }
    assert!(scripts > 0, "web/dist holds the app's script");
}

#[test]
fn the_page_is_titled_steplet() {
    let path = format!("{}/../../web/index.html", env!("CARGO_MANIFEST_DIR"));
    let page = std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{path}: {e}"));
    assert!(page.contains("<title>Steplet</title>"), "web/index.html's title is Steplet");
}
