//! Steplet's own agent loop (the Nebius plan, Task 6), with a stand-in model and stand-in hands: no Linux, no Nebius,
//! nothing spent. Each test drives one run with `drive`, as `run_agent_locked` does, and reads what happened from the
//! run's record, the report, the progress the screen was sent, and Steplet's own log of the run.

mod support;

use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

use serde_json::{Value, json};
use steplet_core::canon::canonicalize;
use steplet_core::header::parse;
use steplet_core::policy::Policy;
use steplet_run::{
    Agent, AgentLimits, Asked, Asker, Check, Done, Event, Finish, GateAnswer, GateRequest, Hands, HandsError, Limit,
    Live, MAX_TOKENS, Message, ModelError, ModelReply, Move, Nanodollars, PER_MOVE, Progress, Report, Role, RunOutcome,
    Scripted, Seen, Step, StopSignal, ToolCall, Trust, Usage, build_report, chat_body, drive, price, specs,
    system_prompt, task_message, worst_case,
};

/// A model Steplet has a price for: $0.30 per million tokens in, $0.90 out, so one stand-in answer (1000 in, 50 out)
/// costs 345 000 nanodollars.
const MODEL: &str = "nvidia/nemotron-3-super-120b-a12b";
const ONE_ANSWER: u64 = 1000 * 300 + 50 * 900;
/// A made-up Nebius key: the text no report, screen or log may show.
const KEY: &str = "v1.STEPLET-TEST-MADE-UP-NEBIUS-KEY-NOT-REAL";
const WORK: &str = "/work/r1/repo";
/// The file's task, as every run here gets it.
const TASK: &str = "Run the tests.";

/// A file that allows only `cargo test` as a command.
fn policy() -> Policy {
    let f = "---\nsteplet: 1\ntitle: t\nsender: s\nexpires: 2026-12-31\nsteps:\n  - a\nallow:\n  commands:\n    - cargo test\n---\n";
    Policy::from_steplet(&parse(&canonicalize(f.as_bytes()).unwrap()).unwrap(), WORK).unwrap()
}

/// What the stand-in hands do with a move, given its time limit and the Stop button.
type Act = Box<dyn FnMut(&Move, Duration, &StopSignal) -> Result<Done, HandsError> + Send>;

/// The stand-in hands: they record each move and the time it was given, then do what the test says.
struct Fake {
    moves: Arc<Mutex<Vec<(Move, Duration)>>>,
    act: Act,
}

impl Hands for Fake {
    fn run(&mut self, mv: &Move, limit: Duration, stop: &StopSignal) -> Result<Done, HandsError> {
        self.moves.lock().unwrap().push((mv.clone(), limit));
        (self.act)(mv, limit, stop)
    }
}

/// A move that ended on its own with `exit`, having printed `output`.
fn printed(exit: i32, output: &str) -> Done {
    Done { exit_code: Some(exit), timed_out: false, output: output.into(), total_bytes: output.len() as u64 }
}

/// An answer from the stand-in model, priced as `Scripted`'s own.
fn answer(text: Option<&str>, calls: Vec<ToolCall>) -> Step {
    Step::Answer(ModelReply {
        text: text.map(str::to_owned),
        finish: if calls.is_empty() { Finish::Stop } else { Finish::ToolCalls },
        calls,
        usage: Usage { prompt_tokens: 1000, completion_tokens: 50 },
        model: "scripted".into(),
        reasoning: None,
    })
}

/// A tool call with its arguments exactly as written.
fn call(id: &str, name: &str, arguments: &str) -> ToolCall {
    ToolCall { id: id.into(), name: name.into(), arguments: arguments.into() }
}

fn run_tests(id: &str) -> Step {
    Scripted::call(id, "run_command", json!({"command": "cargo test"}))
}

/// One run, before it starts.
struct Run {
    steps: Vec<Step>,
    limits: AgentLimits,
    act: Act,
    stop: StopSignal,
}

/// One run, as the test reads it afterwards.
struct Ran {
    outcome: RunOutcome,
    report: Report,
    /// What the model was asked, answer by answer.
    seen: Vec<Seen>,
    moves: Vec<(Move, Duration)>,
    progress: Vec<Progress>,
    log: String,
    took: Duration,
}

