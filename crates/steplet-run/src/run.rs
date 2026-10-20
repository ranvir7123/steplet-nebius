//! One whole run (plan Task 9): the gate, the working copy, the launch and the tripwire watcher, joined, ending in the
//! receiver's report (Task 10).

use std::io;
use std::path::PathBuf;
use std::ptr::null_mut;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::RecvTimeoutError;
use std::thread;
use std::time::{Duration, Instant};

use steplet_core::policy::Policy;
use windows_sys::Win32::Security::Cryptography::{BCRYPT_USE_SYSTEM_PREFERRED_RNG, BCryptGenRandom};

use crate::ask::{Answer, Asker};
use crate::decide::decide;
use crate::gate::{ApiKey, Asked, GateError, GateService, gate_for_run, gate_on_windows, serve};
use crate::launch::{LaunchError, RunHandle, launch};
use crate::live::{Live, Progress, StopSignal};
use crate::lock::RunLock;
use crate::prepare::{PrepareError, changes, discard, prepare, run_config};
use crate::report::{Report, Scrubber, Trust, build_report, read_trust};
use crate::types::{BadValue, Event, GateLink, RunPaths, RunRequest};
use crate::watch::{Tripwire, Watcher};
use crate::wsl;

/// The name Steplet gives its own Linux (Task 1); Windows names the gate program by it.
const DISTRIBUTION: &str = "steplet";
/// How long after its time limit a run that has not ended is stopped from Windows. The time limit inside Linux
/// (`timeout`) normally ends it first.
pub const BACKSTOP: Duration = Duration::from_secs(30);
/// How often the watcher looks whether the agent has ended, while no event comes.
const POLL: Duration = Duration::from_millis(200);
/// How much longer than the ask limit the gate's deadline must be, so the gate never gives up on a decision before
/// the receiver has had their whole time to answer (plan section 3.3).
pub const ASK_MARGIN: Duration = Duration::from_secs(15);
/// How long Steplet waits at most for one call into its Linux that ends what preparing started.
const QUICK_CALL: Duration = Duration::from_secs(10);

/// What a run needs besides the request, its paths and the checker.
#[derive(Clone, Debug)]
pub struct RunSetup {
    /// The gate program on Steplet's Linux disk, where `install_gate` put it.
    pub gate_exe: String,
    /// Steplet's logs folder on Windows (plan section 3): the run's event stream, error output and gate log.
    pub logs: PathBuf,
    /// The Anthropic key, for the gate's key mode (H2).
    pub key: Option<ApiKey>,
    /// Steplet's limit on each gate decision (F36).
    pub decision_deadline: Duration,
    /// The scripted model's address, `http://127.0.0.1:<port>`; tests only.
    pub model_url: Option<String>,
}

/// Why a run could not start. Nothing of the agent ran.
#[derive(Debug)]
pub enum RunError {
    /// A value Steplet will not hand to the gate.
    Refused(BadValue),
    /// Windows gave no random numbers for the run's token.
    Random(io::Error),
    Gate(GateError),
    Prepare(PrepareError),
    Launch(LaunchError),
    /// Another run holds the lock (one run at a time on this computer, phase 4).
    Busy,
    /// Windows would not make the lock; the error says why.
    Lock(io::Error),
    /// The gate's deadline is shorter than the ask limit plus `ASK_MARGIN`: the gate would refuse a question before
    /// the receiver's time to answer it was up.
    Limits,
}

