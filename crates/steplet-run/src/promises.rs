//! The run chooser's lines (phase 4 Task 5, R6, R20): what Steplet may promise about one run of a file, each line
//! with the names of the tests that back it (in this crate's `tests/`), worded no wider than they show. The network
//! line says "on this computer" only in WSL's default network mode, the one mode in which a program listening on
//! Windows was shown unreachable from inside the fence (phase 5 Task 4, choice K4).

use std::path::Path;

use steplet_core::header::Steplet;

/// WSL's network mode on this computer, as far as Steplet can tell from WSL's settings file. Only `Nat` (WSL's
/// default) has been tested from inside the fence; `Other` (for example `mirrored`, where WSL shares Windows' own
/// addresses) and `Unknown` keep the narrower network line.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NetworkMode {
    Nat,
    Other,
    Unknown,
}

/// The network mode the text of `.wslconfig` sets; `None` when there is no such file. The file is read line by line
/// as WSL's settings are written: blank lines, comments (`#` or `;`), `[section]`, and `key = value`. Every
/// `networkingMode` key counts, wherever it stands and in any letter case: `Nat` only when each one says `nat` (or
/// none is there, WSL's default), `Other` when any says something else, and `Unknown` for a line that is none of
/// these. Steplet never changes the file.
pub fn network_mode(wslconfig: Option<&str>) -> NetworkMode {
    let Some(text) = wslconfig else { return NetworkMode::Nat };
    let mut mode = NetworkMode::Nat;
    for line in text.lines().map(str::trim) {
        if line.is_empty() || line.starts_with(['#', ';']) || (line.starts_with('[') && line.ends_with(']')) {
            continue;
        }
        let Some((key, value)) = line.split_once('=') else { return NetworkMode::Unknown };
        let key = key.trim();
        if key.is_empty() {
            return NetworkMode::Unknown;
        }
        if key.eq_ignore_ascii_case("networkingMode") && !value.trim().eq_ignore_ascii_case("nat") {
            mode = NetworkMode::Other;
        }
    }
    mode
}

/// The network mode set by `.wslconfig` in the Windows user's folder (`%USERPROFILE%`, given as `user_folder`).
/// No such file is WSL's default; a file Steplet cannot read, or whose text is not UTF-8, is `Unknown`, and so is a
/// missing user folder.
pub fn read_network_mode(user_folder: Option<&Path>) -> NetworkMode {
    let Some(folder) = user_folder else { return NetworkMode::Unknown };
    match std::fs::read(folder.join(".wslconfig")) {
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => network_mode(None),
        Err(_) => NetworkMode::Unknown,
        Ok(bytes) => match std::str::from_utf8(bytes.strip_prefix(b"\xEF\xBB\xBF").unwrap_or(&bytes)) {
            Ok(text) => network_mode(Some(text)),
            Err(_) => NetworkMode::Unknown,
        },
    }
}

/// One line of the chooser. `items` are the file's own words the line lists (its web sites), shown as file text.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Promise {
    pub text: &'static str,
    pub items: Vec<String>,
    pub tests: &'static [&'static str],
}

const CHECKS: Promise = Promise {
    text: "Steplet checks the agent's commands that change something, its file writes and its web fetches before they \
           happen. If Steplet's checker does not decide in time, or its gate stops working, the action is refused.",
    items: Vec::new(),
    tests: &[
        "a_declared_command_runs_and_what_the_file_did_not_ask_for_does_not",
        "a_gate_that_dies_mid_request_refuses_it_and_the_restarted_gate_answers_the_next",
        "a_decision_that_never_comes_is_refused_at_the_deadline_in_a_real_run",
        "a_writing_command_made_to_skip_the_gate_stops_the_run",
        "a_write_into_steplets_own_folder_is_refused_before_the_gate",
        "a_web_fetch_is_allowed_only_to_a_site_the_file_declares",
    ],
};

/// The agent's web fetches: a site the file does not declare is asked about (R7), so the line says so.
const FETCH_TESTS: &[&str] =
    &["a_web_fetch_is_allowed_only_to_a_site_the_file_declares", "a_yes_allows_exactly_the_request_shown"];

