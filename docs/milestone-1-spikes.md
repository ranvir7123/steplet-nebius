# Milestone 1, phase 0: spike results

Plan: `docs/plans/2026-10-08-milestone-1-phase-0.md`. Each section gives the question, the exact
commands, the pasted output, a verdict (PASS, FAIL, INCONCLUSIVE, or RECORDED when the question has
no pass mark), and what it means. The agent's own words are never evidence. Fake keys in pasted logs
are replaced by labelled placeholders (`spikes/tools/append_results.py`).

## Setup

### Task 2: Rust tools on Windows

Command: `powershell -NoProfile -ExecutionPolicy Bypass -File spikes\tools\setup_windows.ps1`, then one more
fuzz run by hand (last section of the log).

```
--- rustup target add wasm32-unknown-unknown wasm32v1-none
info: downloading component rust-std
info: downloading component rust-std
exit code: 0
--- rustup toolchain install nightly --profile minimal
info: syncing channel updates for nightly-x86_64-pc-windows-msvc
info: latest update on 2026-10-08 for version 1.101.0-nightly (1d81eb4ad 2026-10-07)
info: downloading 3 components

  nightly-x86_64-pc-windows-msvc installed - rustc 1.101.0-nightly (1d81eb4ad 2026-10-07)

info: checking for self-update (current version: 1.29.0)
info: downloading self-update (new version: 1.29.1)
exit code: 0
--- cargo install wasm-bindgen-cli --version 0.2.129 --locked
    Updating crates.io index
 Downloading crates ...
  Downloaded wasm-bindgen-cli v0.2.129
  Installing wasm-bindgen-cli v0.2.129
    Updating crates.io index
warning: package `yoke-derive v0.8.3` in Cargo.lock is yanked in registry `crates-io`
  |
  = help: consider running without --locked
    Updating crates.io index
 Downloading crates ...
  Downloaded anstream v1.0.0
  Downloaded base64 v0.23.1
  Downloaded anstyle-parse v1.0.0
  Downloaded anstyle v1.0.14
  Downloaded bitflags v2.13.2
  Downloaded anstyle-query v1.1.5
  Downloaded anstyle-wincon v3.0.11
  Downloaded base64 v0.13.1
  Downloaded ascii v1.1.0
  Downloaded buf_redux v0.8.4
  Downloaded clap_builder v4.6.7
  Downloaded colorchoice v1.0.5
  Downloaded cfg-if v1.0.5
  Downloaded clap_lex v1.1.1
  Downloaded chunked_transfer v1.5.0
  Downloaded clap v4.6.7
  Downloaded clap_derive v4.6.7
  Downloaded cc v1.4.7
  Downloaded crc32fast v1.5.2
  Downloaded env_logger v0.11.11
  Downloaded crossbeam-utils v0.8.23
  Downloaded crossbeam-epoch v0.9.21
  Downloaded crossbeam-deque v0.8.8
  Downloaded fallible-iterator v0.3.0
  Downloaded env_filter v2.0.0
  Downloaded either v1.18.0
  Downloaded getrandom v0.2.17
  Downloaded filetime v0.2.29
  Downloaded hashbrown v0.16.1
  Downloaded gimli v0.32.3
  Downloaded find-msvc-tools v0.1.13
  Downloaded httparse v1.10.1
  Downloaded httpdate v1.0.3
  Downloaded id-arena v2.3.0
  Downloaded is_terminal_polyfill v1.70.2
  Downloaded indexmap v2.14.2
  Downloaded leb128 v0.2.7
  Downloaded jiff-core v0.1.1
  Downloaded leb128fmt v0.1.0
  Downloaded jiff v0.2.37
  Downloaded mime_guess v2.0.5
  Downloaded multipart v0.18.0
  Downloaded num_cpus v1.17.0
  Downloaded once_cell_polyfill v1.70.2
  Downloaded ppv-lite86 v0.2.21
  Downloaded quick-error v1.2.3
  Downloaded rand_core v0.6.4
  Downloaded rand_chacha v0.3.1
  Downloaded rayon-core v1.13.0
  Downloaded rand v0.8.8
  Downloaded rayon v1.12.0
  Downloaded rustc-demangle v0.1.28
  Downloaded rustls-webpki v0.103.15
  Downloaded safemem v0.3.3
  Downloaded rouille v3.6.2
  Downloaded rustls-pki-types v1.15.1
  Downloaded rustls v0.23.45
  Downloaded ring v0.17.14
  Downloaded shlex v1.3.0
  Downloaded sha1_smol v1.0.1
  Downloaded smallvec v1.16.2
  Downloaded subtle v2.6.1
  Downloaded tempfile v3.27.0
  Downloaded synstructure v0.14.0
  Downloaded syn v3.0.6
  Downloaded threadpool v1.8.1
  Downloaded thiserror v2.0.21
  Downloaded thiserror-impl v2.0.21
  Downloaded tiny_http v0.12.0
  Downloaded unicode-ident v1.0.26
  Downloaded untrusted v0.9.0
  Downloaded unicase v2.9.0
  Downloaded twoway v0.1.8
  Downloaded ureq v3.4.2
  Downloaded ureq-proto v0.6.4
  Downloaded utf8-zero v0.8.1
  Downloaded utf8parse v0.2.2
  Downloaded walrus-macro v0.26.0
  Downloaded wasm-bindgen-shared v0.2.129
  Downloaded wasm-bindgen-test-shared v0.2.129
  Downloaded walrus v0.27.2
  Downloaded wasm-encoder v0.245.1
  Downloaded wasm-bindgen-cli-support v0.2.129
  Downloaded webpki-roots v1.0.9
  Downloaded wasmparser v0.245.1
  Downloaded yoke-derive v0.8.3
  Downloaded zerofrom-derive v0.1.8
  Downloaded zeroize v1.9.0
  Downloaded zerocopy v0.8.58
  Downloaded zlib-rs v0.6.8
   Compiling proc-macro2 v1.0.107
   Compiling unicode-ident v1.0.26
   Compiling quote v1.0.47
   Compiling stable_deref_trait v1.2.1
   Compiling cfg-if v1.0.5
   Compiling serde_core v1.0.229
   Compiling windows-link v0.2.1
   Compiling memchr v2.8.3
   Compiling log v0.4.34
   Compiling crossbeam-utils v0.8.23
   Compiling shlex v2.0.1
   Compiling writeable v0.6.4
   Compiling find-msvc-tools v0.1.13
   Compiling crossbeam-epoch v0.9.21
   Compiling litemap v0.8.3
   Compiling zerocopy v0.8.58
   Compiling windows-sys v0.61.2
   Compiling getrandom v0.2.17
   Compiling icu_properties_data v2.3.0
   Compiling icu_normalizer_data v2.3.0
   Compiling crossbeam-deque v0.8.8
   Compiling utf8_iter v1.0.4
   Compiling itoa v1.0.18
   Compiling serde v1.0.229
   Compiling rayon-core v1.13.0
   Compiling httparse v1.10.1
   Compiling cc v1.4.7
   Compiling once_cell_polyfill v1.70.2
   Compiling once_cell v1.21.4
   Compiling autocfg v1.5.1
   Compiling getrandom v0.4.3
   Compiling ring v0.17.14
   Compiling zeroize v1.9.0
   Compiling equivalent v1.0.2
   Compiling hashbrown v0.17.1
   Compiling smallvec v1.16.2
   Compiling zmij v1.0.23
   Compiling unicase v2.9.0
   Compiling utf8parse v0.2.2
   Compiling anstyle v1.0.14
   Compiling heck v0.5.0
   Compiling rustls-pki-types v1.15.1
   Compiling num-traits v0.2.19
   Compiling anstyle-parse v1.0.0
   Compiling mime_guess v2.0.5
   Compiling anstyle-query v1.1.5
   Compiling anstyle-wincon v3.0.11
   Compiling aho-corasick v1.1.5
   Compiling rand_core v0.6.4
   Compiling untrusted v0.9.0
   Compiling foldhash v0.2.0
   Compiling colorchoice v1.0.5
   Compiling serde_json v1.0.151
   Compiling is_terminal_polyfill v1.70.2
   Compiling anyhow v1.0.104
   Compiling either v1.18.0
   Compiling crc32fast v1.5.2
   Compiling syn v3.0.6
   Compiling regex-syntax v0.8.11
   Compiling percent-encoding v2.3.2
   Compiling anstream v1.0.0
   Compiling syn v2.0.119
   Compiling indexmap v2.14.2
   Compiling hashbrown v0.16.1
   Compiling alloc-no-stdlib v2.0.4
   Compiling fastrand v2.5.0
   Compiling adler2 v2.0.1
   Compiling regex-automata v0.4.18
   Compiling bitflags v2.13.2
   Compiling ppv-lite86 v0.2.21
   Compiling leb128fmt v0.1.0
   Compiling safemem v0.3.3
   Compiling semver v1.0.28
   Compiling mime v0.3.17
   Compiling fallible-iterator v0.3.0
   Compiling wasm-bindgen-shared v0.2.129
   Compiling rustls v0.23.45
   Compiling rand_chacha v0.3.1
   Compiling bytes v1.12.1
   Compiling rayon v1.12.0
   Compiling simd-adler32 v0.3.10
   Compiling rand v0.8.8
   Compiling miniz_oxide v0.9.1
   Compiling walrus-macro v0.26.0
   Compiling gimli v0.32.3
   Compiling synstructure v0.14.0
   Compiling http v1.5.0
   Compiling wasm-encoder v0.245.1
   Compiling rustls-webpki v0.103.15
   Compiling tempfile v3.27.0
   Compiling buf_redux v0.8.4
   Compiling alloc-stdlib v0.2.4
   Compiling form_urlencoded v1.2.2
   Compiling twoway v0.1.8
   Compiling quick-error v1.2.3
   Compiling subtle v2.6.1
   Compiling regex v1.13.1
   Compiling num-conv v0.2.2
   Compiling httpdate v1.0.3
   Compiling id-arena v2.3.0
   Compiling jiff-core v0.1.1
   Compiling zerofrom-derive v0.1.8
   Compiling yoke-derive v0.8.3
   Compiling zerovec-derive v0.11.6
   Compiling displaydoc v0.2.7
   Compiling serde_derive v1.0.229
   Compiling chunked_transfer v1.5.0
   Compiling time-core v0.1.9
   Compiling num_cpus v1.17.0
   Compiling strsim v0.11.1
   Compiling powerfmt v0.2.0
   Compiling clap_lex v1.1.1
   Compiling deranged v0.5.8
   Compiling zerofrom v0.1.8
   Compiling ascii v1.1.0
   Compiling base64 v0.23.1
   Compiling leb128 v0.2.7
   Compiling clap_builder v4.6.7
   Compiling jiff v0.2.37
   Compiling threadpool v1.8.1
   Compiling yoke v0.8.3
   Compiling tiny_http v0.12.0
   Compiling ureq-proto v0.6.4
   Compiling env_filter v2.0.0
   Compiling clap_derive v4.6.7
   Compiling chrono v0.4.45
   Compiling time v0.3.55
   Compiling zerovec v0.11.8
   Compiling zerotrie v0.2.5
   Compiling multipart v0.18.0
   Compiling flate2 v1.1.10
   Compiling brotli-decompressor v5.0.3
   Compiling webpki-roots v1.0.9
   Compiling filetime v0.2.29
   Compiling base64 v0.13.1
   Compiling base64 v0.22.1
   Compiling sha1_smol v1.0.1
   Compiling tinystr v0.8.4
   Compiling potential_utf v0.1.6
   Compiling icu_locale_core v2.3.0
   Compiling utf8-zero v0.8.1
   Compiling rustc-demangle v0.1.28
   Compiling icu_collections v2.3.0
   Compiling clap v4.6.7
   Compiling wasm-bindgen-test-shared v0.2.129
   Compiling shlex v1.3.0
   Compiling env_logger v0.11.11
   Compiling wasmparser v0.245.1
   Compiling icu_provider v2.3.1
   Compiling icu_normalizer v2.3.0
   Compiling icu_properties v2.3.0
   Compiling ureq v3.4.2
   Compiling idna_adapter v1.2.2
   Compiling idna v1.1.0
   Compiling url v2.5.8
   Compiling rouille v3.6.2
   Compiling walrus v0.27.2
   Compiling wasm-bindgen-cli-support v0.2.129
   Compiling wasm-bindgen-cli v0.2.129
    Finished `release` profile [optimized] target(s) in 1m 15s
warning: the following packages contain code that will be rejected by a future version of Rust: buf_redux v0.8.4, multipart v0.18.0
note: to see what the problems were, use the option `--future-incompat-report`, or run `cargo report future-incompatibilities --id 1`
  Installing C:\Users\RANVIR\.cargo\bin\wasm-bindgen-test-runner.exe
  Installing C:\Users\RANVIR\.cargo\bin\wasm-bindgen.exe
  Installing C:\Users\RANVIR\.cargo\bin\wasm2es6js.exe
   Installed package `wasm-bindgen-cli v0.2.129` (executables `wasm-bindgen-test-runner.exe`, `wasm-bindgen.exe`, `wasm2es6js.exe`)
exit code: 0
--- cargo install cargo-fuzz --version 0.13.2 --locked
    Updating crates.io index
 Downloading crates ...
  Downloaded cargo-fuzz v0.13.2
  Installing cargo-fuzz v0.13.2
    Updating crates.io index
    Updating crates.io index
 Downloading crates ...
  Downloaded anyhow v1.0.102
  Downloaded crossbeam-epoch v0.9.18
  Downloaded camino v1.2.2
  Downloaded current_platform v0.2.0
  Downloaded cargo-platform v0.3.3
  Downloaded crossbeam-utils v0.8.21
  Downloaded clap_lex v1.1.0
  Downloaded either v1.16.0
  Downloaded crossbeam-deque v0.8.6
  Downloaded fastrand v2.4.1
  Downloaded cargo_metadata v0.23.1
  Downloaded clap_derive v4.6.1
  Downloaded clap v4.6.1
  Downloaded getrandom v0.4.2
  Downloaded indexmap v2.14.0
  Downloaded clap_builder v4.6.0
  Downloaded memchr v2.8.0
  Downloaded proc-macro2 v1.0.106
  Downloaded thiserror-impl v2.0.18
  Downloaded toml v1.1.2+spec-1.1.0
  Downloaded serde v1.0.228
  Downloaded serde_derive v1.0.228
  Downloaded quote v1.0.45
  Downloaded serde_core v1.0.228
  Downloaded thiserror v2.0.18
  Downloaded serde_json v1.0.150
  Downloaded syn v2.0.117
  Downloaded zmij v1.0.21
  Downloaded toml_parser v1.1.2+spec-1.1.0
  Downloaded toml_writer v1.1.1+spec-1.1.0
  Downloaded winnow v1.0.3
   Compiling serde_core v1.0.228
   Compiling proc-macro2 v1.0.106
   Compiling unicode-ident v1.0.24
   Compiling quote v1.0.45
   Compiling windows-link v0.2.1
   Compiling crossbeam-utils v0.8.21
   Compiling once_cell_polyfill v1.70.2
   Compiling utf8parse v0.2.2
   Compiling anstyle v1.0.14
   Compiling zmij v1.0.21
   Compiling serde v1.0.228
   Compiling getrandom v0.4.2
   Compiling thiserror v2.0.18
   Compiling rayon-core v1.13.0
   Compiling camino v1.2.2
   Compiling serde_json v1.0.150
   Compiling colorchoice v1.0.5
   Compiling windows-sys v0.61.2
   Compiling anstyle-parse v1.0.0
   Compiling is_terminal_polyfill v1.70.2
   Compiling itoa v1.0.18
   Compiling strsim v0.11.1
   Compiling heck v0.5.0
   Compiling winnow v1.0.3
   Compiling anstyle-query v1.1.5
   Compiling anstyle-wincon v3.0.11
   Compiling cfg-if v1.0.4
   Compiling current_platform v0.2.0
   Compiling clap_lex v1.1.0
   Compiling memchr v2.8.0
   Compiling anyhow v1.0.102
   Compiling once_cell v1.21.4
   Compiling toml_writer v1.1.1+spec-1.1.0
   Compiling either v1.16.0
   Compiling fastrand v2.4.1
   Compiling anstream v1.0.0
   Compiling toml_parser v1.1.2+spec-1.1.0
   Compiling num_cpus v1.17.0
   Compiling tempfile v3.27.0
   Compiling clap_builder v4.6.0
   Compiling crossbeam-epoch v0.9.18
   Compiling syn v2.0.117
   Compiling crossbeam-deque v0.8.6
   Compiling rayon v1.12.0
   Compiling semver v1.0.28
   Compiling cargo-platform v0.3.3
   Compiling serde_spanned v1.1.1
   Compiling toml_datetime v1.1.1+spec-1.1.0
   Compiling rustc_version v0.4.1
   Compiling toml v1.1.2+spec-1.1.0
   Compiling serde_derive v1.0.228
   Compiling thiserror-impl v2.0.18
   Compiling clap_derive v4.6.1
   Compiling clap v4.6.1
   Compiling cargo_metadata v0.23.1
   Compiling cargo-fuzz v0.13.2
    Finished `release` profile [optimized] target(s) in 26.97s
  Installing C:\Users\RANVIR\.cargo\bin\cargo-fuzz.exe
   Installed package `cargo-fuzz v0.13.2` (executable `cargo-fuzz.exe`)
exit code: 0
--- versions
rustc 1.98.0 (88d9e12ae 2026-08-18)
rustc 1.101.0-nightly (1d81eb4ad 2026-10-07)
wasm-bindgen 0.2.129
cargo-fuzz 0.13.2
wasm32-unknown-unknown
wasm32v1-none
x86_64-pc-windows-msvc
exit code: 0
--- fuzz smoke test: new crate
    Creating library `fuzz-smoke` package
note: see more `Cargo.toml` keys and their definitions at https://doc.rust-lang.org/cargo/reference/manifest.html
exit code: 0
--- cargo +nightly fuzz init
exit code: 0
--- cargo +nightly fuzz run fuzz_target_1 -- -max_total_time=10
    Updating crates.io index
     Locking 11 packages to highest Rust 1.101.0-nightly compatible versions
 Downloading crates ...
  Downloaded arbitrary v1.5.0
  Downloaded find-msvc-tools v0.1.14
  Downloaded jobserver v0.1.35
  Downloaded cc v1.6.0
  Downloaded libfuzzer-sys v0.4.13
   Compiling getrandom v0.4.3
   Compiling cfg-if v1.0.5
   Compiling find-msvc-tools v0.1.14
   Compiling shlex v2.0.1
   Compiling arbitrary v1.5.0
   Compiling fuzz-smoke v0.1.0 (C:\dev\m1\fuzz-smoke)
   Compiling jobserver v0.1.35
   Compiling cc v1.6.0
   Compiling libfuzzer-sys v0.4.13
   Compiling fuzz-smoke-fuzz v0.0.0 (C:\dev\m1\fuzz-smoke\fuzz)
    Finished `release` profile [optimized + debuginfo] target(s) in 9.32s
    Finished `release` profile [optimized + debuginfo] target(s) in 0.03s
     Running `fuzz\target\x86_64-pc-windows-msvc\release\fuzz_target_1.exe -artifact_prefix=C:\dev\m1\fuzz-smoke\fuzz\artifacts\fuzz_target_1\ -max_total_time=10 C:\dev\m1\fuzz-smoke\fuzz\corpus\fuzz_target_1`
System.Management.Automation.RemoteException
error: process didn't exit successfully: `fuzz\target\x86_64-pc-windows-msvc\release\fuzz_target_1.exe -artifact_prefix=C:\dev\m1\fuzz-smoke\fuzz\artifacts\fuzz_target_1\ -max_total_time=10 C:\dev\m1\fuzz-smoke\fuzz\corpus\fuzz_target_1` (exit code: 0xc0000135, STATUS_DLL_NOT_FOUND)
────────────────────────────────────────────────────────────────────────────────
System.Management.Automation.RemoteException
Error: Fuzz target exited with exit code: 0xc0000135
exit code: 1
--- again without AddressSanitizer
   Compiling arbitrary v1.5.0
   Compiling fuzz-smoke v0.1.0 (C:\dev\m1\fuzz-smoke)
   Compiling libfuzzer-sys v0.4.13
   Compiling fuzz-smoke-fuzz v0.0.0 (C:\dev\m1\fuzz-smoke\fuzz)
error: linking with `link.exe` failed: exit code: 1120
  |
  = note: "C:\\Program Files (x86)\\Microsoft Visual Studio\\2022\\BuildTools\\VC\\Tools\\MSVC\\14.44.35207\\bin\\HostX64\\x64\\link.exe" "/NOLOGO" "C:\\dev\\m1\\fuzz-smoke\\fuzz\\target\\x86_64-pc-windows-msvc\\release\\build\\fuzz-smoke-fuzz/401497b648fd32a9\\out\\rustchDFKO1\\symbols.o" "<2 object files omitted>" "C:\\dev\\m1\\fuzz-smoke\\fuzz\\target\\x86_64-pc-windows-msvc\\release\\build\\libfuzzer-sys\\3bfa80236f473f3b\\out/liblibfuzzer_sys-3bfa80236f473f3b.rlib" "C:\\dev\\m1\\fuzz-smoke\\fuzz\\target\\x86_64-pc-windows-msvc\\release\\build\\arbitrary/1fb584118e8ce86d\\out/libarbitrary-1fb584118e8ce86d.rlib" "<sysroot>\\lib\\rustlib\\x86_64-pc-windows-msvc\\lib/{libstd-*,libpanic_unwind-*,libcfg_if-*,libwindows_link-*,librustc_demangle-*,libstd_detect-*,libhashbrown-*,librustc_std_workspace_alloc-*,libunwind-*,librustc_std_workspace_core-*,liballoc-*,libcore-*,libcompiler_builtins-*}.rlib" "kernel32.lib" "kernel32.lib" "ntdll.lib" "userenv.lib" "ws2_32.lib" "dbghelp.lib" "synchronization.lib" "/defaultlib:msvcrt" "/NXCOMPAT" "/LIBPATH:C:\\dev\\m1\\fuzz-smoke\\fuzz\\target\\x86_64-pc-windows-msvc\\release\\build\\libfuzzer-sys/9d5051796bb281f3\\out" "/OUT:C:\\dev\\m1\\fuzz-smoke\\fuzz\\target\\x86_64-pc-windows-msvc\\release\\build\\fuzz-smoke-fuzz/401497b648fd32a9\\out\\fuzz_target_1.exe" "/OPT:REF,ICF" "/DEBUG" "/PDBALTPATH:%_PDB%" "/NATVIS:<sysroot>\\lib\\rustlib\\etc\\intrinsic.natvis" "/NATVIS:<sysroot>\\lib\\rustlib\\etc\\liballoc.natvis" "/NATVIS:<sysroot>\\lib\\rustlib\\etc\\libcore.natvis" "/NATVIS:<sysroot>\\lib\\rustlib\\etc\\libstd.natvis" "/include:main"
  = note: some arguments are omitted. use `--verbose` to show all linker arguments
  = note:    Creating library C:\dev\m1\fuzz-smoke\fuzz\target\x86_64-pc-windows-msvc\release\build\fuzz-smoke-fuzz\401497b648fd32a9\out\fuzz_target_1.lib and object C:\dev\m1\fuzz-smoke\fuzz\target\x86_64-pc-windows-msvc\release\build\fuzz-smoke-fuzz\401497b648fd32a9\out\fuzz_target_1.exp
          fuzz_target_1.fuzz_target_1.714dc85f0cfdb03b-cgu.0.rcgu.o : error LNK2019: unresolved external symbol __start___sancov_cntrs referenced in function sancov.module_ctor_8bit_counters
          liblibfuzzer_sys-3bfa80236f473f3b.rlib(libfuzzer_sys-3bfa80236f473f3b.libfuzzer_sys.a74fabf965126c85-cgu.0.rcgu.o) : error LNK2001: unresolved external symbol __start___sancov_cntrs
          fuzz_target_1.fuzz_target_1.714dc85f0cfdb03b-cgu.0.rcgu.o : error LNK2019: unresolved external symbol __stop___sancov_cntrs referenced in function sancov.module_ctor_8bit_counters
          liblibfuzzer_sys-3bfa80236f473f3b.rlib(libfuzzer_sys-3bfa80236f473f3b.libfuzzer_sys.a74fabf965126c85-cgu.0.rcgu.o) : error LNK2001: unresolved external symbol __stop___sancov_cntrs
          fuzz_target_1.fuzz_target_1.714dc85f0cfdb03b-cgu.0.rcgu.o : error LNK2019: unresolved external symbol __start___sancov_pcs referenced in function sancov.module_ctor_8bit_counters
          liblibfuzzer_sys-3bfa80236f473f3b.rlib(libfuzzer_sys-3bfa80236f473f3b.libfuzzer_sys.a74fabf965126c85-cgu.0.rcgu.o) : error LNK2001: unresolved external symbol __start___sancov_pcs
          fuzz_target_1.fuzz_target_1.714dc85f0cfdb03b-cgu.0.rcgu.o : error LNK2019: unresolved external symbol __stop___sancov_pcs referenced in function sancov.module_ctor_8bit_counters
          liblibfuzzer_sys-3bfa80236f473f3b.rlib(libfuzzer_sys-3bfa80236f473f3b.libfuzzer_sys.a74fabf965126c85-cgu.0.rcgu.o) : error LNK2001: unresolved external symbol __stop___sancov_pcs
          C:\dev\m1\fuzz-smoke\fuzz\target\x86_64-pc-windows-msvc\release\build\fuzz-smoke-fuzz\401497b648fd32a9\out\fuzz_target_1.exe : fatal error LNK1120: 4 unresolved externals
          
System.Management.Automation.RemoteException
error: could not compile `fuzz-smoke-fuzz` (bin "fuzz_target_1") due to 1 previous error
Error: failed to build fuzz script: "cargo" "build" "--manifest-path" "C:\\dev\\m1\\fuzz-smoke\\fuzz\\Cargo.toml" "--target" "x86_64-pc-windows-msvc" "--release" "--config" "profile.release.debug=\"line-tables-only\"" "--bin" "fuzz_target_1"
exit code: 1
--- again, with the AddressSanitizer runtime folder added to PATH for this command only
INFO: -max_len is not provided; libFuzzer will not generate inputs larger than 4096 bytes
INFO: A corpus is not provided, starting from an empty corpus
#2	INITED cov: 15 ft: 15 corp: 1/1b exec/s: 0 rss: 39Mb
#1048576	pulse  cov: 15 ft: 15 corp: 1/1b lim: 4096 exec/s: 524288 rss: 126Mb
#2097152	pulse  cov: 15 ft: 15 corp: 1/1b lim: 4096 exec/s: 524288 rss: 211Mb
#4194304	pulse  cov: 15 ft: 15 corp: 1/1b lim: 4096 exec/s: 524288 rss: 381Mb
#5510642	DONE   cov: 15 ft: 15 corp: 1/1b lim: 4096 exec/s: 500967 rss: 488Mb
Done 5510642 runs in 11 second(s)
exit code: 0
```

