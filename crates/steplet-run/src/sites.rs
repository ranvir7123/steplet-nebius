//! The listed-sites guard (phase 5 Task 4, choice K4 option B). The agent's fence (Claude Code 2.1.220's own) lets
//! the agent's programs reach only the web sites the file declares; but it resolves each declared name itself and
//! dials whatever address it leads to, with no check on that address (srt 0.0.77 has such a check, the bundled fence
//! does not). So a file that declares a name pointing at this computer, or at a private network, reaches programs
//! there (found in Task 4, Step 4). Before a run, Steplet resolves each declared site where it would lead for the
//! run, refuses the run if one leads to this computer or a private network, and keeps those answers for the run (the
//! fence resolves through `/etc/hosts`, so the answers are pinned there for its length).

use std::collections::HashMap;
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};
use std::time::Duration;

use crate::wsl;

/// Every call the guard makes into Steplet's Linux is given this limit, so a Linux that will not answer ends the call
/// (and lets the run fail closed) instead of hanging the start or the end of a run. It matches the other short WSL
/// calls (launch, the run's clean-up).
const WSL_LIMIT: Duration = Duration::from_secs(10);

/// Whether `ip` is on this computer (Windows or Steplet's Linux), on a private network, or not a plain public address
/// a web site could legitimately use. This is the guard's decision, so it fails closed: it refuses loopback, the
/// unspecified address, every private range, link-local, unique-local and site-local, carrier-grade NAT, the "this
/// network" range (`0.0.0.0/8`), the protocol-assignment range (`192.0.0.0/24`), the benchmarking range
/// (`198.18.0.0/15`, which proxy tools hand out to route back to a proxy on this computer), and the multicast and
/// reserved ranges (`224.0.0.0/4`, `240.0.0.0/4`). The address WSL and Windows use for each other sits in a private
/// range (`172.16/12` on this laptop), so it is covered. A v6 address that carries a v4 inside it (v4-mapped,
/// v4-compatible, 6to4, or NAT64) is judged by that v4, so a public v4 inside a v6 is still allowed.
pub fn leads_to_this_computer_or_private(ip: IpAddr) -> bool {
    match ip {
        IpAddr::V4(v4) => not_a_public_v4(v4),
        IpAddr::V6(v6) => {
            if let Some(v4) = carried_v4(v6) {
                return not_a_public_v4(v4);
            }
            v6.is_loopback()
                || v6.is_unspecified()
                || (v6.segments()[0] & 0xffc0) == 0xfe80 // link-local, fe80::/10
                || (v6.segments()[0] & 0xffc0) == 0xfec0 // site-local, fec0::/10 (deprecated)
                || (v6.octets()[0] & 0xfe) == 0xfc // unique-local, fc00::/7
                || is_local_nat64(v6) // 64:ff9b:1::/48, a local-use NAT64 prefix with no plain v4 to pull out
        }
    }
}

/// Whether a v4 address is anything other than a plain public one. `is_private` covers `10/8`, `172.16/12` and
/// `192.168/16`.
fn not_a_public_v4(v4: Ipv4Addr) -> bool {
    let [a, b, c, _] = v4.octets();
    v4.is_loopback()
        || v4.is_private()
        || v4.is_link_local()
        || v4.is_unspecified()
        || v4.is_broadcast()
        || a == 0 // 0.0.0.0/8, "this network" (RFC 1122)
        || (a == 100 && (64..=127).contains(&b)) // carrier-grade NAT, 100.64.0.0/10 (RFC 6598)
        || (a == 192 && b == 0 && c == 0) // 192.0.0.0/24, IETF protocol assignments
        || (a == 198 && (b == 18 || b == 19)) // 198.18.0.0/15, benchmarking
        || (a & 0xf0) == 0xe0 // 224.0.0.0/4, multicast
        || a >= 240 // 240.0.0.0/4, reserved
}

