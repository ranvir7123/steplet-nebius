//! Steplet's own agent (the Nebius plan, Task 6): the model proposes each move, Steplet's checker decides it, and the
//! fence runs it. The model is called from Windows, so the Nebius key never enters Steplet's Linux; only the moves the
//! checker allowed go in, one at a time, through `Hands`, built from the request the checker saw. The run ends when
//! the model answers without a move, at the step limit, the time limit or the spending cap, at the receiver's Stop, or
//! when the model's service fails in a way trying again would not mend. What happened is told in the same events as a
//! Claude Code run, so one report serves both runners.

use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::mpsc::{self, Receiver, RecvTimeoutError, Sender};
use std::thread;
use std::time::{Duration, Instant};

use serde_json::{Value, json};
use steplet_core::policy::Policy;

use crate::ask::Asker;
use crate::decide::Checker;
use crate::gate::Asked;
use crate::hands::{Done, Hands, HandsError, LinuxHands, Move};
use crate::live::{Live, Progress, StopSignal};
use crate::lock::RunLock;
use crate::model::{Finish, Message, Model, ModelError, ModelReply, ToolCall, ToolSpec, chat_body};
use crate::places::LinuxPaths;
use crate::prepare::{changes, discard, prepare_own};
use crate::price::{Nanodollars, Price, cost, price, worst_case};
use crate::report::{Check, Report, Scrubber, Trust, build_report, raw_log_line};
use crate::run::{RunError, RunOutcome, pin_listed_sites, prepare_until_stopped};
use crate::tools::{specs, system_prompt, task_message, to_move, to_request};
use crate::types::{Event, GateAnswer, GateRequest, RunPaths, RunRequest};

/// The most the model may write in one answer.
pub const MAX_TOKENS: u32 = 4096;
/// The longest one move may run (N4).
pub const PER_MOVE: Duration = Duration::from_secs(10 * 60);
/// The waits before each new try of a model call the service may answer later: three tries more, after 2, 5 and 10 s.
pub const RETRY_WAITS: [Duration; 3] = [Duration::from_secs(2), Duration::from_secs(5), Duration::from_secs(10)];
/// The longest wait Steplet keeps to when the service names its own.
const LONGEST_WAIT: Duration = Duration::from_secs(60);
/// How many answers in a row Steplet could not read end the run.
const UNREADABLE_IN_A_ROW: u32 = 3;
/// How often Steplet looks at the Stop button and the time limit while it waits for the model.
const POLL: Duration = Duration::from_millis(100);
/// How much of a move's output the model gets: the first and the last this many bytes.
const SHOWN_END: usize = 4096;
/// What the model is told when its answer was cut at `MAX_TOKENS` before it asked for a move.
const CUT_SHORT: &str = "Your answer was cut short at the length limit. Carry on, in shorter steps.";

/// What Steplet's own agent needs besides the request, its paths and the checker.
pub struct AgentSetup {
    /// The model, with its key: `TokenFactory` in the app, a stand-in in the tests.
    pub model: Box<dyn Model>,
    /// The most the run may spend on the model.
    pub cap: Nanodollars,
    /// The most answers the model may give.
    pub max_steps: u32,
    /// Steplet's logs folder on Windows, for the run's own record, `<run>.agent.jsonl`.
    pub logs: PathBuf,
    /// The texts no report, live view or log may show: the Nebius key.
    pub secrets: Vec<String>,
}

/// The run's limits, as `drive` keeps to them.
#[derive(Clone, Debug)]
pub struct AgentLimits {
    pub cap: Nanodollars,
    pub max_steps: u32,
    /// When the run's time is up.
    pub deadline: Instant,
    /// The longest one move may run.
    pub per_move: Duration,
    /// The waits before each new try of a model call (`RETRY_WAITS` in a real run); how many there are is how many
    /// times Steplet tries again.
    pub retry_waits: Vec<Duration>,
}

