//! docs/rules.md is written from `rules::RULES`, so it cannot go stale. Write it again with STEPLET_BLESS=1.

use steplet_core::rules::RULES;

const HEAD: &str = "# Steplet's rating rules

This page is written by `cargo test -p steplet-core --test rules_doc` from `crates/steplet-core/src/rules.rs` (decision G3); do not edit it by hand.

A file's colour is its worst finding (decision 21). **Blocked:** the file cannot run. **Red:** it runs only after the receiver types a confirmation sentence. **Amber:** continue only if you trust the sender. **Note:** shown; the file stays green. Rule ids never change.

The phrase rules follow ideas from Cisco's skill-scanner (Apache-2.0, https://github.com/cisco-ai-defense/skill-scanner, read at commit f18074a5), rewritten as Steplet's own word lists.

| Rule | Severity | What it means | What to do |
|---|---|---|---|
";

#[test]
fn docs_rules_md_lists_every_rule() {
    let mut want = String::from(HEAD);
    for r in RULES {
        want.push_str(&format!("| `{}` | {} | {} | {} |\n", r.id, r.severity.name(), r.title, r.advice));
    }
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/../../docs/rules.md");
    if std::env::var_os("STEPLET_BLESS").is_some() {
        std::fs::write(path, &want).unwrap();
    }
    assert_eq!(
        std::fs::read_to_string(path).unwrap_or_default(),
        want,
        "run with STEPLET_BLESS=1 to write docs/rules.md"
    );
}