impl Run {
    /// A run of the model's `steps` within roomy limits, whose every move prints "ok" and exits 0.
    fn new(steps: Vec<Step>) -> Run {
        Run {
            steps,
            limits: AgentLimits {
                cap: Nanodollars(250_000_000),
                max_steps: 20,
                deadline: Instant::now() + Duration::from_secs(60),
                per_move: PER_MOVE,
                retry_waits: vec![Duration::from_millis(10); 3],
            },
            act: Box::new(|_, _, _| Ok(printed(0, "ok\n"))),
            stop: StopSignal::new(),
        }
    }

    fn hands(
        mut self,
        act: impl FnMut(&Move, Duration, &StopSignal) -> Result<Done, HandsError> + Send + 'static,
    ) -> Run {
        self.act = Box::new(act);
        self
    }

    fn go(self) -> Ran {
        let model = Scripted::new(MODEL, self.steps);
        let seen = model.seen();
        let progress = Arc::new(Mutex::new(Vec::new()));
        let shown = Arc::clone(&progress);
        let live = Arc::new(Live {
            on_progress: Box::new(move |p| shown.lock().unwrap().push(p)),
            stop: self.stop,
            answers: None,
            ask_limit: Duration::ZERO,
        });
        let secrets = [KEY];
        let checker = support::checker_as_written(policy());
        let asker = Asker::new(Arc::clone(&live), &secrets);
        let moves = Arc::new(Mutex::new(Vec::new()));
        let mut hands = Fake { moves: Arc::clone(&moves), act: self.act };
        let mut log = Vec::new();
        let started = Instant::now();
        let outcome = drive(
            Agent {
                model: Box::new(model),
                hands: &mut hands,
                checker: &checker,
                asker: &asker,
                live: &live,
                limits: self.limits,
                log: &mut log,
                secrets: &secrets,
            },
            TASK,
            WORK,
        );
        let took = started.elapsed();
        let report = build_report(&outcome, None, Trust::NotRecorded, &secrets);
        let seen = seen.lock().unwrap().clone();
        let moves = moves.lock().unwrap().clone();
        let progress = progress.lock().unwrap().clone();
        Ran { outcome, report, seen, moves, progress, log: String::from_utf8(log).unwrap(), took }
    }
}

impl Ran {
    /// What the model was told about call `id`.
    fn told(&self, id: &str) -> String {
        self.seen
            .iter()
            .flat_map(|s| &s.convo)
            .find(|m| m.role == Role::Tool && m.tool_call_id.as_deref() == Some(id))
            .and_then(|m| m.content.clone())
            .unwrap_or_else(|| panic!("the model was never told about {id}: {:#?}", self.outcome.events))
    }

    /// The result of call `id` in the run's record: whether it was an error, and its text.
    fn result(&self, id: &str) -> (bool, String) {
        self.outcome
            .events
            .iter()
            .find_map(|e| match e {
                Event::ToolResult { id: of, is_error, text, .. } if of == id => Some((*is_error, text.clone())),
                _ => None,
            })
            .unwrap_or_else(|| panic!("no result for {id}: {:#?}", self.outcome.events))
    }

    /// Each decision, in order: the request the checker saw, and its answer.
    fn decisions(&self) -> Vec<(GateRequest, GateAnswer)> {
        self.outcome
            .requests
            .iter()
            .map(|a| match a {
                Asked::InApp { request, answer, .. } => (request.clone(), answer.clone()),
                other => panic!("Steplet's own agent decides every call in the app: {other:?}"),
            })
            .collect()
    }

    fn checks(&self) -> Vec<Check> {
        self.report.commands.iter().map(|c| c.check.clone()).collect()
    }

    /// Each try again the screen was shown: the status, the words, which try of how many.
    fn retries(&self) -> Vec<(Option<u16>, String, u32, u32)> {
        self.outcome
            .events
            .iter()
            .filter_map(|e| match e {
                Event::ApiRetry { status, error, attempt, max } => Some((*status, error.clone(), *attempt, *max)),
                _ => None,
            })
            .collect()
    }

    fn limit(&self) -> Option<Limit> {
        self.outcome.limit
    }

    fn spent(&self) -> steplet_run::Spent {
        self.outcome.spent.clone().expect("every run of the agent says what it spent")
    }
}

