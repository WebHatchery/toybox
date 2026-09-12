use super::*;

pub(super) fn resolve_repair_part(
    session: &mut GameSession,
    data: &GameData,
    walked: &mut f32,
) -> bool {
    // If the other half is already waiting on a bench, go to *that* bench.
    // This is the route the Toy Scanner exists to give the player; without it
    // a closer walks to the nearest bench, is refused because that bench holds
    // someone else's half, and the pair never meets.
    let counterpart = session.carried_counterpart();
    let target = match counterpart {
        Some(location) if location.on_bench => location.position,
        _ => {
            let bench = data.primary_bench();
            WorldPoint {
                x: bench.x,
                y: bench.y,
            }
        }
    };
    walk_to(session, data, target, walked);
    session.interact(data);

    // Only reach for the repair when the bench actually holds a matching pair.
    // Interacting at a bench holding one lone part picks that part back up,
    // which is correct in game and would leave the closer stuck holding it.
    if session.bench_status(data.primary_bench()).stage == BenchStage::Ready
        && matches!(session.interact(data), InteractionResult::Repaired { .. })
    {
        return true;
    }
    if session.active_toy().is_some_and(|toy| toy.is_repair_part()) {
        session.drop_active(data);
    }
    false
}

/// One full restoration errand: take a half, cross the shop for its other half,
/// rejoin them at a bench. Returns the toy index of the repaired toy, now in
/// the closer's hands and ready to shelve.
pub(super) fn restore_one_pair(
    session: &mut GameSession,
    data: &GameData,
    walked: &mut f32,
    actions: &mut usize,
    deferred: &mut HashSet<usize>,
) -> Option<usize> {
    let from = session.player.position.to_vec2();
    let first = nearest_loose_part(session, from, deferred)?;
    let mate = counterpart_index(session, first)?;
    if session.toys[mate].is_held {
        deferred.insert(first);
        return None;
    }
    let mate_is_waiting = session.toys[mate].bench_slot_index.is_some();

    // Half one: fetch it and park it on a bench.
    *actions += 1;
    let first_position = session.toys[first].position;
    walk_to(session, data, first_position, walked);
    if !take_directly(session, data, first) {
        deferred.insert(first);
        return None;
    }
    if resolve_repair_part(session, data, walked) {
        return session
            .active_toy()
            .and_then(|toy| session.toys.iter().position(|other| other.id == toy.id));
    }
    if mate_is_waiting {
        // The real sorting loop can park a part encountered in a pile before
        // the deliberate repair pass begins. Reaching its loose counterpart is
        // already the second half of the errand; do not try to pick the benched
        // part up as though both halves were still on the floor.
        deferred.insert(first);
        return None;
    }
    if session.toys[first].bench_slot_index.is_none() {
        // The bench would not take it — nothing more to try this errand.
        deferred.insert(first);
        return None;
    }

    // Half two: cross the shop for it, then carry it to the bench holding the
    // first half. `resolve_repair_part` routes there via the scanner.
    *actions += 1;
    let mate_position = session.toys[mate].position;
    walk_to(session, data, mate_position, walked);
    if !take_directly(session, data, mate) {
        deferred.insert(mate);
        return None;
    }
    if resolve_repair_part(session, data, walked) {
        // The survivor of a repair is the body, whichever index that is.
        return session
            .active_toy()
            .and_then(|toy| session.toys.iter().position(|other| other.id == toy.id));
    }
    deferred.insert(first);
    deferred.insert(mate);
    None
}

/// Spend credits as soon as they cover the next unlocked tool, cheapest first.
///
/// A player opens the tool screen when the HUD tells them something is
/// affordable, so buying eagerly is the faithful behaviour. Cheapest first
/// matters: the Toy Scanner at one credit arrives on the first completed
/// display, and holding out for a three-credit tool would leave the closer
/// unequipped through the part of the run where help compounds most.
pub(super) fn buy_what_it_can_afford(session: &mut GameSession, data: &GameData) {
    loop {
        let mut affordable: Vec<&toybox_after_hours::data::UpgradeDef> = data
            .upgrades
            .iter()
            .filter(|upgrade| {
                !session.has_upgrade(&upgrade.id)
                    && session.completed_display_count() >= upgrade.unlock_completed_displays
                    && session.available_tool_credits(data) >= upgrade.cost
            })
            .collect();
        affordable.sort_by_key(|upgrade| upgrade.cost);

        let Some(upgrade) = affordable.first() else {
            return;
        };
        if !matches!(
            session.purchase_tool(data, &upgrade.id),
            ToolPurchaseResult::Purchased { .. }
        ) {
            return;
        }
    }
}

