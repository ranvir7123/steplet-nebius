use steplet_core::hosts::{host_of, https_host, imitates, is_exfil_site, is_known_registry, is_punycode};

#[test]
fn the_host_of_an_address() {
    assert_eq!(host_of("https://GitHub.com/octocat/x?y#z").as_deref(), Some("github.com"));
    assert_eq!(host_of("http://example.org").as_deref(), Some("example.org"));
    assert_eq!(host_of("https://user@example.org/"), None);
    assert_eq!(host_of("https://example.org:8443/"), None);
    assert_eq!(host_of("ftp://example.org/"), None);
    assert_eq!(host_of("https://127.0.0.1/"), None);
    assert_eq!(https_host("http://example.org/"), None);
    assert_eq!(https_host("HTTPS://crates.io/api").as_deref(), Some("crates.io"));
}

#[test]
fn lists_and_look_alikes() {
    assert!(is_known_registry("crates.io") && !is_known_registry("example.org"));
    assert!(is_exfil_site("webhook.site") && is_exfil_site("abc.webhook.site"));
    assert!(!is_exfil_site("notwebhook.site") && !is_exfil_site("webhooks.example.org"));
    assert!(is_punycode("xn--80ak6aa92e.invalid") && is_punycode("a.xn--p1ai") && !is_punycode("xnx.example.org"));
    assert_eq!(imitates("githuh.invalid"), Some("github"));
    assert_eq!(imitates("githb.com"), Some("github"));
    assert_eq!(imitates("gihtub.com"), Some("github"));
    assert_eq!(imitates("github.co"), Some("github"));
    for real in ["github.com", "pages.github.io", "objects.githubusercontent.com", "gitlab.com", "example.org", "pypi.org"] {
        assert_eq!(imitates(real), None, "{real}");
    }
}