/// Which of the run's limits ended it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Limit {
    /// The model had given this many answers.
    Steps(u32),
    /// The run's time was up.
    Time,
    /// The next model call could have taken spending past `cap`. `spent` counts the calls that may have cost
    /// something without saying what at the most they could have cost (`Spent::unsure`).
    Spending { spent: Nanodollars, cap: Nanodollars },
}

/// What the run has spent on the model.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Spent {
    /// The model Steplet asked for.
    pub model: String,
    /// Tokens sent and received, as the service counted them.
    pub tokens_in: u64,
    pub tokens_out: u64,
    /// What the answers cost, from the service's own counts.
    pub cost: Nanodollars,
    /// Calls that failed after the service may have charged for them (no answer in time, the connection lost, an
    /// answer that could not be read), each counted at the most it could have cost.
    pub unsure: Nanodollars,
    pub cap: Nanodollars,
    /// How many answers came.
    pub calls: u32,
    /// The most answers the model may give (the step limit).
    pub max_steps: u32,
}

/// Runs `req` as run `paths` with Steplet's own agent, every move decided by `policy`, and returns the receiver's
/// report. The caller holds the one-run lock, as for `run_live_locked`. Each piece of progress goes to `live`, cleaned
/// as the report is, and `live.stop` ends the run at once, the model's call and the move under way included. A model
/// Steplet has no price for is never called (`RunError::NoPrice`), since its spending could not be capped.
pub fn run_agent_locked(
    _held: &RunLock,
    req: &RunRequest,
    paths: &RunPaths,
    policy: &Policy,
    setup: AgentSetup,
    live: Arc<Live>,
) -> Result<Report, RunError> {
    let AgentSetup { model, cap, max_steps, logs, secrets } = setup;
    if price(model.id()).is_none() {
        return Err(RunError::NoPrice(model.id().into()));
    }
    let secrets: Vec<&str> = secrets.iter().map(String::as_str).collect();
    // The moves reach only the declared sites, pinned as for a Claude Code run (K4 B).
    let mut pinned = pin_listed_sites(req, paths.distro_id())?;
    // Steplet's own looks after the run, and its record, do not change whether the run completed.
    let mut after = Vec::new();
    let path = logs.join(format!("{}.agent.jsonl", paths.run_id()));
    let mut log: Box<dyn Write> = match File::create(&path) {
        Ok(file) => Box::new(BufWriter::new(file)),
        Err(e) => {
            after.push(format!("Steplet could not keep the run's own record in {}: {e}", path.display()));
            Box::new(std::io::sink())
        }
    };
    // Each path of the file tools is decided where it really leads in Steplet's Linux (row 30).
    let checker = Checker::new(policy.clone(), Arc::new(LinuxPaths::new(paths.distro_id())));
    let asker = Asker::new(live.clone(), &secrets);
    let outcome = match prepare_until_stopped(paths, &live, || prepare_own(req, paths)) {
        // A Stop while preparing ends what preparing started, so preparing fails: the receiver's Stop, not a fault.
        _ if live.stop.is_set() => {
            RunOutcome { stopped_by_you: true, stopped_at: Some(0), ..RunOutcome::new(Vec::new()) }
        }
        Ok(prepared) => {
            (live.on_progress)(Progress::Prepared(prepared));
            let mut hands = LinuxHands::new(paths);
            let limits = AgentLimits {
                cap,
                max_steps,
                deadline: Instant::now() + req.deadline,
                per_move: PER_MOVE,
                retry_waits: RETRY_WAITS.to_vec(),
            };
            let agent = Agent {
                model,
                hands: &mut hands,
                checker: &checker,
                asker: &asker,
                live: &live,
                limits,
                log: &mut log,
                secrets: &secrets,
            };
            drive(agent, &req.task_text, paths.work_repo())
        }
        Err(e) => {
            let _ = discard(paths);
            return Err(RunError::Prepare(e));
        }
    };
    if let Err(e) = log.flush() {
        after.push(format!("Steplet could not finish the run's own record: {e}"));
    }
    // The look-up program in Steplet's Linux ends here, before Steplet's own looks and the removal.
    drop(checker);
    let agent_ran = !outcome.events.is_empty();
    let mut changed = None;
    if agent_ran {
        changed = changes(paths)
            .map_err(|e| after.push(format!("Steplet could not list the files the run changed: {e}")))
            .ok();
    }
    if let Err(e) = discard(paths) {
        after.push(format!("Steplet could not remove the run's folders: {e}"));
    }
    if let Err(e) = pinned.unpin() {
        after.push(format!("Steplet could not remove this run's web-site pins from Steplet's Linux: {e}"));
    }
    // This runner has no Claude Code config for a folder's trust to be recorded in.
    let mut report = build_report(&outcome, changed.as_deref(), Trust::NotApplicable, &secrets);
    let scrub = Scrubber::new(&secrets);
    report.faults.extend(after.iter().map(|fault| scrub.text(fault)));
    Ok(report)
}

