//! Deterministic replays at shop scale.
//!
//! A scripted closer works the real `GameSession` API — the same `pick_up_toy`,
//! `interact` and `place_active_toy` the player drives — so scoring, mistakes,
//! credits, repairs and completion all come from the game rather than from a
//! model of it. Only travel is modelled: the closer teleports to each target
//! and pays the walk in seconds.
//!
//! Nothing here samples randomness, so two runs of the same scenario produce
//! byte-identical reports. That is what makes these useful for balance work:
//! change a number in `assets/data/*.json` and the difference in the report is
//! entirely attributable to that change.

#[path = "replay/runner.rs"]
mod runner;

use super::*;
use macroquad::prelude::{vec2, Vec2};
use runner::*;
use std::collections::HashSet;
use toybox_after_hours::state::WorldPoint;

/// How the closer decides what to do next.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Strategy {
    /// Shelve whatever is nearest, benching repair parts only when one happens
    /// to be underfoot. This measures the sorting loop.
    NearestFirst,
    /// Hunt broken pairs on purpose: take a half, go fetch its other half from
    /// wherever in the shop it landed, rejoin them, then shelve the result.
    /// This measures the repair loop, which the scatter is designed around.
    Restorer,
    /// Sort every whole toy, spend credits the moment they arrive, then use
    /// the earned tools to hunt and mend every broken pair.
    ///
    /// This is the only complete timed run here: tools do not carry between
    /// shifts, so nobody starts equipped and a finish has to include both the
    /// sorting and restoration loops. The other scenarios isolate one loop or
    /// bracket its tool value; this one actually closes the store.
    Earner,
}

/// A starting loadout and a way of playing, to measure together.
struct Scenario {
    name: &'static str,
    tools: &'static [&'static str],
    strategy: Strategy,
}

const BEGINNER: Scenario = Scenario {
    name: "beginner",
    tools: &[],
    strategy: Strategy::NearestFirst,
};
const MID_UPGRADE: Scenario = Scenario {
    name: "mid-upgrade",
    tools: &["toy_scanner", "sorting_trolley"],
    strategy: Strategy::NearestFirst,
};
const FULLY_EQUIPPED: Scenario = Scenario {
    name: "fully equipped",
    tools: &[
        "toy_scanner",
        "sorting_trolley",
        "grippy_sneakers",
        "long_handled_grabber",
        "managers_nod",
    ],
    strategy: Strategy::NearestFirst,
};
const EARNER: Scenario = Scenario {
    name: "earns tools",
    tools: &[],
    strategy: Strategy::Earner,
};
const RESTORER: Scenario = Scenario {
    name: "restorer",
    tools: &["toy_scanner"],
    strategy: Strategy::Restorer,
};
const RESTORER_EQUIPPED: Scenario = Scenario {
    name: "restorer+tools",
    tools: &[
        "toy_scanner",
        "sorting_trolley",
        "grippy_sneakers",
        "long_handled_grabber",
    ],
    strategy: Strategy::Restorer,
};

/// Seconds of fiddling per interaction, on top of the walk. Not tuned against
/// a real player — it only has to be the same across scenarios for the
/// comparison between them to mean something.
const INTERACTION_SECONDS: f32 = 0.6;

#[derive(Debug, Clone, Copy, PartialEq)]
struct RunReport {
    actions: usize,
    shelved: usize,
    repaired: usize,
    mistakes: u32,
    /// Times the crosshair delivered nothing at all — the closer walked to a
    /// toy, aimed at it, and `E` did not pick anything up.
    whiffs: usize,
    /// Times the crosshair delivered a *different* toy than the one aimed at,
    /// because a neighbour in the pile sat nearer the centre of the view. The
    /// closer still walks away holding something, so this is pile texture, not
    /// lost work.
    grabbed_neighbour: usize,
    /// Times the closer walked to a gap on a shelf and `E` would not shelve
    /// there, because the crosshair offered a different slot that was taken.
    place_whiffs: usize,
    deferred_parts: usize,
    displays_complete: usize,
    tools_owned: usize,
    credits_remaining: usize,
    /// Toys still on the floor when the run stopped. Together with `actions`
    /// this says whether a short run ran out of budget or stalled: a run that
    /// spent its whole budget and left most of the shop loose is doing
    /// something other than shelving.
    still_loose: usize,
    walked_metres: f32,
    minutes: f32,
    finished: bool,
}

