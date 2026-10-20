//! Setting up Steplet's own Linux (phase 4 Task 6): product code now, not a test script. `look` only reads; `install`
//! and `remove` change things, and only Steplet's own Linux.
//!
//! The tests marked `#[ignore]` need WSL: `look_finds_steplets_linux_ready` reads the installed `steplet`, and
//! `install_makes_a_ready_linux_from_nothing` installs a second Linux, `steplet-setup-test`, from the internet (about
//! 1 to 2 GB) and removes it again. Run that one only with the owner's yes (the plan's Z3).

mod support;

use std::time::Instant;

use steplet_run::setup::{self, State, Wsl};

/// What `wsl.exe --version` printed on this laptop on 18 Oct 2026 (the lines unchanged).
const THIS_LAPTOP: &str = "WSL version: 2.6.3.0\r\nKernel version: 6.6.87.2-1\r\nWSLg version: 1.0.71\r\nMSRDC version: \
                           1.2.6353\r\nDirect3D version: 1.611.1-81528511\r\nDXCore version: \
                           10.0.26100.1-240331-1435.ge-release\r\nWindows version: 10.0.26200.9711\r\n";

fn utf16(text: &str) -> Vec<u8> {
    text.encode_utf16().flat_map(u16::to_le_bytes).collect()
}

#[test]
fn reads_wsl_version_output() {
    let ready = Wsl::Ready { version: "2.6.3.0".into() };
    assert_eq!(setup::read_wsl_version(THIS_LAPTOP.as_bytes()), ready, "as UTF-8");
    assert_eq!(setup::read_wsl_version(&utf16(THIS_LAPTOP)), ready, "in UTF-16, as wsl.exe writes it");
    // Made-up test data: a German Windows words the first line differently; the number is what counts.
    assert_eq!(setup::read_wsl_version(&utf16("WSL-Version: 2.6.3.0\r\nKernelversion: 6.6.87.2-1\r\n")), ready);
    assert_eq!(
        setup::read_wsl_version(&utf16("WSL version: 2.4.13.0\r\n")),
        Wsl::TooOld { version: "2.4.13.0".into() },
        "older than the version Steplet was tested on"
    );
    // Made-up test data: the inbox wsl.exe of a Windows without the WSL package does not know --version.
    let inbox = "Invalid command line option: --version\r\nUsage: wsl.exe [Argument] [Options...] [CommandLine]\r\n";
    assert_eq!(setup::read_wsl_version(&utf16(inbox)), Wsl::Missing);
    assert_eq!(setup::read_wsl_version(b""), Wsl::Missing);
}

/// `look.sh` checks every package `setup.sh` installs, so a Linux whose setup stopped half way is not called ready.
#[test]
fn look_checks_every_package_setup_installs() {
    let joined = include_str!("../src/setup.sh").replace("\\\n", " ");
    let install = joined.lines().find(|l| l.starts_with("logged ") && l.contains(" apt-get install ")).unwrap();
    let words = install.split_whitespace();
    let installed: Vec<&str> = words.skip_while(|w| *w != "git").take_while(|w| *w != ">").collect();
    let look = include_str!("../src/look.sh");
    let checks = look.lines().map(str::trim).find(|l| l.starts_with("for p in git ")).unwrap();
    let checked: Vec<&str> = checks["for p in ".len()..].split(';').next().unwrap().split_whitespace().collect();
    assert!(installed.len() >= 10, "{installed:?}");
    assert_eq!(checked, installed);
}

#[test]
fn the_versions_are_the_tested_ones() {
    assert_eq!((setup::LINUX, setup::CLAUDE_VERSION, setup::SRT_VERSION), ("steplet", "2.1.220", "0.0.77"));
}

#[test]
#[ignore = "needs Steplet's Linux"]
fn look_finds_steplets_linux_ready() {
    let gate = std::fs::read(support::gate_program()).unwrap();
    let distro = support::steplet_linux();
    steplet_run::install_gate(&distro, &gate).unwrap();
    let state = setup::look(setup::LINUX, &gate);
    println!("--- {state:#?}");
    assert!(matches!(state.wsl, Wsl::Ready { .. }), "{state:?}");
    assert_eq!(state.linux.as_deref(), Some(distro.as_str()));
    assert_eq!((state.missing, state.trouble), (Vec::<String>::new(), None));
    assert!(state.gate);
    // A gate program Steplet has not installed (made-up bytes) is said to be missing.
    let other = setup::look(setup::LINUX, b"made-up bytes, not a program");
    assert!(!other.gate && other.missing.is_empty() && other.trouble.is_none(), "{other:?}");
}

