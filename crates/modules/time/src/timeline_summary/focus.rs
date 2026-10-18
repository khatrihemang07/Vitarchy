//! Focus stretches: continuous Active time in one app.
//!
//! Edge rules:
//! - Time is Active only when the piece is not Away; Away, unknown Away or a gap ends a stretch.
//! - Another app's Active time shorter than [`GRACE`] does not end the stretch when the app
//!   comes back, and that time does not count toward it. Consecutive pieces of other apps
//!   form one interruption, so its length is their total.
//! - An interruption of [`GRACE`] or more ends the stretch and starts one of its own.
//! - An interruption that never returns to the app (the run ends) just ends the stretch.

use super::clip::Piece;

/// Shorter interruptions than this (seconds) do not break a stretch.
pub const GRACE: i64 = 60;

/// Stretches at least this long (seconds) count as focused time.
pub const FOCUSED: i64 = 25 * 60;

/// The longest stretch and how much of the Active time was focused.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Focus {
    /// The longest stretch, in seconds.
    pub longest_secs: i64,
    /// Active seconds inside stretches of [`FOCUSED`] or more, over all Active seconds;
    /// 0 with no Active time.
    pub share: f64,
}

#[allow(clippy::cast_precision_loss)] // seconds are far below 2^52
/// Finds the Focus stretches in `pieces` (in time order).
#[must_use]
pub fn measure(pieces: &[Piece]) -> Focus {
    let touching_runs: Vec<Vec<AppRun<'_>>> = runs(pieces);
    let total: i64 = touching_runs.iter().flatten().map(|r| r.secs).sum();
    let stretches: Vec<i64> = touching_runs
        .iter()
        .flat_map(|run| stretches(run))
        .collect();
    let focused: i64 = stretches.iter().filter(|s| **s >= FOCUSED).sum();
    Focus {
        longest_secs: stretches.iter().copied().max().unwrap_or(0),
        share: if total > 0 {
            focused as f64 / total as f64
        } else {
            0.0
        },
    }
}

/// Active time in one app, uninterrupted by another app.
struct AppRun<'a> {
    app: &'a str,
    secs: i64,
}

/// Splits the Active pieces into runs of touching pieces, each as consecutive `AppRun`s.
fn runs(pieces: &[Piece]) -> Vec<Vec<AppRun<'_>>> {
    let mut runs: Vec<Vec<AppRun<'_>>> = Vec::new();
    let mut last_end = None;
    for piece in pieces.iter().filter(|p| p.is_active()) {
        if last_end != Some(piece.start_at) {
            runs.push(Vec::new());
        }
        last_end = Some(piece.end_at);
        let run = runs
            .last_mut()
            .expect("a run was just pushed when none was open");
        match run.last_mut() {
            Some(last) if last.app == piece.app => last.secs += piece.secs(),
            _ => run.push(AppRun {
                app: &piece.app,
                secs: piece.secs(),
            }),
        }
    }
    runs
}

/// The stretches in one run: each starts at an `AppRun` and absorbs the app's later ones while
/// the interruptions between them stay under [`GRACE`].
fn stretches(run: &[AppRun<'_>]) -> Vec<i64> {
    let mut found = Vec::new();
    let mut i = 0;
    while i < run.len() {
        let app = run[i].app;
        let mut total = run[i].secs;
        i += 1;
        loop {
            let away_until = run[i..].iter().position(|b| b.app == app).map(|n| i + n);
            let Some(back) = away_until else { break };
            let interruption: i64 = run[i..back].iter().map(|r| r.secs).sum();
            if interruption >= GRACE {
                break;
            }
            total += run[back].secs;
            i = back + 1;
        }
        found.push(total);
    }
    found
}
