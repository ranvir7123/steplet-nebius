//! The ask path (phase 4 Task 4, R7), with the screen played by this test program, so it needs no Linux. When the
//! checker says "ask", the run pauses on a question; only a yes bound to the exact request on screen allows it, and a
//! yes covers that exact request for the rest of the run (J2 B). No answer in time, a stop, or a changed request is a
//! no. The checker's own allow and deny never ask.
//!
//! The test data is short, inert and made up: the key is `support::MADE_UP_KEY`, which is not a key.

mod support;

use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

use serde_json::json;
use steplet_core::canon::canonicalize;
use steplet_core::header::parse;
use steplet_core::policy::Policy;
use steplet_run::{
    Answers, Asker, GateAnswer, GateRequest, Live, Progress, Question, Reply, RunError, RunPaths, RunRequest, RunSetup,
    StopSignal, fingerprint, run_live,
};
use support::MADE_UP_KEY;

const WORK: &str = "/work/r1/repo";
const LIMIT: Duration = Duration::from_millis(200);

fn steplet() -> steplet_core::header::Steplet {
    let f = "---\nsteplet: 1\ntitle: t\nsender: s\nexpires: 2026-12-31\nsteps:\n  - a\nallow:\n  commands:\n    - cargo \
             test\n---\n";
    parse(&canonicalize(f.as_bytes()).unwrap()).unwrap()
}

fn policy() -> Policy {
    Policy::from_steplet(&steplet(), WORK).unwrap()
}

fn bash(command: &str, id: &str) -> GateRequest {
    GateRequest {
        run_token: String::new(),
        tool_name: "Bash".into(),
        input: json!({"command": command, "description": "made-up test call"}),
        tool_use_id: id.into(),
    }
}

/// What the screen does with a question: `Some(reply)` answers it after `delay`, `None` leaves it.
type Screen = dyn Fn(&Question) -> Option<Reply> + Send + Sync;

/// One run's question path, with the screen played by `screen`. Returns the asker, everything sent to the screen,
/// the Stop button and the answers box.
fn asker(screen: Box<Screen>, limit: Duration) -> (Asker, Arc<Mutex<Vec<Progress>>>, StopSignal, Answers) {
    let answers = Answers::default();
    let seen = Arc::new(Mutex::new(Vec::new()));
    let (keep, giving) = (seen.clone(), answers.clone());
    let stop = StopSignal::new();
    let live = Arc::new(Live {
        on_progress: Box::new(move |p| {
            if let Progress::Asking(q) = &p
                && let Some(reply) = screen(q)
            {
                // The screen is another thread: the answer comes a moment after the question.
                let (giving, id) = (giving.clone(), q.id);
                thread::spawn(move || {
                    thread::sleep(Duration::from_millis(20));
                    giving.give(id, reply);
                });
            }
            keep.lock().unwrap().push(p);
        }),
        stop: stop.clone(),
        answers: Some(answers.clone()),
        ask_limit: limit,
    });
    (Asker::new(live, &[MADE_UP_KEY]), seen, stop, answers)
}

fn yes() -> Box<Screen> {
    Box::new(|q| Some(Reply::Yes { fingerprint: q.fingerprint.clone() }))
}

fn questions(seen: &Mutex<Vec<Progress>>) -> Vec<Question> {
    seen.lock()
        .unwrap()
        .iter()
        .filter_map(|p| if let Progress::Asking(q) = p { Some(q.clone()) } else { None })
        .collect()
}

fn closed(seen: &Mutex<Vec<Progress>>) -> Vec<(u64, Reply)> {
    seen.lock()
        .unwrap()
        .iter()
        .filter_map(|p| {
            if let Progress::Answered { question, reply } = p { Some((*question, reply.clone())) } else { None }
        })
        .collect()
}

fn deny(message: &str) -> GateAnswer {
    GateAnswer::Deny { message: message.into() }
}

#[test]
fn a_yes_allows_exactly_the_request_shown() {
    let (asker, seen, _, _) = asker(yes(), LIMIT);
    let req = bash("touch asked.txt", "u1");
    assert_eq!(asker.decide(&policy(), &req), GateAnswer::Allow { updated_input: req.input.clone() });
    let qs = questions(&seen);
    assert_eq!(qs.len(), 1);
    let q = &qs[0];
    assert_eq!(
        (q.tool.as_str(), q.what.as_str(), q.reason.as_str(), &q.input),
        ("Bash", "touch asked.txt", "the file does not ask for this command", &req.input)
    );
    assert_eq!(q.fingerprint, fingerprint(&req));
    assert_eq!(closed(&seen), [(q.id, Reply::Yes { fingerprint: q.fingerprint.clone() })]);
}

