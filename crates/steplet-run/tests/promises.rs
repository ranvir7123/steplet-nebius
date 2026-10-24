//! The run chooser's lines (phase 4 Task 5, R6, R20): each line Steplet may promise for a file names the tests that
//! back it, and is worded no wider than those tests show.

use std::path::Path;

use steplet_core::canon::canonicalize;
use steplet_core::header::{Steplet, parse};
use steplet_run::{NetworkMode, Promise, network_mode, promises, read_network_mode};

fn steplet(allow: &str) -> Steplet {
    let f = format!("---\nsteplet: 1\ntitle: t\nsender: s\nexpires: 2026-12-31\nsteps:\n  - a\n{allow}---\n");
    parse(&canonicalize(f.as_bytes()).unwrap()).unwrap()
}

fn texts(ps: &[Promise]) -> Vec<&str> {
    ps.iter().map(|p| p.text).collect()
}

/// Every test a line names is a test function in this crate's tests, marked `#[test]` (a helper or a commented-out
/// function does not count): renaming or deleting one breaks this test, so a line cannot outlive the evidence for it.
/// Most of them need Steplet's Linux and run with `-- --ignored`.
#[test]
fn every_promise_names_tests_that_exist() {
    let folder = concat!(env!("CARGO_MANIFEST_DIR"), "/tests");
    let mut lines: Vec<String> = Vec::new();
    for entry in std::fs::read_dir(folder).unwrap() {
        let path = entry.unwrap().path();
        if path.extension().is_some_and(|e| e == "rs") {
            lines.extend(std::fs::read_to_string(&path).unwrap().lines().map(str::to_owned));
        }
    }
    let is_test = |name: &str| {
        lines.iter().enumerate().any(|(n, line)| {
            line.starts_with(&format!("fn {name}(")) && lines[n.saturating_sub(3)..n].iter().any(|l| l == "#[test]")
        })
    };
    let modes = [NetworkMode::Nat, NetworkMode::Other, NetworkMode::Unknown];
    for (s, mode) in [steplet(""), steplet("allow:\n  domains:\n    - example.org\n")]
        .into_iter()
        .flat_map(|s| modes.map(|m| (s.clone(), m)))
    {
        for p in promises(&s, mode) {
            assert!(!p.tests.is_empty(), "{:?} names no test", p.text);
            for test in p.tests {
                assert!(is_test(test), "{:?} names {test}, which is not a test here", p.text);
            }
        }
    }
    assert!(!is_test("promises"), "the check itself: a function that is not a test does not count");
}

#[test]
fn the_lines_follow_the_files_sites() {
    let none = promises(&steplet(""), NetworkMode::Other);
    assert_eq!(
        texts(&none),
        [
            "Steplet checks the agent's commands that change something, its file writes and its web fetches before \
             they happen. If Steplet's checker does not decide in time, or its gate stops working, the action is \
             refused.",
            "The agent cannot fetch any web address, unless you say yes to that exact address.",
            "The agent and the programs this task runs can change files only inside the working copy (the programs \
             also in the run's own temporary folder, which Steplet deletes after the run).",
            "Your own files are not inside Steplet's Linux, where this task runs.",
            "Programs this task runs cannot connect to the internet, or to other programs in Steplet's Linux.",
            "Programs this task runs cannot start Windows programs.",
        ]
    );
    assert!(none.iter().all(|p| p.items.is_empty()));

    let one = promises(&steplet("allow:\n  domains:\n    - example.org\n"), NetworkMode::Other);
    assert_eq!(
        (one[1].text, one[1].items.clone()),
        (
            "The agent can fetch only these web addresses, and another one only if you say yes to that exact address:",
            vec!["example.org".to_owned()]
        )
    );
    assert_eq!(
        (one[4].text, one[4].items.clone()),
        (
            "Programs this task runs can connect only to these sites: not to the rest of the internet, or to other \
             programs in Steplet's Linux.",
            vec!["example.org".to_owned()]
        )
    );
    assert!(one[4].tests.contains(&"a_declared_site_is_the_only_one_a_program_can_reach"));
    assert!(none[0].tests.contains(&"a_write_into_steplets_own_folder_is_refused_before_the_gate"), "file writes");
    assert!(none[0].tests.contains(&"a_web_fetch_is_allowed_only_to_a_site_the_file_declares"), "web fetches");
    assert!(none[2].tests.contains(&"the_run_s_temporary_folder_holds_no_key_or_token_and_is_gone_after"));
    assert!(!none[4].tests.contains(&"a_declared_site_is_the_only_one_a_program_can_reach"));
}