/// What happened in a run, from Steplet's own records.
#[derive(Clone, Debug, PartialEq)]
pub struct RunOutcome {
    /// Everything the run did, in order, with Steplet's `SkippedGate` record after each tool call that did not ask
    /// the gate.
    pub events: Vec<Event>,
    /// Everything the gate asked, with Steplet's answers.
    pub requests: Vec<Asked>,
    /// The first tripwire seen. The run was stopped at once, or had already ended when it was seen.
    pub stopped_by: Option<Tripwire>,
    /// How many of `events` had come when Steplet began to stop the run (at a tripwire, or at its time limit). A call
    /// whose result came after that was cut short or refused by the stop.
    pub stopped_at: Option<usize>,
    /// Claude Code's exit code. It is 124 at the time limit and also after a stop, so whether Steplet stopped the
    /// run is `stopped_by` and `faults`, not this.
    pub exit_code: Option<i32>,
    /// What went wrong around the run, in Steplet's words: it could not be stopped or waited for, did not end at
    /// its time limit, or was not removed.
    pub faults: Vec<String>,
    /// The receiver pressed Stop, and Steplet stopped the run as at a tripwire (phase 4).
    pub stopped_by_you: bool,
    /// Every question the receiver was asked, and every yes reused, from Steplet's own record (R7).
    pub answers: Vec<Answer>,
}

/// Runs `req` as run `paths`, with `policy` deciding every request, and returns the receiver's report: `run_live`
/// with no one watching.
pub fn run(req: &RunRequest, paths: &RunPaths, policy: &Policy, setup: &RunSetup) -> Result<Report, RunError> {
    run_live(req, paths, policy, setup, Arc::new(Live::unattended()))
}

/// Runs `req` as run `paths`, with `policy` deciding every request, and returns the receiver's report. Only one run
/// at a time: another one gives `RunError::Busy`. Each piece of progress goes to `live`, cleaned as the report is, and
/// `live.stop` stops the run as a tripwire would (before the agent starts, it never starts). Before the run is
/// removed, Steplet's own `git status` lists what it changed and the run's config is checked for trust (D3). The
/// gate's pipe gets a name and a token no one can guess, made for this run; neither the token nor the key shows in
/// the report, the live log or the raw logs (F48, J6).
pub fn run_live(
    req: &RunRequest,
    paths: &RunPaths,
    policy: &Policy,
    setup: &RunSetup,
    live: Arc<Live>,
) -> Result<Report, RunError> {
    if live.answers.is_some()
        && live.ask_limit.checked_add(ASK_MARGIN).is_none_or(|needed| setup.decision_deadline < needed)
    {
        return Err(RunError::Limits);
    }
    let _one_run = RunLock::take()?;
    let pipe = format!("steplet-{}-{}", paths.run_id(), random_hex::<8>().map_err(RunError::Random)?);
    let token = random_hex::<32>().map_err(RunError::Random)?;
    let secrets: Vec<&str> = setup.key.iter().map(ApiKey::text).chain([token.as_str()]).collect();
    let log = setup.logs.join(format!("{}.gate.log", paths.run_id()));
    // The gate starts from a copy of its own for this run (see `gate_for_run`); `discard` removes it with the run.
    let gate_exe = gate_for_run(paths, &setup.gate_exe).map_err(RunError::Gate)?;
    let link = match GateLink::new(&gate_exe, &pipe, &token, &log.to_string_lossy(), setup.decision_deadline) {
        Ok(link) => link,
        Err(e) => {
            let _ = discard(paths);
            return Err(RunError::Refused(e));
        }
    };
    let checker = policy.clone();
    let program = gate_on_windows(DISTRIBUTION, &gate_exe);
    // With someone to ask, the checker's "ask" becomes a question on their screen; without, a refusal.
    let asker = Arc::new(Asker::new(live.clone(), &secrets));
    let gate = if live.answers.is_some() {
        let asking = asker.clone();
        serve(&link, &program, setup.key.clone(), move |r| asking.decide(&checker, r))
    } else {
        serve(&link, &program, setup.key.clone(), move |r| decide(&checker, r))
    };
    let gate = match gate {
        Ok(gate) => gate,
        Err(e) => {
            let _ = discard(paths);
            return Err(RunError::Gate(e));
        }
    };
    let prepared = prepare_until_stopped(req, paths, &link, &live);
    let started = match prepared {
        // A Stop while preparing ends what preparing started, so preparing fails: the receiver's Stop, not a fault.
        _ if live.stop.is_set() => Ok(None),
        Ok(prepared) => {
            (live.on_progress)(Progress::Prepared(prepared));
            launch(req, paths, &setup.logs, setup.model_url.as_deref(), &secrets).map(Some).map_err(RunError::Launch)
        }
        Err(e) => Err(RunError::Prepare(e)),
    };
    let mut outcome = match started {
        Ok(Some(handle)) => {
            watch_run(handle, gate, policy, paths.work_repo(), req.deadline + BACKSTOP, &live, &secrets)
        }
        // The receiver pressed Stop before the agent started: nothing of the task ran.
        Ok(None) => RunOutcome { stopped_by_you: true, stopped_at: Some(0), ..RunOutcome::new(gate.into_requests()) },
        Err(e) => {
            // Whatever was made goes; the next run's preparation would clear it anyway.
            let _ = discard(paths);
            return Err(e);
        }
    };
    // The run is over: a question still on screen is taken back, and recorded as such.
    asker.take_back_all();
    outcome.answers = asker.answers();
    let agent_ran = !outcome.events.is_empty();
    // Steplet's own looks after the run. They come after the run, so they do not change whether it completed. When
    // the agent never started there is nothing to look at: the copy is the one Steplet made, or not even that.
    let mut after = Vec::new();
    let (mut changed, mut config) = (None, None);
    if agent_ran {
        changed = changes(paths)
            .map_err(|e| after.push(format!("Steplet could not list the files the run changed: {e:?}")))
            .ok();
        config = run_config(paths)
            .map_err(|e| after.push(format!("Steplet could not read the run's config for the trust check: {e:?}")))
            .ok();
    }
    if let Err(e) = discard(paths) {
        after.push(format!("the run's folders could not be removed: {e:?}"));
    }
    let trust = if agent_ran { read_trust(config.as_deref()) } else { Trust::Unknown };
    let mut report = build_report(&outcome, changed.as_deref(), trust, &secrets);
    let scrub = Scrubber::new(&secrets);
    report.faults.extend(after.iter().map(|fault| scrub.text(fault)));
    Ok(report)
}

