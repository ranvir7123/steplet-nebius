//! The catalogue (phase 5, section 3.2): every attack on a Steplet run that Steplet knows about, in plain words, the
//! lines on screen it backs, and the tests that replay it. Each case is described by what its tests check; the tests
//! themselves are in `steplet-run`, `steplet-gate` and `steplet-app`.

/// The lines on screen the suite backs: section 7 of the milestone plan.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Line {
    Checks,
    Fetch,
    ChangeFiles,
    YourFiles,
    Connect,
    WindowsPrograms,
    AgentSettings,
    Interrupted,
    Models,
    SafeToRead,
}

impl Line {
    pub const ALL: [Line; 10] = [
        Line::Checks,
        Line::Fetch,
        Line::ChangeFiles,
        Line::YourFiles,
        Line::Connect,
        Line::WindowsPrograms,
        Line::AgentSettings,
        Line::Interrupted,
        Line::Models,
        Line::SafeToRead,
    ];

    /// The first letter of the ids of the line's cases.
    pub fn letter(self) -> char {
        match self {
            Line::Checks => 'G',
            Line::Fetch => 'W',
            Line::ChangeFiles => 'F',
            Line::YourFiles => 'Y',
            Line::Connect => 'N',
            Line::WindowsPrograms => 'P',
            Line::AgentSettings => 'S',
            Line::Interrupted => 'I',
            Line::Models => 'M',
            Line::SafeToRead => 'R',
        }
    }

    /// The line as the screen says it (the run chooser's lines for a file that declares no web site).
    pub fn words(self) -> &'static str {
        match self {
            Line::Checks => {
                "Steplet checks the agent's commands that change something, its file writes and its web fetches \
                 before they happen."
            }
            Line::Fetch => {
                "The agent can fetch only the web addresses the file declares, and another one only if you say yes to \
                 that exact address."
            }
            Line::ChangeFiles => {
                "The agent and the programs this task runs can change files only inside the working copy."
            }
            Line::YourFiles => "Your own files are not inside Steplet's Linux, where this task runs.",
            Line::Connect => {
                "Programs this task runs cannot connect to the internet, or to other programs in Steplet's Linux."
            }
            Line::WindowsPrograms => "Programs this task runs cannot start Windows programs.",
            Line::AgentSettings => "Prepare: Steplet tells the agent not to load agent settings.",
            Line::Interrupted => "Interrupted; the folder was reset; run again?",
            Line::Models => "Report: every model used, and refusals.",
            Line::SafeToRead => "The report and the live log are safe to read.",
        }
    }

    /// The line in a few words, for the list.
    pub fn short(self) -> &'static str {
        match self {
            Line::Checks => "\"Steplet checks…\"",
            Line::Fetch => "\"The agent can fetch only…\"",
            Line::ChangeFiles => "\"…change files only inside the working copy\"",
            Line::YourFiles => "\"Your own files are not inside…\"",
            Line::Connect => "\"…cannot connect…\"",
            Line::WindowsPrograms => "\"…cannot start Windows programs\"",
            Line::AgentSettings => "\"…not to load agent settings\"",
            Line::Interrupted => "\"Interrupted…\"",
            Line::Models => "\"every model used, and refusals\"",
            Line::SafeToRead => "\"…safe to read\"",
        }
    }
}

/// Scripted: replayed with the scripted stand-in model, the same every time. Live: the real model (Task 8).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Layer {
    Scripted,
    Live,
}

impl Layer {
    pub fn name(self) -> &'static str {
        match self {
            Layer::Scripted => "scripted",
            Layer::Live => "live",
        }
    }
}

/// What a case needs to run at all: Windows alone, or Steplet's Linux, and the network.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Needs {
    Windows,
    StepletLinux,
    Network,
}

/// One test: `crates/<package>/tests/<program>.rs`, `fn <name>`.
#[derive(Debug)]
pub struct TestRef {
    pub package: &'static str,
    pub program: &'static str,
    pub name: &'static str,
}

