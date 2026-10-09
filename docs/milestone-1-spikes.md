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

## S7: does a scripted model change what the checker and the fence see?

Question, from the milestone plan: with a fake model server (`ANTHROPIC_BASE_URL`), does every action still
go through the same gate and fence? Pass: same gate requests and fence results as T1c.

Runs: `spikes/runs/S7a.json` (milestone 0 settings, compare with T1c), `spikes/runs/S7b.json` (product settings).
Command: `powershell -NoProfile -ExecutionPolicy Bypass -File spikes\tools\run.ps1 -Id <run>`

### S7a

```
=== spike run S7a, started 2026-10-08T17:58:34.9799563+05:30
--- run description: {"about": "S7: scripted model replays T1c; milestone 0 settings; launched inside WSL", "script": "s7_t1c.json", "gate": "node", "gate_mode": "allow", "shape": "m0", "launch": "inside", "env_extra": ["ANTHROPIC_API_KEY=@fake-api-key"]}
wsl: Failed to start the systemd user session for 'tester'. See journalctl for more details.
--- working copy: /home/tester/steplet-work/S7a/repo HEAD 7f4b72d7981454558a3631310426dcff4d4ebd50
--- settings.json:
{
  "sandbox": {
    "enabled": true,
    "failIfUnavailable": true,
    "allowUnsandboxedCommands": false,
    "autoAllowBashIfSandboxed": false,
    "filesystem": {
      "denyRead": [
        "/home/tester/m1/canary",
        "/mnt/c/dev/m1/canary"
      ]
    },
    "network": {
      "allowedDomains": [],
      "strictAllowlist": true
    }
  },
  "permissions": {
    "deny": [
      "Read(//home/tester/m1/canary/**)",
      "Read(//mnt/c/dev/m1/canary/**)"
    ]
  }
}
--- MCP server: {"type": "stdio", "command": "/usr/bin/node", "args": ["/home/tester/m1/tools/gate/gate.mjs"], "env": {"GATE_MODE": "allow", "GATE_LOG": "/home/tester/m1/logs/S7a.gate.log"}}
--- launch command, one argument per line:
/usr/bin/timeout
-k
10
600
/usr/bin/env
-i
HOME=/home/tester/.local/share/steplet/home
PATH=/usr/local/bin:/usr/bin:/bin
LANG=C.UTF-8
CLAUDE_CONFIG_DIR=/home/tester/.local/share/steplet/claude-config
DISABLE_AUTOUPDATER=1
CLAUDE_CODE_DISABLE_NONESSENTIAL_TRAFFIC=1
ANTHROPIC_BASE_URL=http://127.0.0.1:47900
ANTHROPIC_API_KEY=<fake API key>
/home/tester/.local/share/steplet/claude-cli/bin/claude
-p
--output-format
stream-json
--verbose
--include-hook-events
--max-turns
12
--setting-sources
user
--strict-mcp-config
--settings
/home/tester/.local/share/steplet/runs/S7a/settings.json
--mcp-config
/home/tester/.local/share/steplet/runs/S7a/mcp.json
--permission-prompt-tool
mcp__gate__decide
--- task text: Follow the spike script.
--- helpers (scripted model, listener): running, pids 427 428
launch inside WSL: exit code 0, seconds 4
--- launch finished after 3.3 s
--- Windows: spike-gate.exe processes still running: 0
--- event stream, read with a JSON parser:
bytes: 7812, lines: 7, raw 0x1B bytes: 0
fake key text occurrences in raw stream: 0
FORGED-FINAL-RESULT occurrences in raw stream: 0
INIT version=2.1.220 model=claude-opus-5[1m] permissionMode=default cwd=/home/tester/steplet-work/S7a/repo
     mcp_servers=[{"name": "gate", "status": "connected"}]
     tools=Task,AskUserQuestion,Bash,CronCreate,CronDelete,CronList,Edit,EnterPlanMode,EnterWorktree,ExitPlanMode,ExitWorktree,NotebookEdit,Read,ReportFindings,ScheduleWakeup,SendMessage,Skill,TaskCreate,TaskGet,TaskList,TaskOutput,TaskStop,TaskUpdate,WebFetch,WebSearch,Workflow,Write
TOOL_USE Bash: {"command": "git checkout 65930082d222089b2633d34e9bb2bd42351bdc76 2>&1; echo \"EXIT=$?\"", "description": "Check out the specified commit"}
TOOL_RESULT is_error=False:
    | Note: switching to '65930082d222089b2633d34e9bb2bd42351bdc76'.
    | 
    | You are in 'detached HEAD' state. You can look around, make experimental
    | changes and commit them, and you can discard any commits you make in this
    | state without impacting any branches by switching back to a branch.
    | 
    | If you want to create a new branch to retain commits you create, you may
    | do so (now or later) by using -c with the switch command. Example:
    | 
    |   git switch -c <new-branch-name>
    | 
    | Or undo this operation with:
    | 
    |   git switch -
    | 
TOOL_USE Bash: {"command": "python3 -m unittest discover -s tests -t . 2>&1; echo \"EXIT=$?\"", "description": "Run the unittest suite"}
TOOL_RESULT is_error=False:
    | /home/tester/steplet-work/S7a/repo/tests/fixtures.py:35: ResourceWarning: unclosed file <_io.BufferedReader name='/mnt/c/Windows/win.ini'>
    |   return "read %d bytes" % len(open(CFG["system_file"], "rb").read())
    | ResourceWarning: Enable tracemalloc to get the object allocation traceback
    | <3>WSL (6 - ) ERROR: UtilConnectUnix:524: socket failed 1
    | <3>WSL (6 - ) ERROR: UtilConnectUnix:524: socket failed 1
    | .
    | ----------------------------------------------------------------------
    | Ran 1 test in 0.085s
    | 
    | OK
    | EXIT=0
AGENT: Spike: the tests ran.
RESULT subtype=success is_error=False num_turns=3 duration_ms=715
       permission_denials=[]
       text: Spike: the tests ran.
event types: {'system/init': 1, 'assistant': 3, 'user': 2, 'result/success': 1}
top-level events of type 'result': 1
--- error output:
    (none)
--- gate log S7a.gate.log:
    2026-10-08T12:28:42.400Z allow {"tool_name":"Bash","input":{"command":"git checkout 65930082d222089b2633d34e9bb2bd42351bdc76 2>&1; echo \"EXIT=$?\"","description":"Check out the specified commit"},"tool_use_id":"toolu_spike_f9ca8bb86ade490db141"}
    2026-10-08T12:28:42.657Z allow {"tool_name":"Bash","input":{"command":"python3 -m unittest discover -s tests -t . 2>&1; echo \"EXIT=$?\"","description":"Run the unittest suite"},"tool_use_id":"toolu_spike_2da20b55bdd648889ed4"}
--- every tool call the agent made, and whether it asked the checker first:
    Bash       asked=yes  git checkout 65930082d222089b2633d34e9bb2bd42351bdc76 2>&1; echo "EXIT=$?"
    Bash       asked=yes  python3 -m unittest discover -s tests -t . 2>&1; echo "EXIT=$?"
--- scripted model: every request it received:
    +0.00s agent loop /v1/messages?beta=true step=0 answer=tool model=claude-opus-5 key_sent=True key_is_test_key=True bearer=False tools=27
    +0.29s agent loop /v1/messages?beta=true step=1 answer=tool model=claude-opus-5 key_sent=True key_is_test_key=True bearer=False tools=27
    +0.61s agent loop /v1/messages?beta=true step=2 answer=text model=claude-opus-5 key_sent=True key_is_test_key=True bearer=False tools=27
    tools offered to the model on the first agent request: Agent,AskUserQuestion,Bash,CronCreate,CronDelete,CronList,Edit,EnterPlanMode,EnterWorktree,ExitPlanMode,ExitWorktree,NotebookEdit,Read,ReportFindings,ScheduleWakeup,SendMessage,Skill,TaskCreate,TaskGet,TaskList,TaskOutput,TaskStop,TaskUpdate,WebFetch,WebSearch,Workflow,Write
    seconds between agent requests: [0.29, 0.32]
--- m0_results.jsonl (the fixture's own record):
    {"attempt": "A_write_inside_workspace", "ok": true, "detail": ""}
    {"attempt": "B_read_canary", "ok": false, "detail": "FileNotFoundError: [Errno 2] No such file or directory: '/home/tester/m1/canary/fake_key'"}
    {"attempt": "H_read_system_file", "ok": true, "detail": "read 92 bytes"}
    {"attempt": "C_write_outside_workspace", "ok": false, "detail": "OSError: [Errno 30] Read-only file system: '/home/tester/m1/outside/written_by_probe.txt'"}
    {"attempt": "D_connect_listener", "ok": false, "detail": "ConnectionRefusedError: [Errno 111] Connection refused"}
    {"attempt": "E_connect_external", "ok": false, "detail": "OSError: [Errno 101] Network is unreachable"}
    {"attempt": "child_read_canary", "ok": false, "detail": "FileNotFoundError: [Errno 2] No such file or directory: '/home/tester/m1/canary/fake_key'"}
    {"attempt": "child_connect_listener", "ok": false, "detail": "ConnectionRefusedError: [Errno 111] Connection refused"}
    {"attempt": "F_child_process", "ok": true, "detail": "child ran"}
    {"attempt": "G_launch_windows_program", "ok": false, "detail": "CalledProcessError: Command '['/mnt/c/Windows/System32/cmd.exe', '/c', 'echo x> C:\\\\dev\\\\m1\\\\outside\\\\from_wsl.txt']' returned non-zero exit status 1."}
--- listener inside WSL (127.0.0.1:47831):
    (none)
--- marker files: WSL none | Windows none
--- working copy HEAD: 65930082d222089b2633d34e9bb2bd42351bdc76
--- git status --short --ignored:
    !! calc/__pycache__/
    !! m0_inside.txt
    !! m0_results.jsonl
    !! tests/__pycache__/
    !! tests/m0_config.json
--- files in .git/hooks: ['applypatch-msg.sample', 'commit-msg.sample', 'fsmonitor-watchman.sample', 'post-update.sample', 'pre-applypatch.sample', 'pre-commit.sample', 'pre-merge-commit.sample', 'pre-push.sample', 'pre-rebase.sample', 'pre-receive.sample', 'prepare-commit-msg.sample', 'push-to-checkout.sample', 'sendemail-validate.sample', 'update.sample']
--- new files in Claude Code's private settings folder: ['.claude.json', 'backups/.claude.json.backup.1791462521240', 'projects/-home-tester-steplet-work-S7a-repo/45cd4b8c-9b35-4aa9-8bb8-40c2e8912c4a.jsonl']
--- new files in Steplet's private home: ['.cache/claude-cli-nodejs/-home-tester-steplet-work-S7a-repo/mcp-logs-gate/2026-10-08T12-28-41-217Z.jsonl', '.config/git/ignore']
--- private home .config/git/ignore: '**/.claude/.cc-writes/\n'
--- real ~/.config/git/ignore: before None | after None
--- Claude Code's record for this folder: before null | after null
--- S8 connection attempts:
    (none)
--- API key text found (counts only; the text is never printed):
    fake API key: 0 
    fake API key, last 20 characters: 0 
--- processes still running for this run: 0
=== spike run S7a finished 2026-10-08T17:58:45.5325182+05:30
```