/// `prepare`, which can take minutes for a large repository, ended at once by the receiver's Stop: while it runs, a
/// second thread looks at the Stop button and, once it is pressed, ends every program of the steplet account (the
/// clone and the checkout run as that account), again and again until preparing has failed.
fn prepare_until_stopped(
    req: &RunRequest,
    paths: &RunPaths,
    link: &GateLink,
    live: &Live,
) -> Result<crate::prepare::Prepared, PrepareError> {
    let done = AtomicBool::new(false);
    thread::scope(|s| {
        s.spawn(|| {
            while !done.load(Ordering::SeqCst) {
                if live.stop.is_set() {
                    let end = ["/usr/bin/pkill", "-KILL", "-u", "steplet"];
                    let _ = wsl::run_within(paths.distro_id(), "root", &end, b"", QUICK_CALL);
                }
                thread::sleep(POLL);
            }
        });
        let prepared = prepare(req, paths, link);
        done.store(true, Ordering::SeqCst);
        prepared
    })
}

/// A running agent, as `watch_run` follows it. `RunHandle` is the real one; the tests also use stand-ins.
pub trait RunningAgent {
    /// The next event, waiting at most `wait`: `Timeout` when none came, `Disconnected` when no more will come.
    fn next_event(&mut self, wait: Duration) -> Result<Event, RecvTimeoutError>;
    /// True once the agent has ended.
    fn ended(&mut self) -> io::Result<bool>;
    /// Stops the agent, as `RunHandle::stop` does.
    fn stop(&mut self) -> io::Result<()>;
    /// Ends the agent's `wsl.exe` from Windows at once, without asking Steplet's Linux.
    fn kill(&mut self) -> io::Result<()>;
    /// Waits for the end, and hands over the events not yet taken, as `RunHandle::finish` does.
    fn finish(self) -> (io::Result<Option<i32>>, Vec<Event>);
}

impl RunningAgent for RunHandle {
    fn next_event(&mut self, wait: Duration) -> Result<Event, RecvTimeoutError> {
        self.events().recv_timeout(wait)
    }

