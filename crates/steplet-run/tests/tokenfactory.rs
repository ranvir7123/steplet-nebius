//! Steplet talking to Nebius Token Factory (the Nebius plan, Task 2), against a stand-in server on this computer: what
//! Steplet sends, how it reads the answer, and how each kind of failure comes back. No real call, no spend.

use std::io::{BufRead, BufReader, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

use serde_json::{Value, json};
use steplet_run::{
    ApiKey, Finish, HttpLimits, Message, Model, ModelError, Nanodollars, TokenFactory, ToolSpec, Usage, cost, price,
    worst_case,
};

const MADE_UP_KEY: &str = "nebius-STEPLET-TEST-MADE-UP-NOT-A-KEY";

/// What the stand-in server does with one request.
#[derive(Clone)]
enum Then {
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
struct Got {
    method: String,
    path: String,
    headers: Vec<(String, String)>,
    body: Vec<u8>,
}

impl Got {
    fn header(&self, name: &str) -> Option<&str> {
        self.headers.iter().find(|(n, _)| n.eq_ignore_ascii_case(name)).map(|(_, v)| v.as_str())
    }
}

struct Server {
    base: String,
    got: Arc<Mutex<Vec<Got>>>,
}

/// A stand-in server on this computer that answers each request with the next of `script`.
fn server(script: Vec<Then>) -> Server {
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

fn ok(body: Value) -> Then {
    Then::Answer { status: 200, headers: Vec::new(), body: body.to_string() }
}

fn status(status: u16, body: &str) -> Then {
    Then::Answer { status, headers: Vec::new(), body: body.into() }
}

fn short() -> HttpLimits {
    HttpLimits { connect: Duration::from_secs(5), reply: Duration::from_secs(3), max_bytes: 64 * 1024 }
}

fn model(s: &Server) -> TokenFactory {
    TokenFactory::at(&s.base, ApiKey::new(MADE_UP_KEY.into()), "nvidia/nemotron-3-super-120b-a12b", short()).unwrap()
}

fn list_files() -> ToolSpec {
    ToolSpec {
        name: "list_files",
        description: "List the files in a folder.",
        parameters: json!({ "type": "object", "properties": { "path": { "type": "string" } }, "required": ["path"] }),
    }
}

fn convo() -> Vec<Message> {
    vec![Message::system("You are Steplet's agent."), Message::user("List the files in /work/demo.")]
}

fn a_tool_call_answer() -> Value {
    json!({
        "id": "chatcmpl-1",
        "model": "nvidia/nemotron-3-super-120b-a12b",
        "choices": [{
            "index": 0,
            "message": {
                "role": "assistant",
                "content": "Looking first.",
                "tool_calls": [{ "id": "call_7", "type": "function",
                                  "function": { "name": "list_files", "arguments": "{\"path\":\"/work/demo\"}" } }]
            },
            "finish_reason": "tool_calls"
        }],
        "usage": { "prompt_tokens": 321, "completion_tokens": 45, "total_tokens": 366 }
    })
}

#[test]
fn sends_the_key_as_a_bearer_header_and_nowhere_else() {
    let s = server(vec![ok(a_tool_call_answer())]);
    model(&s).next(&convo(), &[list_files()], 4096).unwrap();
    let got = s.got.lock().unwrap()[0].clone();
    assert_eq!(got.method, "POST");
    assert_eq!(got.path, "/v1/chat/completions");
    assert_eq!(got.header("authorization"), Some(format!("Bearer {MADE_UP_KEY}").as_str()));
    assert_eq!(got.header("content-type"), Some("application/json"));
    let body = String::from_utf8(got.body.clone()).unwrap();
    assert!(!body.contains(MADE_UP_KEY), "the key is in the body");
    let v: Value = serde_json::from_str(&body).unwrap();
    assert_eq!(v["model"], "nvidia/nemotron-3-super-120b-a12b");
    assert_eq!(v["max_tokens"], 4096);
    assert_eq!(v["tool_choice"], "auto");
    assert_eq!(v["messages"][1]["role"], "user");
    assert_eq!(v["tools"][0]["type"], "function");
    assert_eq!(v["tools"][0]["function"]["name"], "list_files");
}

#[test]
fn reads_tool_calls_text_and_usage() {
    let s = server(vec![ok(a_tool_call_answer())]);
    let reply = model(&s).next(&convo(), &[list_files()], 4096).unwrap();
    assert_eq!(reply.text.as_deref(), Some("Looking first."));
    assert_eq!(reply.finish, Finish::ToolCalls);
    assert_eq!(reply.calls.len(), 1);
    assert_eq!(reply.calls[0].id, "call_7");
    assert_eq!(reply.calls[0].name, "list_files");
    assert_eq!(reply.calls[0].arguments, "{\"path\":\"/work/demo\"}");
    assert_eq!(reply.usage, Usage { prompt_tokens: 321, completion_tokens: 45 });
    assert_eq!(reply.model, "nvidia/nemotron-3-super-120b-a12b");
}

#[test]
fn a_tool_result_goes_back_as_a_tool_message() {
    let s = server(vec![ok(a_tool_call_answer()), ok(a_tool_call_answer())]);
    let mut m = model(&s);
    let reply = m.next(&convo(), &[list_files()], 4096).unwrap();
    let mut c = convo();
    c.push(Message::assistant(&reply));
    c.push(Message::tool("call_7", "a.txt\nb.txt"));
    m.next(&c, &[list_files()], 4096).unwrap();
    let v: Value = serde_json::from_slice(&s.got.lock().unwrap()[1].body).unwrap();
    assert_eq!(v["messages"][2]["role"], "assistant");
    assert_eq!(v["messages"][2]["tool_calls"][0]["id"], "call_7");
    assert_eq!(v["messages"][2]["tool_calls"][0]["function"]["arguments"], "{\"path\":\"/work/demo\"}");
    assert_eq!(v["messages"][3], json!({ "role": "tool", "tool_call_id": "call_7", "content": "a.txt\nb.txt" }));
}

#[test]
fn a_refused_key_is_key_refused() {
    for code in [401, 403] {
        let s = server(vec![status(code, r#"{"detail":"Invalid API key"}"#)]);
        let e = model(&s).next(&convo(), &[], 100).unwrap_err();
        assert_eq!(e, ModelError::KeyRefused { status: code, detail: "Invalid API key".into() }, "status {code}");
        assert_eq!(e.to_string(), "Nebius did not accept the key. Change it in Setup.");
        assert!(!e.worth_retrying());
    }
}

#[test]
fn no_credit_is_said_as_such() {
    let cases = [
        (402, r#"{"detail":"Payment required"}"#),
        (403, r#"{"error":{"message":"Insufficient balance on your account"}}"#),
        (429, r#"{"error":{"message":"insufficient_quota"}}"#),
    ];
    for (code, body) in cases {
        let s = server(vec![status(code, body)]);
        let e = model(&s).next(&convo(), &[], 100).unwrap_err();
        assert!(matches!(e, ModelError::NoCredit { status, .. } if status == code), "status {code}: {e:?}");
        assert!(e.to_string().starts_with("Your Nebius account has no credit left."));
        assert!(!e.worth_retrying());
    }
}

#[test]
fn busy_and_server_errors_are_told_apart() {
    let busy = Then::Answer {
        status: 429,
        headers: vec![("Retry-After", "7".into())],
        body: r#"{"detail":"Rate limit exceeded"}"#.into(),
    };
    let s = server(vec![busy]);
    let e = model(&s).next(&convo(), &[], 100).unwrap_err();
    assert_eq!(e, ModelError::Busy { status: 429, detail: "Rate limit exceeded".into(), retry_after: Some(7) });
    assert!(e.worth_retrying());
    for code in [500, 502, 503] {
        let s = server(vec![status(code, "<html>Bad gateway</html>")]);
        let e = model(&s).next(&convo(), &[], 100).unwrap_err();
        assert!(matches!(e, ModelError::Unavailable { status, .. } if status == code), "status {code}: {e:?}");
        assert!(e.worth_retrying());
    }
    let s = server(vec![status(404, r#"{"detail":"Model not found"}"#)]);
    let e = model(&s).next(&convo(), &[], 100).unwrap_err();
    assert_eq!(e, ModelError::Rejected { status: 404, detail: "Model not found".into() });
    assert!(!e.worth_retrying());
}

#[test]
fn a_server_that_goes_silent_is_too_slow() {
    let s = server(vec![Then::Silent]);
    let started = Instant::now();
    let e = model(&s).next(&convo(), &[], 100).unwrap_err();
    assert_eq!(e, ModelError::TooSlow);
    assert!(e.worth_retrying());
    assert!(started.elapsed() < Duration::from_secs(15), "took {:?}", started.elapsed());
}

#[test]
fn an_oversized_reply_is_refused() {
    let s = server(vec![Then::Huge(200 * 1024)]);
    let e = model(&s).next(&convo(), &[], 100).unwrap_err();
    assert!(matches!(&e, ModelError::BadReply(why) if why.contains("larger than")), "{e:?}");
}

#[test]
fn an_answer_that_is_not_json_is_a_bad_reply() {
    let s = server(vec![status(200, "<html>hello</html>")]);
    let e = model(&s).next(&convo(), &[], 100).unwrap_err();
    assert!(matches!(e, ModelError::BadReply(_)), "{e:?}");
}

#[test]
fn nobody_listening_is_a_network_error() {
    let port = TcpListener::bind("127.0.0.1:0").unwrap().local_addr().unwrap().port();
    let base = format!("http://127.0.0.1:{port}/v1/");
    let mut m = TokenFactory::at(&base, ApiKey::new(MADE_UP_KEY.into()), "x", short()).unwrap();
    let e = m.next(&convo(), &[], 100).unwrap_err();
    assert!(matches!(e, ModelError::Network(_)), "{e:?}");
}

#[test]
fn only_https_or_this_laptop_is_accepted() {
    let key = || ApiKey::new(MADE_UP_KEY.into());
    for bad in ["http://example.com/v1/", "http://localhost:8000/v1/", "ftp://x/", "https://", "https://a b/"] {
        assert!(TokenFactory::at(bad, key(), "x", short()).is_err(), "{bad} was accepted");
    }
    for good in ["https://api.tokenfactory.nebius.com/v1/", "https://example.com:8443/v1", "http://127.0.0.1:1/v1/"] {
        assert!(TokenFactory::at(good, key(), "x", short()).is_ok(), "{good} was refused");
    }
}

#[test]
fn the_key_never_appears_in_an_error() {
    for code in [400, 401, 429, 500] {
        let s = server(vec![Then::Echo(code)]);
        let e = model(&s).next(&convo(), &[], 100).unwrap_err();
        assert!(!format!("{e:?}").contains(MADE_UP_KEY), "status {code}: {e:?}");
        assert!(!e.to_string().contains(MADE_UP_KEY), "status {code}: {e}");
    }
}

#[test]
fn the_models_list_is_read() {
    let list =
        json!({ "object": "list", "data": [{ "id": "nvidia/nemotron-3-super-120b-a12b" }, { "id": "other/x" }] });
    let s = server(vec![ok(list)]);
    let names = model(&s).models().unwrap();
    assert_eq!(names, ["nvidia/nemotron-3-super-120b-a12b", "other/x"]);
    let got = s.got.lock().unwrap()[0].clone();
    assert_eq!((got.method.as_str(), got.path.as_str()), ("GET", "/v1/models"));
}

#[test]
fn cost_counts_both_directions() {
    let super_ = price("nvidia/nemotron-3-super-120b-a12b").unwrap();
    let u = Usage { prompt_tokens: 1_000_000, completion_tokens: 1_000_000 };
    // $0.30 per million in, $0.90 per million out.
    assert_eq!(cost(&u, &super_), Nanodollars::from_cents(120));
    assert_eq!(price("NVIDIA/NEMOTRON-3-SUPER-120B-A12B"), Some(super_));
    assert_eq!(price("some/other-model"), None);
}

#[test]
fn worst_case_is_at_least_the_cost_of_any_reply_within_max_tokens() {
    let p = price("nvidia/nemotron-3-super-120b-a12b").unwrap();
    for (bytes, max) in [(0usize, 1u32), (2_000, 4096), (500_000, 4096), (3_000_000, 600)] {
        let worst = worst_case(bytes, max, &p);
        for prompt in [0, bytes as u64 / 4, bytes as u64, bytes as u64 + 1000] {
            for done in [0, u64::from(max)] {
                let real = cost(&Usage { prompt_tokens: prompt, completion_tokens: done }, &p);
                assert!(real <= worst, "{bytes} bytes, {prompt} sent, {done} received: {real:?} > {worst:?}");
            }
        }
    }
}
