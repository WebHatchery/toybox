use super::*;

#[test]
fn pickup_uses_crosshair_target_instead_of_closest_toy() {
    let data = GameData::load().unwrap();
    let mut session = GameSession::new(&data);
    let aimed_toy_id = session.toys[1].id.clone();

    session.player.position = WorldPoint { x: 5.0, y: 5.0 };
    session.player.yaw = 0.0;
    session.player.pitch = -0.50;

    for toy in &mut session.toys {
        toy.position = WorldPoint { x: 16.8, y: 11.2 };
        toy.is_held = false;
        toy.placed_display_id = None;
        toy.placed_slot_index = None;
    }

    session.toys[0].position = WorldPoint { x: 5.45, y: 5.68 };
    session.toys[1].position = WorldPoint { x: 6.30, y: 5.00 };
    session.rebuild_spatial_index();

    let result = session.interact(&data);

    assert!(matches!(result, InteractionResult::PickedUp { .. }));
    assert_eq!(
        session.active_toy().map(|toy| toy.id.as_str()),
        Some(aimed_toy_id.as_str())
    );
}

#[test]
fn cannot_pick_up_second_toy_while_holding_one() {
    let data = GameData::load().unwrap();
    let mut session = GameSession::new(&data);

    let first_result = session.pick_up_toy(0, &data);
    let second_result = session.pick_up_toy(1, &data);

    assert!(matches!(first_result, InteractionResult::PickedUp { .. }));
    assert!(matches!(second_result, InteractionResult::InventoryFull));
    assert_eq!(session.player.carried_toy_ids.len(), 1);
    assert!(session.toys[0].is_held);
    assert!(!session.toys[1].is_held);
}

/// The Sorting Trolley sells "wheel three toys at once", so pressing `E` on a
/// second toy while already holding one has to load it rather than drop what is
/// in hand. Without this the trolley is unreachable through real input: every
/// gathering attempt drops the armful instead of adding to it, and a replay
/// measured the trolley *losing* to bare hands because of it.
#[test]
fn crosshair_loads_a_second_toy_when_the_trolley_gives_room() {
    let data = GameData::load().unwrap();
    let mut session = GameSession::new(&data);
    session
        .unlocked_upgrade_ids
        .push("sorting_trolley".to_owned());

    let carried_id = session.toys[0].id.clone();
    let aimed_id = session.toys[1].id.clone();
    session.pick_up_toy(0, &data);

    stand_in_the_open_looking_at(&mut session, 1);

    assert!(matches!(
        session.interaction_preview(&data),
        InteractionPreview::Pickup { .. }
    ));
    let result = session.interact(&data);

    assert!(
        matches!(result, InteractionResult::PickedUp { .. }),
        "aiming at a loose toy with room on the trolley dropped instead: {result:?}"
    );
    assert_eq!(session.player.carried_toy_ids, vec![carried_id, aimed_id]);
}

/// The bare-handed carry limit is 1, so the same aim must still drop. A player
/// without the trolley never loses the ability to put a toy down by looking at
/// the floor where another toy happens to lie.
#[test]
fn crosshair_still_drops_when_the_carry_is_already_full() {
    let data = GameData::load().unwrap();
    let mut session = GameSession::new(&data);

    session.pick_up_toy(0, &data);
    stand_in_the_open_looking_at(&mut session, 1);

    assert!(matches!(
        session.interaction_preview(&data),
        InteractionPreview::PutDown
    ));
    assert!(matches!(
        session.interact(&data),
        InteractionResult::Dropped { .. }
    ));
    assert!(session.player.carried_toy_ids.is_empty());
}

/// Put the player in open floor, well clear of any display, aimed squarely at
/// one loose toy with every other toy swept out of the crosshair.
fn stand_in_the_open_looking_at(session: &mut GameSession, toy_index: usize) {
    session.player.position = WorldPoint { x: 9.0, y: 10.8 };
    session.player.yaw = 0.0;
    session.player.pitch = -0.50;

    for toy in session.toys.iter_mut() {
        toy.position = WorldPoint { x: 16.8, y: 20.4 };
        toy.placed_display_id = None;
        toy.placed_slot_index = None;
    }
    session.toys[toy_index].position = WorldPoint { x: 9.55, y: 10.8 };
    session.toys[toy_index].is_held = false;
    session.rebuild_spatial_index();
}

