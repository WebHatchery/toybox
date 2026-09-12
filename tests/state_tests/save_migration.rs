//! Audit of the load path: every shape of save the game has ever written, plus
//! the shapes it never wrote but could be handed, must either load or degrade
//! to a fresh session. Nothing here may panic.

use super::*;
use serde_json::{json, Value};

/// Strip every field added after the pre-expansion release, leaving the save
/// shaped the way an old build wrote it.
fn strip_post_expansion_fields(mut save: Value) -> Value {
    for toy in save["toys"].as_array_mut().unwrap() {
        let toy = toy.as_object_mut().unwrap();
        for field in [
            "spawn_pose",
            "placed_slot_index",
            "bench_slot_index",
            "bench_id",
            "wrong_marker_seconds",
            "repair_state",
        ] {
            toy.remove(field);
        }
    }
    let player = save["player"].as_object_mut().unwrap();
    player.remove("yaw");
    player.remove("pitch");
    // Added with the score screen and the shift deadline. A save written before
    // those has neither key, and in a WebGL build that save is sitting in a
    // real player's localStorage — so the defaults have to hold without a
    // migration step, not merely compile.
    player.remove("repairs");
    save.as_object_mut().unwrap().remove("shift_mode");
    save.as_object_mut().unwrap().remove("shift_seed");
    save
}

#[test]
fn stale_save_with_wrong_toy_count_restocks_fresh() {
    let data = GameData::load().unwrap();
    let mut session = GameSession::new(&data);
    session.toys.truncate(100);
    let stale = serde_json::to_value(session.to_save("2.1.0")).unwrap();

    let migrated = migrate_save_value(Some("2.1.0".to_owned()), stale, &data).unwrap();

    let live_toys = migrated
        .toys
        .iter()
        .filter(|toy| {
            !toy.is_consumed_repair_part() && toy.repair_part_kind() != Some(RepairPartKind::Head)
        })
        .count();
    assert_eq!(live_toys, data.config.toy_count);
    assert_eq!(migrated.version, data.config.version);
}

#[test]
fn pre_expansion_save_loads_without_the_fields_it_never_had() {
    let data = GameData::load().unwrap();
    let mut session = GameSession::new(&data);
    // A pre-expansion store held no broken toys: drop the scattered heads and
    // let the stripped `repair_state` default the bodies back to whole. The
    // live count still has to match, or the restock path fires instead.
    session
        .toys
        .retain(|toy| toy.repair_part_kind() != Some(RepairPartKind::Head));
    assert_eq!(session.toys.len(), data.config.toy_count);
    let legacy =
        strip_post_expansion_fields(serde_json::to_value(session.to_save("2.1.0")).unwrap());

    let migrated = migrate_save_value(Some("2.1.0".to_owned()), legacy, &data).unwrap();
    let session = GameSession::from_save(migrated, &data);

    // Defaults filled in, and load-time repair gave every toy a real pose.
    assert_eq!(session.toys.len(), data.config.toy_count);
    assert!(session
        .toys
        .iter()
        .all(|toy| !toy.spawn_pose.is_uninitialized() || toy.placed_display_id.is_some()));
    assert!(session.player.yaw.is_finite());
    assert!(session.player.pitch.abs() <= GameSession::MAX_LOOK_PITCH);

    // A save with no recorded mode was played against the clock, because that
    // was the only mode there was. Loading it as Relaxed would silently hand
    // someone an untimed run they never chose.
    assert_eq!(session.shift_mode, ShiftMode::Timed);
    assert_eq!(session.shift_seed, CLOSING_SHIFT_SEED);
    assert_eq!(session.player.repairs, 0);
    assert!(
        session.shift_remaining(&data) > 0.0,
        "an old save loaded with the shift already over"
    );
}

#[test]
fn save_wrapped_in_a_data_envelope_is_unwrapped() {
    let data = GameData::load().unwrap();
    let session = GameSession::new(&data);
    let wrapped = json!({
        "version": "2.1.0",
        "data": serde_json::to_value(session.to_save("2.1.0")).unwrap(),
    });

    let migrated = migrate_save_value(Some("2.1.0".to_owned()), wrapped, &data).unwrap();

    assert_eq!(migrated.version, data.config.version);
    assert_eq!(migrated.toys.len(), session.toys.len());
}

