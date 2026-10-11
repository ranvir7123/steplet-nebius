use std::path::PathBuf;
use std::process::ExitCode;

const USAGE: &str = "usage:
  steplet-sign check <file>
  steplet-sign sign <file> --key <private key file, or its .pub when the key is in ssh-agent> [--ssh-keygen <path>]
";

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    match args.first().map(String::as_str) {
        Some("check") if args.len() == 2 => check(PathBuf::from(&args[1])),
        _ => usage(),
    }
}

fn check(path: PathBuf) -> ExitCode {
    match std::fs::read(&path) {
        Err(e) => {
            eprintln!("cannot read {}: {e}", path.display());
            ExitCode::from(2)
        }
        Ok(raw) => {
            let r = steplet_sign::check::check(&path.display().to_string(), &raw, steplet_sign::today());
            print!("{}", r.text);
            if r.ok { ExitCode::SUCCESS } else { ExitCode::from(1) }
        }
    }
}

fn usage() -> ExitCode {
    eprint!("{USAGE}");
    ExitCode::from(2)
}