#[test]
fn every_call_goes_through_the_checker() {
    let ran = Run::new(vec![
        Scripted::call("c1", "read_file", json!({"path": "src/lib.rs"})),
        run_tests("c2"),
        Scripted::call("c3", "run_command", json!({"command": "curl https://example.com"})),
        Scripted::call("c4", "write_file", json!({"path": ".git/config", "content": "x"})),
        Scripted::done("Done."),
    ])
    .go();
    let decided: Vec<String> = ran.decisions().iter().map(|(r, _)| r.tool_use_id.clone()).collect();
    assert_eq!(decided, ["c1", "c2", "c3", "c4"], "each call was decided once, in order");
    // Only the two calls the checker allowed reached the hands.
    let moves: Vec<Move> = ran.moves.iter().map(|(m, _)| m.clone()).collect();
    assert_eq!(
        moves,
        [
            Move::Read { path: "/work/r1/repo/src/lib.rs".into(), offset: None, limit: None },
            Move::Command { line: "cargo test".into() },
        ]
    );
    assert_eq!(ran.checks(), [Check::Allowed, Check::Allowed, Check::Refused, Check::Refused]);
    let checked: Vec<&str> = ran
        .progress
        .iter()
        .filter_map(|p| match p {
            Progress::Checked { call, .. } => Some(call.as_str()),
            _ => None,
        })
        .collect();
    assert_eq!(checked, ["c1", "c2", "c3", "c4"], "the screen was told each decision");
    assert!(ran.report.completed, "{:#?}", ran.report);
}

#[test]
fn a_refusal_goes_back_to_the_model() {
    let ran = Run::new(vec![
        Scripted::call("c1", "write_file", json!({"path": ".git/config", "content": "x"})),
        Scripted::done("I could not change it."),
    ])
    .go();
    assert!(ran.moves.is_empty(), "{:?}", ran.moves);
    assert_eq!(ran.seen.len(), 2, "the model was asked again after the refusal");
    let told = ran.told("c1");
    assert!(told.starts_with("Steplet refused this: "), "{told}");
    assert!(told.ends_with(". It will not run. Carry on another way, or finish."), "{told}");
    assert!(!told.contains(".."), "one full stop between the reason and what follows: {told}");
    assert_eq!(ran.result("c1"), (true, told));
    assert_eq!(ran.checks(), [Check::Refused]);
}

#[test]
fn what_runs_is_what_was_checked() {
    let ran = Run::new(vec![
        Scripted::call("c1", "read_file", json!({"path": "src/lib.rs", "offset": 3, "limit": 10})),
        Scripted::call("c2", "search_text", json!({"path": "src", "pattern": "fn main", "glob": "*.rs"})),
        Scripted::done("Done."),
    ])
    .go();
    let decisions = ran.decisions();
    assert_eq!(decisions[0].0.tool_name, "Read");
    assert_eq!(decisions[0].0.input, json!({"file_path": "/work/r1/repo/src/lib.rs", "offset": 3, "limit": 10}));
    assert_eq!(decisions[1].0.tool_name, "Grep");
    assert_eq!(decisions[1].0.input, json!({"path": "/work/r1/repo/src", "pattern": "fn main", "glob": "*.rs"}));
    // The run's record and the screen show the request the checker saw, not the model's own arguments.
    let uses: Vec<(String, Value)> = ran
        .outcome
        .events
        .iter()
        .filter_map(|e| match e {
            Event::ToolUse { name, input, .. } => Some((name.clone(), input.clone())),
            _ => None,
        })
        .collect();
    let checked: Vec<(String, Value)> = decisions.iter().map(|(r, _)| (r.tool_name.clone(), r.input.clone())).collect();
    assert_eq!(uses, checked);
    let moves: Vec<Move> = ran.moves.iter().map(|(m, _)| m.clone()).collect();
    assert_eq!(
        moves,
        [
            Move::Read { path: "/work/r1/repo/src/lib.rs".into(), offset: Some(3), limit: Some(10) },
            Move::Search { path: "/work/r1/repo/src".into(), regex: "fn main".into(), glob: Some("*.rs".into()) },
        ]
    );
}

