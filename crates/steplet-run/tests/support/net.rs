//! Network test support shared by `contain.rs` and `agent_linux.rs` (test support only): the probes a case plants in a
//! working copy, the listener on Windows, and the knocks that are each case's controls.

use std::io::{BufRead, BufReader};
use std::net::TcpStream;
use std::path::Path;
use std::process::{Command, Stdio};
use std::time::Duration;

use steplet_run::wsl;

/// Test data (made up, inert): a program that tries to fetch two web pages and writes down, for
/// each, whether it got there. It writes to the file named by its argument, or to `probe_results.json`.
pub const SITE_PROBE: &str = r#"import json, os, sys, urllib.error, urllib.request
results = {"proxy": os.environ.get("HTTPS_PROXY") or os.environ.get("https_proxy") or "none"}
for url in ["https://example.org/", "https://example.net/"]:
    try:
        urllib.request.urlopen(url, timeout=20).read(64)
        results[url] = "reached"
    except urllib.error.HTTPError as e:
        # The site itself answered, with an error status: it was reached.
        results[url] = "reached"
    except Exception as e:
        results[url] = "failed: " + type(e).__name__ + ": " + str(e)
with open(sys.argv[1] if len(sys.argv) > 1 else "probe_results.json", "w") as f:
    json.dump(results, f)
print(json.dumps(results))
"#;

/// Test data (made up, inert): a program that tries to connect to each listener by address, and to
/// the file's one real site, in every way the fence offers: directly, and through each proxy named in its environment
/// (HTTP CONNECT and SOCKS5, with the login the proxy's address carries; never printed). Each connection to a listener
/// sends a word naming the way and the target, so a listener's log would show which one got there. It writes down,
/// for each try, whether it got there, in `probe_windows.json`.
pub const WINDOWS_PROBE: &str = r#"import base64, json, os, socket, struct, sys
from urllib.parse import unquote, urlparse

wsl_address, windows_port, linux_port = sys.argv[1], int(sys.argv[2]), int(sys.argv[3])
TARGETS = [
    ("windows-loopback", "127.0.0.1", windows_port),
    ("windows-wsl-address", wsl_address, windows_port),
    ("linux-loopback", "127.0.0.1", linux_port),
    ("declared-site", "example.org", 443),
]

def ways():
    found = {}
    for name, value in sorted(os.environ.items()):
        if name.lower().endswith("_proxy") and "://" in value:
            p = urlparse(value)
            kind = "socks" if p.scheme.startswith("socks") else "http" if p.scheme.startswith("http") else None
            if kind:
                found.setdefault((kind, p.hostname, p.port), p)
    out, count = [("direct", None, None)], {}
    for (kind, _, _), p in found.items():
        count[kind] = count.get(kind, 0) + 1
        out.append(("%s-proxy-%d" % (kind, count[kind]), kind, p))
    return out

def take(s, n):
    data = b""
    while len(data) < n:
        more = s.recv(n - len(data))
        if not more:
            raise OSError("the proxy closed the connection")
        data += more
    return data

def through_http(p, host, port):
    s = socket.create_connection((p.hostname, p.port), timeout=10)
    login = ""
    if p.username:
        word = base64.b64encode(("%s:%s" % (unquote(p.username), unquote(p.password or ""))).encode()).decode()
        login = "Proxy-Authorization: Basic %s\r\n" % word
    s.sendall(("CONNECT %s:%d HTTP/1.1\r\nHost: %s:%d\r\n%s\r\n" % (host, port, host, port, login)).encode())
    reply = b""
    while b"\r\n\r\n" not in reply:
        more = s.recv(1024)
        if not more:
            break
        reply += more
    first = reply.split(b"\r\n")[0].decode("latin-1") or "(no reply)"
    if first.split(" ")[1:2] != ["200"]:
        s.close()
        raise OSError("the proxy said: " + first)
    return s

