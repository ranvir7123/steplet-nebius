//! The "Open .steplet files with Steplet" switch (Task 11, J4 A): entries for this Windows user only, under
//! `HKEY_CURRENT_USER\Software\Classes`, and nothing left when it is turned off. Tried on a made-up file type,
//! `.steplet-test` (Z4: the owner's yes before it runs), never on `.steplet` itself; removed when the test ends.

use std::sync::Mutex;

use steplet_app::register;
use steplet_app::views::Switch;

const EXT: &str = ".steplet-test";
const PROG_ID: &str = "Steplet.TestFile";
const OTHER: &str = "Steplet.OtherTestFile";
/// The tests share the made-up type, so they run one at a time.
static ONE_AT_A_TIME: Mutex<()> = Mutex::new(());

/// Removes the test's entries when it ends, passed or failed.
struct TurnOff;

impl Drop for TurnOff {
    fn drop(&mut self) {
        let _ = register::turn_off(EXT, PROG_ID);
        let _ = register::turn_off(EXT, OTHER);
    }
}

#[test]
fn file_type_switch_round_trip() {
    let _one = ONE_AT_A_TIME.lock().unwrap_or_else(|e| e.into_inner());
    let _cleanup = TurnOff;
    let exe = std::env::current_exe().unwrap();
    let ext_key = format!(r"Software\Classes\{EXT}");
    let prog_key = format!(r"Software\Classes\{PROG_ID}");

    register::turn_off(EXT, PROG_ID).unwrap();
    assert_eq!(register::state(EXT, PROG_ID, &exe), Switch::Off);
    assert!(!register::exists(&ext_key) && !register::exists(&prog_key), "nothing there to start with");

    register::turn_on(EXT, PROG_ID, &exe).unwrap();
    assert_eq!(register::state(EXT, PROG_ID, &exe), Switch::On);
    assert_eq!(register::default_value(&ext_key).as_deref(), Some(PROG_ID), "the file type names Steplet's entry");
    let command = register::default_value(&format!(r"{prog_key}\shell\open\command")).unwrap();
    assert_eq!(command, format!("\"{}\" \"%1\"", exe.display()), "Windows starts this program with the file");
    // Task 15: Explorer's "Select an app" box offers only the entries the file type lists here (Microsoft's
    // OpenWithProgids); without it, a real double-click offered every program but Steplet.
    let open_with = format!(r"{ext_key}\OpenWithProgids");
    assert!(register::has_value(&open_with, PROG_ID), "the file type offers Steplet's entry");
    // The box lists programs, each from its own entry under Applications (Microsoft's application registration);
    // every program it listed had one, and Steplet, without one, stayed out though Windows recommended it.
    let app_key = format!(r"Software\Classes\Applications\{}", exe.file_name().unwrap().to_string_lossy());
    assert!(register::has_value(&format!(r"{app_key}\SupportedTypes"), EXT), "offered for this file type only");
    let app_command = register::default_value(&format!(r"{app_key}\shell\open\command"));
    assert_eq!(app_command.as_deref(), Some(command.as_str()), "the program's entry opens the file the same way");
    reg(&["delete", &format!(r"HKCU\{app_key}"), "/f"]);
    assert_eq!(register::state(EXT, PROG_ID, &exe), Switch::Broken);
    register::turn_on(EXT, PROG_ID, &exe).unwrap();
    assert_eq!(register::state(EXT, PROG_ID, &exe), Switch::On);
    // A switch turned on before Steplet listed its entry there needs turning on again.
    reg(&["delete", &format!(r"HKCU\{open_with}"), "/v", PROG_ID, "/f"]);
    assert_eq!(register::state(EXT, PROG_ID, &exe), Switch::Broken);
    register::turn_on(EXT, PROG_ID, &exe).unwrap();
    assert_eq!(register::state(EXT, PROG_ID, &exe), Switch::On);
    // Microsoft documents both lists' values as REG_NONE, so Windows may rewrite them so: they still count.
    reg(&["add", &format!(r"HKCU\{open_with}"), "/v", PROG_ID, "/t", "REG_NONE", "/f"]);
    reg(&["add", &format!(r"HKCU\{app_key}\SupportedTypes"), "/v", EXT, "/t", "REG_NONE", "/f"]);
    assert_eq!(register::state(EXT, PROG_ID, &exe), Switch::On, "a value with no text is there all the same");

    // The program is not where the entry says any more.
    let moved = exe.with_file_name("steplet-moved-away.exe");
    assert_eq!(register::state(EXT, PROG_ID, &moved), Switch::Broken);

    register::turn_off(EXT, PROG_ID).unwrap();
    assert_eq!(register::state(EXT, PROG_ID, &exe), Switch::Off);
    assert!(!register::exists(&ext_key) && !register::exists(&prog_key), "off leaves nothing");
    assert!(!register::exists(&app_key), "not even the program's entry");

    // Turning Steplet's switch off never takes the file type from another program's entry.
    register::turn_on(EXT, OTHER, &exe).unwrap();
    register::turn_off(EXT, PROG_ID).unwrap();
    assert_eq!(register::default_value(&ext_key).as_deref(), Some(OTHER), "another entry is left alone");
    assert!(register::has_value(&open_with, OTHER), "and stays offered");
    register::turn_off(EXT, OTHER).unwrap();
    assert!(!register::exists(&ext_key), "and its own off removes it");
    assert!(!register::exists(&app_key), "with the program's entry it made");
}

