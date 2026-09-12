//! Which run counts as better, and which mode it counts for.

use super::*;

fn run(toys: usize, mistakes: u32, seconds: f32) -> ShiftRecord {
    ShiftRecord {
        toys_shelved: toys,
        toy_count: 240,
        repairs: 0,
        mistakes,
        zones_restored: 0,
        elapsed_seconds: seconds,
        restored: false,
    }
}

/// Toys first, then mistakes, then time — in that order. Speed settles a tie
/// between two equally clean runs and never buys its way past sloppiness,
/// because the job is putting the shop away, not finishing early.
#[test]
fn a_better_run_shelves_more_then_errs_less_then_finishes_sooner() {
    let baseline = run(200, 3, 1500.0);

    assert!(
        run(201, 9, 1799.0).beats(baseline),
        "more toys should win even when slower and sloppier"
    );
    assert!(
        !run(199, 0, 600.0).beats(baseline),
        "a fast flawless run that shelved less is still less shelved"
    );
    assert!(
        run(200, 2, 1700.0).beats(baseline),
        "same toys, fewer wrong shelves, should win despite being slower"
    );
    assert!(
        !run(200, 4, 100.0).beats(baseline),
        "speed cannot buy off a mistake"
    );
    assert!(
        run(200, 3, 1499.0).beats(baseline),
        "identical otherwise, faster should win"
    );
    assert!(
        !run(200, 3, 1500.0).beats(baseline),
        "an identical run is not an improvement"
    );
}

#[test]
fn the_first_run_of_a_mode_always_sets_the_record() {
    let mut best = BestRuns::default();
    assert_eq!(best.best_for(ShiftMode::Timed), None);

    let first = run(10, 40, 1799.0);
    assert!(best.submit(ShiftMode::Timed, first));
    assert_eq!(best.best_for(ShiftMode::Timed), Some(first));
}

/// A relaxed run has no deadline, so its time is not comparable with a timed
/// one. Pooling them would let an unhurried perfect clear stand as *the*
/// record and leave a Closing Shift player nothing reachable to chase.
#[test]
fn the_two_modes_keep_separate_records() {
    let mut best = BestRuns::default();
    let leisurely = run(240, 0, 5400.0);
    let timed = run(180, 2, 1700.0);

    assert!(best.submit(ShiftMode::Relaxed, leisurely));
    assert!(best.submit(ShiftMode::Timed, timed));

    assert_eq!(best.best_for(ShiftMode::Relaxed), Some(leisurely));
    assert_eq!(best.best_for(ShiftMode::Timed), Some(timed));

    // The far better relaxed run must not displace the timed one.
    assert!(!best.submit(ShiftMode::Timed, run(170, 0, 1000.0)));
    assert_eq!(best.best_for(ShiftMode::Timed), Some(timed));
}

/// Records survive their own round trip, and a file written before a mode
/// existed loads as "no record" rather than refusing.
#[test]
fn records_survive_a_save_and_a_missing_mode() {
    let mut best = BestRuns::default();
    best.submit(ShiftMode::Timed, run(200, 1, 1400.0));

    let json = serde_json::to_string(&best).unwrap();
    let restored: BestRuns = serde_json::from_str(&json).unwrap();
    assert_eq!(restored, best);

    let partial: BestRuns = serde_json::from_str("{}").unwrap();
    assert_eq!(partial, BestRuns::default());
    assert_eq!(partial.best_for(ShiftMode::Timed), None);
}

/// The paths that only bite a real player: a records file that is not there
/// yet, and one that will not parse.
///
/// Slot saves land in the OS app-data directory with no override, so these use
/// a game name of their own and delete it afterwards rather than touching
/// anything the real game owns. Worth the awkwardness: everything else about
/// records is exercised in memory, and "does it survive a round trip through a
/// file" was the one link reasoned about rather than observed.
mod on_disk {
    use super::*;
    use macroquad_toolkit::persistence::get_app_data_path;
    use std::fs;

    /// A game name nobody else uses, so the directory can be removed whole.
    fn scratch_game(tag: &str) -> String {
        format!("toybox_records_test_{tag}")
    }

    fn remove_scratch(game_name: &str) {
        if let Some(path) = get_app_data_path(game_name, "x") {
            if let Some(dir) = path.parent() {
                let _ = fs::remove_dir_all(dir);
            }
        }
    }

    #[test]
    fn a_record_survives_a_round_trip_through_a_file() {
        let game = scratch_game("roundtrip");
        remove_scratch(&game);

        let mut written = BestRuns::default();
        written.submit(ShiftMode::Timed, run(212, 3, 1655.0));
        written.submit(ShiftMode::Relaxed, run(240, 0, 3120.0));
        written
            .save(&game, "records", "1.0.0")
            .expect("save records");

        let read = BestRuns::load(&game, "records", "1.0.0");
        assert_eq!(read, written);
        assert_eq!(read.best_for(ShiftMode::Timed).unwrap().toys_shelved, 212);

        remove_scratch(&game);
    }

    #[test]
    fn a_missing_records_file_is_the_first_run_not_an_error() {
        let game = scratch_game("missing");
        remove_scratch(&game);

        assert_eq!(
            BestRuns::load(&game, "records", "1.0.0"),
            BestRuns::default()
        );
    }

    /// A corrupt file costs the player their records and nothing else. Refusing
    /// to start a shift because a leaderboard will not parse would be a far
    /// worse trade than losing the leaderboard.
    #[test]
    fn a_corrupt_records_file_degrades_to_no_records() {
        let game = scratch_game("corrupt");
        remove_scratch(&game);

        let mut seed = BestRuns::default();
        seed.submit(ShiftMode::Timed, run(100, 0, 900.0));
        seed.save(&game, "records", "1.0.0").expect("seed records");

        let path = get_app_data_path(&game, "save_records.json").expect("records path");
        fs::write(&path, "{ this is not json").expect("corrupt the file");

        assert_eq!(
            BestRuns::load(&game, "records", "1.0.0"),
            BestRuns::default()
        );

        remove_scratch(&game);
    }
}

/// The record is built from the same summary the score screen shows, so the
/// two can never disagree about what the run was worth.
#[test]
fn a_record_carries_what_the_score_screen_reported() {
    let data = GameData::load().unwrap();
    let session = GameSession::new(&data);
    let summary = session.shift_summary(&data);

    let record = ShiftRecord::from_summary(&summary, false);
    assert_eq!(record.toys_shelved, summary.toys_shelved);
    assert_eq!(record.toy_count, summary.toy_count);
    assert_eq!(record.mistakes, summary.mistakes);
    assert_eq!(record.zones_restored, summary.zones_restored);
    assert!(!record.restored);
}