def through_socks(p, host, port):
    s = socket.create_connection((p.hostname, p.port), timeout=10)
    s.sendall(b"\x05\x02\x00\x02" if p.username else b"\x05\x01\x00")
    method = take(s, 2)[1]
    if method == 2:
        user, word = unquote(p.username or "").encode(), unquote(p.password or "").encode()
        s.sendall(b"\x01" + bytes([len(user)]) + user + bytes([len(word)]) + word)
        if take(s, 2)[1] != 0:
            raise OSError("the proxy refused its own login")
    elif method != 0:
        raise OSError("the proxy refused every way to log in")
    name = host.encode()
    s.sendall(b"\x05\x01\x00\x03" + bytes([len(name)]) + name + struct.pack(">H", port))
    reply = take(s, 4)
    if reply[1] != 0:
        s.close()
        raise OSError("the proxy said: SOCKS reply %d" % reply[1])
    take(s, {1: 4, 4: 16}.get(reply[3]) or take(s, 1)[0])
    take(s, 2)
    return s

results = []
for way, kind, p in ways():
    for target, host, port in TARGETS:
        try:
            if kind is None:
                s = socket.create_connection((host, port), timeout=10)
            elif kind == "http":
                s = through_http(p, host, port)
            else:
                s = through_socks(p, host, port)
            if target != "declared-site":
                s.sendall(("inside-%s-%s" % (way, target)).encode())
            s.close()
            results.append({"way": way, "target": target, "reached": True, "detail": "connected"})
        except Exception as e:
            results.append({"way": way, "target": target, "reached": False, "detail": "%s: %s" % (type(e).__name__, e)})
with open("probe_windows.json", "w") as f:
    json.dump(results, f)
for r in results:
    print(r["way"], r["target"], "->", "REACHED" if r["reached"] else r["detail"])
"#;

/// The listener on Windows (`support/windows_listener.py`), run with Windows' own Python on `port` at each of
/// `addresses`, until it is dropped, or until this test program ends (its standard input closes). A case that cannot
/// start it did not run.
pub struct WindowsListener(std::process::Child);

impl WindowsListener {
    pub fn start(port: u16, log: &Path, addresses: &[&str]) -> WindowsListener {
        let script = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/support/windows_listener.py");
        let mut child = match Command::new("python")
            .args(["-I", script, &port.to_string()])
            .arg(log)
            .args(addresses)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
        {
            Ok(child) => child,
            Err(e) => super::did_not_run(&format!("Windows' own Python did not start: {e}")),
        };
        let mut first = String::new();
        let _ = BufReader::new(child.stdout.take().unwrap()).read_line(&mut first);
        println!("--- the Windows listener: {}", first.trim_end());
        if !first.starts_with("listening on") {
            let _ = child.kill();
            let mut why = String::new();
            let _ = std::io::Read::read_to_string(&mut child.stderr.take().unwrap(), &mut why);
            let _ = child.wait();
            super::did_not_run(&format!("the listener on Windows did not start: {}", why.trim_end()));
        }
        WindowsListener(child)
    }
}

impl Drop for WindowsListener {
    fn drop(&mut self) {
        let _ = self.0.kill();
        let _ = self.0.wait();
    }
}

/// The steplet account, outside the fence, connects to `host:port` and sends `word`: "reached", or why not.
pub fn knock(distro: &str, host: &str, port: u16, word: &str) -> String {
    let py = format!(
        "import socket\ntry:\n    s = socket.create_connection(({host:?}, {port}), 5)\n    s.sendall({word:?}.encode())\n    \
         s.close()\n    print('reached')\nexcept OSError as e:\n    print('failed: %s: %s' % (type(e).__name__, e))"
    );
    wsl::run(distro, "steplet", &["/usr/bin/python3", "-I", "-c", &py], b"").unwrap().stdout.trim_end().to_owned()
}

/// A plain Windows program (this test) connects to `host:port` and sends `word`.
pub fn knock_from_windows(host: &str, port: u16, word: &str) -> std::io::Result<()> {
    let to = format!("{host}:{port}").parse().unwrap();
    let mut s = TcpStream::connect_timeout(&to, Duration::from_secs(5))?;
    std::io::Write::write_all(&mut s, word.as_bytes())
}

/// The default route's gateway in Steplet's Linux: the address WSL uses to reach Windows. A run that has none did not
/// run.
pub fn wsl_address(distro: &str) -> String {
    let route = super::root_sh(distro, "ip route show default");
    match route.strip_prefix("default via ").and_then(|r| r.split_whitespace().next()) {
        Some(address) => address.to_owned(),
        None => super::did_not_run(&format!("Steplet's Linux has no default route to Windows: {route:?}")),
    }
}