/// One run of the agent loop, as `drive` takes it: the model, and what decides and runs its moves.
#[doc(hidden)]
pub struct Agent<'a> {
    pub model: Box<dyn Model>,
    pub hands: &'a mut dyn Hands,
    pub checker: &'a Checker,
    /// Asks the receiver when the checker says "ask", and tells the screen each decision.
    pub asker: &'a Asker,
    pub live: &'a Live,
    pub limits: AgentLimits,
    /// Steplet's own record of the run: a line for each answer, each move and each failed call, secrets hidden.
    pub log: &'a mut dyn Write,
    /// The texts no event shown, and no line of the record, may hold.
    pub secrets: &'a [&'a str],
}

/// The agent loop, for a run whose working copy `work` is ready: the model gets the file's `task`, and every move it
/// asks for is decided by the checker and, when allowed, run by the hands, until the run ends. Ends with every
/// question still on screen taken back. `run_agent_locked` is the way in; the tests drive it with a stand-in model
/// and stand-in hands.
#[doc(hidden)]
pub fn drive(agent: Agent<'_>, task: &str, work: &str) -> RunOutcome {
    let Agent { model, hands, checker, asker, live, limits, log, secrets } = agent;
    let spent = Spent {
        model: model.id().into(),
        tokens_in: 0,
        tokens_out: 0,
        cost: Nanodollars(0),
        unsure: Nanodollars(0),
        cap: limits.cap,
        calls: 0,
        max_steps: limits.max_steps,
    };
    let mut run = Driving {
        hands,
        checker,
        asker,
        live,
        limits,
        log,
        secrets,
        scrub: Scrubber::new(secrets),
        outcome: RunOutcome::new(Vec::new()),
        spent,
        models: Vec::new(),
        log_failed: false,
    };
    match price(model.id()) {
        Some(price) => run.go(model, price, task, work),
        None => run.fault(format!("Steplet has no price for the model {}, so it did not call it.", model.id())),
    }
    asker.take_back_all();
    run.outcome.answers = asker.answers();
    run.outcome.spent = Some(run.spent);
    run.outcome
}

/// A run under way.
struct Driving<'a> {
    hands: &'a mut dyn Hands,
    checker: &'a Checker,
    asker: &'a Asker,
    live: &'a Live,
    limits: AgentLimits,
    log: &'a mut dyn Write,
    secrets: &'a [&'a str],
    scrub: Scrubber,
    outcome: RunOutcome,
    spent: Spent,
    /// The models that answered, as the service named them, each once.
    models: Vec<String>,
    log_failed: bool,
}

