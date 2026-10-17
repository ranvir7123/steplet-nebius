//! One whole run (plan Task 9): the gate, the working copy, the launch and the tripwire watcher, joined, ending in the
//! receiver's report (Task 10).

use std::io;
use std::path::PathBuf;
use std::ptr::null_mut;
use std::sync::mpsc::RecvTimeoutError;
use std::thread;
use std::time::{Duration, Instant};

use steplet_core::policy::Policy;
use windows_sys::Win32::Security::Cryptography::{BCRYPT_USE_SYSTEM_PREFERRED_RNG, BCryptGenRandom};

use crate::decide::decide;
use crate::gate::{ApiKey, Asked, GateError, GateService, gate_on_windows, serve};
use crate::launch::{LaunchError, RunHandle, launch};
use crate::prepare::{PrepareError, changes, discard, prepare, run_config};
use crate::report::{Report, Scrubber, build_report, read_trust};
use crate::types::{BadValue, Event, GateLink, RunPaths, RunRequest};
use crate::watch::{Tripwire, Watcher};

/// The name Steplet gives its own Linux (Task 1); Windows names the gate program by it.
const DISTRIBUTION: &str = "steplet";
/// How long after its time limit a run that has not ended is stopped from Windows. The time limit inside Linux
/// (`timeout`) normally ends it first.
pub const BACKSTOP: Duration = Duration::from_secs(30);
/// How often the watcher looks whether the agent has ended, while no event comes.
const POLL: Duration = Duration::from_millis(200);

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
}

