//! A run from the app (Task 12). When Run is pressed the file is checked again (row 30: the bytes previewed, expiry
//! with the time now, the red sentence, what this computer can run), the sender is pinned (S5), and the run goes on
//! a thread of its own through the runner (`steplet_run::run_live`, or a stand-in in tests). Its progress reaches the
//! screen through a `Relay`: each piece as the screen shows it, one question at a time with the taskbar flashing for
//! each. A marker file says a run is going on, so that a run left by a crash is stopped and removed at the next start.

use std::collections::{HashMap, VecDeque};
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, PoisonError};
use std::time::Duration;

use serde::{Deserialize, Serialize};
use serde_json::Value;
use sha2::{Digest, Sha256};
use steplet_core::policy::Policy;
use steplet_core::report::{Report as FileReport, check};
use steplet_core::rules::{Outcome, TIME_EXPIRED};
use steplet_core::sig::SigStatus;
use steplet_run::{
    Answers, ApiKey, Event, Live, PrepareError, Progress, Question, Reply, Report, RunError, RunLock, RunPaths,
    RunRequest, RunSetup, StopSignal, task_text,
};

use crate::pins;
use crate::views::{self, ProgressView, RED_SENTENCE, RunStatus};

/// How long a question waits for its answer; no answer by then is a no (S1).
pub const ASK_LIMIT: Duration = Duration::from_secs(5 * 60);
/// Steplet's limit on each decision of the gate: the ask limit and 15 s (S1).
pub const DECISION_DEADLINE: Duration = Duration::from_secs(5 * 60 + 15);
/// A run's limits (S2).
pub const RUN_LIMIT: Duration = Duration::from_secs(30 * 60);
pub const MAX_TURNS: u32 = 40;

/// The runner, as the session calls it, with the one-run lock the session holds for the run:
/// `steplet_run::run_live_locked` in the app, a stand-in in tests.
pub type RunFn = Arc<
    dyn Fn(&RunLock, &RunRequest, &RunPaths, &Policy, &RunSetup, Arc<Live>) -> Result<Report, RunError> + Send + Sync,
>;

/// The window, as a run needs it.
pub trait Screen: Send + Sync {
    /// One piece of progress for the page (the `run-progress` event).
    fn send(&self, p: ProgressView);
    /// Gets the receiver's attention: a question is on screen (the taskbar button flashes).
    fn attention(&self);
    /// Closes the window: the receiver asked to close it while the run went on, and the run is over.
    fn close(&self);
}

/// Where a run keeps its things on Windows.
#[derive(Clone, Debug)]
pub struct Places {
    /// Written before a run and removed after it: a marker left at start means a crash.
    pub marker: PathBuf,
    /// The senders this computer knows (`None` when Windows does not say where they go).
    pub pins: Option<PathBuf>,
    /// The runner's raw logs (J6).
    pub logs: PathBuf,
}

impl Places {
    /// `%LOCALAPPDATA%\Steplet`'s marker, senders list and logs folder.
    pub fn here() -> Option<Places> {
        let steplet = PathBuf::from(std::env::var_os("LOCALAPPDATA")?).join("Steplet");
        Some(Places {
            marker: steplet.join("run-in-progress.json"),
            pins: Some(steplet.join("senders.json")),
            logs: steplet.join("logs"),
        })
    }
}

/// What setup gives a run.
pub struct Ready {
    /// Steplet's Linux, by its id.
    pub distro_id: String,
    /// This build's gate program on its disk.
    pub gate_exe: String,
    pub key: ApiKey,
}

/// The marker's contents.
#[derive(Serialize, Deserialize)]
struct Marker {
    run_id: String,
    distro_id: String,
}

/// The run going on now, from the moment Run is pressed: a Stop or an answer given while it still starts is kept.
struct Current {
    stop: StopSignal,
    answers: Answers,
}

/// The note for the Open screen after a crash's run was cleared.
const CLEARED: &str = "Steplet was closed while a file was running. It stopped what was left of that run and removed its \
                       folders. Nothing of it is left.";