**Finding:** fuzzing works on Windows (MSVC), but only with the AddressSanitizer runtime folder of the Visual
Studio build tools on `PATH` for the run:
`C:\Program Files (x86)\Microsoft Visual Studio\2022\BuildTools\VC\Tools\MSVC\14.44.35207\bin\Hostx64\x64`.
Without it the fuzz program cannot start (`0xc0000135, STATUS_DLL_NOT_FOUND`); without the sanitizer it does not
link (`unresolved external symbol __start___sancov_cntrs`). Phase 1 runs its fuzz target on Windows with that
folder added to `PATH` for the fuzz command only; Rust inside WSL is not needed for fuzzing.

### Task 3: the test distribution

Created by the user with `wsl --install Ubuntu-26.04 --name steplet-test`, user `tester`, then
`sudo apt-get update && sudo apt-get install -y nodejs npm socat bubblewrap ripgrep git python3`.
Read-only checks: `wsl -d steplet-test --exec /bin/bash /mnt/c/dev/runFile/spikes/wsl/check_distro.sh`.

```
distribution: steplet-test, Ubuntu 26.04.1 LTS, kernel 6.6.87.2-microsoft-standard-WSL2, user tester
nodejs: 22.22.1+dfsg+~cs22.19.15-1ubuntu1
npm: 9.2.0~ds3-1
socat: 1.8.1.1-1ubuntu0.1
bubblewrap: 0.11.1-1ubuntu0.3
ripgrep: 15.1.0-1ubuntu1
git: 1:2.53.0-1ubuntu1
python3: 3.14.3-0ubuntu2
node: v22.22.1
bwrap smoke test: BWRAP_OK
AppArmor limit on user namespaces: setting not present
networking mode: nat
interop sockets in /run/WSL: 1_interop 2_interop 461_interop 5432_interop 
WSL_INTEROP in this session: /run/WSL/5432_interop
a Windows program from here: Microsoft Windows [Version 10.0.26200.9457]
claude on PATH: /mnt/c/Users/RANVIR/AppData/Roaming/npm/claude
~/.claude: absent; ~/.config/git/ignore: absent
distro id: {7d77bf1b-4ab3-40e7-aa63-4241953a1a25}
```