impl Driving<'_> {
    fn go(&mut self, model: Box<dyn Model>, price: Price, task: &str, work: &str) {
        let id = model.id().to_string();
        let tools = specs();
        let brain = match Brain::start(model) {
            Ok(brain) => brain,
            Err(e) => return self.fault(format!("Steplet could not start its call to the model: {e}")),
        };
        // The top of the live log shows the model, the cap and the step limit from the start.
        self.tell_spent();
        self.show(Event::Started {
            model: id.clone(),
            tools: tools.iter().map(|t| t.name.to_string()).collect(),
            // Every move is decided inside Steplet before it runs: there is no gate to fail to connect.
            gate_connected: true,
        });
        let mut convo = vec![Message::system(&system_prompt(work)), Message::user(&task_message(task))];
        let mut answers = 0;
        loop {
            if !self.may_go_on() {
                return;
            }
            if answers >= self.limits.max_steps {
                return self.limit(Limit::Steps(answers));
            }
            let Some(reply) = self.ask(&brain, &id, &price, &convo, &tools) else { return };
            answers += 1;
            self.count(&reply, &price);
            let text = reply.text.as_deref().map(str::trim).filter(|t| !t.is_empty()).map(str::to_owned);
            if let Some(text) = &text {
                self.show(Event::AgentText(text.clone()));
            }
            convo.push(Message::assistant(&reply));
            if reply.calls.is_empty() {
                match &reply.finish {
                    Finish::Length => {
                        convo.push(Message::user(CUT_SHORT));
                        continue;
                    }
                    Finish::Other(why) => {
                        return self.fault(format!(
                            "Nebius ended the model's answer for a reason Steplet does not know ({why}), so the run \
                             ended."
                        ));
                    }
                    // Claude Code's own words for a run that ended on its own, so the report reads both alike.
                    Finish::Stop | Finish::ToolCalls => {
                        self.show(Event::Result {
                            subtype: "success".into(),
                            is_error: false,
                            stop_reason: "end_turn".into(),
                            models: self.models.clone(),
                            text: text.unwrap_or_default(),
                        });
                        self.outcome.exit_code = Some(0);
                        return;
                    }
                }
            }
            for (n, call) in reply.calls.iter().enumerate() {
                let back = if self.may_go_on() { self.make_move(call, work) } else { None };
                match back {
                    Some(text) => convo.push(Message::tool(&call.id, &text)),
                    None => {
                        // The run ends here: the calls not yet decided are listed, and none of them ran.
                        for rest in &reply.calls[n + 1..] {
                            let (name, input) = proposed(rest, work);
                            self.show(Event::ToolUse { id: rest.id.clone(), name, input });
                        }
                        return;
                    }
                }
            }
        }
    }

    /// The model's next answer, or `None` when the run ends here (the reason is on record). Before each try, the
    /// most it could cost is held against the cap.
    fn ask(
        &mut self,
        brain: &Brain,
        id: &str,
        price: &Price,
        convo: &[Message],
        tools: &[ToolSpec],
    ) -> Option<ModelReply> {
        let mut tries_again = 0;
        let mut unreadable = 0;
        loop {
            let most = worst_case(chat_body(id, convo, tools, MAX_TOKENS).len(), MAX_TOKENS, price);
            let spent = self.spent.cost + self.spent.unsure;
            if spent + most > self.limits.cap {
                self.limit(Limit::Spending { spent, cap: self.limits.cap });
                return None;
            }
            let answer = match brain.ask(convo, self.limits.deadline, &self.live.stop) {
                Waited::Answer(answer) => answer,
                // The call is left to end on its own thread; whatever it cost is counted at the most.
                Waited::Stopped => {
                    self.spent.unsure += most;
                    self.stop_for_you();
                    return None;
                }
                Waited::TimeUp => {
                    self.spent.unsure += most;
                    self.limit(Limit::Time);
                    return None;
                }
                Waited::Gone => {
                    self.fault("Steplet's call to the model ended without an answer, so the run ended.".into());
                    return None;
                }
            };
            let e = match answer {
                Ok(reply) => return Some(reply),
                Err(e) => e,
            };
            self.note(json!({"failed": e.to_string()}));
            if matches!(e, ModelError::TooSlow | ModelError::Network(_) | ModelError::BadReply(_)) {
                self.spent.unsure += most;
            }
            if let ModelError::BadReply(_) = e {
                unreadable += 1;
                if unreadable < UNREADABLE_IN_A_ROW {
                    continue;
                }
                self.fault(format!(
                    "Nebius sent {UNREADABLE_IN_A_ROW} answers in a row that Steplet could not read, so the run \
                     ended. The last: {e}"
                ));
                return None;
            }
            let most_tries = self.limits.retry_waits.len();
            if !e.worth_retrying() {
                self.fault(e.to_string());
                return None;
            }
            if tries_again == most_tries {
                self.fault(format!(
                    "{e} Steplet tried again {most_tries} times, then ended the run. It may work if you run it again \
                     later."
                ));
                return None;
            }
            let mut wait = self.limits.retry_waits[tries_again];
            if let ModelError::Busy { retry_after: Some(secs), .. } = e {
                wait = wait.max(Duration::from_secs(secs).min(LONGEST_WAIT));
            }
            tries_again += 1;
            let status = match e {
                ModelError::Busy { status, .. } | ModelError::Unavailable { status, .. } => Some(status),
                _ => None,
            };
            self.show(Event::ApiRetry {
                status,
                error: e.to_string(),
                attempt: tries_again as u32,
                max: most_tries as u32,
            });
            if !self.wait(wait) {
                return None;
            }
        }
    }

    /// Adds an answer's tokens and cost, and tells the screen.
    fn count(&mut self, reply: &ModelReply, price: &Price) {
        let s = &mut self.spent;
        s.tokens_in = s.tokens_in.saturating_add(reply.usage.prompt_tokens);
        s.tokens_out = s.tokens_out.saturating_add(reply.usage.completion_tokens);
        s.cost += cost(&reply.usage, price);
        s.calls += 1;
        self.tell_spent();
        if !reply.model.is_empty() && !self.models.contains(&reply.model) {
            self.models.push(reply.model.clone());
        }
        let calls: Vec<Value> =
            reply.calls.iter().map(|c| json!({"id": c.id, "name": c.name, "arguments": c.arguments})).collect();
        self.note(json!({
            "answer": self.spent.calls,
            "model": reply.model,
            "finish": format!("{:?}", reply.finish),
            "tokens_in": reply.usage.prompt_tokens,
            "tokens_out": reply.usage.completion_tokens,
            "text": reply.text,
            "calls": calls,
        }));
    }

    /// Decides one call and, when it is allowed, runs it. Gives what the model is told, or `None` when the run ends
    /// here (the reason is on record).
    fn make_move(&mut self, call: &ToolCall, work: &str) -> Option<String> {
        let started = Instant::now();
        let (request, answer, unreadable) = match to_request(call, "", work) {
            Ok(request) => {
                self.show(Event::ToolUse {
                    id: call.id.clone(),
                    name: request.tool_name.clone(),
                    input: request.input.clone(),
                });
                let answer = self.asker.decide(self.checker, &request);
                (request, answer, false)
            }
            // A call that cannot be read is refused, and the model is told why; it never reaches the checker.
            Err(why) => {
                let request = as_written(call);
                self.show(Event::ToolUse {
                    id: call.id.clone(),
                    name: request.tool_name.clone(),
                    input: request.input.clone(),
                });
                let message = format!("Steplet could not read this call: {why}");
                (self.live.on_progress)(Progress::Checked {
                    call: self.scrub.text(&call.id),
                    check: Check::Refused,
                    reason: Some(self.scrub.text(&message)),
                });
                (request, GateAnswer::Deny { message }, true)
            }
        };
        self.outcome.requests.push(Asked::InApp {
            request: request.clone(),
            answer: answer.clone(),
            took: started.elapsed(),
        });
        let (back, go_on) = match answer {
            GateAnswer::Deny { message } if unreadable => (
                Back::error(format!(
                    "{message} Nothing ran. Call a tool again with the arguments it takes, or finish."
                )),
                true,
            ),
            GateAnswer::Deny { message } => (
                Back::error(format!(
                    "Steplet refused this: {}. It will not run. Carry on another way, or finish.",
                    message.trim_end().trim_end_matches('.')
                )),
                true,
            ),
            // The move is built from the request the checker allowed, never from the model's own arguments.
            GateAnswer::Allow { updated_input } => self.run_allowed(&GateRequest { input: updated_input, ..request }),
        };
        self.show(Event::ToolResult {
            id: call.id.clone(),
            is_error: back.is_error,
            text: back.text.clone(),
            plain_end: back.plain_end,
        });
        go_on.then_some(back.text)
    }

    /// Runs an allowed request, unless the run has ended meanwhile (the receiver may have taken a while to answer).
    /// True to go on.
    fn run_allowed(&mut self, allowed: &GateRequest) -> (Back, bool) {
        let Some(mv) = to_move(allowed) else {
            return (Back::error("Steplet could not make this into a move, so it did not run.".into()), true);
        };
        if self.live.stop.is_set() {
            self.stop_for_you();
            return (Back::error("Steplet stopped the run before this ran.".into()), false);
        }
        let left = self.limits.deadline.saturating_duration_since(Instant::now());
        if left.is_zero() {
            self.limit(Limit::Time);
            return (Back::error("The run's time was up before this ran.".into()), false);
        }
        let limit = self.limits.per_move.min(left);
        match self.hands.run(&mv, limit, &self.live.stop) {
            Ok(done) => {
                self.note(json!({
                    "move": allowed.tool_use_id,
                    "exit": done.exit_code,
                    "timed_out": done.timed_out,
                    "bytes": done.total_bytes,
                }));
                (said(&mv, &done, limit), true)
            }
            Err(HandsError::Stopped) => {
                self.stop_for_you();
                (Back::error("Steplet stopped the run while this ran, and ended it.".into()), false)
            }
            Err(e) => {
                let why = hands_words(&e);
                self.note(json!({"move": allowed.tool_use_id, "failed": why}));
                self.fault(format!("Steplet could not run the agent's move, so it ended the run: {why}"));
                (Back::error(format!("Steplet could not run this: {why}")), false)
            }
        }
    }

    /// Whether the run may go on: no Stop, and time left. Otherwise the reason is on record.
    fn may_go_on(&mut self) -> bool {
        if self.live.stop.is_set() {
            self.stop_for_you();
            return false;
        }
        if Instant::now() >= self.limits.deadline {
            self.limit(Limit::Time);
            return false;
        }
        true
    }

    /// Waits `wait` before trying again, unless Stop is pressed or the time runs out first. True to go on.
    fn wait(&mut self, wait: Duration) -> bool {
        let until = Instant::now() + wait;
        while Instant::now() < until {
            if !self.may_go_on() {
                return false;
            }
            thread::sleep(POLL.min(until.saturating_duration_since(Instant::now())));
        }
        self.may_go_on()
    }

    fn stop_for_you(&mut self) {
        self.outcome.stopped_by_you = true;
        self.outcome.stopped_at.get_or_insert(self.outcome.events.len());
    }

    fn limit(&mut self, limit: Limit) {
        self.outcome.limit.get_or_insert(limit);
        self.outcome.stopped_at.get_or_insert(self.outcome.events.len());
    }

    fn fault(&mut self, why: String) {
        self.outcome.faults.push(why);
    }

    /// Tells the screen what the run has spent so far.
    fn tell_spent(&self) {
        let s = &self.spent;
        (self.live.on_progress)(Progress::Spent(Spent { model: self.scrub.text(&s.model), ..s.clone() }));
    }

    /// Adds `event` to the run's record, and shows it cleaned as the report would show it.
    fn show(&mut self, event: Event) {
        (self.live.on_progress)(Progress::Event(self.scrub.event(&event)));
        self.outcome.events.push(event);
    }

    /// One line of Steplet's own record, with every secret hidden. A record that cannot be written is said once, as a
    /// fault, and the run goes on.
    fn note(&mut self, line: Value) {
        let mut bytes = raw_log_line(line.to_string().as_bytes(), self.secrets);
        bytes.push(b'\n');
        if let Err(e) = self.log.write_all(&bytes)
            && !self.log_failed
        {
            self.log_failed = true;
            self.fault(format!("Steplet could not write the run's own record: {e}"));
        }
    }
}

