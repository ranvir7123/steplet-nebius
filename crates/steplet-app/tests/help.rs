//! Steplet's own help pages (Task 11, smaller choice S11): a fixed list of addresses, opened in the receiver's
//! browser. Nothing else, and never an address from a file, can be opened this way.

use steplet_app::help;

#[test]
fn only_steplets_own_help_pages_open() {
    assert_eq!(help::address("wsl"), Some("https://learn.microsoft.com/windows/wsl/install"));
    assert_eq!(help::address("anthropic-key"), Some("https://console.anthropic.com/settings/keys"));
    for not_a_topic in ["", "WSL", "https://example.org", "wsl ", "../wsl"] {
        assert_eq!(help::address(not_a_topic), None, "{not_a_topic:?}");
        assert!(help::open(not_a_topic).is_err(), "{not_a_topic:?} opens nothing");
    }
}