/// Task 6's review: `setup.sh` keeps the package tools' long output in log files; when one of them fails, the end of
/// its log is what the receiver sees, and the setup stops there. The script's own helper is run alone, as the
/// `steplet` account, on a made-up command that fails.
#[test]
#[ignore = "needs Steplet's Linux"]
fn a_failed_setup_step_shows_the_end_of_its_log() {
    let setup = include_str!("../src/setup.sh");
    let start = setup.find("\nlogged() {\n").expect("setup.sh has its helper `logged`") + 1;
    let end = start + setup[start..].find("\n}\n").unwrap() + 3;
    let log = "/tmp/steplet-logged-test.txt";
    let script = format!(
        "set -eu\n{}logged {log} sh -c 'echo first; echo the made-up reason it failed; exit 3'\necho not reached\n",
        &setup[start..end]
    );
    let distro = support::steplet_linux();
    let out = steplet_run::wsl::run(&distro, "steplet", &["/bin/bash", "-s"], script.as_bytes()).unwrap();
    steplet_run::wsl::run(&distro, "steplet", &["/usr/bin/rm", "-f", log], b"").unwrap();
    println!("--- {out:#?}");
    assert_eq!(out.status, Some(1));
    assert!(out.stderr.contains("exit code 3") && out.stderr.contains("the made-up reason it failed"), "{out:?}");
    assert!(!out.stdout.contains("not reached"), "the setup stops there");
}

/// Task 6's review: "Try again" runs the whole setup again on a Linux that is already there; the download is skipped.
/// Run on Steplet's own Linux: every step can be repeated, and it is still ready afterwards.
#[test]
#[ignore = "runs the setup again inside Steplet's Linux: needs the network, about 2 minutes"]
fn setup_runs_again_on_a_linux_that_is_there() {
    let gate = std::fs::read(support::gate_program()).unwrap();
    // Where Steplet's Linux is; not used, since it is there.
    let place = std::path::PathBuf::from(std::env::var_os("LOCALAPPDATA").unwrap()).join("Steplet").join("wsl");
    let said = std::cell::RefCell::new(Vec::<String>::new());
    let start = Instant::now();
    let again = setup::install(setup::LINUX, &place, &gate, &|line| {
        println!("setup: {line}");
        said.borrow_mut().push(line.to_owned());
    });
    println!("--- took {:.0} s: {again:?}", start.elapsed().as_secs_f64());
    assert_eq!(again.unwrap(), support::steplet_linux());
    assert!(!said.borrow().iter().any(|l| l.starts_with("Downloading")), "nothing was downloaded again");
    let state = setup::look(setup::LINUX, &gate);
    println!("--- {state:#?}");
    assert_eq!((state.missing, state.trouble, state.gate), (Vec::<String>::new(), None, true));
}

#[test]
#[ignore = "installs a second Linux from the internet: run only with the owner's yes (Z3)"]
fn install_makes_a_ready_linux_from_nothing() {
    const NAME: &str = "steplet-setup-test";
    let gate = std::fs::read(support::gate_program()).unwrap();
    let location =
        std::path::PathBuf::from(std::env::var_os("LOCALAPPDATA").unwrap()).join("Steplet").join("wsl-setup-test");
    assert_eq!(setup::look(NAME, &gate).linux, None, "not installed before the test");
    let start = Instant::now();
    let installed = setup::install(NAME, &location, &gate, &|line| println!("setup: {line}"));
    let took = start.elapsed();
    println!("--- install took {:.0} s: {installed:?}", took.as_secs_f64());
    let state: State = setup::look(NAME, &gate);
    println!("--- {state:#?}");
    let removed = setup::remove(NAME);
    println!("--- removed: {removed:?}");
    let id = installed.unwrap();
    assert_eq!(state.linux.as_deref(), Some(id.as_str()));
    assert_eq!(state.missing, Vec::<String>::new());
    assert!(state.gate);
    removed.unwrap();
    assert_eq!(setup::look(NAME, &gate).linux, None, "gone again");
}