const WRITES: Promise = Promise {
    text: "The agent and the programs this task runs can change files only inside the working copy (the programs also \
           in the run's own temporary folder, which Steplet deletes after the run).",
    items: Vec::new(),
    tests: &[
        "every_try_of_the_hostile_fixture_is_contained",
        "a_write_into_steplets_own_folder_is_refused_before_the_gate",
        "the_run_s_temporary_folder_holds_no_key_or_token_and_is_gone_after",
    ],
};

const YOUR_FILES: Promise = Promise {
    text: "Your own files are not inside Steplet's Linux, where this task runs.",
    items: Vec::new(),
    tests: &["use_case_1_runs_with_none_of_the_receiver_s_files_present"],
};

const WINDOWS_PROGRAMS: Promise = Promise {
    text: "Programs this task runs cannot start Windows programs.",
    items: Vec::new(),
    tests: &["every_try_of_the_hostile_fixture_is_contained"],
};

/// The tests behind the network line: a listener in Steplet's Linux and the internet (the fixture); a file that
/// declares a site whose name leads to this computer is refused (so even the narrower line holds); the one site a
/// file declares; and, for the wider line, a program listening on Windows reached by address.
const REFUSED: &str = "a_run_that_lists_a_site_leading_to_this_computer_is_refused";
const WINDOWS_LISTENER: &str = "a_program_listening_on_windows_cannot_be_reached";
const NETWORK_TESTS: &[&str] = &["every_try_of_the_hostile_fixture_is_contained", REFUSED];
const NETWORK_TESTS_ON_THIS_COMPUTER: &[&str] =
    &["every_try_of_the_hostile_fixture_is_contained", REFUSED, WINDOWS_LISTENER];
const SITES_TESTS: &[&str] =
    &["every_try_of_the_hostile_fixture_is_contained", "a_declared_site_is_the_only_one_a_program_can_reach", REFUSED];
const SITES_TESTS_ON_THIS_COMPUTER: &[&str] = &[
    "every_try_of_the_hostile_fixture_is_contained",
    "a_declared_site_is_the_only_one_a_program_can_reach",
    REFUSED,
    WINDOWS_LISTENER,
];

/// The lines Steplet may show in the run chooser for a run of `s`, in order, on a computer whose WSL runs in `network`
/// mode.
pub fn promises(s: &Steplet, network: NetworkMode) -> Vec<Promise> {
    let sites: Vec<String> = s.allow.domains.iter().map(|d| d.text.clone()).collect();
    let on_this_computer = network == NetworkMode::Nat;
    let (fetch, network) = if sites.is_empty() {
        (
            Promise {
                text: "The agent cannot fetch any web address, unless you say yes to that exact address.",
                items: Vec::new(),
                tests: FETCH_TESTS,
            },
            if on_this_computer {
                Promise {
                    text: "Programs this task runs cannot connect to the internet, or to other programs on this \
                           computer.",
                    items: Vec::new(),
                    tests: NETWORK_TESTS_ON_THIS_COMPUTER,
                }
            } else {
                Promise {
                    text: "Programs this task runs cannot connect to the internet, or to other programs in Steplet's \
                           Linux.",
                    items: Vec::new(),
                    tests: NETWORK_TESTS,
                }
            },
        )
    } else {
        (
            Promise {
                text: "The agent can fetch only these web addresses, and another one only if you say yes to that \
                       exact address:",
                items: sites.clone(),
                tests: FETCH_TESTS,
            },
            if on_this_computer {
                Promise {
                    text: "Programs this task runs can connect only to these sites: not to the rest of the internet, \
                           or to other programs on this computer.",
                    items: sites,
                    tests: SITES_TESTS_ON_THIS_COMPUTER,
                }
            } else {
                Promise {
                    text: "Programs this task runs can connect only to these sites: not to the rest of the internet, \
                           or to other programs in Steplet's Linux.",
                    items: sites,
                    tests: SITES_TESTS,
                }
            },
        )
    };
    vec![CHECKS, fetch, WRITES, YOUR_FILES, network, WINDOWS_PROGRAMS]
}