#[test]
fn a_malformed_tool_call_is_answered_not_run() {
    let ran = Run::new(vec![
        answer(None, vec![call("c1", "run_command", "{not json"), call("c2", "delete_everything", "{}")]),
        Scripted::done("Done."),
    ])
    .go();
    assert!(ran.moves.is_empty(), "{:?}", ran.moves);
    let not_json = ran.told("c1");
    assert!(not_json.starts_with("Steplet could not read this call: The arguments are not JSON: "), "{not_json}");
    assert!(
        not_json.ends_with(" Nothing ran. Call a tool again with the arguments it takes, or finish."),
        "{not_json}"
    );
    let no_such_tool = ran.told("c2");
    assert!(
        no_such_tool.starts_with("Steplet could not read this call: There is no tool named \"delete_everything\"."),
        "{no_such_tool}"
    );
    assert_eq!(ran.checks(), [Check::Refused, Check::Refused]);
    assert!(
        ran.progress.iter().any(|p| matches!(p, Progress::Checked { call, check: Check::Refused, .. } if call == "c1")),
        "the screen was told"
    );
    assert!(ran.report.completed, "{:#?}", ran.report);
}

#[test]
fn three_unreadable_replies_in_a_row_end_the_run() {
    let unreadable = |why: &str| Step::Fail(ModelError::BadReply(why.into()));
    let ran = Run::new(vec![unreadable("one"), unreadable("two"), unreadable("three")]).go();
    assert_eq!(
        ran.outcome.faults,
        [
            "Nebius sent 3 answers in a row that Steplet could not read, so the run ended. The last: Nebius's answer could \
          not be read: three"
        ]
    );
    assert_eq!(ran.seen.len(), 3);
    let spent = ran.spent();
    assert_eq!((spent.calls, spent.cost), (0, Nanodollars(0)));
    assert!(spent.unsure > Nanodollars(0), "each failed call is counted at the most it could have cost");
    assert!(!ran.report.completed);

    // Two, then an answer: the run goes on.
    let ran = Run::new(vec![unreadable("one"), unreadable("two"), Scripted::done("Done.")]).go();
    assert!(ran.outcome.faults.is_empty(), "{:?}", ran.outcome.faults);
    assert!(ran.report.completed, "{:#?}", ran.report);
}

#[test]
fn the_step_limit_ends_the_run() {
    let mut run = Run::new((1..=5).map(|n| run_tests(&format!("c{n}"))).chain([Scripted::done("Done.")]).collect());
    run.limits.max_steps = 2;
    let ran = run.go();
    assert_eq!(ran.limit(), Some(Limit::Steps(2)));
    assert_eq!(ran.seen.len(), 2, "the model gave no more than 2 answers");
    assert_eq!(ran.moves.len(), 2);
    assert!(ran.outcome.faults.is_empty(), "a limit is not a fault: {:?}", ran.outcome.faults);
    assert!(!ran.report.completed);
}

#[test]
fn the_time_limit_ends_the_run() {
    let mut run = Run::new(vec![run_tests("c1"), run_tests("c2"), Scripted::done("Done.")]).hands(|_, limit, _| {
        thread::sleep(limit.min(Duration::from_secs(2)));
        Ok(Done { exit_code: None, timed_out: true, output: String::new(), total_bytes: 0 })
    });
    run.limits.deadline = Instant::now() + Duration::from_millis(300);
    let ran = run.go();
    assert_eq!(ran.limit(), Some(Limit::Time));
    assert!(ran.took < Duration::from_secs(3), "took {:?}", ran.took);
    assert_eq!(ran.moves.len(), 1, "no move after the time was up");
    assert!(
        ran.moves[0].1 <= Duration::from_millis(300),
        "the move had only the run's time left: {:?}",
        ran.moves[0].1
    );
    let (is_error, text) = ran.result("c1");
    assert!(is_error && text.starts_with("Steplet ended this at its time limit of "), "{text}");
    assert_eq!(ran.seen.len(), 1);
    assert!(!ran.report.completed);
}

