//! Runs every job. One Source failing never stops the others.

use std::time::Instant;

use crate::{
    jobs::Job,
    report,
    snapshot::{self, Snapshot, SnapshotError},
};

/// What running all jobs produced.
#[derive(Debug, PartialEq, Eq)]
pub struct Outcome {
    /// One log line per job, in job order.
    pub lines: Vec<String>,
    /// Whether every job succeeded.
    pub all_ok: bool,
}

/// Runs every job with the real [`snapshot::take`].
#[must_use]
pub fn run_all(jobs: &[Job]) -> Outcome {
    run_all_with(jobs, |job| snapshot::take(&job.paths))
}

/// Runs every job with `take`, logging each as it finishes.
pub fn run_all_with(
    jobs: &[Job],
    take: impl Fn(&Job) -> Result<Snapshot, SnapshotError>,
) -> Outcome {
    let mut lines = Vec::new();
    let mut all_ok = true;
    for job in jobs {
        let started = Instant::now();
        match take(job) {
            Ok(taken) => lines.push(report::success_line(job.source, &taken, started.elapsed())),
            Err(error) => {
                all_ok = false;
                lines.push(report::failure_line(job.source, &error, started.elapsed()));
            }
        }
    }
    Outcome { lines, all_ok }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use rusqlite::Connection;
    use tempfile::TempDir;

    use super::*;
    use crate::paths::Paths;

    fn job(source: &'static str, dir: &TempDir, make_source: bool) -> Job {
        let source_db = dir.path().join(format!("{source}.sqlite"));
        if make_source {
            let db = Connection::open(&source_db).unwrap();
            db.execute_batch("CREATE TABLE t (x); INSERT INTO t VALUES (1);")
                .unwrap();
        }
        Job {
            source,
            paths: Paths {
                source: source_db,
                dest_dir: dir.path().join(format!("{source}-out")),
            },
        }
    }

    #[test]
    fn one_source_failing_still_takes_the_other_and_reports_failure() {
        let dir = TempDir::new().unwrap();
        let jobs = [job("toggl", &dir, false), job("clockify", &dir, true)];

        let outcome = run_all(&jobs);

        assert!(!outcome.all_ok);
        assert!(
            outcome.lines[0].contains("source=toggl") && outcome.lines[0].contains("outcome=error")
        );
        assert!(
            outcome.lines[1].contains("source=clockify") && outcome.lines[1].contains("outcome=ok")
        );
        assert!(jobs[1].paths.snapshot().is_file());
        assert!(!jobs[0].paths.snapshot().exists());
    }

    #[test]
    fn every_source_succeeding_is_all_ok() {
        let dir = TempDir::new().unwrap();
        let outcome = run_all(&[job("toggl", &dir, true), job("clockify", &dir, true)]);
        assert!(outcome.all_ok);
        assert_eq!(outcome.lines.len(), 2);
    }

    #[test]
    fn a_failure_before_the_last_job_does_not_skip_it() {
        let dir = TempDir::new().unwrap();
        let jobs = [
            job("a", &dir, true),
            job("b", &dir, true),
            job("c", &dir, true),
        ];
        let outcome = run_all_with(&jobs, |job| {
            if job.source == "a" {
                Err(SnapshotError::SourceMissing(PathBuf::from("/x")))
            } else {
                snapshot::take(&job.paths)
            }
        });
        assert!(!outcome.all_ok);
        assert!(outcome.lines[2].contains("source=c") && outcome.lines[2].contains("outcome=ok"));
    }
}