/// What a call gives back, as the report shows it and the model is told.
struct Back {
    is_error: bool,
    text: String,
    /// The move ended on its own (not stopped, not at its time limit), so no error means exit code 0.
    plain_end: bool,
}

impl Back {
    fn error(text: String) -> Back {
        Back { is_error: true, text, plain_end: false }
    }
}

/// A move's result, in Claude Code's form, so the report reads exit codes for both runners alike: a command's text
/// starts "Exit code N", and only exit code 0 is no error. Its output is cut for the model (`for_the_model`), and the
/// report shows what the model was told.
fn said(mv: &Move, done: &Done, limit: Duration) -> Back {
    let output = for_the_model(&done.output, done.total_bytes);
    let command = matches!(mv, Move::Command { .. });
    match done.exit_code {
        _ if done.timed_out => Back::error(format!(
            "Steplet ended this at its time limit of {} seconds.\n{output}",
            limit.as_secs().max(1)
        )),
        Some(code) if command => {
            Back { is_error: code != 0, text: format!("Exit code {code}\n{output}"), plain_end: true }
        }
        Some(code) => Back { is_error: code != 0, text: output, plain_end: true },
        None => Back::error(format!("It ended without an exit code.\n{output}")),
    }
}

/// A move's output as the model gets it: whole when short, else its first and last `SHOWN_END` bytes with the number
/// cut between them. When the output was longer than Steplet keeps, its end is not known, so only the start is given.
fn for_the_model(output: &str, total_bytes: u64) -> String {
    if output.is_empty() && total_bytes == 0 {
        return "(no output)".into();
    }
    let not_kept = total_bytes.saturating_sub(output.len() as u64);
    if not_kept > 0 {
        let head = &output[..floor(output, SHOWN_END)];
        return format!("{head}\n[{} more bytes not shown]", total_bytes - head.len() as u64);
    }
    if output.len() <= 2 * SHOWN_END {
        return output.into();
    }
    let head = &output[..floor(output, SHOWN_END)];
    let tail = &output[ceil(output, output.len() - SHOWN_END)..];
    format!("{head}\n[{} bytes cut]\n{tail}", output.len() - head.len() - tail.len())
}