/// Runs `req` as run `paths`, with `policy` deciding every request, and returns the receiver's report. Before the run
/// is removed, Steplet's own `git status` lists what it changed and the run's config is checked for trust (D3). The
/// gate's pipe gets a name and a token no one can guess, made for this run; neither the token nor the key shows in
/// the report (F48).
pub fn run(req: &RunRequest, paths: &RunPaths, policy: &Policy, setup: &RunSetup) -> Result<Report, RunError> {
    let pipe = format!("steplet-{}-{}", paths.run_id(), random_hex::<8>().map_err(RunError::Random)?);
    let token = random_hex::<32>().map_err(RunError::Random)?;
    let log = setup.logs.join(format!("{}.gate.log", paths.run_id()));
    let link = GateLink::new(&setup.gate_exe, &pipe, &token, &log.to_string_lossy(), setup.decision_deadline)
        .map_err(RunError::Refused)?;
    let checker = policy.clone();
    let program = gate_on_windows(DISTRIBUTION, &setup.gate_exe);
    let gate = serve(&link, &program, setup.key.clone(), move |r| decide(&checker, r)).map_err(RunError::Gate)?;
    let started = prepare(req, paths, &link)
        .map_err(RunError::Prepare)
        .and_then(|()| launch(req, paths, &setup.logs, setup.model_url.as_deref()).map_err(RunError::Launch));
    let handle = match started {
        Ok(handle) => handle,
        Err(e) => {
            // Whatever was made goes; the next run's preparation would clear it anyway.
            let _ = discard(paths);
            return Err(e);
        }
    };
    let outcome = watch_run(handle, gate, policy, paths.work_repo(), req.deadline + BACKSTOP);
    // Steplet's own looks after the run. They come after the run, so they do not change whether it completed.
    let mut after = Vec::new();
    let changed =
        changes(paths).map_err(|e| after.push(format!("Steplet could not list the files the run changed: {e:?}"))).ok();
    let config = run_config(paths)
        .map_err(|e| after.push(format!("Steplet could not read the run's config for the trust check: {e:?}")))
        .ok();
    if let Err(e) = discard(paths) {
        after.push(format!("the run's folders could not be removed: {e:?}"));
    }
    let secrets: Vec<&str> = setup.key.iter().map(ApiKey::text).chain([token.as_str()]).collect();
    let mut report = build_report(&outcome, changed.as_deref(), read_trust(config.as_deref()), &secrets);
    let scrub = Scrubber::new(&secrets);
    report.faults.extend(after.iter().map(|fault| scrub.text(fault)));
    Ok(report)
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

/// Follows a launched run to its end. Each event is judged as it comes, and the run is stopped at the first
/// tripwire; from then on the gate refuses every request, since stopping takes a moment. If the run has not ended
/// `stop_after` from now, Steplet stops it itself: `run` gives the run's own time limit plus 30 s. The gate service
/// is stopped once the agent has ended.
pub fn watch_run(
    mut handle: impl RunningAgent,
    gate: GateService,
    policy: &Policy,
    work: &str,
    stop_after: Duration,
) -> RunOutcome {
    let mut watcher = Watcher::new(policy, work);
    let mut outcome = RunOutcome {
        events: Vec::new(),
        requests: Vec::new(),
        stopped_by: None,
        stopped_at: None,
        exit_code: None,
        faults: Vec::new(),
    };
    let backstop = Instant::now() + stop_after;
    let mut backstopped = false;
    // Neither Steplet's Linux nor Windows could end the run, so waiting for its end could take for ever.
    let mut gave_up = false;
    let mut next_look = Instant::now();
    loop {
        match handle.next_event(POLL) {
            Ok(event) => {
                if outcome.judge(&mut watcher, event, &gate) && !stop_now(&mut handle, &gate, &mut outcome.faults) {
                    gave_up = true;
                    break;
                }
            }
            Err(RecvTimeoutError::Timeout) => {}
            // No more events will come, but the agent may not have ended yet: keep looking until it has.
            Err(RecvTimeoutError::Disconnected) => thread::sleep(POLL),
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
                gave_up = !stop_now(&mut handle, &gate, &mut outcome.faults);
                break;
            }
        }
        if !backstopped && Instant::now() >= backstop {
            backstopped = true;
            outcome.faults.push("the run did not end at its time limit, so Steplet stopped it".into());
            outcome.stopped_at.get_or_insert(outcome.events.len());
            if !stop_now(&mut handle, &gate, &mut outcome.faults) {
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
        outcome.judge(&mut watcher, event, &gate);
    }
    match code {
        Ok(code) => outcome.exit_code = code,
        Err(e) => outcome.faults.push(format!("Steplet could not wait for the run to end: {e}")),
    }
    // Only now that the agent has ended: a decision still being made was not acted on.
    outcome.requests = gate.into_requests();
    outcome
}

/// Stops the agent at once. The gate refuses everything from now on, then the run is stopped; if that fails, the
/// agent's `wsl.exe` is ended from Windows. False when even that failed, and there is nothing more Steplet can do.
fn stop_now(agent: &mut impl RunningAgent, gate: &GateService, faults: &mut Vec<String>) -> bool {
    gate.refuse_everything();
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
fn random_hex<const N: usize>() -> io::Result<String> {
    let mut bytes = [0u8; N];
    // SAFETY: `bytes` is writable for `N` bytes; the system's preferred generator needs no algorithm handle.
    let status = unsafe { BCryptGenRandom(null_mut(), bytes.as_mut_ptr(), N as u32, BCRYPT_USE_SYSTEM_PREFERRED_RNG) };
    if status != 0 {
        return Err(io::Error::other(format!("Windows gave no random numbers (status {status:#x})")));
    }
    Ok(bytes.iter().map(|b| format!("{b:02x}")).collect())
}

impl RunOutcome {
    /// Adds `event` and the watcher's record of it. True when it set off the run's first tripwire.
    fn judge(&mut self, watcher: &mut Watcher, event: Event, gate: &GateService) -> bool {
        let verdict = watcher.see(&event, &|id| gate.answered(id));
        self.events.push(event);
        self.events.extend(verdict.skipped);
        match verdict.stop {
            Some(why) if self.stopped_by.is_none() => {
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