/// K4 A (22 Oct 2026): the network line says "on this computer" only while WSL runs in its default network mode
/// (`nat`), the one mode in which a program listening on Windows was shown unreachable from inside the fence; in any
/// other mode, or when Steplet cannot tell, the line stays as narrow as phase 4's. Every other line is the same in
/// every mode.
#[test]
fn on_this_computer_only_in_wsl_s_default_network_mode() {
    const WINDOWS: &str = "a_program_listening_on_windows_cannot_be_reached";
    let site = "allow:\n  domains:\n    - example.org\n";
    let network = |allow: &str, mode| promises(&steplet(allow), mode)[4].clone();

    let nat = network("", NetworkMode::Nat);
    assert_eq!(
        nat.text,
        "Programs this task runs cannot connect to the internet, or to other programs on this computer."
    );
    let nat_site = network(site, NetworkMode::Nat);
    assert_eq!(
        nat_site.text,
        "Programs this task runs can connect only to these sites: not to the rest of the internet, or to other \
         programs on this computer."
    );
    assert_eq!(nat_site.items, ["example.org"]);
    assert!(nat.tests.contains(&WINDOWS) && nat_site.tests.contains(&WINDOWS), "the wider line rests on that test");
    const REFUSED: &str = "a_run_that_lists_a_site_leading_to_this_computer_is_refused";
    assert!(
        nat.tests.contains(&REFUSED) && nat_site.tests.contains(&REFUSED),
        "and on the refuse of a bad declared name"
    );

    for mode in [NetworkMode::Other, NetworkMode::Unknown] {
        assert!(network("", mode).tests.contains(&REFUSED), "even the narrower line rests on the refuse: {mode:?}");
        assert!(!network("", mode).tests.contains(&WINDOWS), "but not on the Windows-listener test: {mode:?}");
        assert_eq!(
            network("", mode).text,
            "Programs this task runs cannot connect to the internet, or to other programs in Steplet's Linux.",
            "{mode:?}"
        );
        assert_eq!(
            network(site, mode).text,
            "Programs this task runs can connect only to these sites: not to the rest of the internet, or to other \
             programs in Steplet's Linux.",
            "{mode:?}"
        );
        for allow in ["", site] {
            for p in promises(&steplet(allow), mode) {
                assert!(!p.text.to_lowercase().contains("on this computer"), "{mode:?}: {:?}", p.text);
            }
            let (here, there) = (promises(&steplet(allow), NetworkMode::Nat), promises(&steplet(allow), mode));
            assert_eq!(here.len(), there.len());
            for n in [0, 1, 2, 3, 5] {
                assert_eq!(here[n], there[n], "{mode:?}: only the network line changes");
            }
        }
    }
}

