//! The listed-sites guard (phase 5 Task 4, choice K4 option B): before a run, Steplet resolves each web site the file
//! declares where it would lead for the run, refuses the run if one leads to this computer or a private network, and
//! keeps those answers for the run. Claude Code 2.1.220's own fence resolves a declared name and dials it with no
//! check on the address it resolves to, so without this a file that declares a name pointing at this computer reaches
//! programs on it (found in Step 4). The classifier and the check need no Steplet's Linux; the resolver does.

mod support;

use std::net::IpAddr;

use steplet_run::{
    check_sites, leads_to_this_computer_or_private, pin_sites, resolve_all_in_linux, resolve_in_linux, unpin_sites,
};

fn ip(s: &str) -> IpAddr {
    s.parse().unwrap()
}

/// What counts as "this computer or a private network": loopback, the addresses WSL and Windows use for each other
/// (in a private range), every private range (v4 and v6), link-local, unique-local, site-local, carrier-grade NAT,
/// the "this network" range, benchmarking and protocol-assignment ranges, multicast and reserved ranges, the
/// unspecified address, and any v6 form that carries a non-public v4 inside it (v4-mapped, v4-compatible, 6to4,
/// NAT64). A plain public address — including a public v4 carried inside a v6 — is not.
#[test]
fn an_address_on_this_computer_or_a_private_network_is_known_from_a_public_one() {
    for on_it in [
        "127.0.0.1",
        "127.9.9.9",
        "::1",
        "0.0.0.0",
        "::",
        "0.1.2.3", // 0.0.0.0/8, "this network"
        "10.0.0.5",
        "172.16.0.1",
        "172.27.208.1", // the address WSL uses for Windows on this laptop (172.16/12)
        "172.31.255.255",
        "192.168.1.1",
        "192.0.0.1",  // 192.0.0.0/24, IETF protocol assignments
        "198.18.0.1", // 198.18.0.0/15, benchmarking (abused by local proxies to route to this computer)
        "198.19.255.255",
        "224.0.0.1",       // multicast 224.0.0.0/4
        "239.255.255.255", // multicast
        "240.0.0.1",       // reserved 240.0.0.0/4
        "255.255.255.255", // broadcast
        "169.254.1.1",     // link-local v4
        "fe80::1",         // link-local v6
        "fc00::1",         // unique-local v6
        "fd12:3456::1",
        "fec0::1",          // site-local v6 (deprecated)
        "100.64.0.1",       // carrier-grade NAT
        "::ffff:127.0.0.1", // loopback written as v4-mapped-v6
        "::ffff:10.0.0.1",
        "::0a00:0001",      // v4-compatible ::10.0.0.1
        "2002:a00:1::1",    // 6to4 carrying 10.0.0.1
        "64:ff9b::a00:1",   // NAT64 (well-known) carrying 10.0.0.1
        "64:ff9b:1::a00:1", // NAT64 (local-use) prefix
    ] {
        assert!(leads_to_this_computer_or_private(ip(on_it)), "{on_it} is on this computer or a private network");
    }
    for public in [
        "1.1.1.1",
        "8.8.8.8",
        "93.184.216.34",
        "2606:2800:220:1::1",
        "::ffff:93.184.216.34", // public v4 as v4-mapped-v6
        "100.128.0.1",
        "2002:5db8:d822::1",  // 6to4 carrying the public 93.184.216.34
        "64:ff9b::5db8:d822", // NAT64 carrying the public 93.184.216.34
    ] {
        assert!(!leads_to_this_computer_or_private(ip(public)), "{public} is a public address");
    }
}