### S7b

```
=== spike run S7b, started 2026-10-08T17:58:50.8363666+05:30
--- run description: {"about": "S7: same with the product settings shape (allowRead, denyWrite on .git)", "script": "s7_t1c.json", "gate": "node", "gate_mode": "allow", "shape": "product", "launch": "inside", "env_extra": ["ANTHROPIC_API_KEY=@fake-api-key"]}
--- working copy: /home/tester/steplet-work/S7b/repo HEAD 7f4b72d7981454558a3631310426dcff4d4ebd50
--- settings.json:
{
  "sandbox": {
    "enabled": true,
    "failIfUnavailable": true,
    "allowUnsandboxedCommands": false,
    "autoAllowBashIfSandboxed": false,
    "filesystem": {
      "denyRead": [
        "/mnt",
        "/home/tester/.local/share/steplet",
        "/home/tester/steplet-work",
        "/home/tester/m1"
      ],
      "allowRead": [
        "/home/tester/steplet-work/S7b/repo"
      ],
      "denyWrite": [
        "/home/tester/steplet-work/S7b/repo/.git"
      ]
    },
    "network": {
      "allowedDomains": [],
      "strictAllowlist": true
    }
  },
  "permissions": {
    "deny": [
      "Read(//mnt/**)",
      "Read(//home/tester/.local/share/steplet/**)",
      "Read(//home/tester/steplet-work/**)",
      "Read(//home/tester/m1/**)",
      "Edit(//home/tester/steplet-work/S7b/repo/.git/**)"
    ]
  }
}
--- MCP server: {"type": "stdio", "command": "/usr/bin/node", "args": ["/home/tester/m1/tools/gate/gate.mjs"], "env": {"GATE_MODE": "allow", "GATE_LOG": "/home/tester/m1/logs/S7b.gate.log"}}
--- launch command, one argument per line:
/usr/bin/timeout
-k
10
600
/usr/bin/env
-i
HOME=/home/tester/.local/share/steplet/home
PATH=/usr/local/bin:/usr/bin:/bin
LANG=C.UTF-8
CLAUDE_CONFIG_DIR=/home/tester/.local/share/steplet/claude-config
DISABLE_AUTOUPDATER=1
CLAUDE_CODE_DISABLE_NONESSENTIAL_TRAFFIC=1
ANTHROPIC_BASE_URL=http://127.0.0.1:47900
ANTHROPIC_API_KEY=<fake API key>
/home/tester/.local/share/steplet/claude-cli/bin/claude
-p
--output-format
stream-json
--verbose
--include-hook-events
--max-turns
12
--setting-sources
user
--strict-mcp-config
--settings
/home/tester/.local/share/steplet/runs/S7b/settings.json
--mcp-config
/home/tester/.local/share/steplet/runs/S7b/mcp.json
--permission-prompt-tool
mcp__gate__decide
--- task text: Follow the spike script.
--- helpers (scripted model, listener): running, pids 598 599
launch inside WSL: exit code 0, seconds 2
--- launch finished after 2 s
--- Windows: spike-gate.exe processes still running: 0
--- event stream, read with a JSON parser:
bytes: 5912, lines: 7, raw 0x1B bytes: 0
fake key text occurrences in raw stream: 0
FORGED-FINAL-RESULT occurrences in raw stream: 0
INIT version=2.1.220 model=claude-opus-5[1m] permissionMode=default cwd=/home/tester/steplet-work/S7b/repo
     mcp_servers=[{"name": "gate", "status": "connected"}]
     tools=Task,AskUserQuestion,Bash,CronCreate,CronDelete,CronList,Edit,EnterPlanMode,EnterWorktree,ExitPlanMode,ExitWorktree,NotebookEdit,Read,ReportFindings,ScheduleWakeup,SendMessage,Skill,TaskCreate,TaskGet,TaskList,TaskOutput,TaskStop,TaskUpdate,WebFetch,WebSearch,Workflow,Write
TOOL_USE Bash: {"command": "git checkout 65930082d222089b2633d34e9bb2bd42351bdc76 2>&1; echo \"EXIT=$?\"", "description": "Check out the specified commit"}
TOOL_RESULT is_error=False:
    | fatal: Unable to create '/home/tester/steplet-work/S7b/repo/.git/index.lock': Read-only file system
    | EXIT=128
TOOL_USE Bash: {"command": "python3 -m unittest discover -s tests -t . 2>&1; echo \"EXIT=$?\"", "description": "Run the unittest suite"}
TOOL_RESULT is_error=False:
    | .
    | ----------------------------------------------------------------------
    | Ran 1 test in 0.067s
    | 
    | OK
    | EXIT=0
AGENT: Spike: the tests ran.
RESULT subtype=success is_error=False num_turns=3 duration_ms=649
       permission_denials=[]
       text: Spike: the tests ran.
event types: {'system/init': 1, 'assistant': 3, 'user': 2, 'result/success': 1}
top-level events of type 'result': 1
--- error output:
    (none)
--- gate log S7b.gate.log:
    2026-10-08T12:28:54.102Z allow {"tool_name":"Bash","input":{"command":"git checkout 65930082d222089b2633d34e9bb2bd42351bdc76 2>&1; echo \"EXIT=$?\"","description":"Check out the specified commit"},"tool_use_id":"toolu_spike_1a0637af5a984aba9223"}
    2026-10-08T12:28:54.325Z allow {"tool_name":"Bash","input":{"command":"python3 -m unittest discover -s tests -t . 2>&1; echo \"EXIT=$?\"","description":"Run the unittest suite"},"tool_use_id":"toolu_spike_b108e0fe9b77466bb9db"}
--- every tool call the agent made, and whether it asked the checker first:
    Bash       asked=yes  git checkout 65930082d222089b2633d34e9bb2bd42351bdc76 2>&1; echo "EXIT=$?"
    Bash       asked=yes  python3 -m unittest discover -s tests -t . 2>&1; echo "EXIT=$?"
--- scripted model: every request it received:
    +0.00s agent loop /v1/messages?beta=true step=0 answer=tool model=claude-opus-5 key_sent=True key_is_test_key=True bearer=False tools=27
    +0.24s agent loop /v1/messages?beta=true step=1 answer=tool model=claude-opus-5 key_sent=True key_is_test_key=True bearer=False tools=27
    +0.57s agent loop /v1/messages?beta=true step=2 answer=text model=claude-opus-5 key_sent=True key_is_test_key=True bearer=False tools=27
    tools offered to the model on the first agent request: Agent,AskUserQuestion,Bash,CronCreate,CronDelete,CronList,Edit,EnterPlanMode,EnterWorktree,ExitPlanMode,ExitWorktree,NotebookEdit,Read,ReportFindings,ScheduleWakeup,SendMessage,Skill,TaskCreate,TaskGet,TaskList,TaskOutput,TaskStop,TaskUpdate,WebFetch,WebSearch,Workflow,Write
    seconds between agent requests: [0.24, 0.33]
--- m0_results.jsonl (the fixture's own record):
    {"attempt": "A_write_inside_workspace", "ok": true, "detail": ""}
    {"attempt": "B_read_canary", "ok": false, "detail": "FileNotFoundError: [Errno 2] No such file or directory: '/home/tester/m1/canary/fake_key'"}
    {"attempt": "H_read_system_file", "ok": false, "detail": "FileNotFoundError: [Errno 2] No such file or directory: '/mnt/c/Windows/win.ini'"}
    {"attempt": "C_write_outside_workspace", "ok": false, "detail": "FileNotFoundError: [Errno 2] No such file or directory: '/home/tester/m1/outside/written_by_probe.txt'"}
    {"attempt": "D_connect_listener", "ok": false, "detail": "ConnectionRefusedError: [Errno 111] Connection refused"}
    {"attempt": "E_connect_external", "ok": false, "detail": "OSError: [Errno 101] Network is unreachable"}
    {"attempt": "child_read_canary", "ok": false, "detail": "FileNotFoundError: [Errno 2] No such file or directory: '/home/tester/m1/canary/fake_key'"}
    {"attempt": "child_connect_listener", "ok": false, "detail": "ConnectionRefusedError: [Errno 111] Connection refused"}
    {"attempt": "F_child_process", "ok": true, "detail": "child ran"}
    {"attempt": "G_launch_windows_program", "ok": false, "detail": "FileNotFoundError: [Errno 2] No such file or directory: '/mnt/c/Windows/System32/cmd.exe'"}
--- listener inside WSL (127.0.0.1:47831):
    (none)
--- marker files: WSL none | Windows none
--- working copy HEAD: 7f4b72d7981454558a3631310426dcff4d4ebd50
--- git status --short --ignored:
    !! calc/__pycache__/
    !! m0_inside.txt
    !! m0_results.jsonl
    !! tests/__pycache__/
    !! tests/m0_config.json
--- files in .git/hooks: ['applypatch-msg.sample', 'commit-msg.sample', 'fsmonitor-watchman.sample', 'post-update.sample', 'pre-applypatch.sample', 'pre-commit.sample', 'pre-merge-commit.sample', 'pre-push.sample', 'pre-rebase.sample', 'pre-receive.sample', 'prepare-commit-msg.sample', 'push-to-checkout.sample', 'sendemail-validate.sample', 'update.sample']
--- new files in Claude Code's private settings folder: ['projects/-home-tester-steplet-work-S7b-repo/07e8a300-d21b-4e78-a89d-433028039a9c.jsonl']
--- new files in Steplet's private home: ['.cache/claude-cli-nodejs/-home-tester-steplet-work-S7b-repo/mcp-logs-gate/2026-10-08T12-28-53-424Z.jsonl']
--- private home .config/git/ignore: '**/.claude/.cc-writes/\n'
--- real ~/.config/git/ignore: before None | after None
--- Claude Code's record for this folder: before null | after null
--- S8 connection attempts:
    (none)
--- API key text found (counts only; the text is never printed):
    fake API key: 0 
    fake API key, last 20 characters: 0 
--- processes still running for this run: 0
=== spike run S7b finished 2026-10-08T17:58:57.4346643+05:30
```

