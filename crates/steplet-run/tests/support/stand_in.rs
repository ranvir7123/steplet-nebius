//! A stand-in for Nebius Token Factory on this computer (the Nebius plan, Tasks 2 and 9): it answers each request with
//! the next of a script and keeps every request as it saw it. No real call, no spend.

use std::io::{BufRead, BufReader, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

/// What the stand-in server does with one request.
#[derive(Clone)]
pub enum Then {
    Answer {
        status: u16,
        headers: Vec<(&'static str, String)>,
        body: String,
    },
    /// Reads the request and never answers.
    Silent,
    /// Answers 200 with this many bytes.
    Huge(usize),
    /// Answers with the request's own Authorization header in the body, with this status.
    Echo(u16),
}

/// One request as the stand-in server saw it.
#[derive(Clone, Debug)]
pub struct Got {
    pub method: String,
    pub path: String,
    pub headers: Vec<(String, String)>,
    pub body: Vec<u8>,
}

impl Got {
    pub fn header(&self, name: &str) -> Option<&str> {
        self.headers.iter().find(|(n, _)| n.eq_ignore_ascii_case(name)).map(|(_, v)| v.as_str())
    }
}

pub struct Server {
    pub base: String,
    pub got: Arc<Mutex<Vec<Got>>>,
}

/// A stand-in server on this computer that answers each request with the next of `script`.
pub fn server(script: Vec<Then>) -> Server {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let base = format!("http://127.0.0.1:{}/v1/", listener.local_addr().unwrap().port());
    let got = Arc::new(Mutex::new(Vec::new()));
    let seen = Arc::clone(&got);
    thread::spawn(move || {
        for (then, stream) in script.into_iter().zip(listener.incoming()) {
            let Ok(stream) = stream else { return };
            let seen = Arc::clone(&seen);
            thread::spawn(move || serve(stream, then, &seen));
        }
    });
    Server { base, got }
}

fn serve(stream: TcpStream, then: Then, seen: &Mutex<Vec<Got>>) {
    let mut reader = BufReader::new(stream.try_clone().unwrap());
    let mut line = String::new();
    reader.read_line(&mut line).unwrap();
    let mut parts = line.split_whitespace();
    let method = parts.next().unwrap_or_default().to_string();
    let path = parts.next().unwrap_or_default().to_string();
    let mut headers = Vec::new();
    loop {
        let mut h = String::new();
        reader.read_line(&mut h).unwrap();
        let h = h.trim_end();
        if h.is_empty() {
            break;
        }
        if let Some((n, v)) = h.split_once(':') {
            headers.push((n.trim().to_string(), v.trim().to_string()));
        }
    }
    let length: usize = headers
        .iter()
        .find(|(n, _)| n.eq_ignore_ascii_case("content-length"))
        .map(|(_, v)| v.parse().unwrap())
        .unwrap_or(0);
    let mut body = vec![0; length];
    reader.read_exact(&mut body).unwrap();
    let got = Got { method, path, headers, body };
    seen.lock().unwrap().push(got.clone());
    let mut out = stream;
    let (status, extra, body) = match then {
        Then::Answer { status, headers, body } => (status, headers, body.into_bytes()),
        Then::Silent => {
            thread::sleep(Duration::from_secs(30));
            return;
        }
        Then::Huge(n) => (200, Vec::new(), vec![b'x'; n]),
        Then::Echo(status) => (status, Vec::new(), got.header("authorization").unwrap_or_default().as_bytes().to_vec()),
    };
    let mut head = format!("HTTP/1.1 {status} X\r\nContent-Length: {}\r\nConnection: close\r\n", body.len());
    for (n, v) in extra {
        head.push_str(&format!("{n}: {v}\r\n"));
    }
    head.push_str("\r\n");
    let _ = out.write_all(head.as_bytes());
    let _ = out.write_all(&body);
}