/// WSL's network mode, from the text of `%USERPROFILE%\.wslconfig` (an INI file: `[section]`, `key = value`, and
/// comments). WSL's default is `nat`, so no file, or a file that names no mode, is `Nat`. Any other mode named, wherever
/// it stands, is `Other`; a file Steplet cannot read line by line is `Unknown`. Both keep the narrower line.
#[test]
fn network_mode_reads_the_wslconfig_file() {
    let mode = |text: &str| network_mode(Some(text));
    assert_eq!(network_mode(None), NetworkMode::Nat, "no file: WSL's default");
    assert_eq!(mode("[wsl2]\nmemory=8GB\n"), NetworkMode::Nat, "a file that names no mode");
    assert_eq!(mode("[wsl2]\nnetworkingMode=nat\n"), NetworkMode::Nat);
    assert_eq!(mode("[wsl2]\nnetworkingMode=mirrored\n"), NetworkMode::Other);
    assert_eq!(mode("[wsl2\nnetworkingMode=nat\n"), NetworkMode::Unknown, "a malformed file");

    assert_eq!(mode(""), NetworkMode::Nat, "an empty file");
    assert_eq!(mode("# mine\r\n; also mine\r\n\r\n[wsl2]\r\n  NetworkingMode = NAT  \r\n"), NetworkMode::Nat);
    for other in ["virtioproxy", "bridged", "none", "", "\"nat\"", "nat # the default"] {
        assert_eq!(mode(&format!("[wsl2]\nnetworkingMode={other}\n")), NetworkMode::Other, "{other:?}");
    }
    assert_eq!(mode("[experimental]\nnetworkingMode=mirrored\n"), NetworkMode::Other, "in another section too");
    assert_eq!(mode("[wsl2]\nnetworkingMode=nat\nnetworkingMode=mirrored\n"), NetworkMode::Other, "named twice");
    assert_eq!(mode("[wsl2]\nnetworkingMode nat\n"), NetworkMode::Unknown, "a line with no '='");
    assert_eq!(mode("[wsl2]\n= nat\n"), NetworkMode::Unknown, "a value with no key");
}

/// Steplet reads `.wslconfig` from the Windows user's folder and never changes it. A file it cannot read, or whose
/// text is not UTF-8 (a UTF-8 byte order mark is allowed), is `Unknown`; so is a missing user folder.
#[test]
fn the_mode_is_read_from_the_user_s_folder_and_unreadable_is_unknown() {
    let base = Path::new(env!("CARGO_TARGET_TMPDIR")).join("p5t4-network-mode");
    let _ = std::fs::remove_dir_all(&base);
    let folder = |name: &str, wslconfig: Option<&[u8]>| {
        let f = base.join(name);
        std::fs::create_dir_all(&f).unwrap();
        if let Some(bytes) = wslconfig {
            std::fs::write(f.join(".wslconfig"), bytes).unwrap();
        }
        f
    };
    let utf16: Vec<u8> = [0xFEFF]
        .into_iter()
        .chain("[wsl2]\r\nnetworkingMode=nat\r\n".encode_utf16())
        .flat_map(u16::to_le_bytes)
        .collect();
    let cases: [(&str, Option<&[u8]>, NetworkMode); 5] = [
        ("none", None, NetworkMode::Nat),
        ("nat-with-mark", Some(b"\xEF\xBB\xBF[wsl2]\r\nnetworkingMode=nat\r\n"), NetworkMode::Nat),
        ("mirrored", Some(b"[wsl2]\nnetworkingMode=mirrored\n"), NetworkMode::Other),
        ("utf16", Some(&utf16), NetworkMode::Unknown),
        ("not-text", Some(b"[wsl2]\nnetworkingMode=n\xFFt\n"), NetworkMode::Unknown),
    ];
    for (name, wslconfig, want) in cases {
        let before = wslconfig.map(<[u8]>::to_vec);
        let f = folder(name, wslconfig);
        assert_eq!(read_network_mode(Some(&f)), want, "{name}");
        assert_eq!(std::fs::read(f.join(".wslconfig")).ok(), before, "{name}: the file is unchanged");
    }
    let a_folder = folder("wslconfig-is-a-folder", None);
    std::fs::create_dir(a_folder.join(".wslconfig")).unwrap();
    assert_eq!(read_network_mode(Some(&a_folder)), NetworkMode::Unknown, "a .wslconfig Steplet cannot read");
    assert_eq!(read_network_mode(None), NetworkMode::Unknown, "no user folder");
    std::fs::remove_dir_all(&base).unwrap();
}
