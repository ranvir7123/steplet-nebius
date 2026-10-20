//! The "Open .steplet files with Steplet" switch (Task 11, J4 A): entries for this Windows user only, under
//! `HKEY_CURRENT_USER\Software\Classes`, and nothing left when it is turned off. Tried on a made-up file type,
//! `.steplet-test` (Z4: the owner's yes before it runs), never on `.steplet` itself; removed when the test ends.

use steplet_app::register;
use steplet_app::views::Switch;

const EXT: &str = ".steplet-test";
const PROG_ID: &str = "Steplet.TestFile";
const OTHER: &str = "Steplet.OtherTestFile";

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

    // The program is not where the entry says any more.
    let moved = exe.with_file_name("steplet-moved-away.exe");
    assert_eq!(register::state(EXT, PROG_ID, &moved), Switch::Broken);

    register::turn_off(EXT, PROG_ID).unwrap();
    assert_eq!(register::state(EXT, PROG_ID, &exe), Switch::Off);
    assert!(!register::exists(&ext_key) && !register::exists(&prog_key), "off leaves nothing");

    // Turning Steplet's switch off never takes the file type from another program's entry.
    register::turn_on(EXT, OTHER, &exe).unwrap();
    register::turn_off(EXT, PROG_ID).unwrap();
    assert_eq!(register::default_value(&ext_key).as_deref(), Some(OTHER), "another entry is left alone");
    register::turn_off(EXT, OTHER).unwrap();
    assert!(!register::exists(&ext_key), "and its own off removes it");
}