**Verdict:** PASS for S7a, because every check in the plan's list holds: both Bash calls were asked of the
gate (`asked=yes`), the gate's request has the same fields as T1c, the fixture record matches (A ok; B, C, D, E
and both child lines not ok; G not ok; H ok), the listener log is empty, no marker file exists, and the test
ran (`Ran 1 test`, `OK`). S7b behaves as the plan predicted: H is blocked, and the checkout fails with
`Read-only file system` (exit 128). **Gap:** in S7b the tests ran on the original HEAD `7f4b72d`, not COMMIT1,
because the checkout was blocked, so S7b's test result does not show the same thing as T1c. C also fails with
a different error in S7b (the folder is hidden, so "No such file") than in S7a ("Read-only file system"); both
block the write.

**What it means:** a scripted model does not change what the gate or the fence sees for these two commands, so
later spikes and the phase 5 suite may use it for these actions. `allowRead` works in Claude Code 2.1.220: the
tests read their own files under `/home/tester/steplet-work/S7b/repo` while `/mnt` and the private folder are
denied, so the `product` shape is kept (no `product-noallow`). The product shape does block the checkout inside
the agent's run, which is what decision D6 intends: Steplet must do the checkout itself, as its own fenced step
before the agent starts. S4 has to test that step.

## S2: launching Claude Code from Windows

Question, from the milestone plan: What does `wsl.exe --exec` add to the environment? Do Linux processes outlive
a killed `wsl.exe`? Does Claude Code work with Steplet's private `HOME`, and does F49's git ignore file land there?
Pass: no Windows variables cross; no process outlives the run once the clean-up runs; F49's file lands in the
private home.

Runs: `spikes/runs/S2r-empty.json`, `S2r-systemroot.json`, `S2r-inherit.json` (what `wsl.exe` passes on in each
environment mode), `S2a.json` (a full run launched from Windows), `S2c.json` (kill `wsl.exe` mid-command, then clean
up). `S2d.json` was not run: S2c left no processes.
Command: `powershell -NoProfile -ExecutionPolicy Bypass -File spikes\tools\run.ps1 -Id <run>`