/// Standing at a stocked shelf, looking down at a toy on the floor, `E` must
/// take the toy on the floor.
///
/// Shelf slots are targeted in 2D from yaw alone, so a player aiming at their
/// own feet still "looks at" every slot in front of them — and the shelf branch
/// used to be checked first. A replay driving real input found the closer
/// un-shelving what it had just put away 583 times in one run, mistaking it for
/// the crosshair handing over a neighbouring toy.
#[test]
fn looking_down_at_the_floor_beside_a_shelf_does_not_unshelve_anything() {
    let data = GameData::load().unwrap();
    let mut session = GameSession::new(&data);
    let display = &data.displays[0];

    // Shelve one toy properly, then stand at its slot facing the shelf.
    let shelved_index = session
        .toys
        .iter()
        .position(|toy| toy_matches_display(toy, display) && !toy.is_repair_part())
        .unwrap();
    let shelved_id = session.toys[shelved_index].id.clone();
    session.pick_up_toy(shelved_index, &data);
    session.place_active_toy(0, 0, &data);

    let slot = display_slot_position(display, 0, data.config.room_width);
    session.player.position = slot;
    session.player.yaw = 0.0;

    // A loose toy at the player's feet, everything else swept far away.
    let loose_index = session
        .toys
        .iter()
        .position(|toy| toy.id != shelved_id && toy.placed_display_id.is_none())
        .unwrap();
    let loose_id = session.toys[loose_index].id.clone();
    for (index, toy) in session.toys.iter_mut().enumerate() {
        if index != loose_index && toy.placed_display_id.is_none() {
            toy.position = WorldPoint { x: 30.0, y: 20.0 };
        }
    }
    session.toys[loose_index].position = WorldPoint {
        x: slot.x + 0.55,
        y: slot.y,
    };
    session.rebuild_spatial_index();

    // Aimed at the floor toy, the way the replay's closer aims.
    session.player.pitch = -1.0;

    assert!(
        matches!(
            session.interaction_preview(&data),
            InteractionPreview::Pickup { .. }
        ),
        "the probe is not looking at anything pickable"
    );
    session.interact(&data);

    assert_eq!(
        session.active_toy().map(|toy| toy.id.as_str()),
        Some(loose_id.as_str()),
        "E took something other than the toy underfoot"
    );
    let shelved = session
        .toys
        .iter()
        .find(|toy| toy.id == shelved_id)
        .unwrap();
    assert_eq!(
        shelved.placed_display_id.as_deref(),
        Some(display.id.as_str()),
        "the shelved toy was lifted back off the display"
    );
}

/// The other half of the same rule: looking *at* the shelf still retrieves from
/// it. Fixing the case above by always preferring the floor would make a
/// mis-shelved toy impossible to collect while anything lay nearby.
#[test]
fn looking_at_a_stocked_slot_still_takes_the_toy_off_the_shelf() {
    let data = GameData::load().unwrap();
    let mut session = GameSession::new(&data);
    let display = &data.displays[0];

    let shelved_index = session
        .toys
        .iter()
        .position(|toy| toy_matches_display(toy, display) && !toy.is_repair_part())
        .unwrap();
    let shelved_id = session.toys[shelved_index].id.clone();
    session.pick_up_toy(shelved_index, &data);
    session.place_active_toy(0, 0, &data);

    let slot = display_slot_position(display, 0, data.config.room_width);
    session.player.position = slot;
    session.player.yaw = 0.0;
    // Level, not down at the floor.
    session.player.pitch = 0.0;

    let loose_index = session
        .toys
        .iter()
        .position(|toy| toy.id != shelved_id && toy.placed_display_id.is_none())
        .unwrap();
    for (index, toy) in session.toys.iter_mut().enumerate() {
        if index != loose_index && toy.placed_display_id.is_none() {
            toy.position = WorldPoint { x: 30.0, y: 20.0 };
        }
    }
    session.toys[loose_index].position = WorldPoint {
        x: slot.x + 0.55,
        y: slot.y,
    };
    session.rebuild_spatial_index();

    session.interact(&data);

    assert_eq!(
        session.active_toy().map(|toy| toy.id.as_str()),
        Some(shelved_id.as_str()),
        "a level look at a stocked slot no longer retrieves from the shelf"
    );
}