/// The v4 address a v6 one carries, for the forms a resolver may hand back: v4-mapped (`::ffff:a.b.c.d`),
/// v4-compatible (`::a.b.c.d`, but not `::` or `::1`), 6to4 (`2002::/16`), and the well-known NAT64 prefix
/// (`64:ff9b::/96`). `None` for a v6 that carries no v4.
fn carried_v4(v6: Ipv6Addr) -> Option<Ipv4Addr> {
    if let Some(v4) = v6.to_ipv4_mapped() {
        return Some(v4);
    }
    let seg = v6.segments();
    let low_v4 = || Ipv4Addr::from(((seg[6] as u32) << 16) | seg[7] as u32);
    if seg[0] == 0x2002 {
        // 6to4: the v4 is the two segments after the 2002 prefix.
        return Some(Ipv4Addr::from(((seg[1] as u32) << 16) | seg[2] as u32));
    }
    if seg[0] == 0x0064 && seg[1] == 0xff9b && seg[2..6] == [0, 0, 0, 0] {
        return Some(low_v4()); // NAT64 well-known 64:ff9b::/96
    }
    if seg[..6] == [0, 0, 0, 0, 0, 0] && !(seg[6] == 0 && (seg[7] == 0 || seg[7] == 1)) {
        return Some(low_v4()); // v4-compatible ::a.b.c.d, leaving :: and ::1 to the v6 checks
    }
    None
}

/// The local-use NAT64 prefix `64:ff9b:1::/48` (RFC 8215): a translation prefix whose embedded v4 sits at a
/// non-contiguous offset, so it is refused whole rather than decoded.
fn is_local_nat64(v6: Ipv6Addr) -> bool {
    let seg = v6.segments();
    seg[0] == 0x0064 && seg[1] == 0xff9b && seg[2] == 0x0001
}

/// A web site the file declared that Steplet will not run with, and why, in the receiver's words.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ListedSite {
    pub site: String,
    pub why: String,
}

/// Resolves each `domains` entry with `resolve` (which answers where a name leads, or why it could not). Returns the
/// resolved addresses to pin for the run when every site leads to a public address; refuses, naming the one site,
/// when a site leads to this computer or a private network, or cannot be found at all (fail closed). A file that
/// lists no site passes with nothing to pin.
pub fn check_sites(
    domains: &[String],
    resolve: impl Fn(&str) -> Result<Vec<IpAddr>, String>,
) -> Result<Vec<(String, IpAddr)>, ListedSite> {
    let mut pins = Vec::new();
    for site in domains {
        let found = match resolve(site) {
            Ok(found) if !found.is_empty() => found,
            Ok(_) => return Err(cannot_find(site, "it led nowhere")),
            Err(why) => return Err(cannot_find(site, &why)),
        };
        if found.iter().any(|ip| leads_to_this_computer_or_private(*ip)) {
            let why = "Steplet will not run this file: the web site it lists, \"".to_owned()
                + site
                + "\", leads to this computer or a private network, which the run is not allowed to reach.";
            return Err(ListedSite { site: site.clone(), why });
        }
        pins.extend(found.into_iter().map(|ip| (site.clone(), ip)));
    }
    Ok(pins)
}

fn cannot_find(site: &str, detail: &str) -> ListedSite {
    let why = format!(
        "Steplet will not run this file: it could not find where the web site it lists, \"{site}\", leads ({detail})."
    );
    ListedSite { site: site.to_owned(), why }
}