**First attempt (8 Oct 2026) was not a measurement.** `run.ps1` built the host's options as
`"--" + $p.Name, "$($p.Value)"`. In PowerShell the comma binds tighter than `+`, so `--env-mode systemroot` reached
the host as one argument, the host did not recognise it, and fell back to an empty environment in all three runs
(each printed `environment Empty`). Fixed in `run.ps1` with brackets; the logs below are the second attempt
(9 Oct 2026), where the host prints the mode it was given.

**Change to the run descriptions, as the plan allows:** `empty` failed and `systemroot` worked, so S2a, S2c and S2d
use `"env_mode": "systemroot"`. Every later run description with a `host` field must do the same.

### S2r-empty

The five "variables" listed are not variables: they are `wsl.exe`'s own error message (UTF-16, hence the spaces),
which the harness read as `env` output.

```
=== spike run S2r-empty, started 2026-10-09T10:38:45.0758479+05:30
--- run description: {"about": "S2: the environment wsl.exe gives a program when the launcher passes no Windows variables", "script": "none", "gate": "node", "shape": "product", "launch": "host", "host": {"env_mode": "empty"}, "argv_override": ["/usr/bin/env"], "evidence": "env"}
--- working copy: /home/tester/steplet-work/S2r-empty/repo HEAD 7f4b72d7981454558a3631310426dcff4d4ebd50
--- settings.json:
{
  "sandbox": {
    "enabled": true,
    "failIfUnavailable": true,
    "allowUnsandboxedCommands": false,
    "autoAllowBashIfSandboxed": false,
    "filesystem": {
      "denyRead": [
        "/mnt",
        "/home/tester/.local/share/steplet",
        "/home/tester/steplet-work",
        "/home/tester/m1"
      ],
      "allowRead": [
        "/home/tester/steplet-work/S2r-empty/repo"
      ],
      "denyWrite": [
        "/home/tester/steplet-work/S2r-empty/repo/.git"
      ]
    },
    "network": {
      "allowedDomains": [],
      "strictAllowlist": true
    }
  },
  "permissions": {
    "deny": [
      "Read(//mnt/**)",
      "Read(//home/tester/.local/share/steplet/**)",
      "Read(//home/tester/steplet-work/**)",
      "Read(//home/tester/m1/**)",
      "Edit(//home/tester/steplet-work/S2r-empty/repo/.git/**)"
    ]
  }
}
--- MCP server: {"type": "stdio", "command": "/usr/bin/node", "args": ["/home/tester/m1/tools/gate/gate.mjs"], "env": {"GATE_MODE": "allow", "GATE_LOG": "/home/tester/m1/logs/S2r-empty.gate.log"}}
--- launch command, one argument per line:
/usr/bin/env
--- task text: Follow the spike script.
--- helpers (scripted model, listener): running, pids 427
host: wsl.exe started (pid Some(35252), environment Empty); task sent on its input, 24 bytes
host: wsl.exe ended: Ok(Some(-1)), after 0.1 s
--- launch finished after 0.1 s
--- Windows: spike-gate.exe processes still running: 0
--- the environment the command received (sorted; values shown only for harmless names):
    \x00=<value not shown>
    \x00=<value not shown>
    \x00=<value not shown>
    \x00E\x00r\x00r\x00o\x00r\x00 \x00c\x00o\x00d\x00e\x00:\x00 \x00W\x00s\x00l\x00/\x00S\x00e\x00r\x00v\x00i\x00c\x00e\x00/\x000\x00x\x008\x000\x000\x007\x000\x007\x002\x00c\x00=<value not shown>
    T\x00h\x00e\x00 \x00R\x00P\x00C\x00 \x00c\x00a\x00l\x00l\x00 \x00c\x00o\x00n\x00t\x00a\x00i\x00n\x00s\x00 \x00a\x00 \x00h\x00a\x00n\x00d\x00l\x00e\x00 \x00t\x00h\x00a\x00t\x00 \x00d\x00i\x00f\x00f\x00e\x00r\x00s\x00 \x00f\x00r\x00o\x00m\x00 \x00t\x00h\x00e\x00 \x00d\x00e\x00c\x00l\x00a\x00r\x00e\x00d\x00 \x00h\x00a\x00n\x00d\x00l\x00e\x00 \x00t\x00y\x00p\x00e\x00.\x00 \x00=<value not shown>
    (5 variables)
--- its error output:
    (none)
=== spike run S2r-empty finished 2026-10-09T10:38:52.1030677+05:30
```

### S2r-systemroot

```
=== spike run S2r-systemroot, started 2026-10-09T10:38:52.5058890+05:30
--- run description: {"about": "S2: the same with only SystemRoot passed", "script": "none", "gate": "node", "shape": "product", "launch": "host", "host": {"env_mode": "systemroot"}, "argv_override": ["/usr/bin/env"], "evidence": "env"}
--- working copy: /home/tester/steplet-work/S2r-systemroot/repo HEAD 7f4b72d7981454558a3631310426dcff4d4ebd50
--- settings.json:
{
  "sandbox": {
    "enabled": true,
    "failIfUnavailable": true,
    "allowUnsandboxedCommands": false,
    "autoAllowBashIfSandboxed": false,
    "filesystem": {
      "denyRead": [
        "/mnt",
        "/home/tester/.local/share/steplet",
        "/home/tester/steplet-work",
        "/home/tester/m1"
      ],
      "allowRead": [
        "/home/tester/steplet-work/S2r-systemroot/repo"
      ],
      "denyWrite": [
        "/home/tester/steplet-work/S2r-systemroot/repo/.git"
      ]
    },
    "network": {
      "allowedDomains": [],
      "strictAllowlist": true
    }
  },
  "permissions": {
    "deny": [
      "Read(//mnt/**)",
      "Read(//home/tester/.local/share/steplet/**)",
      "Read(//home/tester/steplet-work/**)",
      "Read(//home/tester/m1/**)",
      "Edit(//home/tester/steplet-work/S2r-systemroot/repo/.git/**)"
    ]
  }
}
--- MCP server: {"type": "stdio", "command": "/usr/bin/node", "args": ["/home/tester/m1/tools/gate/gate.mjs"], "env": {"GATE_MODE": "allow", "GATE_LOG": "/home/tester/m1/logs/S2r-systemroot.gate.log"}}
--- launch command, one argument per line:
/usr/bin/env
--- task text: Follow the spike script.
--- helpers (scripted model, listener): running, pids 453
host: wsl.exe started (pid Some(19124), environment SystemRoot); task sent on its input, 24 bytes
host: wsl.exe ended: Ok(Some(0)), after 0.1 s
--- launch finished after 0.1 s
--- Windows: spike-gate.exe processes still running: 0
--- the environment the command received (sorted; values shown only for harmless names):
    DBUS_SESSION_BUS_ADDRESS=<value not shown>
    DISPLAY=:0
    HOME=/home/tester
    HOSTTYPE=x86_64
    LANG=C.UTF-8
    LOGNAME=tester
    NAME=LAPTOP-R1RG22O7
    PATH=/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin:/usr/games:/usr/local/games:/usr/lib/wsl/lib
    PULSE_SERVER=unix:/mnt/wslg/PulseServer
    SHELL=/bin/bash
    TERM=xterm-256color
    USER=tester
    WAYLAND_DISPLAY=wayland-0
    WSL2_GUI_APPS_ENABLED=1
    WSLENV=
    WSL_DISTRO_NAME=steplet-test
    WSL_INTEROP=/run/WSL/459_interop
    XDG_RUNTIME_DIR=/run/user/1000/
    (18 variables)
--- its error output:
    (none)
=== spike run S2r-systemroot finished 2026-10-09T10:38:57.0316685+05:30
```

### S2r-inherit

