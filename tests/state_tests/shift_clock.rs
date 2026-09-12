//! The shift deadline: when the doors open, and when they never do.

use super::*;

/// Run the clock forward in whole seconds, reporting whether the shift ended
/// during the sweep. A single huge `dt` would work too, but stepping is what a
/// frame loop actually does and it catches an end condition that only fires on
/// an exact boundary.
fn run_clock_for(session: &mut GameSession, data: &GameData, seconds: usize) -> bool {
    (0..seconds).any(|_| session.update_timer(1.0, data))
}

#[test]
fn a_timed_shift_ends_when_the_doors_open() {
    let data = GameData::load().unwrap();
    let mut session = GameSession::new(&data);
    assert_eq!(session.shift_mode, ShiftMode::Timed, "timed is the default");

    let full_shift = data.config.shift_seconds as usize;
    assert!(
        !run_clock_for(&mut session, &data, full_shift - 1),
        "the shift ended early"
    );
    assert_eq!(session.phase, GamePhase::Playing);

    assert!(
        run_clock_for(&mut session, &data, 2),
        "the deadline never fired"
    );
    assert_eq!(session.phase, GamePhase::TimeUp);
    assert!(session.phase.is_over());
}

/// The end has to be announced exactly once. `update_timer` runs every frame,
/// and a notification that re-fires at 60Hz would bury the screen.
#[test]
fn the_doors_open_only_once() {
    let data = GameData::load().unwrap();
    let mut session = GameSession::new(&data);

    let past_the_end = data.config.shift_seconds as usize + 10;
    let announcements = (0..past_the_end)
        .filter(|_| session.update_timer(1.0, &data))
        .count();

    assert_eq!(announcements, 1);
}

#[test]
fn a_relaxed_run_never_runs_out_of_time() {
    let data = GameData::load().unwrap();
    let mut session = GameSession::new(&data);
    session.shift_mode = ShiftMode::Relaxed;

    let three_shifts = data.config.shift_seconds as usize * 3;
    assert!(
        !run_clock_for(&mut session, &data, three_shifts),
        "a relaxed run ended itself"
    );
    assert_eq!(session.phase, GamePhase::Playing);
    // The clock still runs, so a relaxed run can be compared with a timed one.
    assert!(session.player.elapsed_seconds >= three_shifts as f32);
}

/// A time-up ending must freeze the shop. The phase guards already do this for
/// movement and interaction, but nothing else pins it, and a deadline that
/// leaves the player sorting is not a deadline.
#[test]
fn a_finished_shift_stops_accepting_work() {
    let data = GameData::load().unwrap();
    let mut session = GameSession::new(&data);
    session.phase = GamePhase::TimeUp;

    let before = session.player.position;
    session.move_player(vec2(1.0, 1.0), &data, 1.0);
    assert_eq!(session.player.position.x, before.x);
    assert_eq!(session.player.position.y, before.y);

    assert!(matches!(
        session.interact(&data),
        InteractionResult::NothingNearby
    ));
    assert!(session.player.carried_toy_ids.is_empty());

    // The clock stops too, so a score screen does not tick upward behind it.
    let elapsed = session.player.elapsed_seconds;
    session.update_timer(5.0, &data);
    assert_eq!(session.player.elapsed_seconds, elapsed);
}

/// The mistake penalty pushes `elapsed_seconds` forward, which in a timed shift
/// is the whole point — a sloppy closer runs out of night sooner. Shelving a toy
/// on the wrong display near the end must be able to cost the shift, otherwise
/// the penalty is only a bigger number on the score screen.
#[test]
fn a_mistake_near_closing_can_cost_the_shift() {
    let data = GameData::load().unwrap();
    let mut session = GameSession::new(&data);
    let display = &data.displays[0];
    let toy_index = session
        .toys
        .iter()
        .position(|toy| !toy_matches_display(toy, display) && !toy.is_repair_part())
        .unwrap();

    // Half a penalty short of opening: the clock alone would not end the run.
    session.player.elapsed_seconds =
        data.config.shift_seconds - data.config.mistake_penalty_seconds * 0.5;

    session.pick_up_toy(toy_index, &data);
    let result = session.place_active_toy(0, 0, &data);
    assert!(matches!(
        result,
        InteractionResult::Placed {
            was_wrong: true,
            ..
        }
    ));
    assert!(session.player.elapsed_seconds > data.config.shift_seconds);

    // The penalty does not end the shift by itself — the next tick notices.
    assert_eq!(session.phase, GamePhase::Playing);
    assert!(session.update_timer(0.016, &data));
    assert_eq!(session.phase, GamePhase::TimeUp);
}

/// The HUD clock says which way it runs, and says it correctly.
///
/// The two modes draw an identical status panel — same stopwatch, same colour,
/// same position — so an unlabelled "07:11" meant seven minutes gone in a
/// relaxed run and seven minutes left in a timed one. A caption that disagreed
/// with the direction would be worse than none, so it is derived from
/// `shows_countdown` rather than written out beside it.
#[test]
fn the_clock_caption_says_which_way_the_clock_runs() {
    for mode in [ShiftMode::Timed, ShiftMode::Relaxed] {
        let caption = mode.clock_caption();
        assert_eq!(
            caption == "LEFT",
            mode.shows_countdown(),
            "{mode:?} counts {} but its clock is captioned {caption:?}",
            if mode.shows_countdown() { "down" } else { "up" },
        );
    }

    // Distinct, or the caption tells the player nothing they did not have.
    assert_ne!(
        ShiftMode::Timed.clock_caption(),
        ShiftMode::Relaxed.clock_caption()
    );
}
