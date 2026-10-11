use steplet_core::canon::canonicalize;
use steplet_core::header::parse;

fn body_line(f: &str) -> u32 {
    parse(&canonicalize(f.as_bytes()).unwrap()).unwrap().body_line
}

#[test]
fn the_body_starts_after_the_closing_line() {
    let head = "---\nsteplet: 1\ntitle: t\nsender: s\nexpires: 2026-10-25\nsteps:\n  - a\n---\n";
    assert_eq!(body_line(&format!("{head}# Body\ntext\n")), 9);
    assert_eq!(body_line(head), 9);
}

#[test]
fn signature_lines_count_as_lines_of_the_file() {
    let signed = include_str!("vectors/valid.steplet");
    let first = signed.lines().position(|l| l == "# Count the README lines").unwrap() as u32 + 1;
    assert_eq!(body_line(signed), first);
}