/// Resolves every name in `names` in Steplet's Linux, as the steplet account (the account the agent's programs run as,
/// so it sees the same names the run would), in one call: a run that lists many sites makes one short WSL call, not
/// one for each, so a Linux that will not answer cannot stall the start of a run (the call is bounded by `WSL_LIMIT`).
/// The map gives, for each name that resolved, every address it leads to; a name that did not resolve is left out, and
/// the check then refuses the run for it (fail closed). Each `name` is a declared domain, so it holds only lower-case
/// letters, digits, dots and hyphens. An `Err` means the look-up itself could not run (no Python, a WSL error, or the
/// time limit), and the caller refuses the run.
pub fn resolve_all_in_linux(distro: &str, names: &[String]) -> Result<HashMap<String, Vec<IpAddr>>, String> {
    if names.is_empty() {
        return Ok(HashMap::new());
    }
    // One name may fail to resolve without stopping the rest: it is simply left out, so the check refuses it.
    const LOOK: &str = "import socket, sys\n\
        for name in sys.argv[1:]:\n    \
            try:\n        \
                infos = socket.getaddrinfo(name, None, type=socket.SOCK_STREAM)\n    \
            except OSError:\n        \
                continue\n    \
            seen = set()\n    \
            for info in infos:\n        \
                ip = info[4][0].split('%')[0]\n        \
                if ip not in seen:\n            \
                    seen.add(ip)\n            \
                    print(name, ip)\n";
    let mut argv = vec!["/usr/bin/python3", "-I", "-c", LOOK];
    argv.extend(names.iter().map(String::as_str));
    let out = wsl::run_within(distro, "steplet", &argv, b"", WSL_LIMIT).map_err(|e| e.to_string())?;
    if out.status != Some(0) {
        let said = if out.stderr.trim().is_empty() { out.stdout.trim() } else { out.stderr.trim() };
        return Err(said.to_owned());
    }
    let mut map: HashMap<String, Vec<IpAddr>> = HashMap::new();
    for line in out.stdout.lines() {
        let Some((name, addr)) = line.split_once(' ') else { continue };
        let ip = addr.trim().parse::<IpAddr>().map_err(|e| format!("{line:?}: {e}"))?;
        map.entry(name.to_owned()).or_default().push(ip);
    }
    Ok(map)
}

/// Resolves one `name` in Steplet's Linux (as `resolve_all_in_linux` does for one), to every address it leads to. An
/// error means the name could not be found or the look-up could not run, which makes the check refuse.
pub fn resolve_in_linux(distro: &str, name: &str) -> Result<Vec<IpAddr>, String> {
    let names = [name.to_owned()];
    match resolve_all_in_linux(distro, &names)?.remove(name) {
        Some(ips) if !ips.is_empty() => Ok(ips),
        _ => Err(format!("the name {name:?} led nowhere")),
    }
}

/// The lines that mark the run's pins inside `/etc/hosts`.
const PIN_BEGIN: &str = "# steplet-run pins begin";
const PIN_END: &str = "# steplet-run pins end";

/// Where a pinned name's missing address family is sent, so a lookup for that family cannot fall through to the public
/// name service and reach a rebinding address. Both go nowhere: `192.0.2.1` is documentation-only (TEST-NET-1,
/// RFC 5737) and `100::1` is discard-only (RFC 6666); neither is routed on the public internet.
const V4_SINK: &str = "192.0.2.1";
const V6_SINK: &str = "100::1";

/// Keeps `pins` (a run's listed sites and the addresses they led to) fixed for the run, by writing them into Steplet's
/// Linux `/etc/hosts`: the fence resolves a name through `/etc/hosts` first, so each listed site reaches the address
/// it led to at the start, not wherever the name may point later. The lines go in one marked block; a block a crashed
/// run left behind is replaced, so a run never inherits another's pins. `unpin_sites` removes the block when the run
/// ends.
pub fn pin_sites(distro: &str, pins: &[(String, IpAddr)]) -> Result<(), String> {
    rewrite_hosts(distro, &hosts_block(pins))
}

/// The marked block of pin lines for `pins` (a run's listed sites and the addresses they led to), or the empty string
/// when there are none. Each name's real addresses are written, grouped, in order; and for the one address family a
/// name has no real address in, a sink line is written too (I6), so a lookup of that family finds the name in
/// `/etc/hosts` and cannot fall through to the public name service and reach a rebinding address.
fn hosts_block(pins: &[(String, IpAddr)]) -> String {
    if pins.is_empty() {
        return String::new();
    }
    let mut names: Vec<&str> = Vec::new();
    for (name, _) in pins {
        if !names.contains(&name.as_str()) {
            names.push(name);
        }
    }
    let mut block = format!("{PIN_BEGIN}\n");
    for name in names {
        let ips: Vec<IpAddr> = pins.iter().filter(|(n, _)| n == name).map(|(_, ip)| *ip).collect();
        for ip in &ips {
            block.push_str(&format!("{ip} {name}\n"));
        }
        if !ips.iter().any(IpAddr::is_ipv4) {
            block.push_str(&format!("{V4_SINK} {name}\n"));
        }
        if !ips.iter().any(IpAddr::is_ipv6) {
            block.push_str(&format!("{V6_SINK} {name}\n"));
        }
    }
    block.push_str(PIN_END);
    block.push('\n');
    block
}