#[test]
fn the_spending_cap_ends_the_run_before_the_call_that_could_pass_it() {
    // Enough for what one answer really costs, but not for the most it could cost: it is never asked.
    let mut run = Run::new(vec![Scripted::done("Done.")]);
    run.limits.cap = Nanodollars(ONE_ANSWER);
    let ran = run.go();
    assert!(ran.seen.is_empty(), "the model was called: {:?}", ran.seen);
    assert_eq!(ran.limit(), Some(Limit::Spending { spent: Nanodollars(0), cap: Nanodollars(ONE_ANSWER) }));
    assert!(ran.outcome.faults.is_empty(), "a limit is not a fault: {:?}", ran.outcome.faults);

    // Room for the most the first call could cost, and no more: exactly one call is made, since the second's most
    // (on top of what the first really cost, with a longer conversation) would pass the cap.
    let first = [Message::system(&system_prompt(WORK)), Message::user(&task_message(TASK))];
    let most = worst_case(chat_body(MODEL, &first, &specs(), MAX_TOKENS).len(), MAX_TOKENS, &price(MODEL).unwrap());
    let mut run = Run::new(vec![run_tests("c1"), run_tests("c2"), Scripted::done("Done.")]);
    run.limits.cap = most;
    let ran = run.go();
    assert_eq!(ran.seen.len(), 1, "exactly one model call");
    assert_eq!(ran.seen[0].convo, first, "the call measured is the call made");
    assert_eq!(ran.limit(), Some(Limit::Spending { spent: Nanodollars(ONE_ANSWER), cap: most }));
    assert_eq!(ran.moves.len(), 1, "the move the one answer asked for ran");

    // With room for a few: the run ends at the cap, with every call it made answered and counted, and never past it.
    let mut run = Run::new((1..=80).map(|n| run_tests(&format!("c{n}"))).collect());
    run.limits.cap = Nanodollars(20_000_000);
    run.limits.max_steps = 100;
    let ran = run.go();
    let spent = ran.spent();
    assert!(matches!(ran.limit(), Some(Limit::Spending { .. })), "{:?}", ran.limit());
    assert!(ran.seen.len() >= 2, "{}", ran.seen.len());
    assert_eq!(ran.seen.len(), spent.calls as usize, "no call was made that the cap had not room for");
    assert_eq!(spent.cost, Nanodollars(ONE_ANSWER * u64::from(spent.calls)));
    assert!(spent.cost + spent.unsure <= Nanodollars(20_000_000), "{spent:?}");
    let last_shown = ran.progress.iter().rev().find_map(|p| match p {
        Progress::Spent(s) => Some(s.clone()),
        _ => None,
    });
    assert_eq!(last_shown, Some(spent), "the screen was shown the spending after each answer");
}

#[test]
fn stop_during_a_model_call_ends_the_run_promptly() {
    let run = Run::new(vec![Step::Hang(Duration::from_secs(30))]);
    let stop = run.stop.clone();
    let presser = thread::spawn(move || {
        thread::sleep(Duration::from_millis(200));
        stop.stop();
    });
    let ran = run.go();
    presser.join().unwrap();
    assert!(ran.took < Duration::from_secs(5), "took {:?}", ran.took);
    assert!(ran.outcome.stopped_by_you);
    assert!(ran.outcome.faults.is_empty(), "{:?}", ran.outcome.faults);
    assert!(ran.spent().unsure > Nanodollars(0), "the call cut off is counted at the most it could have cost");

    // During a move: the hands end it, and the model is not asked again.
    let run = Run::new(vec![run_tests("c1"), Scripted::done("Done.")]).hands(|_, _, stop| {
        let until = Instant::now() + Duration::from_secs(10);
        while !stop.is_set() && Instant::now() < until {
            thread::sleep(Duration::from_millis(10));
        }
        Err(HandsError::Stopped)
    });
    let stop = run.stop.clone();
    let presser = thread::spawn(move || {
        thread::sleep(Duration::from_millis(200));
        stop.stop();
    });
    let ran = run.go();
    presser.join().unwrap();
    assert!(ran.took < Duration::from_secs(5), "took {:?}", ran.took);
    assert!(ran.outcome.stopped_by_you);
    assert_eq!(ran.seen.len(), 1);
    assert!(ran.outcome.faults.is_empty(), "{:?}", ran.outcome.faults);
}

#[test]
fn busy_nebius_is_retried_and_shown() {
    let busy = || Step::Fail(ModelError::Busy { status: 429, detail: "slow down".into(), retry_after: None });
    let ran = Run::new(vec![
        busy(),
        Step::Fail(ModelError::Unavailable { status: 503, detail: "down".into() }),
        Scripted::done("Done."),
    ])
    .go();
    assert_eq!(
        ran.retries(),
        [
            (Some(429), "Nebius is busy (status 429).".to_string(), 1, 3),
            (Some(503), "Nebius had a problem on its side (status 503).".to_string(), 2, 3),
        ]
    );
    assert_eq!(ran.seen.len(), 3);
    assert!(ran.report.completed, "{:#?}", ran.report);

    // Busy every time: three tries more, then the run ends, in plain words.
    let ran = Run::new(vec![busy(), busy(), busy(), busy(), Scripted::done("Done.")]).go();
    assert_eq!(ran.retries().len(), 3);
    assert_eq!(ran.seen.len(), 4);
    assert_eq!(
        ran.outcome.faults,
        ["Nebius is busy (status 429). Steplet tried again 3 times, then ended the run. It may work if you run it \
          again later."]
    );
    assert!(!ran.report.completed);
}

