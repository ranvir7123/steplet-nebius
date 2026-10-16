//! Prints the settings file and MCP config Steplet writes for one run of a `.steplet` file, with a sample pipe name,
//! token and log. Usage: `cargo run -p steplet-run --offline --example settings -- <file.steplet> <run id>`.

use std::time::Duration;

use steplet_core::canon::canonicalize;
use steplet_core::header::parse;
use steplet_run::{GateLink, RunPaths, mcp_json, settings_json};

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let [_, file, run_id] = args.as_slice() else { panic!("usage: settings <file.steplet> <run id>") };
    let bytes = std::fs::read(file).expect("the file can be read");
    let steplet = parse(&canonicalize(&bytes).expect("the file is clean text")).expect("the header reads");
    let paths = RunPaths::new("{00000000-0000-0000-0000-000000000000}", run_id).expect("a plain run id");
    let log = format!(r"C:\Users\sample\AppData\Local\Steplet\logs\{run_id}.gate.log");
    let (pipe, deadline) = (format!("steplet-{run_id}"), Duration::from_secs(30));
    let gate = GateLink::new("/opt/steplet/gate/steplet-gate.exe", &pipe, "sampletoken", &log, deadline)
        .expect("plain gate values");
    println!("--- {}", paths.settings());
    print!("{}", settings_json(&steplet, &paths, &gate));
    println!("--- {}", paths.mcp());
    print!("{}", mcp_json(&gate));
}