/// Removes what a run left on Steplet's Linux: `steplet_run::discard` in the app, a stand-in in tests.
pub type DiscardFn = Arc<dyn Fn(&RunPaths) -> Result<(), PrepareError> + Send + Sync>;

/// The note for the Open screen when a crash's run could not be removed yet.
const LEFT_BEHIND: &str = "Steplet was closed while a file was running, and could not yet remove everything that run \
                           left in its Linux. It tries again the next time it starts.";

pub struct Session {
    run: RunFn,
    discard: DiscardFn,
    now: Arc<dyn Fn() -> i64 + Send + Sync>,
    places: Places,
    /// The run of this window, from the moment Run is pressed until the run is over.
    current: Arc<Mutex<Option<Current>>>,
    /// The receiver chose "Stop and close": the window closes once the run is over. Read and changed only while
    /// `current` is locked, so the run's end never misses it.
    close_after: Arc<AtomicBool>,
    /// The last run that finished in this window: its report, for "Save report…", and its file.
    last: Arc<Mutex<Option<Last>>>,
}

struct Last {
    /// The SHA-256 of the file that ran.
    sha: String,
    report: views::ReportView,
}

impl Session {
    pub fn new(run: RunFn, discard: DiscardFn, now: Arc<dyn Fn() -> i64 + Send + Sync>, places: Places) -> Session {
        Session {
            run,
            discard,
            now,
            places,
            current: Arc::default(),
            close_after: Arc::default(),
            last: Arc::default(),
        }
    }

    /// Starts the run of the file read as `raw`, previewed with the SHA-256 `previewed`; `typed` is the red file's
    /// sentence; `allow_model_switch` is "Run again, allowing another model" (E1). A refusal is Steplet's own
    /// sentence; once started, the run's end comes as progress. The run counts as going on from the first moment, so
    /// a Stop, or closing the window, while Steplet still checks the file is kept, and a second Run is refused.
    pub fn start(
        &self,
        raw: &[u8],
        previewed: &str,
        typed: Option<&str>,
        allow_model_switch: bool,
        ready: Result<Ready, String>,
        screen: Arc<dyn Screen>,
    ) -> Result<(), String> {
        let (stop, answers) = (StopSignal::new(), Answers::default());
        {
            let mut current = lock(&self.current);
            if current.is_some() {
                return Err("A file is running in this window already.".into());
            }
            *current = Some(Current { stop: stop.clone(), answers: answers.clone() });
            // A close asked for while an earlier run was ending is not carried over to this one.
            self.close_after.store(false, Ordering::SeqCst);
        }
        let started = self.begin(raw, previewed, typed, allow_model_switch, ready, screen.clone(), stop, answers);
        if started.is_err() {
            let close = {
                let mut current = lock(&self.current);
                *current = None;
                self.close_after.swap(false, Ordering::SeqCst)
            };
            if close {
                screen.close();
            }
        }
        started
    }

