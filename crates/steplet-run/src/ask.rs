//! The question path (phase 4 Task 4, R7): the gate's decision function for a live run. The checker's allow and deny
//! go back at once. Its "ask" becomes a question on the receiver's screen, bound to the exact request by its
//! fingerprint; the run's call waits for the answer, at most the ask limit, and stops waiting at once when the run is
//! being stopped. Only a yes carrying the question's own fingerprint allows it, and the allow hands back exactly the
//! request that was asked. A yes covers that exact request for the rest of the run, never anything broader, and is
//! kept in memory only (J2 B).

use std::collections::{HashMap, HashSet};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Condvar, Mutex, PoisonError};
use std::time::{Duration, Instant};

use serde_json::Value;
use sha2::{Digest, Sha256};
use steplet_core::policy::{Decision, Policy};

use crate::decide::{check, no_one_to_ask};
use crate::live::{Live, Progress, StopSignal};
use crate::report::Scrubber;
use crate::types::{GateAnswer, GateRequest};
use crate::watch::what;

/// How often a waiting question looks whether the run is being stopped.
const LOOK: Duration = Duration::from_millis(50);

const MISMATCH: &str = "the answer did not match the request on screen, so Steplet refused it";

/// One question for the receiver. `what` and `input` are cleaned for the screen; `fingerprint` is of the request
/// itself.
#[derive(Clone, Debug, PartialEq)]
pub struct Question {
    pub id: u64,
    pub tool: String,
    pub what: String,
    pub input: Value,
    /// Why Steplet asks: the checker's reason.
    pub reason: String,
    pub fingerprint: String,
    pub asked_at: Instant,
}

/// What became of a question.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Reply {
    /// The receiver said yes to the request with this fingerprint.
    Yes {
        fingerprint: String,
    },
    No,
    /// No answer within the ask limit.
    NoAnswer,
    /// The run was being stopped, so Steplet took the question back.
    Withdrawn,
    /// Not asked: the receiver had already said yes to this exact request in this run.
    Reused,
}

/// One question and its reply, for the report. `call` is the tool call's id.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Answer {
    pub call: String,
    pub tool: String,
    pub what: String,
    pub reply: Reply,
}

#[derive(Debug, Default)]
struct Waiting {
    open: HashSet<u64>,
    given: HashMap<u64, Reply>,
}

/// Where the screen puts the receiver's answers. Clones share one box.
#[derive(Clone, Debug, Default)]
pub struct Answers(Arc<(Mutex<Waiting>, Condvar)>);

impl Answers {
    /// Gives `reply` to question `question`. False when there is no such open question: never asked, already
    /// answered, or closed by its time limit or a stop.
    pub fn give(&self, question: u64, reply: Reply) -> bool {
        let (waiting, ready) = &*self.0;
        let mut w = waiting.lock().unwrap_or_else(PoisonError::into_inner);
        if !w.open.remove(&question) {
            return false;
        }
        w.given.insert(question, reply);
        ready.notify_all();
        true
    }

    fn open(&self, question: u64) {
        self.0.0.lock().unwrap_or_else(PoisonError::into_inner).open.insert(question);
    }

    /// The reply to `question`: the receiver's, or `NoAnswer` at `until`, or `Withdrawn` once `stop` is set. The
    /// question is closed either way.
    fn wait(&self, question: u64, until: Instant, stop: &StopSignal) -> Reply {
        let (waiting, ready) = &*self.0;
        let mut w = waiting.lock().unwrap_or_else(PoisonError::into_inner);
        loop {
            if let Some(reply) = w.given.remove(&question) {
                return reply;
            }
            let now = Instant::now();
            let closed = if stop.is_set() {
                Reply::Withdrawn
            } else if now >= until {
                Reply::NoAnswer
            } else {
                w = ready.wait_timeout(w, (until - now).min(LOOK)).unwrap_or_else(PoisonError::into_inner).0;
                continue;
            };
            w.open.remove(&question);
            return closed;
        }
    }
}

/// One run's question path. Holds the run's approvals (fingerprints, in memory only), its question numbers and its
/// record of answers.
pub struct Asker {
    live: Arc<Live>,
    scrub: Scrubber,
    last: AtomicU64,
    approved: Mutex<HashSet<String>>,
    record: Mutex<Vec<Answer>>,
}