#[test]
fn a_yes_with_another_fingerprint_is_a_no() {
    let (asker, seen, _, _) = asker(Box::new(|_| Some(Reply::Yes { fingerprint: "0".repeat(64) })), LIMIT);
    let answer = asker.decide(&policy(), &bash("touch asked.txt", "u1"));
    assert_eq!(answer, deny("the answer did not match the request on screen, so Steplet refused it"));
    assert_eq!(closed(&seen).len(), 1);
    assert_eq!(closed(&seen)[0].1, Reply::No);
}

#[test]
fn no_answer_within_the_limit_is_a_no() {
    let (asker, seen, _, _) = asker(Box::new(|_| None), LIMIT);
    let start = Instant::now();
    let answer = asker.decide(&policy(), &bash("touch asked.txt", "u1"));
    let took = start.elapsed();
    assert_eq!(answer, deny("no answer in time, so Steplet refused it"));
    assert!(took >= LIMIT && took < Duration::from_secs(1), "{took:?}");
    assert_eq!(closed(&seen)[0].1, Reply::NoAnswer);
}

#[test]
fn a_stop_withdraws_the_open_question_at_once() {
    let (asker, seen, stop, _) = asker(Box::new(|_| None), Duration::from_secs(10));
    let start = Instant::now();
    let pressing = stop.clone();
    thread::spawn(move || {
        thread::sleep(Duration::from_millis(100));
        pressing.stop();
    });
    let answer = asker.decide(&policy(), &bash("touch asked.txt", "u1"));
    let took = start.elapsed();
    assert!(matches!(answer, GateAnswer::Deny { .. }), "{answer:?}");
    assert!(took < Duration::from_millis(500), "{took:?}");
    assert_eq!(closed(&seen)[0].1, Reply::Withdrawn);
}

#[test]
fn the_same_exact_request_later_in_the_run_is_allowed_without_asking_again() {
    let (asker, seen, _, _) = asker(yes(), LIMIT);
    let (first, again) = (bash("touch asked.txt", "u1"), bash("touch asked.txt", "u2"));
    assert!(matches!(asker.decide(&policy(), &first), GateAnswer::Allow { .. }));
    assert_eq!(asker.decide(&policy(), &again), GateAnswer::Allow { updated_input: again.input.clone() });
    assert_eq!(questions(&seen).len(), 1, "asked once (J2 B)");
    let replies: Vec<Reply> = asker.answers().into_iter().map(|a| a.reply).collect();
    assert_eq!(replies, [Reply::Yes { fingerprint: fingerprint(&first) }, Reply::Reused]);
}

#[test]
fn a_request_that_differs_by_one_character_asks_again() {
    let (asker, seen, _, _) = asker(yes(), LIMIT);
    assert!(matches!(asker.decide(&policy(), &bash("touch a.txt", "u1")), GateAnswer::Allow { .. }));
    assert!(matches!(asker.decide(&policy(), &bash("touch b.txt", "u2")), GateAnswer::Allow { .. }));
    assert_eq!(questions(&seen).len(), 2);
    assert_ne!(fingerprint(&bash("touch a.txt", "u1")), fingerprint(&bash("touch b.txt", "u1")));
    assert_eq!(
        fingerprint(&bash("touch a.txt", "u1")),
        fingerprint(&bash("touch a.txt", "u2")),
        "the call id is not part of it"
    );
}

#[test]
fn the_checkers_allow_and_deny_never_ask() {
    let (asker, seen, _, _) = asker(yes(), LIMIT);
    assert!(matches!(asker.decide(&policy(), &bash("cargo test", "u1")), GateAnswer::Allow { .. }));
    assert!(matches!(asker.decide(&policy(), &bash("sudo make install", "u2")), GateAnswer::Deny { .. }));
    let write = GateRequest {
        tool_name: "Write".into(),
        input: json!({"file_path": "/tmp/x", "content": "made up"}),
        ..bash("", "u3")
    };
    assert!(matches!(asker.decide(&policy(), &write), GateAnswer::Deny { .. }), "J1: refused without a question");
    assert!(seen.lock().unwrap().is_empty(), "nothing was asked");
    assert!(asker.answers().is_empty());
}