/// The largest character boundary of `s` at or before `at`.
fn floor(s: &str, at: usize) -> usize {
    (0..=at.min(s.len())).rev().find(|&i| s.is_char_boundary(i)).unwrap_or(0)
}

/// The smallest character boundary of `s` at or after `at`.
fn ceil(s: &str, at: usize) -> usize {
    (at..=s.len()).find(|&i| s.is_char_boundary(i)).unwrap_or(s.len())
}

/// A call that cannot be read, as a request for the record: its tool's name, and its arguments as JSON when they are
/// JSON, else as they came.
fn as_written(call: &ToolCall) -> GateRequest {
    let input = serde_json::from_str(&call.arguments).unwrap_or_else(|_| Value::String(call.arguments.clone()));
    GateRequest { run_token: String::new(), tool_name: call.name.clone(), input, tool_use_id: call.id.clone() }
}

/// A call's tool and input as the checker would have seen them, for a call the run ended before deciding.
fn proposed(call: &ToolCall, work: &str) -> (String, Value) {
    let r = to_request(call, "", work).unwrap_or_else(|_| as_written(call));
    (r.tool_name, r.input)
}

/// What went wrong with the hands, in Steplet's words.
fn hands_words(e: &HandsError) -> String {
    match e {
        HandsError::Linux(why) => format!("Steplet's Linux could not run it: {why}"),
        HandsError::Layout(why) => format!("the run's folders were not as Steplet made them: {why}"),
        HandsError::Unreadable(why) => format!("Steplet could not read what its Linux answered: {why}"),
        HandsError::Stopped => "the run was stopped".into(),
    }
}