    #[allow(clippy::too_many_arguments)]
    fn begin(
        &self,
        raw: &[u8],
        previewed: &str,
        typed: Option<&str>,
        allow_model_switch: bool,
        ready: Result<Ready, String>,
        screen: Arc<dyn Screen>,
        stop: StopSignal,
        answers: Answers,
    ) -> Result<(), String> {
        let sha: String = Sha256::digest(raw).iter().map(|b| format!("{b:02x}")).collect();
        if sha != previewed {
            return Err("This file changed after you opened it. Open it again to see what it asks for now.".into());
        }
        // E1: another model only right after a model refused this file's run in this window, whatever the page asks.
        let offered = lock(&self.last).as_ref().is_some_and(|l| l.sha == sha && l.report.offers_model_switch);
        if allow_model_switch && !offered {
            return Err(
                "Steplet lets Claude Code switch models only right after a model refused this file's run.".into()
            );
        }
        let report = check(raw, (self.now)());
        // The same colour and Run status the preview showed: the file's findings and this computer's (row 22).
        let here = crate::open::here(&report, &pins::load_at(self.places.pins.as_deref()));
        let file = runnable(&report, &here, typed)?;
        let ready = ready?;
        // One run at a time on this computer, from now until this run is over: refused now if another window runs
        // one. While it is held, the marker below is this run's, and no other window's clean-up touches it.
        let one_run = RunLock::take().map_err(|e| views::run_failure(&e).1.reason)?;
        // A run of this window that crashed left its folders behind: they go before this run starts.
        if self.places.marker.exists() && self.clear_held().is_err() {
            return Err(
                "Steplet could not yet remove what an earlier run left in its Linux, so it did not start this \
                        one. Try again in a moment."
                    .into(),
            );
        }
        let run_id = format!("app-{}-{}", (self.now)().max(0), std::process::id());
        let paths = RunPaths::new(&ready.distro_id, &run_id)
            .map_err(|_| "Steplet's Linux is not set up as Steplet expects. Check again in Setup.".to_string())?;
        let policy = Policy::from_steplet(&file, paths.work_repo())
            .map_err(|_| "Steplet could not make the checker for this file, so nothing ran.".to_string())?;
        let repository = file.repository.clone().ok_or("This file names no repository.")?;
        let req = RunRequest {
            repo_url: repository.url.text.clone(),
            commit: repository.commit.text.clone(),
            task_text: task_text(&file, &repository.commit.text),
            steplet: file,
            max_turns: MAX_TURNS,
            deadline: RUN_LIMIT,
            allow_model_switch,
        };
        // The sender is pinned while Steplet holds the lock, so two windows never write the list at once.
        if let (Some(path), Some(SigStatus::Valid { fingerprint })) = (&self.places.pins, &report.signature) {
            // A list Steplet cannot read is kept as it is; the preview already said it cannot warn.
            let _ = pins::record(path, &req.steplet.sender.text, fingerprint);
        }
        let setup = RunSetup {
            gate_exe: ready.gate_exe,
            logs: self.places.logs.clone(),
            key: Some(ready.key),
            decision_deadline: DECISION_DEADLINE,
            model_url: None,
        };
        let marker = Marker { run_id: run_id.clone(), distro_id: ready.distro_id.clone() };
        write_marker(&self.places.marker, &marker).map_err(|e| {
            format!("Steplet could not note that a run is starting, so it did not start it. {}", views::windows_says(e))
        })?;

        let relay = Mutex::new(Relay::new(views::Limits { ask: ASK_LIMIT, run: RUN_LIMIT }));
        let to_screen = screen.clone();
        let live = Arc::new(Live {
            on_progress: Box::new(move |p| {
                // Sent while the relay is held: progress made on two threads at once (the gate's and the event
                // stream's) reaches the screen in the order the relay made it.
                let mut relay = lock(&relay);
                let out = relay.relay(&p);
                for view in out.views {
                    to_screen.send(view);
                }
                if out.flash {
                    to_screen.attention();
                }
            }),
            stop: stop.clone(),
            answers: Some(answers),
            ask_limit: ASK_LIMIT,
        });
        let (run, current, close_after, marker_path, last) = (
            self.run.clone(),
            self.current.clone(),
            self.close_after.clone(),
            self.places.marker.clone(),
            self.last.clone(),
        );
        std::thread::spawn(move || {
            let end = if stop.is_set() {
                // Stop came while Steplet was still checking the file: the agent never starts.
                ProgressView::Failed {
                    stage: "Starting the run".into(),
                    failure: views::failure("You stopped the run before it started. Nothing ran.", None),
                }
            } else {
                // A crash inside the run still ends it here, so the window is not left "running".
                let ran = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                    run(&one_run, &req, &paths, &policy, &setup, live)
                }));
                match ran {
                    Ok(Ok(report)) => ProgressView::Finished { report: views::report_view(&report) },
                    Ok(Err(e)) => {
                        let (stage, failure) = views::run_failure(&e);
                        ProgressView::Failed { stage, failure }
                    }
                    Err(_) => ProgressView::Failed {
                        stage: "Running the file".into(),
                        failure: views::failure(
                            "Something went wrong inside Steplet, so it ended the run. Its folders are removed before \
                             the next run, or the next time Steplet starts.",
                            None,
                        ),
                    },
                }
            };
            // After a crash the run's folders may still be there: the marker stays, so they are removed before the
            // next run. The marker goes before the lock, so no window ever finds a marker of a run that is over.
            let crashed = matches!(end, ProgressView::Failed { ref stage, .. } if stage == "Running the file");
            if !crashed {
                let _ = std::fs::remove_file(&marker_path);
            }
            drop(one_run);
            if let ProgressView::Finished { report } = &end {
                *lock(&last) = Some(Last { sha, report: report.clone() });
            }
            let close = {
                let mut current = lock(&current);
                *current = None;
                close_after.swap(false, Ordering::SeqCst)
            };
            screen.send(end);
            if close {
                screen.close();
            }
        });
        Ok(())
    }

    /// The receiver's answer to question `question`, bound to the request's fingerprint; false when no such
    /// question is open.
    pub fn answer(&self, question: u32, yes: bool, fingerprint: &str) -> bool {
        let reply = if yes { Reply::Yes { fingerprint: fingerprint.into() } } else { Reply::No };
        lock(&self.current).as_ref().is_some_and(|c| c.answers.give(question.into(), reply))
    }

    /// The receiver's Stop: the run stops as at a tripwire, and its folders are removed (D4). Pressed while the run
    /// still starts, it is kept, and the agent never starts.
    pub fn stop(&self) {
        if let Some(c) = lock(&self.current).as_ref() {
            c.stop.stop();
        }
    }

    /// The report of the last run that finished in this window.
    pub fn last_report(&self) -> Option<views::ReportView> {
        lock(&self.last).as_ref().map(|l| l.report.clone())
    }

    pub fn running(&self) -> bool {
        lock(&self.current).is_some()
    }

    /// Whether the window may close now; while a run goes on (or starts) it asks first.
    pub fn may_close(&self) -> bool {
        !self.running()
    }

    /// The receiver chose to close the window while the run goes on: Stop, and close once the run is over. False
    /// when no run goes on any more: the window closes now.
    pub fn stop_and_close(&self) -> bool {
        let current = lock(&self.current);
        let Some(c) = current.as_ref() else { return false };
        self.close_after.store(true, Ordering::SeqCst);
        c.stop.stop();
        true
    }

    /// At Steplet's start: a marker left behind means Steplet was closed or crashed during a run. That run is
    /// stopped and its folders removed, and the Open screen gets a note. A run that another Steplet window is doing
    /// right now holds the one-run lock, and its marker is left alone. Nothing to clear: `None`.
    pub fn clear_leftover(&self) -> Option<String> {
        if !self.places.marker.exists() {
            return None;
        }
        let Ok(_one) = RunLock::take() else { return None };
        match self.clear_held() {
            Ok(true) => Some(CLEARED.into()),
            Ok(false) => None,
            Err(said) => Some(said),
        }
    }

    /// With the one-run lock held: removes what the run named in the marker left, then the marker. `Ok(true)` when a
    /// run was removed, `Ok(false)` when the marker named no run Steplet could have started, and the note to show when
    /// the run could not be removed yet (the marker stays, for the next try).
    fn clear_held(&self) -> Result<bool, String> {
        let Ok(text) = std::fs::read(&self.places.marker) else { return Ok(false) };
        let paths =
            serde_json::from_slice::<Marker>(&text).ok().and_then(|m| RunPaths::new(&m.distro_id, &m.run_id).ok());
        let Some(paths) = paths else {
            // Not a marker Steplet wrote: nothing can be stopped by it, so it goes.
            let _ = std::fs::remove_file(&self.places.marker);
            return Ok(false);
        };
        match (self.discard)(&paths) {
            Ok(()) => {
                let _ = std::fs::remove_file(&self.places.marker);
                Ok(true)
            }
            Err(_) => Err(LEFT_BEHIND.into()),
        }
    }
}

