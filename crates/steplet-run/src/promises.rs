//! The run chooser's lines (phase 4 Task 5, R6, R20): what Steplet may promise about one run of a file, each line
//! with the names of the tests that back it (in this crate's `tests/`), worded no wider than they show. A line that
//! needs a test not yet run (a program listening on the Windows side, so "on this computer") is not offered.

use steplet_core::header::Steplet;

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

/// The lines Steplet may show in the run chooser for a run of `s`, in order.
pub fn promises(s: &Steplet) -> Vec<Promise> {
    let sites: Vec<String> = s.allow.domains.iter().map(|d| d.text.clone()).collect();
    let (fetch, network) = if sites.is_empty() {
        (
            Promise {
                text: "The agent cannot fetch any web address, unless you say yes to that exact address.",
                items: Vec::new(),
                tests: FETCH_TESTS,
            },
            Promise {
                text: "Programs this task runs cannot connect to the internet, or to other programs in Steplet's Linux.",
                items: Vec::new(),
                tests: &["every_try_of_the_hostile_fixture_is_contained"],
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
            Promise {
                text: "Programs this task runs can connect only to these sites: not to the rest of the internet, or to \
                       other programs in Steplet's Linux.",
                items: sites,
                tests: &[
                    "every_try_of_the_hostile_fixture_is_contained",
                    "a_declared_site_is_the_only_one_a_program_can_reach",
                ],
            },
        )
    };
    vec![CHECKS, fetch, WRITES, YOUR_FILES, network, WINDOWS_PROGRAMS]
}