impl RunReport {
    fn line(&self, name: &str) -> String {
        format!(
            "{name:>15}: {:>5} actions  {:>5} shelved  {:>5} loose  {:>3} repaired  \
             {:>2} mistakes  {:>4} deferred  {:>4} whiffed  {:>4} neighbour  \
             {:>4} noslot  {:>2} displays  {:>1} tools  {:>2} credits  \
             {:>6.0}m walked  {:>6.1} min  {}",
            self.actions,
            self.shelved,
            self.still_loose,
            self.repaired,
            self.mistakes,
            self.deferred_parts,
            self.whiffs,
            self.grabbed_neighbour,
            self.place_whiffs,
            self.displays_complete,
            self.tools_owned,
            self.credits_remaining,
            self.walked_metres,
            self.minutes,
            if self.finished { "FINISHED" } else { "partial" }
        )
    }
}

/// Walk the closer to arm's length of `target` and turn to face it, charging
/// the clock for the distance. Stopping short matters: standing exactly on a
/// toy puts it at zero range, and the game's crosshair targeting rejects
/// anything that is not actually in front of the player.
/// Mirrors `PLAYER_EYE_HEIGHT` in `state/interactions.rs`, which is private.
const EYE_HEIGHT: f32 = 1.08;
/// Roughly where a loose toy's aim centre sits above the floor.
const TOY_AIM_HEIGHT: f32 = 0.22;
/// How far short of a target the closer stops.
const STANDOFF: f32 = 0.55;

fn walk_to(session: &mut GameSession, data: &GameData, target: WorldPoint, walked: &mut f32) {
    let from = session.player.position.to_vec2();
    let to_target = target.to_vec2() - from;
    let distance = to_target.length();
    let arrival = if distance > STANDOFF {
        target.to_vec2() - to_target.normalize() * STANDOFF
    } else {
        from
    };

    let speed = data.config.player_speed * session.speed_multiplier(data);
    session.player.position = WorldPoint::from_vec2(arrival);
    if to_target.length_squared() > f32::EPSILON {
        session.player.yaw = to_target.y.atan2(to_target.x);
    }
    session.player.elapsed_seconds += distance / speed.max(0.1) + INTERACTION_SECONDS;
    *walked += distance;
}

/// Pick a toy up the way a player does: stand in front of it, look at it, and
/// press E — but only once the game agrees that E will pick something up.
///
/// `interact` dispatches on context, so a script cannot simply press E and
/// assume. Near a display while holding something it shelves instead, which an
/// earlier version of this harness discovered by inventing mis-shelvings that
/// were its own fault. `interaction_preview` is the game's own answer to "what
/// would E do here", so checking it first is what makes driving the input
/// layer faithful rather than reckless.
///
/// Returns the toy actually picked up, which is not always the one aimed at:
/// in a pile the crosshair takes whatever is nearest the centre of the view.
/// Callers must work with what they got rather than forcing what they wanted —
/// forcing it is how the first attempt at this ended up holding one toy and
/// shelving it on another's display.
fn aim_and_pick_up(
    session: &mut GameSession,
    data: &GameData,
    intended: usize,
    whiffs: &mut usize,
    neighbours: &mut usize,
) -> Option<usize> {
    // Look down at the floor, not out across it. A fixed shallow pitch aims
    // over the top of a toy standing at arm's length and misses almost every
    // time — the first measured pass reported an 89% miss rate for exactly
    // this reason, and read it as the pile being crowded.
    let ground = session
        .player
        .position
        .to_vec2()
        .distance(session.toys[intended].position.to_vec2())
        .max(STANDOFF * 0.5);
    session.player.pitch = ((TOY_AIM_HEIGHT - EYE_HEIGHT) / ground)
        .atan()
        .clamp(-GameSession::MAX_LOOK_PITCH, GameSession::MAX_LOOK_PITCH);

    if !matches!(
        session.interaction_preview(data),
        InteractionPreview::Pickup { .. }
    ) {
        *whiffs += 1;
        return None;
    }

    if !matches!(session.interact(data), InteractionResult::PickedUp { .. }) {
        *whiffs += 1;
        return None;
    }

    let held = session.active_toy()?.id.clone();
    let got = session.toys.iter().position(|toy| toy.id == held)?;
    if got != intended {
        *neighbours += 1;
    }
    Some(got)
}