/// A fake resolver: each site leads where the map says, or cannot be found when it is not in the map.
fn leads<'a>(map: &'a [(&'a str, &'a [&'a str])]) -> impl Fn(&str) -> Result<Vec<IpAddr>, String> + 'a {
    move |name: &str| {
        map.iter()
            .find(|(n, _)| *n == name)
            .map(|(_, ips)| ips.iter().map(|s| ip(s)).collect())
            .ok_or_else(|| "no such name".to_owned())
    }
}

/// The check returns the resolved addresses to pin when every listed site leads to a public address; it refuses,
/// naming the one site, when a site leads to this computer or a private network, or cannot be found at all (fail
/// closed). A file that lists no site passes with nothing to pin.
#[test]
fn the_check_refuses_a_listed_site_that_leads_to_this_computer() {
    let map: &[(&str, &[&str])] =
        &[("good.example", &["93.184.216.34", "1.1.1.1"]), ("bad.example", &["10.0.0.1"]), ("empty.example", &[])];

    let pins = check_sites(&["good.example".to_owned()], leads(map)).unwrap();
    assert_eq!(pins, [("good.example".to_owned(), ip("93.184.216.34")), ("good.example".to_owned(), ip("1.1.1.1"))]);

    assert_eq!(check_sites(&[], leads(map)).unwrap(), Vec::new(), "a file that lists no site passes");

    let refused = check_sites(&["good.example".to_owned(), "bad.example".to_owned()], leads(map)).unwrap_err();
    assert_eq!(refused.site, "bad.example");
    assert!(refused.why.contains("this computer or a private network"), "{}", refused.why);

    // A name that resolves to no address at all, and a name that cannot be found, are both refused (fail closed).
    let nowhere = check_sites(&["empty.example".to_owned()], leads(map)).unwrap_err();
    assert_eq!(nowhere.site, "empty.example");
    assert!(nowhere.why.to_lowercase().contains("led nowhere"), "{}", nowhere.why);

    let missing = check_sites(&["nowhere.example".to_owned()], leads(map)).unwrap_err();
    assert_eq!(missing.site, "nowhere.example");
    assert!(missing.why.to_lowercase().contains("could not find"), "{}", missing.why);
}

/// Steplet's Linux resolves a name where it really leads, as the steplet account: a name in `/etc/hosts` leads to the
/// address written there (that is also how a run's pins hold for the run), a public name leads to public addresses
/// only, and a name that does not exist cannot be found (so the check refuses, fail closed). The control: the made-up
/// name is not a real address until it is in `/etc/hosts`.
#[test]
#[ignore = "needs Steplet's Linux and the network"]
fn steplet_s_linux_resolves_a_name_where_it_leads() {
    let _linux = support::hold_linux();
    let distro = support::steplet_linux();
    let names = support::add_to_hosts(&distro, "203.0.113.7 pinned.steplet-test.example\n");

    let pinned = resolve_in_linux(&distro, "pinned.steplet-test.example").unwrap();
    println!("--- pinned.steplet-test.example leads to {pinned:?}");
    assert_eq!(pinned, [ip("203.0.113.7")], "the pinned name leads to the address in /etc/hosts");

    let public = resolve_in_linux(&distro, "example.org").unwrap();
    println!("--- example.org leads to {public:?}");
    assert!(!public.is_empty() && public.iter().all(|a| !leads_to_this_computer_or_private(*a)), "{public:?}");

    let missing = resolve_in_linux(&distro, "no-such-name.steplet-test.example");
    println!("--- a name that does not exist: {missing:?}");
    assert!(missing.is_err(), "a name that does not exist cannot be found");

    // The product path resolves every declared site in one call (not one per name): the map holds each name that
    // resolved and leaves out one that cannot be found, so the check refuses it.
    let names_at_once = [
        "pinned.steplet-test.example".to_owned(),
        "example.org".to_owned(),
        "no-such-name.steplet-test.example".to_owned(),
    ];
    let all = resolve_all_in_linux(&distro, &names_at_once).unwrap();
    println!("--- one call for all the names: {all:?}");
    assert_eq!(all.get("pinned.steplet-test.example"), Some(&vec![ip("203.0.113.7")]), "{all:?}");
    assert!(
        all.get("example.org")
            .is_some_and(|a| !a.is_empty() && a.iter().all(|x| !leads_to_this_computer_or_private(*x))),
        "{all:?}"
    );
    assert!(!all.contains_key("no-such-name.steplet-test.example"), "a name that cannot be found is left out: {all:?}");

    drop(names);
    let gone = resolve_in_linux(&distro, "pinned.steplet-test.example");
    assert!(gone.is_err(), "the made-up name is not a real address once it is out of /etc/hosts");
}

/// The run's pins are written into Steplet's Linux `/etc/hosts` and taken out byte for byte afterwards (I7): the
/// mechanism behind "keeps those answers fixed for the run". While the block is in place a pinned name resolves to the
/// pinned address; a name pinned with one address family only gets a sink line for the other, so a lookup of that
/// family cannot fall through to the public name service (I6); and a run with no sites leaves the file exactly as it
/// was. Made-up names under `.example` only; the file is read and put back as root.
#[test]
#[ignore = "needs Steplet's Linux"]
fn the_runs_pins_are_written_to_etc_hosts_and_taken_out_byte_for_byte() {
    let _linux = support::hold_linux();
    let distro = support::steplet_linux();
    // Clear any block a crashed run left, then snapshot the file.
    unpin_sites(&distro).unwrap();
    let before = support::etc_hosts(&distro);

    let pins = vec![
        ("pinned4.steplet-test.example".to_owned(), ip("203.0.113.7")), // v4 only
        ("pinned6.steplet-test.example".to_owned(), ip("2606:2800:220:1::1")), // v6 only
    ];
    pin_sites(&distro, &pins).unwrap();

    // While the block is in place the pinned name resolves to the pinned address (the fence resolves through
    // /etc/hosts, so this is what holds a site fixed for the run).
    let got = resolve_in_linux(&distro, "pinned4.steplet-test.example").unwrap();
    println!("--- pinned4.steplet-test.example leads to {got:?}");
    assert!(got.contains(&ip("203.0.113.7")), "the pinned name leads to the pinned address: {got:?}");

    // The family each name lacks is present as a sink, so a one-family lookup cannot fall through to DNS (I6).
    let during = String::from_utf8_lossy(&support::etc_hosts(&distro)).into_owned();
    println!("--- /etc/hosts while pinned:\n{during}");
    assert!(during.contains("100::1 pinned4.steplet-test.example"), "a v4-only name needs a v6 sink: {during}");
    assert!(during.contains("192.0.2.1 pinned6.steplet-test.example"), "a v6-only name needs a v4 sink: {during}");

    // After the run the block is gone and the file is exactly what it was.
    unpin_sites(&distro).unwrap();
    assert_eq!(support::etc_hosts(&distro), before, "/etc/hosts is restored byte for byte");

    // A run with no sites writes no block and leaves the file unchanged.
    pin_sites(&distro, &[]).unwrap();
    assert_eq!(support::etc_hosts(&distro), before, "a run with no sites leaves /etc/hosts unchanged");
}