#[test]
fn unreadable_saves_degrade_to_a_fresh_session() {
    let data = GameData::load().unwrap();

    for junk in [
        json!(null),
        json!("not a save"),
        json!(42),
        json!([]),
        json!({}),
        json!({ "toys": "not an array" }),
        // Right shape, required field missing.
        json!({ "version": "2.1.0", "player": {}, "toys": [] }),
    ] {
        let migrated = migrate_save_value(None, junk.clone(), &data)
            .unwrap_or_else(|error| panic!("{junk} should not error: {error}"));
        let live_toys = migrated
            .toys
            .iter()
            .filter(|toy| {
                !toy.is_consumed_repair_part()
                    && toy.repair_part_kind() != Some(RepairPartKind::Head)
            })
            .count();
        assert_eq!(live_toys, data.config.toy_count, "for {junk}");
    }
}

#[test]
fn a_save_carrying_a_toy_it_never_flagged_as_held_is_reconciled() {
    let data = GameData::load().unwrap();
    let mut session = GameSession::new(&data);
    let carried_id = session.toys[0].id.clone();

    // Both halves of the desync an older writer (or a hand-edited save) can
    // leave behind: carried but not held, and held but not carried.
    session.player.carried_toy_ids = vec![carried_id.clone()];
    session.player.active_carry_index = 0;
    session.toys[0].is_held = false;
    session.toys[1].is_held = true;

    let reloaded = GameSession::from_save(session.to_save("2.1.0"), &data);

    let carried = reloaded
        .toys
        .iter()
        .find(|toy| toy.id == carried_id)
        .unwrap();
    assert!(
        carried.is_held,
        "a toy in carried_toy_ids must be flagged held or it renders on the floor"
    );
    assert_eq!(
        reloaded.active_toy().map(|toy| toy.id.as_str()),
        Some(carried_id.as_str())
    );

    let stray = &reloaded.toys[1];
    assert!(
        !stray.is_held,
        "a held toy nobody is carrying is unreachable: pickup targeting skips held toys"
    );
}

#[test]
fn the_retired_tag_lantern_id_still_grants_the_scanner() {
    let data = GameData::load().unwrap();
    let mut session = GameSession::new(&data);
    session.unlocked_upgrade_ids = vec!["tag_lantern".to_owned()];

    let reloaded = GameSession::from_save(session.to_save("2.1.0"), &data);

    assert!(reloaded.scanner_enabled(&data));
    // The retired id must still spend its credit, or the tool comes out free.
    assert_eq!(reloaded.available_tool_credits(&data), 0);
}

/// The player's own save, written to a real file and read back.
///
/// Every other test here hands `migrate_save_value` a `Value` it built in
/// memory. That is the arrangement in which a save bug cannot happen — and it
/// is not the code that runs: a save written by this build carries the current
/// version, so `load_from_slot_with_migration` takes its *fast path* and
/// deserialises the wrapper directly, never calling `migrate_save_value` at
/// all. The migration tests cannot reach it.
///
/// This matters because that exact pair of calls silently dropped the best-run
/// records for two iterations, caught only once something went through a file.
/// Records were a leaderboard; this is the player's shift.
mod on_disk {
    use super::*;
    use macroquad_toolkit::persistence::{
        get_app_data_path, load_from_slot_with_migration, save_to_slot_with_version, slot_exists,
    };
    use std::fs;

    fn scratch_game(tag: &str) -> String {
        format!("toybox_session_test_{tag}")
    }

    fn remove_scratch(game_name: &str) {
        if let Some(path) = get_app_data_path(game_name, "x") {
            if let Some(dir) = path.parent() {
                let _ = fs::remove_dir_all(dir);
            }
        }
    }

    /// Play a little: shelve one toy, carry another, bank a mistake and a
    /// repair, and buy a tool. A save of an untouched session would round-trip
    /// even if the payload were being dropped entirely.
    fn played_session(data: &GameData) -> GameSession {
        let mut session = GameSession::new_with_seed(data, 0x5EED_2026_AA55_0240);
        let display = &data.displays[0];

        let right = session
            .toys
            .iter()
            .position(|toy| toy_matches_display(toy, display) && !toy.is_repair_part())
            .unwrap();
        session.pick_up_toy(right, data);
        session.place_active_toy(0, 0, data);

        let wrong = session
            .toys
            .iter()
            .position(|toy| !toy_matches_display(toy, display) && !toy.is_repair_part())
            .unwrap();
        session.pick_up_toy(wrong, data);
        session.place_active_toy(0, 1, data);

        let carried = session
            .toys
            .iter()
            .position(|toy| toy.placed_display_id.is_none() && !toy.is_held)
            .unwrap();
        session.pick_up_toy(carried, data);

        session.player.repairs = 3;
        session.player.elapsed_seconds = 421.5;
        session.shift_mode = ShiftMode::Relaxed;
        session.unlocked_upgrade_ids.push("toy_scanner".to_owned());
        session
    }

