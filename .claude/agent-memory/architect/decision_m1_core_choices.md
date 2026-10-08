---
name: decision-m1-core-choices
description: 2026-10-08 milestone 1 core/screens tech choices - own YAML-subset parser, ssh-key for SSHSIG, no_std steplet-core crate, Svelte 5 shared findings view, corpus/fuzz approach; rejected options and why
metadata:
  type: project
---

**Advised 2026-10-08 (architect, milestone 1 core + screens; runner side belongs to another architect). Owner has not yet approved.**

- Header parser: our own parser for a small block-only YAML subset; types from our schema, never YAML implicit typing. saphyr-parser used only as a test oracle (differential test: anything we accept parses the same there). Runner-up: saphyr-parser 0.1.0 event layer. Rejected: serde_yaml (deprecated), serde_yml (RUSTSEC-2025-0068), serde_norway / serde_yaml_ng (unsafe libyaml ports, serde level hides aliases), serde-saphyr (serde level), noyalib (serde_yml author). Flip: if senders must hand-write arbitrary YAML styles.
- Signing: RustCrypto ssh-key 0.6.7, no default features, alloc+ed25519. Its PublicKey::verify checks key and namespace; we must also reject non-Ed25519, non-sha512, version>1. Sender CLI shells out to ssh-keygen -Y sign then re-verifies with core. Runner-up: own SSHSIG parsing on ed25519-dalek.
- Layout: steplet-core (#![no_std]+alloc: canon, header, sig, rules, policy incl. R8 matcher, display segments), steplet-wasm (wasm-bindgen), steplet-sign CLI, src-tauri app (OS facts in, decisions out). WASM via cargo + pinned wasm-bindgen-cli, not wasm-pack.
- Front end: Svelte 5 + TS + Vite, npm workspace web/shared, web/app, web/check; one FileText component is the only path for file text; check page CSP connect-src 'none' with embedded wasm. Runner-up React 19.
- Corpus: ordinary (use cases + converted repo READMEs) and hostile (positive + near-miss per rule id); insta golden files, proptest, native-vs-WASM parity diff, cargo-fuzz (WSL2 fallback).

**Facts measured 2026-10-08 on this machine:** Windows ssh-keygen 9.5p2 signs/verifies SSHSIG with namespace runfile-v1; failure exit code is 127; re-signing is byte-identical (Ed25519 deterministic); Git Bash PATH finds Git's ssh-keygen 10.2 first, so call C:\Windows\System32\OpenSSH\ssh-keygen.exe by full path. No wasm32 target installed; WASM builds unverified.

**How to apply:** do not re-open without new evidence; namespace rename (runfile-v1 vs steplet) is an open naming item for the owner. Related: [[decision-signing-v1]], [[risk-rating-decision]].
