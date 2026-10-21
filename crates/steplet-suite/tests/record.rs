//! The suite's records in `docs/suite/` (phase 5 Task 1): named by day, Claude Code version and layer, never written
//! over, and counted for a key's text before they are kept. Written under this test program's own folder in `target`.

use std::path::{Path, PathBuf};

use steplet_suite::catalogue::Layer;
use steplet_suite::record::{Day, keep, record_name};
use steplet_suite::runner::MADE_UP_KEY;

fn empty_folder(name: &str) -> PathBuf {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR")).join(name);
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

#[test]
fn a_record_is_named_by_day_version_and_layer() {
    let day = Day { year: 2026, month: 10, day: 3 };
    assert_eq!(record_name(&day, "2.1.220", Layer::Scripted, None), "2026-10-03-claude-2.1.220-scripted.txt");
    assert_eq!(record_name(&day, "2.1.220", Layer::Live, None), "2026-10-03-claude-2.1.220-live.txt");
    assert_eq!(
        record_name(&day, "2.1.220", Layer::Scripted, Some("G")),
        "2026-10-03-claude-2.1.220-scripted-only-G.txt"
    );
    assert_eq!(
        record_name(&day, "2.1.220", Layer::Scripted, Some("I,M,R,Y")),
        "2026-10-03-claude-2.1.220-scripted-only-I-M-R-Y.txt"
    );
}

#[test]
fn a_second_record_the_same_day_does_not_replace_the_first() {
    let dir = empty_folder("records-twice");
    let first = keep(&dir, "2026-10-03-claude-2.1.220-scripted.txt", "the first run\n", &[]).unwrap();
    let second = keep(&dir, "2026-10-03-claude-2.1.220-scripted.txt", "the second run\n", &[]).unwrap();
    assert_eq!(first.path, dir.join("2026-10-03-claude-2.1.220-scripted.txt"));
    assert_eq!(second.path, dir.join("2026-10-03-claude-2.1.220-scripted-2.txt"));
    assert_eq!(std::fs::read_to_string(&first.path).unwrap(), "the first run\n");
    assert_eq!(std::fs::read_to_string(&second.path).unwrap(), "the second run\n");
    let names: Vec<String> =
        std::fs::read_dir(&dir).unwrap().map(|e| e.unwrap().file_name().to_string_lossy().into_owned()).collect();
    assert_eq!(names.len(), 2, "no part-written file is left: {names:?}");
}

#[test]
fn a_key_in_a_record_is_counted_and_hidden_before_it_is_kept() {
    let dir = empty_folder("records-key");
    let keys = [("the made-up key", MADE_UP_KEY)];
    let text = format!("row one\nit printed {MADE_UP_KEY}\nand again:{MADE_UP_KEY}.\n");
    let kept = keep(&dir, "with-key.txt", &text, &keys).unwrap();
    assert_eq!(kept.counts, [("the made-up key".to_owned(), 2)]);
    assert!(!kept.clean());
    let on_disk = std::fs::read_to_string(&kept.path).unwrap();
    assert_eq!(
        on_disk,
        "row one\nit printed [hidden by Steplet]\nand again:[hidden by Steplet].\n\
         Counted before this record was kept: the made-up key 2 times. The suite does not pass while a key's text is \
         in its record; each one above was replaced by [hidden by Steplet].\n"
    );
    assert!(!on_disk.contains(MADE_UP_KEY));

    // The control: a record without the key is kept as it is, and says it was counted.
    let kept = keep(&dir, "without-key.txt", "row one\nrow two\n", &keys).unwrap();
    assert_eq!(kept.counts, [("the made-up key".to_owned(), 0)]);
    assert!(kept.clean());
    assert_eq!(
        std::fs::read_to_string(&kept.path).unwrap(),
        "row one\nrow two\nCounted before this record was kept: the made-up key 0 times.\n"
    );
}
