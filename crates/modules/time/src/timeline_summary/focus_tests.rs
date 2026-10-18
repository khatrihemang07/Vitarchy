//! Tests for `focus`.
#![allow(clippy::float_cmp)] // the expected shares are exact binary fractions

use super::focus::*;
use super::test_support::{active_s, piece_s};
use crate::timeline::model::Away;

const MIN: i64 = 60;

#[test]
fn nothing_active_means_no_stretch_and_no_share() {
    let nothing = measure(&[piece_s("Code", 0, 1800, Away::Yes)]);
    assert_eq!((nothing.longest_secs, nothing.share), (0, 0.0));
}

#[test]
fn touching_pieces_of_one_app_make_one_stretch() {
    let focus = measure(&[
        active_s("Code", 0, 20 * MIN),
        active_s("Code", 20 * MIN, 40 * MIN),
    ]);
    assert_eq!(focus.longest_secs, 40 * MIN);
    assert_eq!(focus.share, 1.0);
}

#[test]
fn an_interruption_of_59_seconds_keeps_the_stretch_and_is_not_counted() {
    let focus = measure(&[
        active_s("Code", 0, 15 * MIN),
        active_s("Mail", 15 * MIN, 15 * MIN + 59),
        active_s("Code", 15 * MIN + 59, 30 * MIN + 59),
    ]);
    assert_eq!(focus.longest_secs, 30 * MIN);
}

#[test]
fn an_interruption_of_60_seconds_breaks_the_stretch() {
    let focus = measure(&[
        active_s("Code", 0, 15 * MIN),
        active_s("Mail", 15 * MIN, 16 * MIN),
        active_s("Code", 16 * MIN, 31 * MIN),
    ]);
    assert_eq!(focus.longest_secs, 15 * MIN);
    assert_eq!(focus.share, 0.0);
}

#[test]
fn a_long_interruption_is_its_own_stretch() {
    let focus = measure(&[
        active_s("Code", 0, 10 * MIN),
        active_s("Mail", 10 * MIN, 40 * MIN),
    ]);
    assert_eq!(focus.longest_secs, 30 * MIN);
    assert_eq!(focus.share, 0.75);
}

#[test]
fn a_gap_or_away_ends_the_stretch_even_for_the_same_app() {
    let gap = measure(&[
        active_s("Code", 0, 15 * MIN),
        active_s("Code", 15 * MIN + 10, 30 * MIN),
    ]);
    assert_eq!(gap.longest_secs, 15 * MIN);
    let away = measure(&[
        active_s("Code", 0, 15 * MIN),
        piece_s("Code", 15 * MIN, 15 * MIN + 10, Away::Yes),
        active_s("Code", 15 * MIN + 10, 30 * MIN),
    ]);
    assert_eq!(away.longest_secs, 15 * MIN);
}

#[test]
fn a_short_interruption_at_the_end_of_a_run_does_not_extend_it() {
    let focus = measure(&[
        active_s("Code", 0, 10 * MIN),
        active_s("Mail", 10 * MIN, 10 * MIN + 30),
    ]);
    assert_eq!(focus.longest_secs, 10 * MIN);
}

#[test]
fn the_share_counts_only_focused_stretches_of_all_active_time() {
    // 30 min focused in Code, then after a gap 10 min in Mail.
    let focus = measure(&[
        active_s("Code", 0, 30 * MIN),
        active_s("Mail", 40 * MIN, 50 * MIN),
    ]);
    assert_eq!(focus.share, 0.75);
}