impl Asker {
    /// The question path of a run followed by `live`; `secrets` (the key, the run's token) never reach the screen.
    pub fn new(live: Arc<Live>, secrets: &[&str]) -> Asker {
        Asker {
            live,
            scrub: Scrubber::new(secrets),
            last: AtomicU64::new(0),
            approved: Mutex::new(HashSet::new()),
            record: Mutex::new(Vec::new()),
        }
    }

    /// The gate's answer to `req`; blocks at most the ask limit, and stops waiting once the run is being stopped.
    pub fn decide(&self, policy: &Policy, req: &GateRequest) -> GateAnswer {
        let reason = match check(policy, req) {
            Err(message) => return deny(message),
            Ok(Decision::Allow) => return allow(req),
            Ok(Decision::Deny { reason, .. }) => return deny(reason),
            Ok(Decision::Ask { reason }) => reason,
        };
        let Some(answers) = &self.live.answers else { return no_one_to_ask(reason) };
        let print = fingerprint(req);
        let (tool, what) = (self.scrub.text(&req.tool_name), self.scrub.text(&what(&req.tool_name, &req.input)));
        let note = |reply: &Reply| {
            let answer = Answer {
                call: self.scrub.text(&req.tool_use_id),
                tool: tool.clone(),
                what: what.clone(),
                reply: reply.clone(),
            };
            self.record.lock().unwrap_or_else(PoisonError::into_inner).push(answer);
        };
        if self.approved.lock().unwrap_or_else(PoisonError::into_inner).contains(&print) {
            note(&Reply::Reused);
            return allow(req);
        }
        let id = self.last.fetch_add(1, Ordering::SeqCst) + 1;
        answers.open(id);
        let asked_at = Instant::now();
        let question = Question {
            id,
            tool: tool.clone(),
            what: what.clone(),
            input: self.scrub.value(&req.input),
            reason: reason.into(),
            fingerprint: print.clone(),
            asked_at,
        };
        (self.live.on_progress)(Progress::Asking(question));
        let (reply, answer) = match answers.wait(id, asked_at + self.live.ask_limit, &self.live.stop) {
            Reply::Yes { fingerprint } if fingerprint == print => {
                self.approved.lock().unwrap_or_else(PoisonError::into_inner).insert(print);
                (Reply::Yes { fingerprint }, allow(req))
            }
            Reply::Yes { .. } | Reply::Reused => (Reply::No, deny(MISMATCH)),
            Reply::No => (Reply::No, deny("the receiver said no")),
            Reply::NoAnswer => (Reply::NoAnswer, deny("no answer in time, so Steplet refused it")),
            Reply::Withdrawn => {
                (Reply::Withdrawn, deny("Steplet is stopping this run, so it took the question back and refused it"))
            }
        };
        note(&reply);
        (self.live.on_progress)(Progress::Answered { question: id, reply });
        answer
    }

    /// Every question asked and every reused yes, in order.
    pub fn answers(&self) -> Vec<Answer> {
        self.record.lock().unwrap_or_else(PoisonError::into_inner).clone()
    }
}

/// The fingerprint of an exact request: SHA-256, in lower-case hexadecimal, of the tool's name, a zero byte, and the
/// request's input as JSON with every object's keys in sorted order. Any change to the request changes it; the tool
/// call's id is not part of it.
pub fn fingerprint(req: &GateRequest) -> String {
    let mut text = String::new();
    canonical(&req.input, &mut text);
    let mut h = Sha256::new();
    h.update(req.tool_name.as_bytes());
    h.update([0]);
    h.update(text.as_bytes());
    h.finalize().iter().map(|b| format!("{b:02x}")).collect()
}

/// `v` as compact JSON with object keys sorted, whatever order the JSON library keeps them in.
fn canonical(v: &Value, out: &mut String) {
    match v {
        Value::Object(fields) => {
            let mut keys: Vec<&String> = fields.keys().collect();
            keys.sort();
            out.push('{');
            for (n, k) in keys.into_iter().enumerate() {
                if n > 0 {
                    out.push(',');
                }
                out.push_str(&Value::String(k.clone()).to_string());
                out.push(':');
                canonical(&fields[k], out);
            }
            out.push('}');
        }
        Value::Array(items) => {
            out.push('[');
            for (n, x) in items.iter().enumerate() {
                if n > 0 {
                    out.push(',');
                }
                canonical(x, out);
            }
            out.push(']');
        }
        other => out.push_str(&other.to_string()),
    }
}

fn allow(req: &GateRequest) -> GateAnswer {
    GateAnswer::Allow { updated_input: req.input.clone() }
}

fn deny(message: &str) -> GateAnswer {
    GateAnswer::Deny { message: message.into() }
}