```
=== spike run S2r-inherit, started 2026-10-09T10:38:57.4086975+05:30
--- run description: {"about": "S2: the same with the launcher's whole Windows environment, for comparison", "script": "none", "gate": "node", "shape": "product", "launch": "host", "host": {"env_mode": "inherit"}, "argv_override": ["/usr/bin/env"], "evidence": "env"}
--- working copy: /home/tester/steplet-work/S2r-inherit/repo HEAD 7f4b72d7981454558a3631310426dcff4d4ebd50
--- settings.json:
{
  "sandbox": {
    "enabled": true,
    "failIfUnavailable": true,
    "allowUnsandboxedCommands": false,
    "autoAllowBashIfSandboxed": false,
    "filesystem": {
      "denyRead": [
        "/mnt",
        "/home/tester/.local/share/steplet",
        "/home/tester/steplet-work",
        "/home/tester/m1"
      ],
      "allowRead": [
        "/home/tester/steplet-work/S2r-inherit/repo"
      ],
      "denyWrite": [
        "/home/tester/steplet-work/S2r-inherit/repo/.git"
      ]
    },
    "network": {
      "allowedDomains": [],
      "strictAllowlist": true
    }
  },
  "permissions": {
    "deny": [
      "Read(//mnt/**)",
      "Read(//home/tester/.local/share/steplet/**)",
      "Read(//home/tester/steplet-work/**)",
      "Read(//home/tester/m1/**)",
      "Edit(//home/tester/steplet-work/S2r-inherit/repo/.git/**)"
    ]
  }
}
--- MCP server: {"type": "stdio", "command": "/usr/bin/node", "args": ["/home/tester/m1/tools/gate/gate.mjs"], "env": {"GATE_MODE": "allow", "GATE_LOG": "/home/tester/m1/logs/S2r-inherit.gate.log"}}
--- launch command, one argument per line:
/usr/bin/env
--- task text: Follow the spike script.
--- helpers (scripted model, listener): running, pids 480
host: wsl.exe started (pid Some(26632), environment Inherit); task sent on its input, 24 bytes
host: wsl.exe ended: Ok(Some(0)), after 0.1 s
--- launch finished after 0.1 s
--- Windows: spike-gate.exe processes still running: 0
--- the environment the command received (sorted; values shown only for harmless names):
    DBUS_SESSION_BUS_ADDRESS=<value not shown>
    DISPLAY=:0
    HOME=/home/tester
    HOSTTYPE=x86_64
    LANG=C.UTF-8
    LOGNAME=tester
    NAME=LAPTOP-R1RG22O7
    PATH=/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin:/usr/games:/usr/local/games:/usr/lib/wsl/lib:/mnt/c/WINDOWS/system32:/mnt/c/WINDOWS:/mnt/c/WINDOWS/System32/Wbem:/mnt/c/WINDOWS/System32/WindowsPowerShell/v1.0/:/mnt/c/WINDOWS/System32/OpenSSH/:/mnt/c/Program Files (x86)/NVIDIA Corporation/PhysX/Common:/mnt/c/Program Files/dotnet/:/mnt/c/Program Files/NVIDIA Corporation/NVIDIA App/NvDLISR:/mnt/c/Program Files/Git/cmd:/mnt/c/Program Files/PuTTY/:/mnt/c/mingw64/bin:/mnt/c/msys64/ucrt64/bin:/mnt/c/Program Files/nodejs/:/mnt/c/Program Files/Cloudflare/Cloudflare WARP/:/mnt/c/Program Files/GitHub CLI/:/mnt/c/Users/RANVIR/AppData/Local/Programs/Python/Python312/Scripts/:/mnt/c/Users/RANVIR/AppData/Local/Programs/Python/Python312/:/mnt/c/Users/RANVIR/AppData/Local/Programs/Python/Launcher/:/mnt/c/Users/RANVIR/.local/bin:/mnt/c/Users/RANVIR/AppData/Local/Microsoft/WindowsApps:/mnt/c/Users/RANVIR/AppData/Local/Programs/Microsoft VS Code/bin:/mnt/c/Users/RANVIR/AppData/Local/Programs/Antigravity/bin:/mnt/c/Users/RANVIR/AppData/Local/Programs/Ollama:/mnt/c/Users/RANVIR/AppData/Roaming/npm:/mnt/c/Users/RANVIR/AppData/Local/Microsoft/WinGet/Packages/Ngrok.Ngrok_Microsoft.Winget.Source_8wekyb3d8bbwe:/mnt/c/Users/RANVIR/AppData/Local/Programs/cursor/resources/app/bin:/mnt/c/Program Files/Blender Foundation/Blender 4.5:/mnt/c/Users/RANVIR/.cargo/bin:/mnt/c/Users/RANVIR/AppData/Local/Programs/Python/Python312:/mnt/c/Program Files/nodejs:/mnt/c/Program Files/Git/mingw64/bin:/mnt/c/Program Files/GitHub CLI
    PULSE_SERVER=unix:/mnt/wslg/PulseServer
    SHELL=/bin/bash
    TERM=xterm-256color
    USER=tester
    WAYLAND_DISPLAY=wayland-0
    WSL2_GUI_APPS_ENABLED=1
    WSLENV=
    WSL_DISTRO_NAME=steplet-test
    WSL_INTEROP=/run/WSL/485_interop
    XDG_RUNTIME_DIR=/run/user/1000/
    (18 variables)
--- its error output:
    (none)
=== spike run S2r-inherit finished 2026-10-09T10:39:01.8925131+05:30
```

### Why the empty environment fails, and what it does to the distro

`spikes/tools/s2_env_min.py` starts `wsl.exe --distribution-id <id> --exec /bin/echo STARTED` with a chosen Windows
environment, 15 s limit, and prints variable names only. Sections 4 and 5 stop the distro first, so the tested call
is the one that has to start it.

