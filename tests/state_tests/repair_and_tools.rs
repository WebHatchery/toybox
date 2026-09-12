use super::*;

#[test]
fn broken_part_must_be_repaired_before_display() {
    let data = GameData::load().unwrap();
    let mut session = GameSession::new(&data);
    let robot_display_index = data
        .displays
        .iter()
        .position(|display| display.category == ToyCategory::ActionFigures)
        .unwrap();
    let body_index = session
        .toys
        .iter()
        .position(|toy| toy.repair_part_kind() == Some(RepairPartKind::Body))
        .unwrap();

    session.pick_up_toy(body_index, &data);
    let result = session.place_active_toy(robot_display_index, 0, &data);

    assert!(matches!(result, InteractionResult::NeedsRepair { .. }));
    assert!(session.active_toy().unwrap().is_repair_part());
    assert_eq!(session.total_placed_toys(), 0);
}

#[test]
fn repair_bench_repairs_matching_benched_parts() {
    let data = GameData::load().unwrap();
    let mut session = GameSession::new(&data);
    let (body_id, pair_repair_id) = session
        .toys
        .iter()
        .find_map(|toy| match &toy.repair_state {
            RepairState::BrokenPart {
                repair_id,
                part: RepairPartKind::Body,
                ..
            } => Some((toy.id.clone(), repair_id.clone())),
            _ => None,
        })
        .unwrap();
    let head_id = session
        .toys
        .iter()
        .find(|toy| {
            matches!(
                &toy.repair_state,
                RepairState::BrokenPart {
                    repair_id,
                    part: RepairPartKind::Head,
                    ..
                } if *repair_id == pair_repair_id
            )
        })
        .unwrap()
        .id
        .clone();

    let bench = data.primary_bench();
    session.player.position = WorldPoint {
        x: bench.x,
        y: bench.y,
    };

    let body_index = session
        .toys
        .iter()
        .position(|toy| toy.id == body_id)
        .unwrap();
    session.pick_up_toy(body_index, &data);
    let result = session.interact(&data);

    assert!(matches!(
        result,
        InteractionResult::PlacedOnRepairBench { .. }
    ));
    assert!(session.player.carried_toy_ids.is_empty());
    assert_eq!(
        session
            .toys
            .iter()
            .find(|toy| toy.id == body_id)
            .unwrap()
            .bench_slot_index,
        Some(0)
    );

    let head_index = session
        .toys
        .iter()
        .position(|toy| toy.id == head_id)
        .unwrap();
    session.pick_up_toy(head_index, &data);
    let result = session.interact(&data);

    assert!(matches!(
        result,
        InteractionResult::PlacedOnRepairBench { .. }
    ));
    assert!(session.player.carried_toy_ids.is_empty());

    let result = session.interact(&data);

    assert!(matches!(result, InteractionResult::Repaired { .. }));
    assert_eq!(session.player.carried_toy_ids.len(), 1);
    let active_toy = session.active_toy().unwrap();
    assert_eq!(active_toy.id, body_id);
    assert!(!active_toy.is_repair_part());
    assert!(active_toy.bench_slot_index.is_none());
    let home_display = data
        .displays
        .iter()
        .find(|display| display.category == active_toy.category)
        .unwrap();
    assert!(toy_matches_display(active_toy, home_display));
    assert!(session
        .toys
        .iter()
        .find(|toy| toy.id == head_id)
        .unwrap()
        .is_consumed_repair_part());
}

#[test]
fn parts_bench_at_the_nearest_bench() {
    let data = GameData::load().unwrap();
    assert!(data.layout.benches.len() >= 2, "expected multiple benches");
    let second_bench = &data.layout.benches[1];
    let mut session = GameSession::new(&data);

    let head_index = session
        .toys
        .iter()
        .position(|toy| toy.repair_part_kind() == Some(RepairPartKind::Head))
        .unwrap();
    session.pick_up_toy(head_index, &data);
    session.player.position = WorldPoint {
        x: second_bench.x,
        y: second_bench.y,
    };

    let result = session.interact(&data);

    assert!(matches!(
        result,
        InteractionResult::PlacedOnRepairBench { .. }
    ));
    assert_eq!(
        session.toys[head_index].bench_id.as_deref(),
        Some(second_bench.id.as_str())
    );
    assert!(session.toys[head_index].bench_slot_index.is_some());
}