#[test]
fn interact_places_held_toy_on_floor_when_no_target_is_active() {
    let data = GameData::load().unwrap();
    let mut session = GameSession::new(&data);
    let toy_id = session.toys[0].id.clone();

    session.pick_up_toy(0, &data);
    session.player.position = WorldPoint { x: 9.0, y: 10.8 };
    session.player.yaw = 0.0;
    session.player.pitch = 0.0;

    let result = session.interact(&data);

    assert!(matches!(result, InteractionResult::Dropped { .. }));
    assert!(session.player.carried_toy_ids.is_empty());

    let dropped = session.toys.iter().find(|toy| toy.id == toy_id).unwrap();
    assert!(!dropped.is_held);
    assert!(dropped.placed_display_id.is_none());
    assert!(dropped.placed_slot_index.is_none());
    assert!(dropped.position.x > session.player.position.x + 0.7);
    assert!((dropped.position.y - session.player.position.y).abs() < 0.01);
}

#[test]
fn can_pick_up_toy_from_display_slot() {
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
    session.player.position = display_slot_position(display, 0, data.config.room_width);
    session.player.yaw = 0.0;

    let result = session.interact(&data);

    assert!(matches!(result, InteractionResult::PickedUp { .. }));
    let picked_up = session.toys.iter().find(|toy| toy.id == toy_id).unwrap();
    assert!(picked_up.is_held);
    assert!(picked_up.placed_display_id.is_none());
    assert!(picked_up.placed_slot_index.is_none());
    assert_eq!(session.total_placed_toys(), 0);
}

#[test]
fn spatial_grid_tracks_pickup_place_and_drop() {
    let data = GameData::load().unwrap();
    let mut session = GameSession::new(&data);

    let toy_index = session
        .toys
        .iter()
        .position(|toy| !toy.is_repair_part() && toy.placed_display_id.is_none())
        .unwrap();
    let spawn = session.toys[toy_index].position.to_vec2();
    assert!(session
        .spatial()
        .indices_near(spawn, 0.1)
        .contains(&toy_index));

    session.pick_up_toy(toy_index, &data);
    assert!(!session
        .spatial()
        .indices_near(spawn, 0.1)
        .contains(&toy_index));

    let _ = session.place_active_toy(0, 0, &data);
    let placed = session.toys[toy_index].position.to_vec2();
    assert!(session
        .spatial()
        .indices_near(placed, 0.1)
        .contains(&toy_index));

    session.pick_up_toy(toy_index, &data);
    session.drop_active(&data).unwrap();
    let dropped = session.toys[toy_index].position.to_vec2();
    assert!(session
        .spatial()
        .indices_near(dropped, 0.1)
        .contains(&toy_index));
}

#[test]
fn movement_follows_player_yaw() {
    let data = GameData::load().unwrap();
    let mut session = GameSession::new(&data);
    let start = session.player.position;

    session.player.yaw = 0.0;
    session.move_player(vec2(0.0, 1.0), &data, 0.25);

    assert!(session.player.position.x > start.x);
    assert!((session.player.position.y - start.y).abs() < 0.001);
}

#[test]
fn player_collides_with_aisle_shelving() {
    let data = GameData::load().unwrap();
    let mut session = GameSession::new(&data);
    let shelf = &data.layout.shelving[0];

    session.player.position = WorldPoint {
        x: shelf.x + shelf.w * 0.5,
        y: shelf.y - 1.0,
    };
    session.player.yaw = std::f32::consts::FRAC_PI_2;
    for _ in 0..40 {
        session.move_player(vec2(0.0, 1.0), &data, 0.1);
    }

    let final_y = session.player.position.y;
    assert!(
        final_y > shelf.y - 1.0 + 0.2,
        "player should approach shelf"
    );
    assert!(
        final_y < shelf.y - 0.40,
        "player should stop at the shelf edge, got y {final_y}"
    );
}

#[test]
fn look_yaw_is_not_clamped_to_one_turn() {
    let data = GameData::load().unwrap();
    let mut session = GameSession::new(&data);
    session.player.yaw = 0.0;

    session.update_player_look(std::f32::consts::TAU * 2.25, 0.0);

    assert!(session.player.yaw > std::f32::consts::TAU * 2.0);
}