### Task 4: Steplet's folders, Claude Code 2.1.220, srt 0.0.77, spike data

Command: `wsl -d steplet-test --exec /bin/bash /mnt/c/dev/runFile/spikes/wsl/setup_steplet.sh`

```
wsl: Failed to start the systemd user session for 'tester'. See journalctl for more details.
--- Claude Code 2.1.220 into /home/tester/.local/share/steplet/claude-cli

added 3 packages in 1m
version: 2.1.220 (Claude Code)
the file that runs: /home/tester/.local/share/steplet/claude-cli/lib/node_modules/@anthropic-ai/claude-code/bin/claude.exe
0000000 177   E   L   F
--- srt 0.0.77 into /home/tester/.local/share/steplet/srt

added 5 packages in 4s
srt@1.0.0 /home/tester/.local/share/steplet/srt
└─┬ @anthropic-ai/sandbox-runtime@0.0.77
  ├── @pondwader/socks5-server@1.0.10
  ├── commander@12.1.0
  ├── node-forge@1.4.0
  └── zod@3.25.76

Usage: srt [options] [command] [command...]

Run commands in a sandbox with network and filesystem restrictions

Arguments:
  command                      command to run in the sandbox

Options:
  -V, --version                output the version number
  -d, --debug                  enable debug logging
  -s, --settings <path>        path to config file (default:
                               ~/.srt-settings.json)
  -c <command>                 run command string directly (like sh -c), no
                               escaping applied
  --control-fd <fd>            read config updates from an inherited file
                               descriptor >= 3 (JSON lines protocol; give srt a
                               dedicated read-only end — see the README)
  -h, --help                   display help for command

Commands:
--- fixture (milestone 0's test repository)
fixture HEAD: 7f4b72d7981454558a3631310426dcff4d4ebd50; COMMIT1 present: commit
--- fake keys (random, made for these tests, never real)
fixture fake key: 48 bytes; fake API key: 52 bytes
--- Node stand-in gate

added 94 packages in 5s
gate packages load: yes
--- folders
total 32
drwx------ 8 tester tester 4096 Oct  8 12:25 .
drwxr-xr-x 3 tester tester 4096 Oct  8 12:25 ..
drwxr-xr-x 4 tester tester 4096 Oct  8 12:26 claude-cli
drwxr-xr-x 2 tester tester 4096 Oct  8 12:25 claude-config
drwxr-xr-x 2 tester tester 4096 Oct  8 12:25 home
drwxr-xr-x 2 tester tester 4096 Oct  8 12:25 runs
drwx------ 2 tester tester 4096 Oct  8 12:26 secrets
drwxr-xr-x 3 tester tester 4096 Oct  8 12:26 srt
~/.claude: absent; ~/.claude.json: absent
```