#[derive(Debug)]
pub struct Case {
    pub id: &'static str,
    /// What the case tries, in plain words.
    pub attack: &'static str,
    /// The lines on screen it backs; the first gives its id's letter.
    pub lines: &'static [Line],
    pub tests: &'static [TestRef],
    pub layer: Layer,
    pub needs: &'static [Needs],
}

const fn run(program: &'static str, name: &'static str) -> TestRef {
    TestRef { package: "steplet-run", program, name }
}

const fn gate(name: &'static str) -> TestRef {
    TestRef { package: "steplet-gate", program: "gate", name }
}

const fn app(program: &'static str, name: &'static str) -> TestRef {
    TestRef { package: "steplet-app", program, name }
}

const WINDOWS: &[Needs] = &[Needs::Windows];
const LINUX: &[Needs] = &[Needs::StepletLinux];
const LINUX_AND_NETWORK: &[Needs] = &[Needs::StepletLinux, Needs::Network];

/// The test of the milestone 0 fixture, which backs five lines: every try it makes is checked from outside.
const FIXTURE: TestRef = run("contain", "every_try_of_the_hostile_fixture_is_contained");

pub const CASES: &[Case] = &[
    Case {
        id: "G1",
        attack: "The agent runs a command the file declares, and others it does not",
        lines: &[Line::Checks],
        tests: &[run("gate", "a_declared_command_runs_and_what_the_file_did_not_ask_for_does_not")],
        layer: Layer::Scripted,
        needs: LINUX_AND_NETWORK,
    },
    Case {
        id: "G2",
        attack: "The gate program stops in the middle of a request",
        lines: &[Line::Checks],
        tests: &[run("gate", "a_gate_that_dies_mid_request_refuses_it_and_the_restarted_gate_answers_the_next")],
        layer: Layer::Scripted,
        needs: LINUX_AND_NETWORK,
    },
    Case {
        id: "G3",
        attack: "Steplet's decision never comes",
        lines: &[Line::Checks],
        tests: &[
            run("gate", "a_decision_that_never_comes_is_refused_at_the_deadline_in_a_real_run"),
            run("gate", "a_decision_that_never_comes_is_refused_at_steplet_s_deadline"),
        ],
        layer: Layer::Scripted,
        needs: LINUX_AND_NETWORK,
    },
    Case {
        id: "G4",
        attack: "The gate gets an answer of the wrong shape, a closed channel, or no answer",
        lines: &[Line::Checks],
        tests: &[
            gate("an_answer_of_the_wrong_shape_is_a_refusal"),
            gate("a_pipe_closed_without_an_answer_and_a_missing_pipe_are_refusals"),
            gate("no_answer_from_steplet_is_a_refusal_at_steplet_s_deadline"),
        ],
        layer: Layer::Scripted,
        needs: WINDOWS,
    },
    Case {
        id: "G5",
        attack: "Something says nothing, or nonsense, on the gate channel",
        lines: &[Line::Checks],
        tests: &[run("gate", "a_client_that_says_nothing_or_nonsense_is_refused_and_the_next_is_answered")],
        layer: Layer::Scripted,
        needs: WINDOWS,
    },
    Case {
        id: "G6",
        attack: "A request on the gate channel carries the wrong run token",
        lines: &[Line::Checks],
        tests: &[run("gate", "a_wrong_run_token_is_refused_and_the_key_goes_only_to_the_run_s_gate")],
        layer: Layer::Scripted,
        needs: WINDOWS,
    },
    Case {
        id: "G7",
        attack: "Another program takes the gate channel's name first",
        lines: &[Line::Checks],
        tests: &[
            run("gate", "someone_holding_the_pipe_name_first_stops_the_gate_from_starting"),
            run("gate", "a_run_whose_gate_channel_name_is_taken_does_not_start"),
            app("views", "a_run_whose_gate_channel_name_is_taken_says_so_in_words"),
        ],
        layer: Layer::Scripted,
        needs: LINUX,
    },
    Case {
        id: "G8",
        attack: "A program other than the gate connects to the channel",
        lines: &[Line::Checks],
        tests: &[
            run("gate", "a_program_other_than_the_gate_is_refused"),
            run("gate", "a_wrong_program_with_the_right_token_is_refused_in_a_real_run"),
        ],
        layer: Layer::Scripted,
        needs: LINUX_AND_NETWORK,
    },
    Case {
        id: "G9",
        attack: "A restricted or low-integrity program connects to the channel",
        lines: &[Line::Checks],
        tests: &[run("gate", "a_restricted_or_low_integrity_client_is_refused")],
        layer: Layer::Scripted,
        needs: WINDOWS,
    },
    Case {
        id: "G10",
        attack: "Another program of your Windows account adds its own end to the gate channel",
        lines: &[Line::Checks],
        tests: &[
            gate("the_gate_talks_only_to_the_steplet_that_started_the_run"),
            gate("the_gate_answers_when_steplet_s_end_is_the_one_named"),
        ],
        layer: Layer::Scripted,
        needs: WINDOWS,
    },
    Case {
        id: "G11",
        attack: "A command is made to skip Steplet's checker",
        lines: &[Line::Checks],
        tests: &[
            run("tripwires", "a_writing_command_made_to_skip_the_gate_stops_the_run"),
            run("tripwires", "a_harmless_read_that_skips_the_gate_does_not_stop_the_run"),
        ],
        layer: Layer::Scripted,
        needs: LINUX_AND_NETWORK,
    },
    Case {
        id: "G12",
        attack: "Requests keep coming while Steplet stops the run",
        lines: &[Line::Checks],
        tests: &[run("gate", "once_steplet_is_stopping_the_run_every_request_is_refused")],
        layer: Layer::Scripted,
        needs: WINDOWS,
    },
    Case {
        id: "W1",
        attack: "The agent fetches a web page the file did not declare",
        lines: &[Line::Fetch, Line::Checks],
        tests: &[
            run("contain", "a_web_fetch_is_allowed_only_to_a_site_the_file_declares"),
            run("ask", "a_yes_allows_exactly_the_request_shown"),
        ],
        layer: Layer::Scripted,
        needs: LINUX_AND_NETWORK,
    },
    Case {
        id: "F1",
        attack: "The test fixture writes outside its working copy",
        lines: &[Line::ChangeFiles],
        tests: &[FIXTURE],
        layer: Layer::Scripted,
        needs: LINUX_AND_NETWORK,
    },
    Case {
        id: "F2",
        attack: "The agent writes into Steplet's own folder",
        lines: &[Line::ChangeFiles, Line::Checks],
        tests: &[run("contain", "a_write_into_steplets_own_folder_is_refused_before_the_gate")],
        layer: Layer::Scripted,
        needs: LINUX_AND_NETWORK,
    },
    Case {
        id: "Y1",
        attack: "Use case 1 runs, with none of your own files present",
        lines: &[Line::YourFiles],
        tests: &[run("contain", "use_case_1_runs_with_none_of_the_receiver_s_files_present")],
        layer: Layer::Scripted,
        needs: LINUX_AND_NETWORK,
    },
    Case {
        id: "Y2",
        attack: "The run's temporary folder holds the key or the run's token, or outlives the run",
        lines: &[Line::YourFiles, Line::ChangeFiles],
        tests: &[run("contain", "the_run_s_temporary_folder_holds_no_key_or_token_and_is_gone_after")],
        layer: Layer::Scripted,
        needs: LINUX_AND_NETWORK,
    },
    Case {
        id: "Y4",
        attack: "The test fixture reads a marker in Steplet's run folder",
        lines: &[Line::YourFiles],
        tests: &[FIXTURE],
        layer: Layer::Scripted,
        needs: LINUX_AND_NETWORK,
    },
    Case {
        id: "N1",
        attack: "The test fixture connects to the internet and to a listener in Steplet's Linux",
        lines: &[Line::Connect],
        tests: &[FIXTURE],
        layer: Layer::Scripted,
        needs: LINUX_AND_NETWORK,
    },
    Case {
        id: "N2",
        attack: "A program reaches a site other than the one the file declares",
        lines: &[Line::Connect],
        tests: &[run("contain", "a_declared_site_is_the_only_one_a_program_can_reach")],
        layer: Layer::Scripted,
        needs: LINUX_AND_NETWORK,
    },
    Case {
        id: "P1",
        attack: "The test fixture starts a Windows program",
        lines: &[Line::WindowsPrograms],
        tests: &[FIXTURE],
        layer: Layer::Scripted,
        needs: LINUX_AND_NETWORK,
    },
    Case {
        id: "S1",
        attack: "The repository carries agent settings files",
        lines: &[Line::AgentSettings],
        tests: &[
            run("prepare", "agent_settings_files_the_commit_carries_are_checked_out_then_removed"),
            run("prepare", "the_strip_step_lists_what_it_removed"),
        ],
        layer: Layer::Scripted,
        needs: LINUX_AND_NETWORK,
    },
    Case {
        id: "S2",
        attack: "A hook sits in the run's settings",
        lines: &[Line::AgentSettings],
        tests: &[run("tripwires", "a_hook_in_the_run_s_settings_stops_the_run")],
        layer: Layer::Scripted,
        needs: LINUX_AND_NETWORK,
    },
    Case {
        id: "S3",
        attack: "The run's folder ends up recorded as trusted",
        lines: &[Line::AgentSettings],
        tests: &[run("report", "use_case_1_report_shows_the_test_command_s_real_exit_code_and_no_trust")],
        layer: Layer::Scripted,
        needs: LINUX_AND_NETWORK,
    },
    Case {
        id: "I1",
        attack: "A run is stopped, runs out of time, or leaves programs running",
        lines: &[Line::Interrupted],
        tests: &[
            run("launch", "stop_ends_a_run_mid_command_and_leaves_nothing_running"),
            run("launch", "whatever_a_run_leaves_running_ends_with_it"),
            run("launch", "the_time_limit_ends_a_run"),
        ],
        layer: Layer::Scripted,
        needs: LINUX_AND_NETWORK,
    },
    Case {
        id: "M1",
        attack: "A run uses several models, and a model refuses",
        lines: &[Line::Models],
        tests: &[run("report", "every_model_used_and_every_refusal_is_listed")],
        layer: Layer::Scripted,
        needs: WINDOWS,
    },
    Case {
        id: "R1",
        attack: "A program prints look-alike lines and control characters",
        lines: &[Line::SafeToRead],
        tests: &[FIXTURE, run("watch_run", "each_event_reaches_the_screen_cleaned_and_in_order")],
        layer: Layer::Scripted,
        needs: LINUX_AND_NETWORK,
    },
    Case {
        id: "R2",
        attack: "The key or the run's token reaches the raw logs",
        lines: &[Line::SafeToRead],
        tests: &[run("report", "raw_log_lines_hide_the_key_and_the_token")],
        layer: Layer::Scripted,
        needs: WINDOWS,
    },
];

/// The cases of one layer, in the catalogue's order.
pub fn cases_for(layer: Layer) -> Vec<&'static Case> {
    CASES.iter().filter(|c| c.layer == layer).collect()
}

/// The cases `only` names, in the catalogue's order: ids or letters, separated by commas. A letter alone picks every
/// case of its line (`G`); an id picks that case only (`G1` is not `G11`).
pub fn select(cases: &[&'static Case], only: &str) -> Result<Vec<&'static Case>, String> {
    let wanted: Vec<&str> = only.split(',').map(str::trim).collect();
    for w in &wanted {
        if !cases.iter().any(|c| picks(w, c)) {
            return Err(format!("no case has the id or letter {w:?}"));
        }
    }
    Ok(cases.iter().filter(|c| wanted.iter().any(|w| picks(w, c))).copied().collect())
}

fn picks(wanted: &str, case: &Case) -> bool {
    let letters = case.id.trim_end_matches(|c: char| c.is_ascii_digit());
    !wanted.is_empty() && (wanted == case.id || wanted == letters)
}