    fn ended(&mut self) -> io::Result<bool> {
        RunHandle::ended(self)
    }

    fn stop(&mut self) -> io::Result<()> {
        RunHandle::stop(self)
    }

    fn kill(&mut self) -> io::Result<()> {
        RunHandle::kill(self)
    }

    fn finish(self) -> (io::Result<Option<i32>>, Vec<Event>) {
        RunHandle::finish(self)
    }
}

/// Follows a launched run to its end. Each event is judged as it comes, sent to `live` cleaned of `secrets` (with
/// Steplet's record after a call that skipped the gate), and the run is stopped at the first tripwire or when the
/// receiver presses Stop; from then on the gate refuses every request, since stopping takes a moment. If the run has
/// not ended `stop_after` from now, Steplet stops it itself: `run` gives the run's own time limit plus 30 s. The gate
/// service is stopped once the agent has ended.
pub fn watch_run(
    mut handle: impl RunningAgent,
    gate: GateService,
    policy: &Policy,
    work: &str,
    stop_after: Duration,
    live: &Live,
    secrets: &[&str],
) -> RunOutcome {
    let mut watcher = Watcher::new(policy, work);
    let scrub = Scrubber::new(secrets);
    let show = |e: &Event| (live.on_progress)(Progress::Event(scrub.event(e)));
    let mut outcome = RunOutcome::new(Vec::new());
    let backstop = Instant::now() + stop_after;
    let mut backstopped = false;
    // Neither Steplet's Linux nor Windows could end the run, so waiting for its end could take for ever.
    let mut gave_up = false;
    let mut next_look = Instant::now();
    loop {
        match handle.next_event(POLL) {
            Ok(event) => {
                // After the receiver's Stop, a tripwire is still recorded, but the run is already being stopped.
                if outcome.judge(&mut watcher, event, &gate, &show)
                    && !outcome.stopped_by_you
                    && !stop_now(&mut handle, &gate, &mut outcome.faults, &live.stop)
                {
                    gave_up = true;
                    break;
                }
            }
            Err(RecvTimeoutError::Timeout) => {}
            // No more events will come, but the agent may not have ended yet: keep looking until it has.
            Err(RecvTimeoutError::Disconnected) => thread::sleep(POLL),
        }
        // The receiver's Stop, looked at on every turn; once Steplet is already stopping the run, it changes nothing.
        if live.stop.is_set() && !outcome.stopped_by_you && outcome.stopped_at.is_none() {
            outcome.stopped_by_you = true;
            outcome.stopped_at = Some(outcome.events.len());
            if !stop_now(&mut handle, &gate, &mut outcome.faults, &live.stop) {
                gave_up = true;
                break;
            }
        }
        // Looked at on time even while events keep coming.
        if Instant::now() < next_look {
            continue;
        }
        next_look = Instant::now() + POLL;
        match handle.ended() {
            Ok(true) => break,
            Ok(false) => {}
            Err(e) => {
                outcome.faults.push(format!("Steplet could not tell whether the run had ended: {e}"));
                // It may still be running: it is stopped, as at a tripwire, before Steplet waits for its end.
                outcome.stopped_at.get_or_insert(outcome.events.len());
                gave_up = !stop_now(&mut handle, &gate, &mut outcome.faults, &live.stop);
                break;
            }
        }
        if !backstopped && Instant::now() >= backstop {
            backstopped = true;
            outcome.faults.push("the run did not end at its time limit, so Steplet stopped it".into());
            outcome.stopped_at.get_or_insert(outcome.events.len());
            if !stop_now(&mut handle, &gate, &mut outcome.faults, &live.stop) {
                gave_up = true;
                break;
            }
        }
    }
    let (code, rest) = if gave_up {
        // Only what has already come is read; `run` then goes on to remove the run's folders and make the report.
        outcome.faults.push("Steplet did not wait for the run to end, since it could not end it".into());
        (Ok(None), already_come(&mut handle))
    } else {
        handle.finish()
    };
    for event in rest {
        outcome.judge(&mut watcher, event, &gate, &show);
    }
    match code {
        Ok(code) => outcome.exit_code = code,
        Err(e) => outcome.faults.push(format!("Steplet could not wait for the run to end: {e}")),
    }
    // Only now that the agent has ended: a decision still being made was not acted on.
    outcome.requests = gate.into_requests();
    outcome
}