/// Removes the run's pin block from `/etc/hosts`.
pub fn unpin_sites(distro: &str) -> Result<(), String> {
    rewrite_hosts(distro, "")
}

fn rewrite_hosts(distro: &str, block: &str) -> Result<(), String> {
    let current =
        wsl::run_within(distro, "root", &["/usr/bin/cat", "/etc/hosts"], b"", WSL_LIMIT).map_err(|e| e.to_string())?;
    let after = hosts_after_rewrite(&current, block)?;
    // One atomic swap as root: write a temp beside /etc/hosts (same file system, so `mv` is a rename, never a
    // half-written file), give it the usual mode (mktemp makes it private, which would stop the steplet account from
    // resolving names), then move it over /etc/hosts. A temp is left behind only if a step fails, and is removed then.
    const SWAP: &str = "set -e\n\
        tmp=$(mktemp /etc/hosts.steplet.XXXXXX)\n\
        trap 'rm -f \"$tmp\"' EXIT\n\
        cat > \"$tmp\"\n\
        chmod 644 \"$tmp\"\n\
        mv -f \"$tmp\" /etc/hosts\n";
    let out =
        wsl::run_within(distro, "root", &["/bin/sh", "-c", SWAP], &after, WSL_LIMIT).map_err(|e| e.to_string())?;
    if out.status != Some(0) {
        return Err(out.stderr);
    }
    Ok(())
}

/// The bytes to write back over `/etc/hosts`: the current file with any stale pin block stripped, then `block`
/// appended. Refuses (fail closed) when the read of `/etc/hosts` did not finish cleanly — a `wsl.exe` that exits
/// non-zero or is ended by a signal may print its own error, or nothing, and that must never be written back.
fn hosts_after_rewrite(current: &wsl::Output, block: &str) -> Result<Vec<u8>, String> {
    if current.status != Some(0) {
        return Err(format!(
            "Steplet could not read /etc/hosts to keep the run's sites fixed (exit {:?})",
            current.status
        ));
    }
    let mut after = hosts_without_block(&current.stdout_bytes);
    after.extend_from_slice(block.as_bytes());
    Ok(after)
}

/// `text` with any `PIN_BEGIN`..`PIN_END` block removed, every other line kept byte for byte (so a line that is not
/// UTF-8, and is not Steplet's, is never changed).
fn hosts_without_block(text: &[u8]) -> Vec<u8> {
    let mut kept = Vec::new();
    let mut skipping = false;
    for line in text.split_inclusive(|b| *b == b'\n') {
        let trimmed = trim_eol(line);
        if trimmed == PIN_BEGIN.as_bytes() {
            skipping = true;
        } else if skipping {
            if trimmed == PIN_END.as_bytes() {
                skipping = false;
            }
        } else {
            kept.extend_from_slice(line);
        }
    }
    kept
}

