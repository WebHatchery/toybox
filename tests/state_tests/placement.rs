use super::*;

#[test]
fn correct_placement_completes_a_display() {
    let data = GameData::load().unwrap();
    let mut session = GameSession::new(&data);
    let display = &data.displays[0];
    let mut matching_toy_ids: Vec<(usize, String)> = session
        .toys
        .iter()
        .filter(|toy| toy_matches_display(toy, display))
        .map(|toy| (toy.slot_number, toy.id.clone()))
        .collect();
    matching_toy_ids.sort_by_key(|(slot_number, _)| *slot_number);

    for (slot_index, (_, toy_id)) in matching_toy_ids
        .into_iter()
        .take(display.capacity)
        .enumerate()
    {
        let toy_index = session
            .toys
            .iter()
            .position(|toy| toy.id == toy_id)
            .unwrap();
        session.pick_up_toy(toy_index, &data);
        session.player.position = WorldPoint {
            x: display.x + display.w * 0.5,
            y: display.y + display.h * 0.5,
        };
        let _ = session.place_active_toy(0, slot_index, &data);
    }

    assert!(session.is_display_complete(&display.id));
    assert_eq!(session.completed_display_count(), 1);
}

#[test]
fn placement_uses_target_slot() {
    let data = GameData::load().unwrap();
    let mut session = GameSession::new(&data);
    let display = &data.displays[0];
    let toy_id = session
        .toys
        .iter()
        .find(|toy| toy_matches_display(toy, display) && toy.slot_number == 4)
        .unwrap()
        .id
        .clone();
    let toy_index = session
        .toys
        .iter()
        .position(|toy| toy.id == toy_id)
        .unwrap();

    session.pick_up_toy(toy_index, &data);
    session.player.position = WorldPoint {
        x: display.x + display.w * 0.5,
        y: display.y + display.h * 0.5,
    };
    let _ = session.place_active_toy(0, 0, &data);

    let placed = session.toys.iter().find(|toy| toy.id == toy_id).unwrap();
    let expected = display_slot_position(display, 0, data.config.room_width);
    assert_eq!(placed.placed_slot_index, Some(0));
    assert!((placed.position.x - expected.x).abs() < f32::EPSILON);
    assert!((placed.position.y - expected.y).abs() < f32::EPSILON);
}

#[test]
fn wrong_placement_still_places_toy_and_counts_mistake() {
    let data = GameData::load().unwrap();
    let mut session = GameSession::new(&data);
    let display = &data.displays[0];
    let toy_index = session
        .toys
        .iter()
        .position(|toy| !toy_matches_display(toy, display) && !toy.is_repair_part())
        .unwrap();
    let toy_id = session.toys[toy_index].id.clone();

    session.pick_up_toy(toy_index, &data);
    let _ = session.place_active_toy(0, 0, &data);

    let placed = session.toys.iter().find(|toy| toy.id == toy_id).unwrap();
    assert_eq!(
        placed.placed_display_id.as_deref(),
        Some(display.id.as_str())
    );
    assert!(placed.wrong_marker_seconds > 0.0);
    assert_eq!(session.player.mistakes, 1);

    session.update_timer(GameSession::WRONG_MARKER_SECONDS + 0.1, &data);
    let placed = session.toys.iter().find(|toy| toy.id == toy_id).unwrap();
    assert_eq!(placed.wrong_marker_seconds, 0.0);
}