#[test]
fn a_refused_key_ends_the_run_with_plain_words() {
    let ran = Run::new(vec![
        Step::Fail(ModelError::KeyRefused { status: 401, detail: "invalid key".into() }),
        Scripted::done("Done."),
    ])
    .go();
    assert_eq!(ran.outcome.faults, ["Nebius did not accept the key. Change it in Setup."]);
    assert_eq!(ran.seen.len(), 1, "a refused key is not tried again");
    assert!(ran.retries().is_empty());
    assert!(!ran.report.completed);

    let ran = Run::new(vec![Step::Fail(ModelError::NoCredit { status: 402, detail: "no credit".into() })]).go();
    assert_eq!(ran.outcome.faults, ["Your Nebius account has no credit left. Add credit at tokenfactory.nebius.com."]);
    assert_eq!(ran.seen.len(), 1);
}

#[test]
fn the_report_shows_steplets_exit_code_not_the_models_claim() {
    let ran = Run::new(vec![run_tests("c1"), Scripted::done("All 12 tests pass.")])
        .hands(|_, _, _| Ok(printed(1, "test result: FAILED. 0 passed; 12 failed\n")))
        .go();
    let command = &ran.report.commands[0];
    assert_eq!(command.command, "cargo test");
    assert_eq!(command.exit_code, Some(1));
    assert_eq!(command.ended_in_error, Some(true));
    assert!(command.output.starts_with("Exit code 1\n"), "{}", command.output);
    // The model's claim is shown only as what it says.
    assert_eq!(ran.report.agent_says, "All 12 tests pass.");
}

#[test]
fn huge_output_is_cut_for_the_model() {
    let mut n = 0;
    let ran = Run::new(vec![run_tests("c1"), run_tests("c2"), Scripted::done("Done.")])
        .hands(move |_, _, _| {
            n += 1;
            Ok(match n {
                // 5 MiB printed, of which Steplet's Linux kept the first 1 MiB.
                1 => Done { exit_code: Some(0), timed_out: false, output: "x".repeat(1 << 20), total_bytes: 5 << 20 },
                _ => printed(0, &format!("{}{}", "a".repeat(10_000), "z".repeat(10_000))),
            })
        })
        .go();
    let first = ran.told("c1");
    assert!(first.len() <= 8704, "{} bytes", first.len());
    assert!(first.starts_with("Exit code 0\nxxxx"), "{}", &first[..20]);
    assert!(first.ends_with(&format!("[{} more bytes not shown]", (5 << 20) - 4096)), "{}", &first[first.len() - 40..]);
    let second = ran.told("c2");
    assert!(second.len() <= 8704, "{} bytes", second.len());
    assert!(second.ends_with(&"z".repeat(4096)), "the end of what it printed is kept");
    assert!(second.contains(&format!("[{} bytes cut]", 20_000 - 2 * 4096)), "the cut is said");
}

#[test]
fn the_nebius_key_never_appears_in_the_report_or_logs() {
    let ran = Run::new(vec![
        answer(
            Some(&format!("I will use {KEY} now.")),
            vec![call("c1", "run_command", r#"{"command": "cargo test"}"#)],
        ),
        Scripted::call("c2", "run_command", json!({"command": format!("echo {KEY}")})),
        Scripted::done(&format!("Finished; the key was {KEY}.")),
    ])
    .hands(|_, _, _| Ok(printed(0, &format!("{KEY}\n"))))
    .go();
    // The key really was in what the model wrote and the move printed, so the test can fail.
    assert!(format!("{:?}", ran.outcome.events).contains(KEY));
    assert!(!format!("{:?}", ran.report).contains(KEY), "in the report: {:#?}", ran.report);
    assert!(!format!("{:?}", ran.progress).contains(KEY), "on the screen: {:#?}", ran.progress);
    assert!(!ran.log.is_empty(), "Steplet kept its own log of the run");
    assert!(!ran.log.contains(KEY), "in the log: {}", ran.log);
}