/// The file in `report`, if it may run now: not blocked (expiry is checked with the time now), runnable on this
/// computer, and a red file only with its sentence typed exactly.
fn runnable(
    report: &FileReport,
    here: &views::Here,
    typed: Option<&str>,
) -> Result<steplet_core::header::Steplet, String> {
    let blocked = report.rating.outcome == Outcome::Blocked;
    if blocked && report.rating.findings.iter().any(|f| f.rule.id == TIME_EXPIRED.id) {
        return Err("This file expired after you opened it.".into());
    }
    let Some(file) = report.steplet.clone() else { return Err("This file cannot run.".into()) };
    match views::run_status(report, here).1 {
        RunStatus::Blocked => Err("This file cannot run.".into()),
        RunStatus::NotRunnable { .. } => Err("Steplet cannot run this file on this computer.".into()),
        RunStatus::Red if typed != Some(RED_SENTENCE) => {
            Err("This file is red: type the sentence exactly before you run it.".into())
        }
        RunStatus::Red | RunStatus::Amber | RunStatus::Green => Ok(file),
    }
}

fn lock<T>(m: &Mutex<T>) -> std::sync::MutexGuard<'_, T> {
    m.lock().unwrap_or_else(PoisonError::into_inner)
}

fn write_marker(path: &std::path::Path, m: &Marker) -> std::io::Result<()> {
    if let Some(dir) = path.parent() {
        std::fs::create_dir_all(dir)?;
    }
    std::fs::write(path, serde_json::to_vec(m).map_err(std::io::Error::other)?)
}