/// Is nothing shelved in this slot yet?
///
/// `GameSession` knows this privately, but the closer needs it too: with real
/// aiming it cannot pick a slot, only stand in front of one, so it has to walk
/// the shelf to a free spot the way a player scanning for a gap does.
fn slot_is_free(
    session: &GameSession,
    display: &toybox_after_hours::data::DisplayDef,
    slot_index: usize,
) -> bool {
    !session.toys.iter().any(|toy| {
        toy.placed_display_id.as_deref() == Some(display.id.as_str())
            && toy.placed_slot_index == Some(slot_index)
    })
}

fn next_free_slot(
    session: &GameSession,
    display: &toybox_after_hours::data::DisplayDef,
    from: usize,
) -> Option<usize> {
    (from..display.capacity).find(|slot| slot_is_free(session, display, *slot))
}

/// Shelve the active toy the way a player does: walk to the gap, face it, and
/// press E only once the game agrees E will shelve.
///
/// The counterpart to `aim_and_pick_up`, and the half the replay was missing.
/// Going straight to `place_active_toy` charged a run for *finding* a toy but
/// not for putting it away, which is a walk to a specific spot on a specific
/// fixture for every single toy — and, with a trolley, a separate walk per toy
/// in the armful, because the slot just filled is no longer the one the
/// crosshair offers.
fn aim_and_place(
    session: &mut GameSession,
    data: &GameData,
    display_index: usize,
    slot_index: usize,
    walked: &mut f32,
    place_whiffs: &mut usize,
) -> Option<InteractionResult> {
    let display = &data.displays[display_index];
    let slot = display_slot_position(display, slot_index, data.config.room_width);
    walk_to(session, data, slot, walked);
    // Shelf slots are targeted from yaw alone, so pitch cannot change the
    // outcome — but leaving it aimed at the floor from the last pickup would
    // misrepresent where the player is looking when they press E.
    session.player.pitch = 0.0;

    if !matches!(
        session.interaction_preview(data),
        InteractionPreview::PlaceOnShelf
    ) {
        *place_whiffs += 1;
        return None;
    }
    Some(session.interact(data))
}

/// Take a *named* toy, bypassing the crosshair.
///
/// The restoration errand needs one specific half, and at 4000 loose toys the
/// crosshair cannot reliably deliver a named toy — measured, the closer failed
/// to dig out its intended part on essentially every attempt even given six
/// tries each, and completed zero repairs. Sorting does not have this problem
/// because any toy in the pile is a fine toy to shelve, which is why that
/// strategy runs on real input above.
///
/// So this measures the errand's *travel and rejoin* cost with the digging
/// abstracted away. The digging cost is real and is not in these numbers; see
/// the aim-miss column on the sorting rows for its scale.
fn take_directly(session: &mut GameSession, data: &GameData, toy_index: usize) -> bool {
    matches!(
        session.pick_up_toy(toy_index, data),
        InteractionResult::PickedUp { .. }
    )
}

/// The closest toy still needing work, found through the real spatial grid so
/// the search cost does not dominate a 4000-action run.
fn nearest_loose_toy(session: &GameSession, from: Vec2, skip: &HashSet<usize>) -> Option<usize> {
    nearest_loose_toy_matching(session, from, skip, |_| true)
}

/// The nearest whole toy that can be shelved without a repair. A complete
/// closer deliberately finishes this pass before starting the cross-zone pair
/// hunt, so the two costs remain visible while still sharing one real session.
fn nearest_loose_whole_toy(
    session: &GameSession,
    from: Vec2,
    skip: &HashSet<usize>,
) -> Option<usize> {
    nearest_loose_toy_matching(session, from, skip, |toy| !toy.is_repair_part())
}