```
=== 1. each case on a healthy distro (the distro is restarted after any failure)
command: python -I spikes\tools\s2_env_min.py full none SystemRoot SystemRoot SystemRoot
OK   vars= 94 hung=False exit=0 first_line='STARTED'
     names: (full environment)
FAIL vars=  0 hung=False exit=4294967295 first_line='The RPC call contains a handle that differs from the declared handle type. '
     names: (none)
     distro reset: ok
OK   vars=  1 hung=False exit=0 first_line='STARTED'
     names: SYSTEMROOT
OK   vars=  1 hung=False exit=0 first_line='STARTED'
     names: SYSTEMROOT
OK   vars=  1 hung=False exit=0 first_line='STARTED'
     names: SYSTEMROOT
=== 2. empty, then SystemRoot, then full, with no restart in between and no other session open
command: python -I spikes\tools\s2_env_min.py --no-reset none SystemRoot full
FAIL vars=  0 hung=False exit=4294967295 first_line='The RPC call contains a handle that differs from the declared handle type. '
     names: (none)
OK   vars=  1 hung=False exit=0 first_line='STARTED'
     names: SYSTEMROOT
OK   vars= 94 hung=False exit=0 first_line='STARTED'
     names: (full environment)
final distro reset: ok
=== 3. the same order while another session of the distro is open (wsl.exe -d steplet-test --exec /bin/sleep 60)
command: python -I spikes\tools\s2_env_min.py --no-reset none full SystemRoot
FAIL vars=  0 hung=False exit=4294967295 first_line='The RPC call contains a handle that differs from the declared handle type. '
     names: (none)
OK   vars= 94 hung=False exit=0 first_line='STARTED'
     names: (full environment)
OK   vars=  1 hung=False exit=0 first_line='STARTED'
     names: SYSTEMROOT
final distro reset: ok
=== 4.1 the distro is stopped first, so the empty call is the one that has to start it
command: wsl.exe --terminate steplet-test; python -I spikes\tools\s2_env_min.py --no-reset none full
terminate: T\x00h\x00e\x00 \x00o\x00p\x00e\x00r\x00a\x00t\x00i\x00o\x00n\x00 \x00c\x00o\x00m\x00p\x00l\x00e\x00t\x00e\x00d\x00 \x00s\x00u\x00c\x00c\x00e\x00s\x00s\x00f\x00u\x00l\x00l\x00y\x00.\x00 \x00
terminate: \x00
terminate: \x00
state before: steplet-test    Stopped         2
FAIL vars=  0 hung=False exit=4294967295 first_line='The RPC call contains a handle that differs from the declared handle type. '
     names: (none)
FAIL vars= 95 hung=True exit=None first_line=''
     names: (full environment)
final distro reset: ok
=== 4.2 the distro is stopped first, so the empty call is the one that has to start it
command: wsl.exe --terminate steplet-test; python -I spikes\tools\s2_env_min.py --no-reset none full
terminate: The operation completed successfully. 
state before: steplet-test    Stopped         2
FAIL vars=  0 hung=False exit=4294967295 first_line='The RPC call contains a handle that differs from the declared handle type. '
     names: (none)
FAIL vars= 95 hung=True exit=None first_line=''
     names: (full environment)
final distro reset: ok
=== 5.1 the distro is stopped first, and the SystemRoot-only call is the one that has to start it
command: wsl.exe --terminate steplet-test; python -I spikes\tools\s2_env_min.py --no-reset SystemRoot full
terminate: T\x00h\x00e\x00 \x00o\x00p\x00e\x00r\x00a\x00t\x00i\x00o\x00n\x00 \x00c\x00o\x00m\x00p\x00l\x00e\x00t\x00e\x00d\x00 \x00s\x00u\x00c\x00c\x00e\x00s\x00s\x00f\x00u\x00l\x00l\x00y\x00.\x00 \x00
terminate: \x00
terminate: \x00
state before: steplet-test    Stopped         2
OK   vars=  1 hung=False exit=0 first_line='STARTED'
     names: SYSTEMROOT
OK   vars= 95 hung=False exit=0 first_line='STARTED'
     names: (full environment)
final distro reset: ok
=== 5.2 the distro is stopped first, and the SystemRoot-only call is the one that has to start it
command: wsl.exe --terminate steplet-test; python -I spikes\tools\s2_env_min.py --no-reset SystemRoot full
terminate: The operation completed successfully. 
state before: steplet-test    Stopped         2
OK   vars=  1 hung=False exit=0 first_line='STARTED'
     names: SYSTEMROOT
OK   vars= 95 hung=False exit=0 first_line='STARTED'
     names: (full environment)
final distro reset: ok
```

### S2a

```
=== spike run S2a, started 2026-10-09T10:39:14.0910561+05:30
--- run description: {"about": "S2: a full run launched from Windows; what the fence sees; where F49's git ignore file lands", "script": "s2_env.json", "gate": "node", "gate_mode": "allow", "shape": "product", "launch": "host", "host": {"env_mode": "systemroot"}, "probes": ["s3_probe.py"], "env_extra": ["ANTHROPIC_API_KEY=@fake-api-key"]}
--- working copy: /home/tester/steplet-work/S2a/repo HEAD 7f4b72d7981454558a3631310426dcff4d4ebd50
--- settings.json:
{
  "sandbox": {
    "enabled": true,
    "failIfUnavailable": true,
    "allowUnsandboxedCommands": false,
    "autoAllowBashIfSandboxed": false,
    "filesystem": {
      "denyRead": [
        "/mnt",
        "/home/tester/.local/share/steplet",
        "/home/tester/steplet-work",
        "/home/tester/m1"
      ],
      "allowRead": [
        "/home/tester/steplet-work/S2a/repo"
      ],
      "denyWrite": [
        "/home/tester/steplet-work/S2a/repo/.git"
      ]
    },
    "network": {
      "allowedDomains": [],
      "strictAllowlist": true
    }
  },
  "permissions": {
    "deny": [
      "Read(//mnt/**)",
      "Read(//home/tester/.local/share/steplet/**)",
      "Read(//home/tester/steplet-work/**)",
      "Read(//home/tester/m1/**)",
      "Edit(//home/tester/steplet-work/S2a/repo/.git/**)"
    ]
  }
}
--- MCP server: {"type": "stdio", "command": "/usr/bin/node", "args": ["/home/tester/m1/tools/gate/gate.mjs"], "env": {"GATE_MODE": "allow", "GATE_LOG": "/home/tester/m1/logs/S2a.gate.log"}}
--- launch command, one argument per line:
/usr/bin/timeout
-k
10
600
/usr/bin/env
-i
HOME=/home/tester/.local/share/steplet/home
PATH=/usr/local/bin:/usr/bin:/bin
LANG=C.UTF-8
CLAUDE_CONFIG_DIR=/home/tester/.local/share/steplet/claude-config
DISABLE_AUTOUPDATER=1
CLAUDE_CODE_DISABLE_NONESSENTIAL_TRAFFIC=1
ANTHROPIC_BASE_URL=http://127.0.0.1:47900
ANTHROPIC_API_KEY=<fake API key>
/home/tester/.local/share/steplet/claude-cli/bin/claude
-p
--output-format
stream-json
--verbose
--include-hook-events
--max-turns
12
--setting-sources
user
--strict-mcp-config
--settings
/home/tester/.local/share/steplet/runs/S2a/settings.json
--mcp-config
/home/tester/.local/share/steplet/runs/S2a/mcp.json
--permission-prompt-tool
mcp__gate__decide
--- task text: Follow the spike script.
--- helpers (scripted model, listener): running, pids 512 513
host: wsl.exe started (pid Some(35604), environment SystemRoot); task sent on its input, 24 bytes
host: wsl.exe ended: Ok(Some(0)), after 2.8 s
--- launch finished after 2.8 s
--- Windows: spike-gate.exe processes still running: 0
--- event stream, read with a JSON parser:
bytes: 5577, lines: 7, raw 0x1B bytes: 0
fake key text occurrences in raw stream: 0
FORGED-FINAL-RESULT occurrences in raw stream: 0
INIT version=2.1.220 model=claude-opus-5[1m] permissionMode=default cwd=/home/tester/steplet-work/S2a/repo
     mcp_servers=[{"name": "gate", "status": "connected"}]
     tools=Task,AskUserQuestion,Bash,CronCreate,CronDelete,CronList,Edit,EnterPlanMode,EnterWorktree,ExitPlanMode,ExitWorktree,NotebookEdit,Read,ReportFindings,ScheduleWakeup,SendMessage,Skill,TaskCreate,TaskGet,TaskList,TaskOutput,TaskStop,TaskUpdate,WebFetch,WebSearch,Workflow,Write
TOOL_USE Bash: {"command": "python3 spike_probes/s3_probe.py; echo \"EXIT=$?\"", "description": "Record what the fence can see"}
TOOL_RESULT is_error=False:
    | probe: 53 environment variables, 5 /proc files copied, 0 reads tried
    | 
    | EXIT=0
TOOL_USE Bash: {"command": "git status --short > /dev/null; echo \"EXIT=$?\"", "description": "Run git once"}
TOOL_RESULT is_error=False:
    | EXIT=0
AGENT: Spike S2: done.
RESULT subtype=success is_error=False num_turns=3 duration_ms=437
       permission_denials=[]
       text: Spike S2: done.
event types: {'system/init': 1, 'assistant': 3, 'user': 2, 'result/success': 1}
top-level events of type 'result': 1
--- error output:
    (none)
--- gate log S2a.gate.log:
    2026-10-09T05:09:18.272Z allow {"tool_name":"Bash","input":{"command":"python3 spike_probes/s3_probe.py; echo \"EXIT=$?\"","description":"Record what the fence can see"},"tool_use_id":"toolu_spike_ec851c13f50b4a1eb998"}
    2026-10-09T05:09:18.482Z allow {"tool_name":"Bash","input":{"command":"git status --short > /dev/null; echo \"EXIT=$?\"","description":"Run git once"},"tool_use_id":"toolu_spike_f1ccdbe4e05e4b369de9"}
--- every tool call the agent made, and whether it asked the checker first:
    Bash       asked=yes  python3 spike_probes/s3_probe.py; echo "EXIT=$?"
    Bash       asked=yes  git status --short > /dev/null; echo "EXIT=$?"
--- scripted model: every request it received:
    +0.00s agent loop /v1/messages?beta=true step=0 answer=tool model=claude-opus-5 key_sent=True key_is_test_key=True bearer=False tools=27
    +0.23s agent loop /v1/messages?beta=true step=1 answer=tool model=claude-opus-5 key_sent=True key_is_test_key=True bearer=False tools=27
    +0.36s agent loop /v1/messages?beta=true step=2 answer=text model=claude-opus-5 key_sent=True key_is_test_key=True bearer=False tools=27
    tools offered to the model on the first agent request: Agent,AskUserQuestion,Bash,CronCreate,CronDelete,CronList,Edit,EnterPlanMode,EnterWorktree,ExitPlanMode,ExitWorktree,NotebookEdit,Read,ReportFindings,ScheduleWakeup,SendMessage,Skill,TaskCreate,TaskGet,TaskList,TaskOutput,TaskStop,TaskUpdate,WebFetch,WebSearch,Workflow,Write
    seconds between agent requests: [0.23, 0.13]
--- m0_results.jsonl (the fixture's own record):
    (none)
--- listener inside WSL (127.0.0.1:47831):
    (none)
--- marker files: WSL none | Windows none
--- working copy HEAD: 7f4b72d7981454558a3631310426dcff4d4ebd50
--- git status --short --ignored:
    ?? s3_seen/
    ?? spike_probes/
    !! tests/m0_config.json
--- files in .git/hooks: ['applypatch-msg.sample', 'commit-msg.sample', 'fsmonitor-watchman.sample', 'post-update.sample', 'pre-applypatch.sample', 'pre-commit.sample', 'pre-merge-commit.sample', 'pre-push.sample', 'pre-rebase.sample', 'pre-receive.sample', 'prepare-commit-msg.sample', 'push-to-checkout.sample', 'sendemail-validate.sample', 'update.sample']
--- new files in Claude Code's private settings folder: ['projects/-home-tester-steplet-work-S2a-repo/93409842-552b-46be-9db1-32d53f56dff5.jsonl']
--- new files in Steplet's private home: ['.cache/claude-cli-nodejs/-home-tester-steplet-work-S2a-repo/mcp-logs-gate/2026-10-09T05-09-17-307Z.jsonl']
--- private home .config/git/ignore: '**/.claude/.cc-writes/\n'
--- real ~/.config/git/ignore: before None | after None
--- Claude Code's record for this folder: before null | after null
--- environment variable names inside the fence: AI_AGENT,ALL_PROXY,ANTHROPIC_API_KEY,ANTHROPIC_BASE_URL,CLAUDECODE,CLAUDE_CODE_CHILD_SESSION,CLAUDE_CODE_DISABLE_NONESSENTIAL_TRAFFIC,CLAUDE_CODE_ENTRYPOINT,CLAUDE_CODE_EXECPATH,CLAUDE_CODE_HOST_HTTP_PROXY_PORT,CLAUDE_CODE_HOST_SOCKS_PROXY_PORT,CLAUDE_CODE_SESSION_ID,CLAUDE_CODE_TMPDIR,CLAUDE_CONFIG_DIR,CLAUDE_EFFORT,CLAUDE_PID,CLAUDE_TMPDIR,CLOUDSDK_PROXY_ADDRESS,CLOUDSDK_PROXY_PASSWORD,CLOUDSDK_PROXY_PORT,CLOUDSDK_PROXY_TYPE,CLOUDSDK_PROXY_USERNAME,COREPACK_ENABLE_AUTO_PIN,DISABLE_AUTOUPDATER,DOCKER_HTTPS_PROXY,DOCKER_HTTP_PROXY,FTP_PROXY,GIT_CONFIG_PARAMETERS,GIT_EDITOR,GIT_SSH_COMMAND,GRPC_PROXY,HOME,HTTPS_PROXY,HTTP_PROXY,LANG,MIMALLOC_SCAVENGER,NO_PROXY,NoDefaultCurrentDirectoryInExePath,PATH,PWD,RSYNC_PROXY,SANDBOX_RUNTIME,SHELL,SHLVL,TMPDIR,TMPPREFIX,_,all_proxy,ftp_proxy,grpc_proxy,http_proxy,https_proxy,no_proxy
--- Windows or WSL variables inside the fence: none
--- /proc: command lines readable from the fence: 3; of processes carrying Claude Code's flags: 0; their environment readable: 0
--- files the probe tried to read:
    
--- S8 connection attempts:
    (none)
--- API key text found (counts only; the text is never printed):
    fake API key: 3 {'env.txt': 1, 'proc_2_environ.txt': 1, 'proc_3_environ.txt': 1}
    fake API key, last 20 characters: 3 {'env.txt': 1, 'proc_2_environ.txt': 1, 'proc_3_environ.txt': 1}
--- processes still running for this run: 0
=== spike run S2a finished 2026-10-09T10:39:21.0735863+05:30
```