/// What the screen gets for one piece of the runner's progress.
#[derive(Debug, Default, PartialEq)]
pub struct Relayed {
    pub views: Vec<ProgressView>,
    /// A question came on screen: get the receiver's attention.
    pub flash: bool,
}

/// The runner's progress, as the screen shows it: each call's request kept so its result gets its exit code (H13),
/// and the questions one at a time, the later ones waiting their turn.
pub struct Relay {
    limits: views::Limits,
    calls: HashMap<String, (String, Value)>,
    showing: Option<u64>,
    waiting: VecDeque<Question>,
}

impl Relay {
    pub fn new(limits: views::Limits) -> Relay {
        Relay { limits, calls: HashMap::new(), showing: None, waiting: VecDeque::new() }
    }

    pub fn relay(&mut self, p: &Progress) -> Relayed {
        let mut out = Relayed::default();
        match p {
            Progress::Asking(q) if self.showing.is_none() => self.show(q, &mut out),
            Progress::Asking(q) => {
                self.waiting.push_back(q.clone());
                out.views.push(self.waiting_count());
            }
            Progress::Answered { question, .. } if self.showing == Some(*question) => {
                out.views.extend(views::progress_view(p, None, self.limits));
                self.showing = None;
                if let Some(next) = self.waiting.pop_front() {
                    self.show(&next, &mut out);
                    out.views.push(self.waiting_count());
                }
            }
            Progress::Answered { question, .. } => {
                // Closed before it came on screen (a stop, or the run's end): it only stops waiting.
                self.waiting.retain(|q| q.id != *question);
                out.views.push(self.waiting_count());
            }
            Progress::Event(e) => {
                let call = match e {
                    Event::ToolUse { id, name, input } => {
                        self.calls.insert(id.clone(), (name.clone(), input.clone()));
                        None
                    }
                    Event::ToolResult { id, .. } => self.calls.get(id).map(|(n, i)| (n.as_str(), i)),
                    _ => None,
                };
                out.views.extend(views::progress_view(p, call, self.limits));
                // A call that never reached the gate: its check, in the report's own words, as for any other call.
                if let Event::SkippedGate { id, harmless, .. } = e {
                    let check = views::check_view(&steplet_run::Check::NotAsked { harmless: *harmless });
                    out.views.push(ProgressView::Checked { call: id.clone(), check, reason: None });
                }
            }
            Progress::Prepared(_) | Progress::Checked { .. } => {
                out.views.extend(views::progress_view(p, None, self.limits));
            }
        }
        out
    }

    fn show(&mut self, q: &Question, out: &mut Relayed) {
        self.showing = Some(q.id);
        out.views.extend(views::progress_view(&Progress::Asking(q.clone()), None, self.limits));
        out.flash = true;
    }

    fn waiting_count(&self) -> ProgressView {
        ProgressView::Waiting { count: self.waiting.len() as u32 }
    }
}