/// The model, on a thread of its own, so that a call that hangs never holds up the Stop button or the time limit.
struct Brain {
    asks: Sender<Vec<Message>>,
    answers: Receiver<Result<ModelReply, ModelError>>,
}

/// How waiting for the model ended.
enum Waited {
    Answer(Result<ModelReply, ModelError>),
    Stopped,
    TimeUp,
    /// The model's thread ended without an answer.
    Gone,
}

impl Brain {
    fn start(mut model: Box<dyn Model>) -> std::io::Result<Brain> {
        let (asks, asked) = mpsc::channel::<Vec<Message>>();
        let (answer, answers) = mpsc::channel();
        // Once the run is over the thread is no longer waited for: a call still under way ends on its own, its answer
        // goes nowhere, and the thread ends with it.
        thread::Builder::new().name("steplet-model".into()).spawn(move || {
            let tools = specs();
            for convo in asked {
                if answer.send(model.next(&convo, &tools, MAX_TOKENS)).is_err() {
                    break;
                }
            }
        })?;
        Ok(Brain { asks, answers })
    }

    /// The model's answer to `convo`, unless Stop is pressed or `deadline` passes first.
    fn ask(&self, convo: &[Message], deadline: Instant, stop: &StopSignal) -> Waited {
        if self.asks.send(convo.to_vec()).is_err() {
            return Waited::Gone;
        }
        loop {
            match self.answers.recv_timeout(POLL) {
                Ok(answer) => return Waited::Answer(answer),
                Err(RecvTimeoutError::Disconnected) => return Waited::Gone,
                Err(RecvTimeoutError::Timeout) => {}
            }
            if stop.is_set() {
                return Waited::Stopped;
            }
            if Instant::now() >= deadline {
                return Waited::TimeUp;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn short_output_reaches_the_model_whole() {
        assert_eq!(for_the_model("ok\n", 3), "ok\n");
        assert_eq!(for_the_model("", 0), "(no output)");
    }

    #[test]
    fn long_output_keeps_its_start_and_end() {
        let out = format!("{}{}{}", "a".repeat(5000), "b".repeat(10_000), "c".repeat(5000));
        let shown = for_the_model(&out, out.len() as u64);
        assert!(shown.starts_with(&"a".repeat(4096)), "start");
        assert!(shown.ends_with(&"c".repeat(4096)), "end");
        assert!(shown.contains(&format!("[{} bytes cut]", out.len() - 2 * 4096)), "{}", &shown[4090..4130]);
    }

    #[test]
    fn output_longer_than_steplet_keeps_gives_only_its_start() {
        let kept = "x".repeat(1 << 20);
        let shown = for_the_model(&kept, 5 << 20);
        assert_eq!(shown, format!("{}\n[{} more bytes not shown]", "x".repeat(4096), (5 << 20) - 4096));
    }

    #[test]
    fn output_is_never_cut_inside_a_character() {
        let out = "é".repeat(6000);
        let shown = for_the_model(&out, out.len() as u64);
        assert!(shown.contains("bytes cut]"), "cut");
        let kept = "é".repeat(3000);
        assert!(for_the_model(&kept, (kept.len() + 10) as u64).contains("more bytes not shown"));
    }
}