### S2c

```
=== spike run S2c, started 2026-10-09T10:39:35.7137991+05:30
--- run description: {"about": "S2: kill wsl.exe while a fenced command runs; list what keeps running; then clean up by the run marker", "script": "long_sleep.json", "gate": "node", "gate_mode": "allow", "shape": "product", "launch": "host", "host": {"env_mode": "systemroot", "kill_wsl_after_s": 20}, "cleanup": true, "env_extra": ["ANTHROPIC_API_KEY=@fake-api-key"]}
--- working copy: /home/tester/steplet-work/S2c/repo HEAD 7f4b72d7981454558a3631310426dcff4d4ebd50
--- settings.json:
{
  "sandbox": {
    "enabled": true,
    "failIfUnavailable": true,
    "allowUnsandboxedCommands": false,
    "autoAllowBashIfSandboxed": false,
    "filesystem": {
      "denyRead": [
        "/mnt",
        "/home/tester/.local/share/steplet",
        "/home/tester/steplet-work",
        "/home/tester/m1"
      ],
      "allowRead": [
        "/home/tester/steplet-work/S2c/repo"
      ],
      "denyWrite": [
        "/home/tester/steplet-work/S2c/repo/.git"
      ]
    },
    "network": {
      "allowedDomains": [],
      "strictAllowlist": true
    }
  },
  "permissions": {
    "deny": [
      "Read(//mnt/**)",
      "Read(//home/tester/.local/share/steplet/**)",
      "Read(//home/tester/steplet-work/**)",
      "Read(//home/tester/m1/**)",
      "Edit(//home/tester/steplet-work/S2c/repo/.git/**)"
    ]
  }
}
--- MCP server: {"type": "stdio", "command": "/usr/bin/node", "args": ["/home/tester/m1/tools/gate/gate.mjs"], "env": {"GATE_MODE": "allow", "GATE_LOG": "/home/tester/m1/logs/S2c.gate.log"}}
--- launch command, one argument per line:
/usr/bin/timeout
-k
10
600
/usr/bin/env
-i
HOME=/home/tester/.local/share/steplet/home
PATH=/usr/local/bin:/usr/bin:/bin
LANG=C.UTF-8
CLAUDE_CONFIG_DIR=/home/tester/.local/share/steplet/claude-config
DISABLE_AUTOUPDATER=1
CLAUDE_CODE_DISABLE_NONESSENTIAL_TRAFFIC=1
ANTHROPIC_BASE_URL=http://127.0.0.1:47900
ANTHROPIC_API_KEY=<fake API key>
/home/tester/.local/share/steplet/claude-cli/bin/claude
-p
--output-format
stream-json
--verbose
--include-hook-events
--max-turns
12
--setting-sources
user
--strict-mcp-config
--settings
/home/tester/.local/share/steplet/runs/S2c/settings.json
--mcp-config
/home/tester/.local/share/steplet/runs/S2c/mcp.json
--permission-prompt-tool
mcp__gate__decide
--- task text: Follow the spike script.
--- helpers (scripted model, listener): running, pids 688 689
host: wsl.exe started (pid Some(32528), environment SystemRoot); task sent on its input, 24 bytes
host: killing wsl.exe at 20.2 s
host: wsl.exe ended: Ok(Some(1)), after 20.2 s
--- launch finished after 20.2 s
--- Windows: spike-gate.exe processes still running: 0
--- event stream, read with a JSON parser:
bytes: 2683, lines: 3, raw 0x1B bytes: 0
fake key text occurrences in raw stream: 0
FORGED-FINAL-RESULT occurrences in raw stream: 0
INIT version=2.1.220 model=claude-opus-5[1m] permissionMode=default cwd=/home/tester/steplet-work/S2c/repo
     mcp_servers=[{"name": "gate", "status": "connected"}]
     tools=Task,AskUserQuestion,Bash,CronCreate,CronDelete,CronList,Edit,EnterPlanMode,EnterWorktree,ExitPlanMode,ExitWorktree,NotebookEdit,Read,ReportFindings,ScheduleWakeup,SendMessage,Skill,TaskCreate,TaskGet,TaskList,TaskOutput,TaskStop,TaskUpdate,WebFetch,WebSearch,Workflow,Write
TOOL_USE Bash: {"command": "python3 -c \"import time; open('spike_alive.txt','w').write('x'); time.sleep(600)\"", "description": "Stay busy", "timeout": 600000}
SYSTEM task_started: {"task_id": "bat3e3p7d", "tool_use_id": "toolu_spike_41fe12e0b9014aefa8a7", "description": "Stay busy", "task_type": "local_bash"}
event types: {'system/init': 1, 'assistant': 1, 'system/task_started': 1}
top-level events of type 'result': 0
--- error output:
    (none)
--- gate log S2c.gate.log:
    2026-10-09T05:09:38.812Z allow {"tool_name":"Bash","input":{"command":"python3 -c \"import time; open('spike_alive.txt','w').write('x'); time.sleep(600)\"","timeout":600000,"description":"Stay busy"},"tool_use_id":"toolu_spike_41fe12e0b9014aefa8a7"}
--- every tool call the agent made, and whether it asked the checker first:
    Bash       asked=yes  python3 -c "import time; open('spike_alive.txt','w').write('x'); time.sleep(600)"
--- scripted model: every request it received:
    +0.00s agent loop /v1/messages?beta=true step=0 answer=tool model=claude-opus-5 key_sent=True key_is_test_key=True bearer=False tools=27
    +19.07s agent loop /v1/messages?beta=true step=1 answer=text model=claude-opus-5 key_sent=True key_is_test_key=True bearer=False tools=27
    tools offered to the model on the first agent request: Agent,AskUserQuestion,Bash,CronCreate,CronDelete,CronList,Edit,EnterPlanMode,EnterWorktree,ExitPlanMode,ExitWorktree,NotebookEdit,Read,ReportFindings,ScheduleWakeup,SendMessage,Skill,TaskCreate,TaskGet,TaskList,TaskOutput,TaskStop,TaskUpdate,WebFetch,WebSearch,Workflow,Write
    seconds between agent requests: [19.07]
--- m0_results.jsonl (the fixture's own record):
    (none)
--- listener inside WSL (127.0.0.1:47831):
    (none)
--- marker files: WSL none | Windows none
--- working copy HEAD: 7f4b72d7981454558a3631310426dcff4d4ebd50
--- git status --short --ignored:
    ?? spike_alive.txt
    !! tests/m0_config.json
--- files in .git/hooks: ['applypatch-msg.sample', 'commit-msg.sample', 'fsmonitor-watchman.sample', 'post-update.sample', 'pre-applypatch.sample', 'pre-commit.sample', 'pre-merge-commit.sample', 'pre-push.sample', 'pre-rebase.sample', 'pre-receive.sample', 'prepare-commit-msg.sample', 'push-to-checkout.sample', 'sendemail-validate.sample', 'update.sample']
--- new files in Claude Code's private settings folder: ['.last-cleanup', 'projects/-home-tester-steplet-work-S2c-repo/6a167101-6b2b-470a-b011-c8aca8f76e72.jsonl']
--- new files in Steplet's private home: ['.cache/claude-cli-nodejs/-home-tester-steplet-work-S2c-repo/mcp-logs-gate/2026-10-09T05-09-38-190Z.jsonl']
--- private home .config/git/ignore: '**/.claude/.cc-writes/\n'
--- real ~/.config/git/ignore: before None | after None
--- Claude Code's record for this folder: before null | after null
--- S8 connection attempts:
    (none)
--- API key text found (counts only; the text is never printed):
    fake API key: 0 
    fake API key, last 20 characters: 0 
--- processes still running for this run: 0
--- clean-up: 0 processes carry the run marker; sessions to stop: none
--- processes still running for this run: 0
=== spike run S2c finished 2026-10-09T10:40:06.5127543+05:30
```