fn nearest_loose_toy_matching(
    session: &GameSession,
    from: Vec2,
    skip: &HashSet<usize>,
    matches: impl Fn(&ToyState) -> bool,
) -> Option<usize> {
    let mut radius = 2.0_f32;
    while radius <= 64.0 {
        let mut best: Option<(usize, f32)> = None;
        for index in session.spatial().indices_near(from, radius) {
            let toy = &session.toys[index];
            if skip.contains(&index)
                || toy.is_held
                || toy.placed_display_id.is_some()
                || toy.bench_slot_index.is_some()
                || toy.is_consumed_repair_part()
                || !matches(toy)
            {
                continue;
            }
            let distance = toy.position.to_vec2().distance(from);
            if best.is_none_or(|(_, best_distance)| distance < best_distance) {
                best = Some((index, distance));
            }
        }
        if let Some((index, _)) = best {
            return Some(index);
        }
        radius *= 2.0;
    }
    None
}

/// The nearest loose whole toy that belongs on `display`.
fn nearest_matching_toy(
    session: &GameSession,
    data: &GameData,
    display: &toybox_after_hours::data::DisplayDef,
    from: Vec2,
) -> Option<usize> {
    let mut radius = 2.0_f32;
    while radius <= 16.0 {
        let mut best: Option<(usize, f32)> = None;
        for index in session.spatial().indices_near(from, radius) {
            let toy = &session.toys[index];
            if toy.is_held
                || toy.placed_display_id.is_some()
                || toy.bench_slot_index.is_some()
                || toy.is_consumed_repair_part()
                || toy.is_repair_part()
                || !toy_matches_display(toy, display)
            {
                continue;
            }
            let distance = toy.position.to_vec2().distance(from);
            if best.is_none_or(|(_, best_distance)| distance < best_distance) {
                best = Some((index, distance));
            }
        }
        if let Some((index, _)) = best {
            return Some(index);
        }
        radius *= 2.0;
    }
    let _ = data;
    None
}

/// The nearest loose repair part, wherever in the shop it is. The restorer
/// searches the whole room rather than a local radius: the errand is the point.
fn nearest_loose_part(session: &GameSession, from: Vec2, skip: &HashSet<usize>) -> Option<usize> {
    session
        .toys
        .iter()
        .enumerate()
        .filter(|(index, toy)| {
            !skip.contains(index)
                && toy.is_repair_part()
                && !toy.is_held
                && toy.bench_slot_index.is_none()
        })
        .min_by(|(_, left), (_, right)| {
            let left_distance = left.position.to_vec2().distance_squared(from);
            let right_distance = right.position.to_vec2().distance_squared(from);
            left_distance.total_cmp(&right_distance)
        })
        .map(|(index, _)| index)
}

/// The other half of a broken toy: the part sharing its `repair_id`.
fn counterpart_index(session: &GameSession, toy_index: usize) -> Option<usize> {
    let RepairState::BrokenPart { repair_id, .. } = &session.toys[toy_index].repair_state else {
        return None;
    };
    let own_id = &session.toys[toy_index].id;
    session.toys.iter().position(|toy| {
        &toy.id != own_id
            && matches!(
                &toy.repair_state,
                RepairState::BrokenPart { repair_id: other, .. } if other == repair_id
            )
    })
}

/// Where a toy belongs: the nearest display of its category and theme that
/// still has a free slot.
///
/// Every category owns four displays, so "the first one that matches" is not a
/// shelf-choosing rule, it is a way to ignore three quarters of the shop. A
/// closer using it fills 1000 of the 4000 slots, and then every remaining toy
/// it picks up is carried to a shelf that has been full for hours and dropped
/// again — 9700 wasted actions in an 8000-action shift, which is what made a
/// full shop look unfinishable. Room first, then distance, because a player
/// who walks to a full shelf has to walk somewhere else anyway.
fn home_display_index(
    data: &GameData,
    toy: &ToyState,
    session: &GameSession,
    from: Vec2,
) -> Option<usize> {
    data.displays
        .iter()
        .enumerate()
        .filter(|(_, display)| {
            toy_matches_display(toy, display) && next_free_slot(session, display, 0).is_some()
        })
        .min_by(|(_, left), (_, right)| {
            display_centre(left)
                .distance_squared(from)
                .total_cmp(&display_centre(right).distance_squared(from))
        })
        .map(|(index, _)| index)
}

fn display_centre(display: &toybox_after_hours::data::DisplayDef) -> Vec2 {
    vec2(display.x + display.w * 0.5, display.y + display.h * 0.5)
}