/// A full front row must not shadow the rows behind it.
///
/// Slots are laid out five to a row, so a display's capacity is its depth. Slot
/// targeting scores by distance, so the nearest slot wins — and once row one is
/// full, standing at the shelf offers a taken slot and `E` refuses, with rows
/// two and three empty a few centimetres further back. The scaling replay put
/// numbers on it: 2 refusals per run at capacity 12, 1133 at 20, 15793 at 200,
/// where the shop simply cannot be filled.
///
/// Holding a toy, the player's intent is unambiguous — they want somewhere to
/// put it — so the search should only consider slots that can actually take it.
#[test]
fn a_full_front_row_does_not_block_the_rows_behind_it() {
    let data = GameData::load().unwrap();
    let mut session = GameSession::new(&data);
    let display = &data.displays[0];
    let columns = display.capacity.clamp(1, 5);
    assert!(
        display.capacity > columns,
        "display 0 is a single row, so this proves nothing"
    );

    let matching: Vec<String> = session
        .toys
        .iter()
        .filter(|toy| toy_matches_display(toy, display) && !toy.is_repair_part())
        .take(columns + 1)
        .map(|toy| toy.id.clone())
        .collect();

    // Fill the whole front row.
    for (slot_index, toy_id) in matching.iter().take(columns).enumerate() {
        let toy_index = session
            .toys
            .iter()
            .position(|toy| &toy.id == toy_id)
            .unwrap();
        session.pick_up_toy(toy_index, &data);
        session.place_active_toy(0, slot_index, &data);
    }

    // Stand at the front row holding one more, facing the shelf.
    let front = display_slot_position(display, 2, data.config.room_width);
    let spare = session
        .toys
        .iter()
        .position(|toy| toy.id == matching[columns])
        .unwrap();
    session.pick_up_toy(spare, &data);
    let spare_id = session.toys[spare].id.clone();
    session.player.position = front;
    session.player.yaw = 0.0;
    session.player.pitch = 0.0;

    assert!(
        matches!(
            session.interaction_preview(&data),
            InteractionPreview::PlaceOnShelf
        ),
        "the shelf refused a toy while it still had {} free slots",
        display.capacity - columns
    );

    let result = session.interact(&data);
    assert!(
        matches!(
            result,
            InteractionResult::Placed {
                was_wrong: false,
                ..
            }
        ),
        "expected a placement, got {result:?}"
    );

    let placed = session.toys.iter().find(|toy| toy.id == spare_id).unwrap();
    assert_eq!(
        placed.placed_display_id.as_deref(),
        Some(display.id.as_str())
    );
    assert!(
        placed.placed_slot_index.is_some_and(|slot| slot >= columns),
        "the toy went into the front row, which was already full"
    );
}

#[test]
fn cannot_place_into_filled_slot() {
    let data = GameData::load().unwrap();
    let mut session = GameSession::new(&data);
    let display = &data.displays[0];
    let toy_ids: Vec<String> = session
        .toys
        .iter()
        .filter(|toy| toy_matches_display(toy, display))
        .take(2)
        .map(|toy| toy.id.clone())
        .collect();

    let first_index = session
        .toys
        .iter()
        .position(|toy| toy.id == toy_ids[0])
        .unwrap();
    session.pick_up_toy(first_index, &data);
    let _ = session.place_active_toy(0, 0, &data);

    let second_index = session
        .toys
        .iter()
        .position(|toy| toy.id == toy_ids[1])
        .unwrap();
    session.pick_up_toy(second_index, &data);
    let result = session.place_active_toy(0, 0, &data);

    assert!(matches!(result, InteractionResult::ShelfSlotUnavailable));
    assert!(
        session
            .toys
            .iter()
            .find(|toy| toy.id == toy_ids[1])
            .unwrap()
            .is_held
    );
    assert_eq!(
        session
            .placed_toys_for_display(&display.id)
            .filter(|toy| toy.placed_slot_index == Some(0))
            .count(),
        1
    );
}

#[test]
fn wrong_toys_do_not_complete_display() {
    let data = GameData::load().unwrap();
    let mut session = GameSession::new(&data);
    let display = &data.displays[0];
    let wrong_toy_ids: Vec<String> = session
        .toys
        .iter()
        .filter(|toy| !toy_matches_display(toy, display) && !toy.is_repair_part())
        .take(display.capacity)
        .map(|toy| toy.id.clone())
        .collect();

    for (slot_index, toy_id) in wrong_toy_ids.into_iter().enumerate() {
        let toy_index = session
            .toys
            .iter()
            .position(|toy| toy.id == toy_id)
            .unwrap();
        session.pick_up_toy(toy_index, &data);
        let _ = session.place_active_toy(0, slot_index, &data);
    }

    assert_eq!(session.total_placed_toys(), display.capacity);
    assert!(!session.is_display_complete(&display.id));
}

#[test]
fn full_shelf_rejects_extra_toy() {
    let data = GameData::load().unwrap();
    let mut session = GameSession::new(&data);
    let display = &data.displays[0];
    let mut matching_toy_ids: Vec<(usize, String)> = session
        .toys
        .iter()
        .filter(|toy| toy_matches_display(toy, display))
        .map(|toy| (toy.slot_number, toy.id.clone()))
        .collect();
    matching_toy_ids.sort_by_key(|(slot_number, _)| *slot_number);

    for (slot_index, (_, toy_id)) in matching_toy_ids
        .into_iter()
        .take(display.capacity)
        .enumerate()
    {
        let toy_index = session
            .toys
            .iter()
            .position(|toy| toy.id == toy_id)
            .unwrap();
        session.pick_up_toy(toy_index, &data);
        let _ = session.place_active_toy(0, slot_index, &data);
    }

    let extra_toy_index = session
        .toys
        .iter()
        .position(|toy| !toy_matches_display(toy, display) && !toy.is_repair_part())
        .unwrap();
    session.pick_up_toy(extra_toy_index, &data);
    let result = session.place_active_toy(0, 0, &data);

    assert!(matches!(result, InteractionResult::ShelfFull));
}