#[test]
fn a_lone_benched_part_names_the_counterpart_still_missing() {
    let data = GameData::load().unwrap();
    let mut session = GameSession::new(&data);
    let bench = data.primary_bench();
    session.player.position = WorldPoint {
        x: bench.x,
        y: bench.y,
    };

    let body_index = session
        .toys
        .iter()
        .position(|toy| toy.repair_part_kind() == Some(RepairPartKind::Body))
        .unwrap();
    let repaired_name = match &session.toys[body_index].repair_state {
        RepairState::BrokenPart { repaired_name, .. } => repaired_name.clone(),
        _ => unreachable!("selected a broken part"),
    };
    session.pick_up_toy(body_index, &data);
    session.interact(&data);

    assert_eq!(
        session.lone_benched_part(&data),
        Some((repaired_name, RepairPartKind::Head))
    );

    // Aiming at the waiting part still offers to take it back off the bench;
    // the guidance fills the prompt only when nothing else is targeted.
    session.player.yaw = 0.0;
    assert!(matches!(
        session.interaction_preview(&data),
        InteractionPreview::AwaitingRepairMatch {
            missing_part: RepairPartKind::Head,
            ..
        }
    ));

    // Once both halves are on the bench there is nothing left to hunt for.
    let head_index = session
        .toys
        .iter()
        .position(|toy| toy.repair_part_kind() == Some(RepairPartKind::Head))
        .unwrap();
    session.pick_up_toy(head_index, &data);
    session.interact(&data);
    assert_eq!(session.lone_benched_part(&data), None);
}

#[test]
fn the_scanner_locates_the_carried_parts_counterpart() {
    let data = GameData::load().unwrap();
    let mut session = GameSession::new(&data);

    // Nothing carried, nothing to locate.
    assert!(session.carried_counterpart().is_none());

    let (body_index, pair_repair_id) = session
        .toys
        .iter()
        .enumerate()
        .find_map(|(index, toy)| match &toy.repair_state {
            RepairState::BrokenPart {
                repair_id,
                part: RepairPartKind::Body,
                ..
            } => Some((index, repair_id.clone())),
            _ => None,
        })
        .unwrap();
    let head_index = session
        .toys
        .iter()
        .position(|toy| {
            matches!(
                &toy.repair_state,
                RepairState::BrokenPart {
                    repair_id,
                    part: RepairPartKind::Head,
                    ..
                } if *repair_id == pair_repair_id
            )
        })
        .unwrap();
    let head_position = session.toys[head_index].position;

    session.pick_up_toy(body_index, &data);
    let located = session.carried_counterpart().unwrap();

    assert_eq!(located.part, RepairPartKind::Head);
    assert!(!located.on_bench);
    assert_eq!(located.position.x, head_position.x);
    assert_eq!(located.position.y, head_position.y);

    // A whole toy has no counterpart to point at.
    session.drop_active(&data);
    let whole_index = session
        .toys
        .iter()
        .position(|toy| !toy.is_repair_part() && !toy.is_consumed_repair_part())
        .unwrap();
    session.pick_up_toy(whole_index, &data);
    assert!(session.carried_counterpart().is_none());

    // Once the counterpart is benched the scanner says so, so the player knows
    // to head for the bench rather than hunt the floor.
    session.drop_active(&data);
    let bench = data.primary_bench();
    session.player.position = WorldPoint {
        x: bench.x,
        y: bench.y,
    };
    session.pick_up_toy(head_index, &data);
    session.interact(&data);
    session.pick_up_toy(body_index, &data);

    let located = session.carried_counterpart().unwrap();
    assert_eq!(located.part, RepairPartKind::Head);
    assert!(located.on_bench);
}

#[test]
fn bench_status_reports_every_stage_for_the_beacon() {
    let data = GameData::load().unwrap();
    let mut session = GameSession::new(&data);
    let bench = data.primary_bench();
    session.player.position = WorldPoint {
        x: bench.x,
        y: bench.y,
    };

    assert_eq!(
        session.bench_status(bench),
        BenchStatus {
            filled: 0,
            capacity: bench.capacity,
            stage: BenchStage::Empty,
        }
    );

    let (body_id, pair_repair_id) = session
        .toys
        .iter()
        .find_map(|toy| match &toy.repair_state {
            RepairState::BrokenPart {
                repair_id,
                part: RepairPartKind::Body,
                ..
            } => Some((toy.id.clone(), repair_id.clone())),
            _ => None,
        })
        .unwrap();
    let body_index = session
        .toys
        .iter()
        .position(|toy| toy.id == body_id)
        .unwrap();
    session.pick_up_toy(body_index, &data);
    session.interact(&data);

    assert_eq!(session.bench_status(bench).stage, BenchStage::AwaitingMatch);
    assert_eq!(session.bench_status(bench).filled, 1);

    // Mismatched is no longer reachable by placing — the bench refuses a part
    // from another break — but a save written before that fix still holds one,
    // and the beacon has to report it. Stage it the way such a save would.
    let stranger_index = session
        .toys
        .iter()
        .position(|toy| {
            matches!(
                &toy.repair_state,
                RepairState::BrokenPart {
                    repair_id,
                    part: RepairPartKind::Head,
                    ..
                } if *repair_id != pair_repair_id
            )
        })
        .unwrap();
    session.toys[stranger_index].bench_id = Some(bench.id.clone());
    session.toys[stranger_index].bench_slot_index = Some(1);
    let mut session = GameSession::from_save(session.to_save("3.0.0"), &data);
    session.player.position = WorldPoint {
        x: bench.x,
        y: bench.y,
    };

    assert_eq!(session.bench_status(bench).stage, BenchStage::Mismatched);
    assert_eq!(session.bench_status(bench).filled, bench.capacity);

    // Swap the stranger for the real counterpart and the beacon goes green.
    session.pick_up_toy(stranger_index, &data);
    let matching_head_index = session
        .toys
        .iter()
        .position(|toy| {
            matches!(
                &toy.repair_state,
                RepairState::BrokenPart {
                    repair_id,
                    part: RepairPartKind::Head,
                    ..
                } if *repair_id == pair_repair_id
            )
        })
        .unwrap();
    session.drop_active(&data);
    session.pick_up_toy(matching_head_index, &data);
    session.interact(&data);

    assert_eq!(session.bench_status(bench).stage, BenchStage::Ready);
}

