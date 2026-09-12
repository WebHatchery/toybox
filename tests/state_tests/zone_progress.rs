use super::*;

/// Total zone capacity must account for every toy in the shop, or the HUD
/// percentages are measured against a denominator that quietly loses displays.
#[test]
fn every_display_lands_in_exactly_one_zone() {
    let data = GameData::load().unwrap();
    let session = GameSession::new(&data);
    let progress = session.zone_progress(&data);

    assert_eq!(progress.len(), data.layout.zones.len());
    let total_capacity: usize = progress.iter().map(|zone| zone.capacity).sum();
    assert_eq!(total_capacity, data.config.toy_count);
}

/// Placed + broken + still-to-find must exactly reconstruct an aisle. If they
/// do not, the panel is telling the player to go looking for toys that are
/// really sitting in halves — the exact confusion `broken` exists to remove.
#[test]
fn every_aisle_slot_is_accounted_for_as_shelved_broken_or_missing() {
    let data = GameData::load().unwrap();
    let session = GameSession::new(&data);

    let mut total_broken = 0;
    for zone in session.zone_progress(&data) {
        assert_eq!(
            zone.placed + zone.broken + zone.still_to_find(),
            zone.capacity,
            "zone {} does not add up",
            zone.zone_index
        );
        total_broken += zone.broken;
    }

    // Every break must land in some aisle: a body whose category and theme
    // matched no display would silently vanish from the accounting.
    let bodies = session
        .toys
        .iter()
        .filter(|toy| toy.repair_part_kind() == Some(RepairPartKind::Body))
        .count();
    assert!(bodies > 0, "the shop broke nothing, so this proves nothing");
    assert_eq!(total_broken, bodies);
}

/// Shelving every whole toy in an aisle is not enough to restore it, and the
/// readout has to say why rather than leaving a bare shortfall.
#[test]
fn an_aisle_of_only_broken_remainders_reports_nothing_left_to_find() {
    let data = GameData::load().unwrap();
    let mut session = GameSession::new(&data);

    // Displays 0-3 are the plush wall/bin/table, all in the same zone.
    for display_index in 0..4 {
        let display = &data.displays[display_index];
        let matching: Vec<String> = session
            .toys
            .iter()
            .filter(|toy| toy_matches_display(toy, display) && toy.placed_display_id.is_none())
            .map(|toy| toy.id.clone())
            .collect();
        for (slot_index, toy_id) in matching.into_iter().enumerate() {
            let Some(toy_index) = session.toys.iter().position(|toy| toy.id == toy_id) else {
                continue;
            };
            session.pick_up_toy(toy_index, &data);
            session.place_active_toy(display_index, slot_index, &data);
        }
    }

    let plush = session.zone_progress(&data)[0];
    assert!(plush.broken > 0, "the plush aisle broke nothing");
    assert_eq!(
        plush.still_to_find(),
        0,
        "every whole plush toy is shelved, so nothing is left to hunt"
    );
    assert!(
        !plush.is_restored(),
        "an aisle with toys still in halves is not restored"
    );
}

#[test]
fn a_fresh_shop_reads_zero_everywhere_it_has_shelves() {
    let data = GameData::load().unwrap();
    let session = GameSession::new(&data);

    for zone in session.zone_progress(&data) {
        if zone.has_displays() {
            assert_eq!(zone.placed, 0);
            assert_eq!(zone.fraction(), 0.0);
        } else {
            // Nothing to tidy reads as done, not as permanently 0% complete.
            assert_eq!(zone.fraction(), 1.0);
        }
    }
}

#[test]
fn shelving_a_display_moves_only_its_own_zone() {
    let data = GameData::load().unwrap();
    let mut session = GameSession::new(&data);
    let before = session.zone_progress(&data);

    complete_display_by_index(&mut session, &data, 0);
    let after = session.zone_progress(&data);

    let moved: Vec<usize> = before
        .iter()
        .zip(&after)
        .filter(|(before, after)| before.placed != after.placed)
        .map(|(_, after)| after.zone_index)
        .collect();
    assert_eq!(moved.len(), 1, "one display should move one zone");

    let zone = after[moved[0]];
    assert_eq!(zone.placed, data.displays[0].capacity);
    assert!(zone.fraction() > 0.0 && zone.fraction() <= 1.0);
}

#[test]
fn wrongly_shelved_toys_do_not_count_toward_a_zone() {
    let data = GameData::load().unwrap();
    let mut session = GameSession::new(&data);

    // A toy whose category does not match display 0, shelved there anyway.
    let display = &data.displays[0];
    let wrong_index = session
        .toys
        .iter()
        .position(|toy| !toy.is_repair_part() && !toy.is_held && !toy_matches_display(toy, display))
        .unwrap();
    session.pick_up_toy(wrong_index, &data);
    session.place_active_toy(0, 0, &data);

    let placed: usize = session
        .zone_progress(&data)
        .iter()
        .map(|zone| zone.placed)
        .sum();
    assert_eq!(placed, 0, "a mis-shelved toy must not read as progress");
}

fn complete_display_by_index(session: &mut GameSession, data: &GameData, display_index: usize) {
    let display = &data.displays[display_index];
    let mut matching: Vec<(usize, String)> = session
        .toys
        .iter()
        .filter(|toy| {
            toy_matches_display(toy, display) && toy.placed_display_id.is_none() && !toy.is_held
        })
        .map(|toy| (toy.slot_number, toy.id.clone()))
        .collect();
    matching.sort_by_key(|(slot_number, _)| *slot_number);

    for (slot_index, (_, toy_id)) in matching.into_iter().take(display.capacity).enumerate() {
        let toy_index = session
            .toys
            .iter()
            .position(|toy| toy.id == toy_id)
            .unwrap();
        session.pick_up_toy(toy_index, data);
        let _ = session.place_active_toy(display_index, slot_index, data);
    }
}
