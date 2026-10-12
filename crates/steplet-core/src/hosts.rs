//! Web addresses and host names (G9): the host an address really goes to, and the rating's lists of hosts.

use alloc::string::String;

use crate::header::is_domain;

fn split_scheme(url: &str) -> Option<(bool, &str)> {
    let b = url.as_bytes();
    if b.len() >= 8 && b[..8].eq_ignore_ascii_case(b"https://") {
        Some((true, &url[8..]))
    } else if b.len() >= 7 && b[..7].eq_ignore_ascii_case(b"http://") {
        Some((false, &url[7..]))
    } else {
        None
    }
}

fn host(rest: &str) -> Option<String> {
    let host = rest.split(['/', '?', '#']).next().unwrap_or("").to_ascii_lowercase();
    is_domain(&host).then_some(host)
}

/// The host of an `http://` or `https://` address, in lower case, if it is a host name `allow: domains` could
/// hold: no user name, no port, no IP address.
pub fn host_of(url: &str) -> Option<String> {
    host(split_scheme(url)?.1)
}

/// As `host_of`, for `https://` addresses only.
pub fn https_host(url: &str) -> Option<String> {
    match split_scheme(url)? {
        (true, rest) => host(rest),
        (false, _) => None,
    }
}

/// G9: package registries Steplet knows. In `allow: domains` they give no finding.
pub const KNOWN_REGISTRIES: &[&str] = &[
    "crates.io", "index.crates.io", "static.crates.io", "registry.npmjs.org", "registry.yarnpkg.com", "pypi.org",
    "files.pythonhosted.org", "proxy.golang.org", "sum.golang.org", "repo.maven.apache.org", "repo1.maven.org",
    "plugins.gradle.org", "services.gradle.org", "rubygems.org", "api.nuget.org", "repo.packagist.org", "github.com",
    "codeload.github.com", "objects.githubusercontent.com",
];

/// G9: sites often used to collect stolen data: paste sites, file drops, request catchers and tunnels. Any name
/// below one of these counts too.
pub const EXFIL_SITES: &[&str] = &[
    "pastebin.com", "paste.ee", "hastebin.com", "transfer.sh", "file.io", "0x0.st", "webhook.site", "requestbin.net",
    "pipedream.net", "beeceptor.com", "ngrok.io", "ngrok.app", "ngrok-free.app", "trycloudflare.com", "serveo.net",
    "localhost.run", "discord.com", "discordapp.com", "telegram.org", "interact.sh", "oast.fun", "oast.live",
    "oast.me", "oast.online", "oast.pro", "oast.site", "burpcollaborator.net",
];

pub fn is_known_registry(host: &str) -> bool {
    KNOWN_REGISTRIES.contains(&host)
}

pub fn is_exfil_site(host: &str) -> bool {
    EXFIL_SITES.iter().any(|s| host == *s || host.strip_suffix(s).is_some_and(|rest| rest.ends_with('.')))
}

/// True when a part of `host` is written in punycode (`xn--…`): letters of another alphabet.
pub fn is_punycode(host: &str) -> bool {
    host.split('.').any(|label| label.starts_with("xn--"))
}

/// Well-known sites a look-alike may imitate: the name before the ending, and the endings that are really theirs.
const FAMOUS: &[(&str, &[&str])] = &[
    ("github", &["com", "io", "dev"]),
    ("gitlab", &["com", "io"]),
    ("bitbucket", &["org"]),
    ("codeberg", &["org"]),
    ("crates", &["io"]),
    ("npmjs", &["com", "org"]),
    ("python", &["org"]),
    ("anthropic", &["com"]),
    ("openai", &["com"]),
    ("golang", &["org"]),
    ("nodejs", &["org"]),
    ("docker", &["com", "io"]),
];

/// The famous name `host` imitates: the name before its ending is one change away from a famous one (`githb.com`,
/// `gihtub.com`), or is the famous name with another ending (`github.co`). The ending is the last part only, so
/// endings like `co.uk` are not understood.
pub fn imitates(host: &str) -> Option<&'static str> {
    let mut labels = host.rsplit('.');
    let ending = labels.next()?;
    let name = labels.next()?;
    FAMOUS
        .iter()
        .find(|(famous, endings)| (name == *famous && !endings.contains(&ending)) || one_change_apart(name, famous))
        .map(|(famous, _)| *famous)
}

/// True when `a` and `b` differ by exactly one change: one letter replaced, added or removed, or two neighbouring
/// letters swapped. Only for famous names of five letters or more.
fn one_change_apart(a: &str, b: &str) -> bool {
    let (a, b) = (a.as_bytes(), b.as_bytes());
    if a == b || b.len() < 5 {
        return false;
    }
    let same = a.iter().zip(b).take_while(|(x, y)| x == y).count();
    let (ra, rb) = (&a[same..], &b[same..]);
    match ra.len() as isize - rb.len() as isize {
        0 => ra[1..] == rb[1..] || (ra.len() >= 2 && ra[0] == rb[1] && ra[1] == rb[0] && ra[2..] == rb[2..]),
        1 => ra[1..] == *rb,
        -1 => *ra == rb[1..],
        _ => false,
    }
}
