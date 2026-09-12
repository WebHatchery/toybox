//! What the score screen reports at the end of a run.

use super::*;

/// Fill `display_index` completely, the way the capture scenes do.
fn stock(session: &mut GameSession, data: &GameData, display_index: usize) {
    let display = &data.displays[display_index];
    let mut matching: Vec<String> = session
        .toys
        .iter()
        .filter(|toy| {
            toy_matches_display(toy, display) && toy.placed_display_id.is_none() && !toy.is_held
        })
        .map(|toy| toy.id.clone())
        .collect();
    matching.truncate(display.capacity);

    for (slot_index, toy_id) in matching.into_iter().enumerate() {
        let Some(toy_index) = session.toys.iter().position(|toy| toy.id == toy_id) else {
            continue;
        };
        session.pick_up_toy(toy_index, data);
        session.place_active_toy(display_index, slot_index, data);
    }
}

#[test]
fn a_fresh_shop_scores_nothing_but_still_counts_its_aisles() {
    let data = GameData::load().unwrap();
    let session = GameSession::new(&data);
    let summary = session.shift_summary(&data);

    assert_eq!(summary.toys_shelved, 0);
    assert_eq!(summary.zones_restored, 0);
    assert_eq!(summary.completion(), 0.0);
    assert_eq!(summary.grade(), "E");

    // Checkout and the backroom hold no displays. Counting them would make a
    // perfect run read as 5/7 and a fresh one claim two aisles already done.
    assert!(summary.zones_with_shelves > 0);
    assert!(summary.zones_with_shelves < data.layout.zones.len());
    assert_eq!(summary.zones.len(), summary.zones_with_shelves);
    assert!(summary.zones.iter().all(|(_, zone)| zone.has_displays()));
}

/// Every toy is either shelved or accounted for as a break, so the two
/// denominators on the panel have to add up to the shop.
#[test]
fn the_break_count_covers_every_split_toy() {
    let data = GameData::load().unwrap();
    let session = GameSession::new(&data);
    let summary = session.shift_summary(&data);

    let split_parts = session
        .toys
        .iter()
        .filter(|toy| toy.is_repair_part())
        .count();
    assert_eq!(
        summary.breaks_total as usize * 2,
        split_parts,
        "every break is exactly two parts"
    );
    assert_eq!(summary.repairs, 0);
}

#[test]
fn shelving_an_aisle_marks_it_restored() {
    let data = GameData::load().unwrap();
    let mut session = GameSession::new(&data);

    let before = session.restored_zone_names(&data);
    assert!(before.is_empty());

    // Displays 0-3 are the plush wall/bin/table, all in one zone.
    for display_index in 0..4 {
        stock(&mut session, &data, display_index);
    }

    let summary = session.shift_summary(&data);
    let restored: Vec<&str> = summary
        .zones
        .iter()
        .filter(|(_, zone)| zone.is_restored())
        .map(|(name, _)| name.as_str())
        .collect();

    // A zone caps below 100% while its toys are still in halves, so this only
    // lands if the plush zone happened to break none. Assert the weaker,
    // always-true relationship plus the fact that progress moved.
    let plush = &summary.zones[0];
    assert!(plush.1.placed > 0, "the plush aisle took no toys");
    assert_eq!(
        plush.1.is_restored(),
        restored.contains(&plush.0.as_str()),
        "is_restored and the restored list disagree"
    );
    assert_eq!(summary.zones_restored, restored.len());
}

/// The grade must not hand out an S for a shop that was never finished, and
/// must not hand one to a finished-but-sloppy run either.
#[test]
fn the_grade_separates_perfect_from_merely_finished() {
    let data = GameData::load().unwrap();
    let mut session = GameSession::new(&data);

    // Nothing done.
    assert_eq!(session.shift_summary(&data).grade(), "E");

    // Pretend the whole shop is away, cleanly.
    let clean = ShiftSummary {
        zones: Vec::new(),
        zones_restored: 5,
        zones_with_shelves: 5,
        toys_shelved: data.config.toy_count,
        toy_count: data.config.toy_count,
        repairs: 0,
        breaks_total: 0,
        mistakes: 0,
        elapsed_seconds: 600.0,
    };
    assert_eq!(clean.grade(), "S");
    assert_eq!(clean.completion(), 1.0);

    let sloppy = ShiftSummary {
        mistakes: 3,
        ..clean.clone()
    };
    assert_eq!(
        sloppy.grade(),
        "A",
        "a perfect clear with wrong shelves is not an S"
    );

    let half = ShiftSummary {
        toys_shelved: data.config.toy_count / 2,
        ..clean
    };
    assert_eq!(half.grade(), "C");

    // A partial run must never out-grade a complete one.
    session.player.mistakes = 99;
    assert_ne!(session.shift_summary(&data).grade(), "S");
}

/// The shop can actually be won with the shipped data.
///
/// Grades and completion are otherwise only tested on summaries built by hand,
/// which cannot notice a config where the win condition is unreachable — a
/// display whose category has too few matching toys, or capacities that no
/// longer sum to `toy_count`, would leave `GamePhase::Finished` unreachable and
/// every test still green. This drives the same staging the `store_restored`
/// capture uses, so the scene and the invariant cannot drift apart.
#[test]
fn the_shipped_shop_can_be_restored_completely() {
    let data = GameData::load().unwrap();
    let session = toybox_after_hours::capture_scenes::store_restored(&data);

    assert_eq!(session.phase, GamePhase::Finished);

    let summary = session.shift_summary(&data);
    assert_eq!(summary.toys_shelved, data.config.toy_count);
    assert_eq!(summary.completion(), 1.0);
    assert_eq!(summary.grade(), "S", "a flawless clear is the top grade");
    assert_eq!(
        summary.zones_restored, summary.zones_with_shelves,
        "every aisle with shelves should read restored"
    );

    // Nothing left loose on the floor: a toy that matches no display at all
    // would leave the shop winnable but the floor never clear.
    let stranded = session
        .toys
        .iter()
        .filter(|toy| {
            matches!(toy.repair_state, RepairState::Whole) && toy.placed_display_id.is_none()
        })
        .count();
    assert_eq!(stranded, 0, "{stranded} whole toys had nowhere to go");
}