#[test]
fn a_bench_refuses_a_part_from_a_different_break() {
    let data = GameData::load().unwrap();
    let mut session = GameSession::new(&data);
    let bench = data.primary_bench();
    session.player.position = WorldPoint {
        x: bench.x,
        y: bench.y,
    };

    let body_index = session
        .toys
        .iter()
        .position(|toy| toy.repair_part_kind() == Some(RepairPartKind::Body))
        .unwrap();
    let benched_repair_id = session.toys[body_index].repair_state.clone();
    session.pick_up_toy(body_index, &data);
    session.interact(&data);

    // A head from a different break shares no repair_id with the benched body.
    let stranger_index = session
        .toys
        .iter()
        .position(|toy| {
            toy.repair_part_kind() == Some(RepairPartKind::Head)
                && !matches!(
                    (&toy.repair_state, &benched_repair_id),
                    (
                        RepairState::BrokenPart { repair_id: left, .. },
                        RepairState::BrokenPart { repair_id: right, .. },
                    ) if left == right
                )
        })
        .unwrap();
    session.pick_up_toy(stranger_index, &data);

    assert!(matches!(
        session.interaction_preview(&data),
        InteractionPreview::RepairMismatch
    ));

    // The prompt warns and the placement is actually refused, so the bench
    // cannot be filled with two halves that will never join.
    assert!(matches!(
        session.interact(&data),
        InteractionResult::RepairMismatch
    ));
    assert_eq!(
        session.bench_status(bench).filled,
        1,
        "the waiting half should still be alone on the bench"
    );
    assert!(
        session.active_toy().is_some(),
        "a refused part stays in the player's hands rather than vanishing"
    );
}

#[test]
fn tool_purchases_use_completed_display_credits() {
    let data = GameData::load().unwrap();
    let mut session = GameSession::new(&data);

    assert_eq!(session.carry_limit(&data), 1);
    assert!(!session.scanner_enabled(&data));
    assert!(matches!(
        session.purchase_tool(&data, "toy_scanner"),
        ToolPurchaseResult::Locked { .. }
    ));

    complete_display_by_index(&mut session, &data, 0);
    assert_eq!(session.available_tool_credits(&data), 1);

    let result = session.purchase_tool(&data, "toy_scanner");
    assert!(matches!(
        result,
        ToolPurchaseResult::Purchased {
            ref tool_name,
            remaining_credits: 0,
        } if tool_name == "Toy Scanner"
    ));
    assert!(session.scanner_enabled(&data));
    assert_eq!(session.carry_limit(&data), 1);

    complete_display_by_index(&mut session, &data, 1);
    assert_eq!(session.available_tool_credits(&data), 1);
    assert!(matches!(
        session.purchase_tool(&data, "small_trolley"),
        ToolPurchaseResult::NoToolsAvailable
    ));
    assert_eq!(session.carry_limit(&data), 1);
}

fn complete_display_by_index(session: &mut GameSession, data: &GameData, display_index: usize) {
    let display = &data.displays[display_index];
    let mut matching_toy_ids: Vec<(usize, String)> = session
        .toys
        .iter()
        .filter(|toy| {
            // Same-theme displays share matching toys: leave already-shelved
            // ones on their display.
            toy_matches_display(toy, display) && toy.placed_display_id.is_none() && !toy.is_held
        })
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
        session.pick_up_toy(toy_index, data);
        let _ = session.place_active_toy(display_index, slot_index, data);
    }
}
