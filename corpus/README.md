# The rating's test corpus

Made by `make_corpus.py` (milestone 1, phase 2; decisions G3 and G4). Every `.steplet` file has an expected answer
next to it, the `.json` that `steplet_core::report::check(file, 1791720000).to_json()` gives (1,791,720,000 seconds
is 11 Oct 2026, 12:00 UTC). The native tests (`cargo test -p steplet-core --test corpus`) and the WASM build
(`crates/steplet-wasm/parity.cjs`) must both give exactly these answers.

- `hostile/<rule id>.hit.steplet` must trigger that rule, and nothing worse; `hostile/<rule id>.near.steplet`,
  almost the same, must not trigger it. Each pair is one small change to one green file. The `sig.invalid` pair is
  phase 1's signed test vectors.
- `ordinary/` holds what senders really write, including a few realistic things that may raise findings, so the
  green, amber and red counts are honest. `uc5-lookalike.steplet` is use case 5's file: it looks exactly like use
  case 1, because its attack is in the repository, not in the file.
- `ordinary/example-run-tests.steplet` is a copy of `examples/run-tests.steplet`. Its expiry must stay within 365
  days of the fixed "now" (so before 12 Oct 2027), or the copy turns blocked (`time.too-far`).

To change the corpus: edit `make_corpus.py`, run `python -I corpus/make_corpus.py`, run the corpus test, read every
difference, then write the expected answers again:

    $env:STEPLET_BLESS = "1"; cargo test -p steplet-core --offline --test corpus --test rules_doc; Remove-Item Env:STEPLET_BLESS

The phrase rules follow ideas from Cisco's skill-scanner (Apache-2.0), see `docs/rules.md`.