/// Task 4's review: a yes must cover exactly the request on screen, so a request Steplet cannot show exactly is
/// refused without a question: one that holds a character that does not show (here an escape in the request's
/// description, which the checker does not read), or the receiver's key, which the screen would have to hide.
#[test]
fn a_request_steplet_cannot_show_exactly_is_refused_without_asking() {
    let (asker, seen, _, _) = asker(yes(), LIMIT);
    let mut escape = bash("touch made-up.txt", "u1");
    escape.input["description"] = "made up \u{1b}[31mtest".into();
    let key = bash(&format!("touch {MADE_UP_KEY}"), "u2");
    // A command with such a character is already refused by the checker; a file's content it does not read.
    let space = GateRequest {
        tool_name: "Write".into(),
        input: json!({"file_path": format!("{WORK}/made-up.txt"), "content": "made up\u{a0}text"}),
        ..bash("", "u3")
    };
    for req in [&escape, &key, &space] {
        let answer = asker.decide(&policy(), req);
        assert!(
            matches!(&answer, GateAnswer::Deny { message } if message.contains("cannot show")),
            "{:?}: {answer:?}",
            req.input
        );
    }
    assert!(questions(&seen).is_empty(), "none of them was asked");
    // The same request without them is asked, as the control.
    assert!(matches!(asker.decide(&policy(), &bash("touch made-up.txt", "u4")), GateAnswer::Allow { .. }));
    assert_eq!(questions(&seen).len(), 1);
}

#[test]
fn two_questions_at_once_each_get_their_own_answer() {
    let (asker, seen, _, answers) = asker(Box::new(|_| None), Duration::from_secs(10));
    let asker = Arc::new(asker);
    let ask = |command: &'static str, id: &'static str| {
        let asker = asker.clone();
        thread::spawn(move || asker.decide(&policy(), &bash(command, id)))
    };
    let (a, b) = (ask("touch a.txt", "u1"), ask("touch b.txt", "u2"));
    let start = Instant::now();
    while questions(&seen).len() < 2 {
        assert!(start.elapsed() < Duration::from_secs(5), "both questions came");
        thread::sleep(Duration::from_millis(10));
    }
    let qs = questions(&seen);
    let of = |what: &str| qs.iter().find(|q| q.what == what).unwrap().clone();
    let (qa, qb) = (of("touch a.txt"), of("touch b.txt"));
    // Answered in reverse order: the second question first.
    assert!(answers.give(qb.id, Reply::No));
    assert!(answers.give(qa.id, Reply::Yes { fingerprint: qa.fingerprint.clone() }));
    assert!(matches!(a.join().unwrap(), GateAnswer::Allow { .. }));
    assert!(matches!(b.join().unwrap(), GateAnswer::Deny { .. }));
    assert!(!answers.give(qa.id, Reply::No), "an answered question takes no second answer");
}

#[test]
fn run_live_refuses_an_ask_limit_the_gate_deadline_cannot_cover() {
    let live = Arc::new(Live {
        on_progress: Box::new(|_| {}),
        stop: StopSignal::new(),
        answers: Some(Answers::default()),
        ask_limit: Duration::from_secs(300),
    });
    let req = RunRequest {
        steplet: steplet(),
        repo_url: "https://github.com/octocat/Hello-World.git".into(),
        commit: "7fd1a60b01f91b314f59955a4e4d4e80d8edf11d".into(),
        task_text: "made up".into(),
        max_turns: 1,
        deadline: Duration::from_secs(60),
    };
    // A made-up distribution: nothing may reach WSL.
    let paths = RunPaths::new("{00000000-0000-0000-0000-000000000000}", "p4t4x").unwrap();
    let setup = RunSetup {
        gate_exe: "/opt/steplet/gate/steplet-gate.exe".into(),
        logs: std::env::temp_dir(),
        key: None,
        decision_deadline: Duration::from_secs(310),
        model_url: None,
    };
    let got = run_live(&req, &paths, &policy(), &setup, live);
    assert!(matches!(got, Err(RunError::Limits)), "{got:?}");
}
