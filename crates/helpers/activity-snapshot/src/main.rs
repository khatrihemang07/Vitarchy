//! Entry point: take one Snapshot per Source, log one structured line each, exit non-zero if any failed.

use std::env;
use std::path::PathBuf;
use std::process::ExitCode;

use activity_snapshot::{jobs::resolve_jobs_on_disk, run::run_all};

fn main() -> ExitCode {
    let Some(home) = env::var_os("HOME").map(PathBuf::from) else {
        eprintln!("level=error msg=\"snapshot failed\" outcome=error kind=no_home");
        return ExitCode::FAILURE;
    };
    let jobs = resolve_jobs_on_disk(&home, |name| env::var_os(name));
    let outcome = run_all(&jobs);
    for line in &outcome.lines {
        eprintln!("{line}");
    }
    if outcome.all_ok {
        ExitCode::SUCCESS
    } else {
        ExitCode::FAILURE
    }
}