/// Stops the agent at once. The gate refuses everything from now on, every question still on screen is taken back
/// (`stop` is set), then the run is stopped; if that fails, the agent's `wsl.exe` is ended from Windows. False when
/// even that failed, and there is nothing more Steplet can do.
fn stop_now(agent: &mut impl RunningAgent, gate: &GateService, faults: &mut Vec<String>, stop: &StopSignal) -> bool {
    gate.refuse_everything();
    stop.stop();
    let Err(e) = agent.stop() else { return true };
    faults.push(format!("Steplet could not stop the run: {e}"));
    match agent.kill() {
        Ok(()) => true,
        Err(e) => {
            faults.push(format!("Steplet could not end the run from Windows either: {e}"));
            false
        }
    }
}

/// The events that have already come, read without waiting for more, and for at most a second, since an agent that
/// could not be ended may go on talking.
fn already_come(agent: &mut impl RunningAgent) -> Vec<Event> {
    let until = Instant::now() + Duration::from_secs(1);
    let mut events = Vec::new();
    while Instant::now() < until {
        match agent.next_event(Duration::ZERO) {
            Ok(event) => events.push(event),
            Err(_) => break,
        }
    }
    events
}

/// `N` random bytes from Windows' own generator, as lower-case hexadecimal.
pub(crate) fn random_hex<const N: usize>() -> io::Result<String> {
    let mut bytes = [0u8; N];
    // SAFETY: `bytes` is writable for `N` bytes; the system's preferred generator needs no algorithm handle.
    let status = unsafe { BCryptGenRandom(null_mut(), bytes.as_mut_ptr(), N as u32, BCRYPT_USE_SYSTEM_PREFERRED_RNG) };
    if status != 0 {
        return Err(io::Error::other(format!("Windows gave no random numbers (status {status:#x})")));
    }
    Ok(bytes.iter().map(|b| format!("{b:02x}")).collect())
}

impl RunOutcome {
    /// An outcome with nothing on record yet but the gate's `requests`.
    fn new(requests: Vec<Asked>) -> RunOutcome {
        RunOutcome {
            events: Vec::new(),
            requests,
            stopped_by: None,
            stopped_at: None,
            exit_code: None,
            faults: Vec::new(),
            stopped_by_you: false,
            answers: Vec::new(),
        }
    }

    /// Adds `event` and the watcher's record of it, and shows both. True when it set off the run's first tripwire;
    /// after the receiver's Stop none counts, since the run was already being stopped for them.
    fn judge(&mut self, watcher: &mut Watcher, event: Event, gate: &GateService, show: &dyn Fn(&Event)) -> bool {
        let verdict = watcher.see(&event, &|id| gate.answered(id));
        show(&event);
        if let Some(skipped) = &verdict.skipped {
            show(skipped);
        }
        self.events.push(event);
        self.events.extend(verdict.skipped);
        match verdict.stop {
            Some(why) if self.stopped_by.is_none() && !self.stopped_by_you => {
                self.stopped_by = Some(why);
                self.stopped_at.get_or_insert(self.events.len());
                true
            }
            _ => false,
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn random_hex_is_fresh_lower_case_hexadecimal_of_the_length_asked() {
        let (a, b) = (super::random_hex::<32>().unwrap(), super::random_hex::<32>().unwrap());
        assert_eq!(a.len(), 64);
        assert!(a.bytes().all(|c| c.is_ascii_digit() || (b'a'..=b'f').contains(&c)), "{a}");
        assert_ne!(a, b, "two runs never share a token");
        assert_eq!(super::random_hex::<8>().unwrap().len(), 16);
    }
}