/// `line` with any trailing newline and carriage-return bytes removed.
fn trim_eol(line: &[u8]) -> &[u8] {
    let mut end = line.len();
    while end > 0 && (line[end - 1] == b'\n' || line[end - 1] == b'\r') {
        end -= 1;
    }
    &line[..end]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_pinned_name_gets_a_sink_for_the_family_it_lacks_so_a_one_family_lookup_cannot_fall_through() {
        let pins = vec![
            ("v4only.example".to_owned(), "93.184.216.34".parse().unwrap()),
            ("v6only.example".to_owned(), "2606:2800:220:1::1".parse().unwrap()),
            ("both.example".to_owned(), "1.2.3.4".parse().unwrap()),
            ("both.example".to_owned(), "2001:db8::1".parse().unwrap()),
        ];
        let block = hosts_block(&pins);

        // The real addresses are written, grouped under each name.
        assert!(block.contains("93.184.216.34 v4only.example\n"), "{block}");
        assert!(block.contains("2606:2800:220:1::1 v6only.example\n"), "{block}");

        // The family a name lacks is sent nowhere, so a lookup of that family cannot fall through to DNS.
        assert!(block.contains(&format!("{V6_SINK} v4only.example\n")), "a v4-only name needs a v6 sink: {block}");
        assert!(block.contains(&format!("{V4_SINK} v6only.example\n")), "a v6-only name needs a v4 sink: {block}");

        // A dual-stack name needs no sink.
        assert!(!block.contains(&format!("{V6_SINK} both.example")), "{block}");
        assert!(!block.contains(&format!("{V4_SINK} both.example")), "{block}");

        // The block is wrapped in the markers, and no sites means no block at all.
        assert!(block.starts_with(&format!("{PIN_BEGIN}\n")) && block.trim_end().ends_with(PIN_END), "{block}");
        assert_eq!(hosts_block(&[]), "", "a run with no sites writes no block");
    }

    #[test]
    fn a_failed_read_of_etc_hosts_refuses_rather_than_writing_its_error_back() {
        let block = format!("{PIN_BEGIN}\n1.2.3.4 a.example\n{PIN_END}\n");
        // A good read: the new file is the old file (any stale block stripped) with the fresh block appended.
        let ok = out(Some(0), b"127.0.0.1 localhost\n");
        assert_eq!(hosts_after_rewrite(&ok, &block).unwrap(), format!("127.0.0.1 localhost\n{block}").into_bytes());
        // A read that failed (non-zero exit) or was ended by a signal is refused, so a passing WSL error — whose
        // output might be its own message, in two-byte text, or nothing — can never be written back over /etc/hosts.
        assert!(hosts_after_rewrite(&out(Some(1), b"wsl: some error"), &block).is_err(), "a non-zero exit is refused");
        assert!(hosts_after_rewrite(&out(None, b""), "").is_err(), "a read ended by a signal is refused");
    }

    #[test]
    fn a_non_utf8_line_that_is_not_steplets_is_kept_byte_for_byte() {
        // A line with a byte that is not UTF-8 (here 0xff), and which is not inside a pin block, must survive exactly:
        // the old code turned it into the replacement character and wrote that back.
        let mut input = b"127.0.0.1 localhost\n".to_vec();
        input.extend_from_slice(&[0xff, 0xfe, b' ', b'x', b'\n']);
        input.extend_from_slice(format!("{PIN_BEGIN}\n9.9.9.9 old.example\n{PIN_END}\n").as_bytes());
        let mut expected = b"127.0.0.1 localhost\n".to_vec();
        expected.extend_from_slice(&[0xff, 0xfe, b' ', b'x', b'\n']);
        assert_eq!(hosts_without_block(&input), expected);
    }

    fn out(status: Option<i32>, stdout_bytes: &[u8]) -> wsl::Output {
        wsl::Output { status, stdout: String::new(), stderr: String::new(), stdout_bytes: stdout_bytes.to_vec() }
    }

    #[test]
    fn a_pin_block_is_stripped_wherever_it_sits_and_other_lines_are_untouched() {
        let plain = b"127.0.0.1 localhost\n::1 localhost\n";
        assert_eq!(hosts_without_block(plain), plain.to_vec(), "with no block, nothing changes");

        let with_block = format!("127.0.0.1 localhost\n{PIN_BEGIN}\n93.184.216.34 a.example\n{PIN_END}\n10.0.0.2 b\n");
        assert_eq!(hosts_without_block(with_block.as_bytes()), b"127.0.0.1 localhost\n10.0.0.2 b\n".to_vec());

        // Steplet writes a begin and an end together as one atomic swap, so it never leaves a block with no end;
        // should a foreign dangling begin ever appear, everything from it is dropped, so no stale pin survives a run.
        let unclosed = format!("keep me\n{PIN_BEGIN}\n1.2.3.4 x.example\n");
        assert_eq!(hosts_without_block(unclosed.as_bytes()), b"keep me\n".to_vec());

        // Writing a fresh block over an old one keeps only the new.
        let old = format!("base\n{PIN_BEGIN}\nold\n{PIN_END}\n");
        let mut fresh = hosts_without_block(old.as_bytes());
        fresh.extend_from_slice(format!("{PIN_BEGIN}\n5.5.5.5 new.example\n{PIN_END}\n").as_bytes());
        assert_eq!(fresh, format!("base\n{PIN_BEGIN}\n5.5.5.5 new.example\n{PIN_END}\n").into_bytes());
    }
}