    #[test]
    fn a_session_survives_a_round_trip_through_a_file() {
        let data = GameData::load().unwrap();
        let game = scratch_game("roundtrip");
        let slot = "session";
        remove_scratch(&game);

        let before = played_session(&data);
        let version = &data.config.version;
        save_to_slot_with_version(&game, slot, &before.to_save(version), version)
            .expect("save session");
        assert!(slot_exists(&game, slot));

        let loaded: SaveData =
            load_from_slot_with_migration(&game, slot, version, |detected, value| {
                // Reaching here means the fast path did *not* run, which is
                // itself worth failing on: it would mean a save this build just
                // wrote does not carry this build's version.
                panic!("unexpected migration from {detected:?}: {value}")
            })
            .expect("load session");
        let after = GameSession::from_save(loaded, &data);

        assert_eq!(after.shift_mode, ShiftMode::Relaxed);
        assert_eq!(after.shift_seed, before.shift_seed);
        assert_eq!(after.player.mistakes, before.player.mistakes);
        assert_eq!(after.player.repairs, 3);
        assert_eq!(after.player.elapsed_seconds, 421.5);
        assert_eq!(after.unlocked_upgrade_ids, before.unlocked_upgrade_ids);
        assert_eq!(after.toys.len(), before.toys.len());
        assert_eq!(after.total_placed_toys(), before.total_placed_toys());
        assert!(
            after.total_placed_toys() > 0,
            "nothing was shelved to check"
        );
        assert_eq!(
            after.player.carried_toy_ids, before.player.carried_toy_ids,
            "the toy in hand did not come back"
        );
        assert_eq!(
            after.active_toy().map(|toy| toy.id.clone()),
            before.active_toy().map(|toy| toy.id.clone())
        );

        remove_scratch(&game);
    }

    /// A save from an older build reaches `migrate_save_value` through the real
    /// load, not just through a direct call.
    #[test]
    fn an_older_saves_version_routes_it_through_migration() {
        let data = GameData::load().unwrap();
        let game = scratch_game("migrate");
        let slot = "session";
        remove_scratch(&game);

        let session = GameSession::new(&data);
        save_to_slot_with_version(&game, slot, &session.to_save("2.1.0"), "2.1.0")
            .expect("save old session");

        let loaded: SaveData =
            load_from_slot_with_migration(&game, slot, &data.config.version, |detected, value| {
                assert_eq!(detected.as_deref(), Some("2.1.0"));
                migrate_save_value(detected, value, &data)
            })
            .expect("load old session");

        assert_eq!(loaded.version, data.config.version);
        assert_eq!(loaded.toys.len(), session.toys.len());

        remove_scratch(&game);
    }

    /// Quitting to the title mid-shift keeps the run, and quitting once the
    /// shift is over leaves the resumable one alone.
    ///
    /// "Quit to Title" was the only way out of a run from the UI and it wrote
    /// nothing — `Ctrl+S` was the sole path to the save slot — so leaving threw
    /// the shift away and then offered "Continue" backed by whatever older save
    /// was on disk. The second half matters just as much: a finished run must
    /// not overwrite a live one, or Continue resumes onto a score screen for a
    /// shift that already ended.
    #[test]
    fn quitting_mid_shift_keeps_the_run_and_quitting_after_it_does_not() {
        let data = GameData::load().unwrap();
        let game = scratch_game("quit_to_title");
        let slot = "session";
        remove_scratch(&game);

        let version = &data.config.version;
        let live = played_session(&data);
        assert!(live.phase.should_save_on_leaving());
        save_to_slot_with_version(&game, slot, &live.to_save(version), version).expect("save live");

        // A run the clock ended: still worth plenty of progress, so a version
        // that wrote it anyway would look fine until Continue was pressed.
        let mut over = played_session(&data);
        over.phase = GamePhase::TimeUp;
        assert!(!over.phase.should_save_on_leaving());
        if over.phase.should_save_on_leaving() {
            save_to_slot_with_version(&game, slot, &over.to_save(version), version)
                .expect("save finished");
        }

        let loaded: SaveData = load_from_slot_with_migration(&game, slot, version, |_, _| {
            Err("a save this build wrote must not need migrating".to_owned())
        })
        .expect("load session");
        let resumed = GameSession::from_save(loaded, &data);

        assert_eq!(
            resumed.phase,
            GamePhase::Playing,
            "Continue would resume onto a score screen"
        );
        assert_eq!(resumed.total_placed_toys(), live.total_placed_toys());
        assert!(resumed.total_placed_toys() > 0, "nothing was shelved");
        assert_eq!(
            resumed.player.carried_toy_ids, live.player.carried_toy_ids,
            "the toy in hand did not survive the quit"
        );

        remove_scratch(&game);
    }
}