#[test]
fn turning_off_gives_the_file_type_back_to_the_program_that_had_it() {
    let _one = ONE_AT_A_TIME.lock().unwrap_or_else(|e| e.into_inner());
    let _cleanup = TurnOff;
    /// Removes the made-up type's own key too, which the program "that had it" left behind.
    struct Gone;
    impl Drop for Gone {
        fn drop(&mut self) {
            let _ = std::process::Command::new(r"C:\Windows\System32\reg.exe")
                .args(["delete", r"HKCU\Software\Classes\.steplet-test", "/f"])
                .output();
        }
    }
    let _gone = Gone;
    let exe = std::env::current_exe().unwrap();
    let ext_key = format!(r"Software\Classes\{EXT}");
    // Another program owned the made-up type before (its entry's name is all it needs here).
    register::set_default(&ext_key, "SomeOther.TestFile").unwrap();
    register::turn_on(EXT, PROG_ID, &exe).unwrap();
    assert_eq!(register::state(EXT, PROG_ID, &exe), Switch::On);
    register::turn_on(EXT, PROG_ID, &exe).unwrap();
    register::turn_off(EXT, PROG_ID).unwrap();
    assert_eq!(
        register::default_value(&ext_key).as_deref(),
        Some("SomeOther.TestFile"),
        "given back, even after two ons"
    );
    assert!(!register::exists(&format!(r"Software\Classes\{PROG_ID}")), "Steplet's own entry is gone");
    assert!(!register::exists(&format!(r"{ext_key}\OpenWithProgids")), "and no longer offered");
}

/// Runs Windows' own reg.exe, for what the switch itself never does (a test standing for an older switch).
fn reg(args: &[&str]) {
    let out = std::process::Command::new(r"C:\Windows\System32\reg.exe").args(args).output().unwrap();
    assert!(out.status.success(), "reg {args:?}: {}", String::from_utf8_lossy(&out.stderr));
}

/// A program entry under Applications that Steplet did not make (another program with the same file name) is never
/// changed by turning the switch on, nor removed by turning it off (the code review of Task 15).
#[test]
fn a_program_entry_steplet_did_not_make_is_left_alone() {
    let _one = ONE_AT_A_TIME.lock().unwrap_or_else(|e| e.into_inner());
    let _cleanup = TurnOff;
    const APP_KEY: &str = r"Software\Classes\Applications\steplet-made-up-test.exe";
    /// Removes the made-up program entry, which stands for another program's and so is never Steplet's to remove.
    struct Gone;
    impl Drop for Gone {
        fn drop(&mut self) {
            let _ = std::process::Command::new(r"C:\Windows\System32\reg.exe")
                .args(["delete", &format!(r"HKCU\{APP_KEY}"), "/f"])
                .output();
        }
    }
    let _gone = Gone;
    let exe = std::path::PathBuf::from(r"C:\made-up\steplet-made-up-test.exe");
    let theirs = r#""C:\made-up\another-program.exe" "%1""#;
    register::set_default(&format!(r"{APP_KEY}\shell\open\command"), theirs).unwrap();
    reg(&["add", &format!(r"HKCU\{APP_KEY}\SupportedTypes"), "/v", ".another", "/t", "REG_SZ", "/d", "", "/f"]);

    register::turn_on(EXT, PROG_ID, &exe).unwrap();
    let command = format!(r"{APP_KEY}\shell\open\command");
    assert_eq!(register::default_value(&command).as_deref(), Some(theirs), "its command is not changed");
    assert!(!register::has_value(&format!(r"{APP_KEY}\SupportedTypes"), EXT), "nor this type added to it");
    assert!(!register::exists(&format!(r"{APP_KEY}\DefaultIcon")), "nor an icon");

    register::turn_off(EXT, PROG_ID).unwrap();
    assert_eq!(register::default_value(&command).as_deref(), Some(theirs), "and turning off leaves it");
    assert!(register::has_value(&format!(r"{APP_KEY}\SupportedTypes"), ".another"), "with all it held");
}
