//! Entry point: take one Snapshot, log one structured line, exit non-zero on failure.

use std::env;
use std::path::PathBuf;
use std::process::ExitCode;
use std::time::Instant;

use activity_snapshot::paths::Paths;
use activity_snapshot::{report, snapshot};

fn main() -> ExitCode {
    let Some(home) = env::var_os("HOME").map(PathBuf::from) else {
        eprintln!("level=error msg=\"snapshot failed\" outcome=error kind=no_home");
        return ExitCode::FAILURE;
    };
    let paths = Paths::resolve(&home, |name| env::var_os(name));

    let started = Instant::now();
    match snapshot::take(&paths) {
        Ok(taken) => {
            eprintln!("{}", report::success_line(&taken, started.elapsed()));
            ExitCode::SUCCESS
        }
        Err(error) => {
            eprintln!("{}", report::failure_line(&error, started.elapsed()));
            ExitCode::FAILURE
        }
    }
}
