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
        Some("sign") => match sign_args(&args[1..]) {
            Some((file, key, keygen)) => {
                let req = steplet_sign::sign::SignRequest { file: &file, key: &key, ssh_keygen: &keygen, today: steplet_sign::today() };
                match steplet_sign::sign::sign(&req) {
                    Ok(msg) => {
                        println!("{msg}");
                        ExitCode::SUCCESS
                    }
                    Err(msg) => {
                        println!("{msg}");
                        ExitCode::from(1)
                    }
                }
            }
            None => usage(),
        },
        _ => usage(),
    }
}

fn sign_args(rest: &[String]) -> Option<(PathBuf, PathBuf, PathBuf)> {
    let (mut file, mut key, mut keygen) = (None, None, None);
    let mut it = rest.iter();
    while let Some(a) = it.next() {
        match a.as_str() {
            "--key" => key = Some(PathBuf::from(it.next()?)),
            "--ssh-keygen" => keygen = Some(PathBuf::from(it.next()?)),
            _ if file.is_none() && !a.starts_with("--") => file = Some(PathBuf::from(a)),
            _ => return None,
        }
    }
    Some((file?, key?, keygen.unwrap_or_else(|| PathBuf::from(steplet_sign::sign::DEFAULT_SSH_KEYGEN))))
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