### Task 5: self-tests of the harness

Scripted model, inside the test distribution (`python3 -I spikes/tools/selftest_fake_model.py`):

```
wsl: Failed to start the systemd user session for 'tester'. See journalctl for more details.
fake model on 127.0.0.1:47999 with 3 steps
PASS step 0 answers with a Bash tool call
PASS streamed step 0 rebuilds the same tool input
PASS step 1 refuses, with the extra field merged
PASS a repeat at step 1 gets the text after the refusal, under the model asked for
PASS a request without tools gets 'ok'
PASS count_tokens answers
PASS six requests logged
PASS the key matched and its text is not in the log
failures: 0
```

Gate and pipe, on Windows (`spikes\tools\selftest_gate.ps1`). The first version piped the MCP lines through
PowerShell, which put a byte-order mark (`EF BB BF`) in front of the first line, so the gate could not read the
opening message and every case failed with 2 replies instead of 3, although each decision was right. The test now
hands the gate a file; the gate is unchanged.

```
host: pipe \\.\pipe\steplet-spike-selftest created, gate mode allow
PASS host allows -> allow (1.0 s, 3 replies)
    {"id":3,"jsonrpc":"2.0","result":{"content":[{"text":"{\"behavior\":\"allow\",\"updatedInput\":{\"command\":\"echo hi\"}}","type":"text"}]}}
host: pipe \\.\pipe\steplet-spike-selftest created, gate mode allow
PASS wrong run token -> deny (1.0 s, 3 replies)
    {"id":3,"jsonrpc":"2.0","result":{"content":[{"text":"{\"behavior\":\"deny\",\"message\":\"Steplet spike host: wrong run token\"}","type":"text"}]}}
host: pipe \\.\pipe\steplet-spike-selftest created, gate mode deny
PASS host denies -> deny (1.0 s, 3 replies)
    {"id":3,"jsonrpc":"2.0","result":{"content":[{"text":"{\"behavior\":\"deny\",\"message\":\"Steplet spike host: denied by the spike\"}","type":"text"}]}}
host: pipe \\.\pipe\steplet-spike-selftest created, gate mode drop
PASS host closes without answering -> deny (1.0 s, 3 replies)
    {"id":3,"jsonrpc":"2.0","result":{"content":[{"text":"{\"behavior\":\"deny\",\"message\":\"Steplet's checker refused: no answer\"}","type":"text"}]}}
host: pipe \\.\pipe\steplet-spike-selftest created, gate mode stall
PASS host stalls -> deny at the host's deadline (2.1 s, 3 replies)
    {"id":3,"jsonrpc":"2.0","result":{"content":[{"text":"{\"behavior\":\"deny\",\"message\":\"Steplet spike host: no decision before the deadline\"}","type":"text"}]}}
PASS no host at all -> deny (1.0 s, 3 replies)
    {"id":3,"jsonrpc":"2.0","result":{"content":[{"text":"{\"behavior\":\"deny\",\"message\":\"Steplet's checker refused: cannot reach Steplet (The system cannot find the file specified. (os error 2))\"}","type":"text"}]}}
--- gate log:
1791462451.217 pid=82640 started; 96 environment variables (names not shown in the self-test)
1791462451.218 pid=82640 tool="Bash" behavior="allow" ms=0 input={"command":"echo hi"}
1791462451.219 pid=82640 standard input closed; exiting
1791462456.203 pid=89048 started; 96 environment variables (names not shown in the self-test)
1791462456.204 pid=89048 tool="Bash" behavior="deny" ms=0 input={"command":"echo hi"}
1791462456.204 pid=89048 standard input closed; exiting
1791462461.236 pid=69652 started; 96 environment variables (names not shown in the self-test)
1791462461.237 pid=69652 tool="Bash" behavior="deny" ms=0 input={"command":"echo hi"}
1791462461.237 pid=69652 standard input closed; exiting
1791462466.268 pid=60692 started; 96 environment variables (names not shown in the self-test)
1791462466.269 pid=60692 tool="Bash" behavior="deny" ms=0 input={"command":"echo hi"}
1791462466.269 pid=60692 standard input closed; exiting
1791462471.309 pid=21428 started; 96 environment variables (names not shown in the self-test)
1791462472.812 pid=21428 tool="Bash" behavior="deny" ms=1502 input={"command":"echo hi"}
1791462472.813 pid=21428 standard input closed; exiting
1791462475.611 pid=16348 started; 96 environment variables (names not shown in the self-test)
1791462475.612 pid=16348 tool="Bash" behavior="deny" ms=1 input={"command":"echo hi"}
1791462475.613 pid=16348 standard input closed; exiting
--- host log:
1791462451.218 tool="Bash" token_ok=true mode=allow behavior="allow" ms=0
1791462451.468 tasklist:
INFO: No tasks are running which match the specified criteria.

1791462456.204 tool="Bash" token_ok=false mode=allow behavior="deny" ms=0
1791462456.447 tasklist:
INFO: No tasks are running which match the specified criteria.

1791462461.237 tool="Bash" token_ok=true mode=deny behavior="deny" ms=0
1791462461.474 tasklist:
INFO: No tasks are running which match the specified criteria.

1791462466.269 tool="Bash" mode=drop: closed without answering
1791462466.507 tasklist:
INFO: No tasks are running which match the specified criteria.

1791462471.572 tasklist:
"Image Name","PID","Session Name","Session#","Mem Usage","Status","User Name","CPU Time","Window Title"
"spike-gate.exe","21428","Console","1","4,028 K","Unknown","LAPTOP-R1RG22O7\RANVIR","0:00:00","N/A"

1791462472.812 tool="Bash" token_ok=true mode=stall behavior="deny" ms=1502
failures: 0
```