pub(super) fn run(scenario: &Scenario, data: &GameData, action_budget: usize) -> RunReport {
    let mut session = GameSession::new_with_seed(data, CLOSING_SHIFT_SEED);
    for tool in scenario.tools {
        session.unlocked_upgrade_ids.push((*tool).to_owned());
    }

    // Where each display's next free slot is. Placement rejects a taken slot,
    // so the closer fills each shelf left to right like a person would.

    let mut walked = 0.0;
    let (mut shelved, mut repaired, mut actions) = (0usize, 0usize, 0usize);
    let (mut whiffs, mut neighbours, mut place_whiffs) = (0usize, 0usize, 0usize);
    // Parts the closer gave up on. Two mismatched halves fill a bench and
    // every later part then cycles pick-up, walk, refuse, drop forever, with
    // the closer parked at the bench so the dropped part is always the nearest
    // thing to it. Deferring is what a player does; without it the run stalls.
    let mut deferred: HashSet<usize> = HashSet::new();
    // `Earner` is one continuous run with two legible phases. Earlier it used
    // the nearest-first loop alone, stopped with every broken pair still on the
    // floor, and was nevertheless reported as a complete twenty-minute shift.
    let mut repairing_remainder = false;
    let mut repairs_complete = false;

    while actions < action_budget {
        if scenario.strategy == Strategy::Earner {
            buy_what_it_can_afford(&mut session, data);
        }
        if scenario.strategy == Strategy::Restorer
            || (scenario.strategy == Strategy::Earner && repairing_remainder)
        {
            // Hunt a pair. On success the closer is holding a repaired toy and
            // falls through to shelving it like any other.
            let before = actions;
            match restore_one_pair(&mut session, data, &mut walked, &mut actions, &mut deferred) {
                Some(_) => repaired += 1,
                None => {
                    if actions == before {
                        if scenario.strategy == Strategy::Earner {
                            // Return for any whole toys the real crosshair or a
                            // temporarily crowded slot made us defer. With the
                            // repair parts gone, that cleanup pass gets a clear
                            // aim rather than repeating the same obstruction.
                            repairing_remainder = false;
                            repairs_complete = true;
                            deferred.clear();
                            continue;
                        }
                        break; // no pairs left to chase
                    }
                    continue;
                }
            }
        } else {
            let next = if scenario.strategy == Strategy::Earner {
                nearest_loose_whole_toy(&session, session.player.position.to_vec2(), &deferred)
            } else {
                nearest_loose_toy(&session, session.player.position.to_vec2(), &deferred)
            };
            let Some(first_index) = next else {
                if scenario.strategy == Strategy::Earner && !repairs_complete {
                    // Parts brushed aside while digging whole toys out of a
                    // pile become eligible again for the deliberate repair
                    // pass. Deferred whole toys get another try once those
                    // obstructions have been removed.
                    deferred.clear();
                    repairing_remainder = true;
                    continue;
                }
                break;
            };
            actions += 1;
            let target = session.toys[first_index].position;
            walk_to(&mut session, data, target, &mut walked);
            let Some(held) = aim_and_pick_up(
                &mut session,
                data,
                first_index,
                &mut whiffs,
                &mut neighbours,
            ) else {
                deferred.insert(first_index);
                continue;
            };

            if session.toys[held].is_repair_part() {
                if scenario.strategy == Strategy::Earner {
                    // Keep the first phase a sorting measurement. A repair part
                    // grabbed from beside the intended whole toy is set back
                    // down and revisited once the whole floor is clear. Skip
                    // the obstructed intended target for this pass so the
                    // closer does not pick the same dropped part forever.
                    session.drop_active(data);
                    deferred.insert(first_index);
                    continue;
                }
                if resolve_repair_part(&mut session, data, &mut walked) {
                    repaired += 1;
                } else {
                    deferred.insert(held);
                    continue;
                }
            }
        }

        // Gather an armful bound for the same display before walking there.
        // A one-at-a-time routine would show no benefit from a bigger carry
        // limit at all, so this is what makes the trolley measurable.
        let Some(active) = session.active_toy().cloned() else {
            continue;
        };
        let Some(display_index) =
            home_display_index(data, &active, &session, session.player.position.to_vec2())
        else {
            session.drop_active(data);
            continue;
        };
        let display = &data.displays[display_index];

        while session.player.carried_toy_ids.len() < session.carry_limit(data)
            && actions < action_budget
        {
            let from = session.player.position.to_vec2();
            let Some(extra) = nearest_matching_toy(&session, data, display, from) else {
                break;
            };
            actions += 1;
            let extra_position = session.toys[extra].position;
            walk_to(&mut session, data, extra_position, &mut walked);
            if aim_and_pick_up(&mut session, data, extra, &mut whiffs, &mut neighbours).is_none() {
                break;
            }
        }

        // Unload the armful, walking to a free slot for each toy in it.
        while !session.player.carried_toy_ids.is_empty() {
            // An armful gathered for one display can still contain a stray: the
            // crosshair hands over whatever sat nearest the centre of the view,
            // not always the toy aimed at. A player checks what is in hand
            // before shelving it. Forcing it here would manufacture mistakes
            // and blind the run's mistake count to real placement bugs.
            let belongs = session
                .active_toy()
                .is_some_and(|toy| toy_matches_display(toy, display));
            let free_slot = next_free_slot(&session, display, 0);
            let (true, Some(slot)) = (belongs, free_slot) else {
                session.drop_active(data);
                continue;
            };

            match aim_and_place(
                &mut session,
                data,
                display_index,
                slot,
                &mut walked,
                &mut place_whiffs,
            ) {
                Some(InteractionResult::Placed { was_wrong, .. }) => {
                    if !was_wrong {
                        shelved += 1;
                    }
                }
                // Dropped rather than retried: a whiff here means the crosshair
                // offered a different slot than the gap walked to, and trying
                // the same spot again would loop forever.
                _ => {
                    session.drop_active(data);
                }
            }
        }
    }

    RunReport {
        actions,
        shelved,
        repaired,
        mistakes: session.player.mistakes,
        whiffs,
        grabbed_neighbour: neighbours,
        place_whiffs,
        deferred_parts: deferred.len(),
        displays_complete: session.completed_display_count(),
        tools_owned: session.unlocked_upgrade_ids.len(),
        credits_remaining: session.available_tool_credits(data),
        still_loose: session
            .toys
            .iter()
            .filter(|toy| {
                !toy.is_held && toy.placed_display_id.is_none() && !toy.is_consumed_repair_part()
            })
            .count(),
        walked_metres: walked,
        minutes: session.player.elapsed_seconds / 60.0,
        finished: session.phase == GamePhase::Finished,
    }
}