/// Bench a carried repair part, repairing if its other half is already there.
/// Returns true when the closer walks away holding a repaired toy.
/// A bare-handed closer must be able to empty the floor of whole toys, and to
/// do it without spinning. Both halves matter: a closer that only ever walks to
/// the *first* display of each category fills a quarter of the shop and then
/// carries every remaining toy to a shelf that has been full for hours, and the
/// only symptom is a run that burns its whole budget with the floor still
/// covered. Asserting on the budget is what hid that for so long.
/// Nothing the renderer reads may reach the score. These five values decide
/// which toys get drawn, at what detail, and how much of the view is culled —
/// change them to their most aggressive settings and the run must come out
/// byte-identical. If one ever leaks into targeting or placement, a player who
/// nudged their view distance would quietly be playing a different game.
/// The repair loop is the half of the game the scatter is built around: cross
/// the shop for a missing head, rejoin it, shelve the whole toy. A nearest-first
/// closer never runs that errand, so it needs measuring on its own terms.
/// Can a shift actually be finished before the doors open?
///
/// `shift_seconds` was set from the bare-handed replay, but nobody plays a
/// bare-handed run: tools do not carry between shifts, so every timed run
/// starts with nothing and earns its way up. This measures that run, and the
/// two bracketing loadouts alongside it, against the real deadline.
/// A display has to stay fillable to its back row.
///
/// Slots are laid out five to a row, so capacity decides depth: 12 slots is
/// three rows, 200 is forty. Shelf targeting used to take the nearest slot in
/// the crosshair and only then ask whether it was free, so a full front row
/// shadowed everything behind it — walking to a gap in row three still offered
/// the taken slot in row one, and `E` refused. The sweep measured 2 refusals a
/// run at capacity 12, 7173 at 100, and 15793 at 200 where the shop could not
/// be filled at all.
///
/// `targeted_empty_display_slot` searches free slots only now, and this watches
/// that it stays that way. Nothing else would fail if the two came apart again:
/// displays would just quietly stop accepting toys past their first row.
/// What a wrong shelf should cost, expressed in the only unit the player
/// experiences: another toy's worth of work.
///
/// A mis-shelving already costs real time — the toy sits in a slot without
/// counting, so it blocks its display and has to be collected and re-shelved.
/// `mistake_penalty_seconds` is the surcharge on top, and the rule it encodes
/// is "one more toy". That is a number nobody can eyeball, because seconds per
/// toy falls out of shop size, walking speed and tools all at once; this ties
/// the two together so a retune of any of them shows up here instead of
/// silently making the penalty trivial or savage.
/// The comparison the balance TODO needs: same shop, same script, different
/// loadout. Prints the table so a JSON retune can be re-measured.
/// The diagnostic whole-shop report. Slow by design — run it explicitly with
/// `cargo test --release full_shift -- --ignored --nocapture` when retuning.
/// What a shift costs at each shop size, for deciding `toy_count`.
///
/// Every display holds the same number, so one knob moves the whole shop and
/// keeps the capacity-equals-`toy_count` invariant intact. Run it with
/// `cargo test --release shop_scale -- --ignored --nocapture`.
///
/// Note that `displays` stays near zero at every size: a display cannot
/// complete until its broken toys are rejoined, and this closer defers the
/// repair errand rather than running it. Read the shelved/minutes columns for
/// length and `the_restoration_errand_is_winnable` for the other half.
/// The crosshair hands over the toy you aimed at.
///
/// Two different failures used to be added together and reported as "the
/// crosshair misses half the time", so they are counted apart:
///
/// - *whiffed* — the crosshair delivered nothing at all. This is the cone
///   being wrong, and it should be nearly zero.
/// - *neighbour* — it delivered a different toy than the one aimed at, because
///   something in the pile sat nearer the centre of the view. Not lost work
///   (the closer still walks away holding a toy) but not what was asked for.
///
/// Both were once justified by "a floor buried in toys", measured when the shop
/// held 4000 of them: neighbour grabs ran at roughly 45% of pickups. The shop
/// ships 240 now and the rate collapsed to a few per run, which makes that
/// justification obsolete rather than merely dated — and left the recorded
/// figures wrong by an order of magnitude with nothing to notice. These gates
/// sit well above the shipped rates and well below the old ones, so density
/// creeping back up shows here instead of in prose nobody re-measures.
#[path = "replay/test_part_1.rs"]
mod test_part_1;