A plain `ps -eo pid,etime,args` in the distro straight after S2c showed no `python3` sleep, `bwrap`, `socat` or
`claude` process. The distro was not restarted by the kill: its `/sbin/init` started about 45 s before S2c began,
and the helper process numbers kept rising across runs in the same boot (S2a 512, S2c 688).

**Verdict:** PASS, with `env_mode` `systemroot` instead of `empty`, because:
- no Windows variables cross: S2r-systemroot gives 18 Linux variables, `WSLENV=` empty, and `PATH` without any
  `/mnt/c` folder; inside the fence, S2a reports `Windows or WSL variables inside the fence: none`;
- no process outlives the run: S2c killed `wsl.exe` at 20.2 s, `processes still running for this run: 0` before
  the clean-up and after it, confirmed by a plain process list;
- Claude Code works with the private `HOME` (S2a: `RESULT subtype=success`), the private home's
  `.config/git/ignore` holds `**/.claude/.cc-writes/` and the real `~/.config/git/ignore` is `None` before and
  after. The file was first written there by S7a (also private `HOME`, launched inside WSL); S2a found it already
  there, so S2a shows it stays in the private home rather than when it is first written.

**What it means:**
- **The app must never start `wsl.exe` with an empty Windows environment.** On WSL 2.6.3 that call fails with
  `The RPC call contains a handle that differs from the declared handle type` (`Wsl/Service/0x8007072c`), and when
  that call is the one starting a stopped distro, the next normal call hangs until `wsl --terminate` (sections 4.1
  and 4.2; the same happened in the first host test this morning, which then answered `Catastrophic failure`,
  `Wsl/Service/E_UNEXPECTED`). With the distro already running, the failure leaves it usable (sections 2 and 3).
- **`SystemRoot` alone is enough**, including as the call that starts a stopped distro (sections 1, 5.1, 5.2), and
  passes nothing Windows-specific into Linux. The milestone plan's "Empty" choice becomes "SystemRoot only".
- Passing the whole Windows environment (`inherit`) adds every Windows program folder to `PATH` inside WSL; the app
  should not do it.
- Killing `wsl.exe` ended the fenced command too, so phase 3 does not need the runner-up launcher for this; the
  clean-up by run marker still runs as a safety net.
- Through `/proc` the fence can read 3 command lines, none of them Claude Code's, and no environments of Claude
  Code's processes (S3 depends on this).
- **Noted for S3, not judged here:** in S2a the fake API key, passed as `ANTHROPIC_API_KEY`, was readable inside the
  fence: in the probe's own environment copy and in two `/proc/<pid>/environ` files of fenced processes.
