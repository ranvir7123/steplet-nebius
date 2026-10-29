//! Setup (Task 11): the glue between the Setup screen and `steplet_run::setup`. What is set up comes from the
//! runner's `look`, whether a key is saved from Credential Manager (`key`), and the switch from the registry
//! (`register`). Steplet's Linux is `steplet_run::setup::LINUX`, its disk in `%LOCALAPPDATA%\Steplet\wsl`.

use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};

use steplet_run::setup::{self as run_setup, LINUX, State};
use steplet_run::{RunError, RunLock};

use crate::key;
use crate::register;
use crate::views::{self, FailureView, SetupView, Shown, Switch};

/// The gate program's file name: it sits next to Steplet's own program (H8: one gate per build of Steplet).
const GATE_FILE: &str = "steplet-gate.exe";

/// This program, as Windows would start it.
pub fn this_program() -> Option<PathBuf> {
    std::env::current_exe().ok()
}

/// The gate program of this build, read from beside this program.
pub fn gate_program() -> Result<Vec<u8>, String> {
    let path = this_program()
        .map(|p| p.with_file_name(GATE_FILE))
        .ok_or("Windows did not say where Steplet's own program is, so Steplet cannot find its gate program.")?;
    std::fs::read(&path).map_err(|e| {
        format!(
            "Steplet could not read its gate program ({GATE_FILE}, next to Steplet). {}",
            crate::views::windows_says(e)
        )
    })
}

/// Where Steplet's Linux keeps its disk.
fn install_folder() -> Result<PathBuf, String> {
    std::env::var_os("LOCALAPPDATA")
        .map(|d| PathBuf::from(d).join("Steplet").join("wsl"))
        .ok_or_else(|| "Windows did not say where this user's app data goes.".into())
}

/// The switch for this program.
pub fn switch() -> Switch {
    match this_program() {
        Some(exe) => register::state(register::EXT, register::PROG_ID, &exe),
        None => Switch::Off,
    }
}

/// What is set up on this computer. It changes nothing; it may start Steplet's Linux to look inside.
pub fn look() -> State {
    // Without its gate program, the look still reports everything else; the gate line then says it is not there.
    let gate = gate_program().unwrap_or_default();
    run_setup::look(LINUX, &gate)
}

/// The Setup screen's data: the checklist from `state`, which keys are saved in `entries`, and the switch. It holds
/// whether each key is saved, never a key.
pub fn view(state: &State, entries: key::Entries, switch: Switch) -> SetupView {
    let saved = |target: &str| matches!(key::load(target), Ok(Some(_)));
    let keys = views::Keys { nebius: saved(entries.nebius), anthropic: saved(entries.anthropic) };
    views::setup_view(state, keys, switch)
}

/// The Setup screen as it is now on this computer.
pub fn view_here() -> SetupView {
    view(&look(), key::ENTRIES, switch())
}

/// Installs Steplet's Linux, or finishes an install that stopped half way. Each line it prints goes to `line`, as
/// file text (it is what Linux programs print).
pub fn install(line: &dyn Fn(Shown)) -> Result<(), FailureView> {
    let words = |reason: String| views::failure(&reason, None);
    let gate = gate_program().map_err(words)?;
    let folder = install_folder().map_err(words)?;
    run_setup::install(LINUX, &folder, &gate, &|l| line(views::setup_line(l)))
        .map(|_| ())
        .map_err(|e| views::setup_failure(&e))
}

/// What a run needs from setup, if everything is there: Steplet's Linux by its id, this build's gate program on its
/// disk, and the receiver's key. Otherwise Steplet's sentence saying what to do.
pub fn ready_for_run(state: &State) -> Result<crate::session::Ready, String> {
    if !view(state, key::ENTRIES, Switch::Off).ready {
        return Err("Set up Steplet first: Setup shows what is missing.".into());
    }
    let distro_id = state.linux.clone().ok_or("Set up Steplet first: its own Linux is not installed.")?;
    let gate = gate_program()?;
    let key = match key::load(key::ANTHROPIC_TARGET) {
        Ok(Some(key)) => key,
        Ok(None) => return Err("Steplet needs your Anthropic key. Add it in Setup.".into()),
        Err(e) => {
            return Err(format!(
                "Steplet could not read your key from Windows' Credential Manager. {}",
                crate::views::windows_says(e)
            ));
        }
    };
    Ok(crate::session::Ready {
        distro_id,
        gate_exe: format!("/opt/steplet/gate/{}", steplet_run::gate_file_name(&gate)),
        key,
    })
}

/// One install or removal of Steplet's Linux at a time, and never while a file runs: each double-clicked file opens
/// in a Steplet program of its own (S9), so the work also holds Windows' one-run lock, which every window's run takes.
/// The mark that one is going on is cleared however the work ends, a crash of the work included, so a crash never
/// blocks the next try.
#[derive(Debug, Default)]
pub struct OneAtATime(AtomicBool);

impl OneAtATime {
    /// Runs `work`, or refuses at once while another one runs, here or in another window, or a file runs.
    pub fn run<T>(&self, work: impl FnOnce() -> Result<T, FailureView>) -> Result<T, FailureView> {
        if self.0.swap(true, Ordering::SeqCst) {
            return Err(views::failure("Steplet is already changing its Linux. Wait for that to finish.", None));
        }
        struct Clear<'a>(&'a AtomicBool);
        impl Drop for Clear<'_> {
            fn drop(&mut self) {
                self.0.store(false, Ordering::SeqCst);
            }
        }
        let _clear = Clear(&self.0);
        let _no_run = RunLock::take().map_err(|e| match e {
            RunError::Lock(why) => views::failure(
                &format!(
                    "Windows would not let Steplet make sure that no file runs while it changes its Linux, so \
                     nothing changed. {}",
                    views::windows_says(why)
                ),
                None,
            ),
            _ => views::failure(views::ANOTHER_WINDOW, None),
        })?;
        work()
    }
}

/// Removes Steplet's Linux and everything in it ("Start over"). Nothing of the receiver's is in it.
pub fn remove() -> Result<(), FailureView> {
    run_setup::remove(LINUX).map_err(|e| views::setup_failure(&e))
}
