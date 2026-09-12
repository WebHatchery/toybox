use super::*;

#[test]
fn a_replay_is_reproducible() {
    let data = GameData::load().unwrap();
    assert_eq!(run(&BEGINNER, &data, 300), run(&BEGINNER, &data, 300));
}
#[test]
fn the_closer_clears_the_floor_without_misshelving() {
    let data = GameData::load().unwrap();
    let budget = data.config.toy_count * 4;
    let report = run(&BEGINNER, &data, budget);

    assert!(report.actions < budget, "the closer never ran out of work");
    assert_eq!(
        report.mistakes, 0,
        "the closer only ever targets a toy's own display, so any mistake is \
         a bug in placement or matching"
    );
    assert!(report.walked_metres > 0.0);

    // What is left on the floor should be the repair parts this strategy
    // deliberately defers, not whole toys it could not find a home for.
    // Saturating because the deferred set can outnumber what is actually loose:
    // a toy the crosshair handed over as a neighbour gets shelved even though
    // its index was written off earlier.
    let whole_left = report.still_loose.saturating_sub(report.deferred_parts);
    assert!(
        whole_left * 20 < data.config.toy_count,
        "{whole_left} whole toys never reached a shelf out of {}: the closer is \
         running out of somewhere to put them",
        data.config.toy_count
    );
}
#[test]
fn render_settings_cannot_move_the_score() {
    let baseline = GameData::load().unwrap();
    let mut altered = baseline.clone();
    // Deliberately below `interaction_radius` (1.35). A render value larger
    // than the reach cannot change a targeting decision even if gameplay does
    // read it, so a gentler setting here makes the whole test vacuous.
    altered.config.toy_render_distance = 0.4;
    altered.config.toy_lod_distance = 0.25;
    altered.config.toy_pose_distance = 0.25;
    altered.config.toy_view_cull_min_dot = 0.98;
    altered.config.toy_always_draw_radius = 0.0;

    // Crosshair targeting, which the scripted runs below deliberately bypass.
    // `interaction_preview` is the public read of `targeted_loose_toy_index`,
    // the one place a render distance could plausibly be mistaken for a
    // gameplay reach.
    let mut session = GameSession::new_with_seed(&baseline, CLOSING_SHIFT_SEED);
    let toy = session
        .toys
        .iter()
        .position(|toy| !toy.is_held && toy.placed_display_id.is_none())
        .unwrap();
    let toy_position = session.toys[toy].position.to_vec2();
    session.player.position = WorldPoint::from_vec2(toy_position - vec2(0.5, 0.0));
    session.player.yaw = 0.0;
    session.player.pitch = -0.42;
    assert!(
        matches!(
            session.interaction_preview(&baseline),
            InteractionPreview::Pickup { .. }
        ),
        "the targeting probe must actually be looking at a toy, or it proves nothing"
    );
    assert!(
        matches!(
            session.interaction_preview(&altered),
            InteractionPreview::Pickup { .. }
        ),
        "render distance changed what the crosshair can pick up"
    );

    assert_eq!(
        run(&BEGINNER, &baseline, 250),
        run(&BEGINNER, &altered, 250),
        "render distance / LOD / culling changed the sorting run"
    );
    assert_eq!(
        run(&RESTORER, &baseline, 150),
        run(&RESTORER, &altered, 150),
        "render distance / LOD / culling changed the restoration run"
    );
}
#[test]
fn the_restoration_errand_is_winnable() {
    let data = GameData::load().unwrap();

    for scenario in [RESTORER, RESTORER_EQUIPPED] {
        let report = run(&scenario, &data, 200);
        println!("{}", report.line(scenario.name));
        assert!(
            report.repaired > 0,
            "{} completed no repairs: the hunt-and-rejoin loop is unplayable,              not merely slow",
            scenario.name
        );
        assert!(
            report.shelved >= report.repaired,
            "every repaired toy should reach its shelf"
        );
    }
}
#[test]
fn the_deadline_is_reachable_by_a_closer_who_buys_tools() {
    let data = GameData::load().unwrap();
    let budget = data.config.toy_count * 4;
    let limit = data.config.shift_seconds / 60.0;

    let reports: Vec<(&str, RunReport)> = [BEGINNER, EARNER, FULLY_EQUIPPED]
        .iter()
        .map(|scenario| (scenario.name, run(scenario, &data, budget)))
        .collect();

    for (name, report) in &reports {
        println!(
            "{}   [deadline {limit:.0} min, {:+.1}]",
            report.line(name),
            limit - report.minutes
        );
    }

    let earner = reports[1].1;
    // The shipped counts pin the user-facing promise alongside the phase so a
    // future config change cannot turn "finished" into a partial report without
    // making this test explain the new shop.
    let expected_repairs = (data.config.toy_count as f32 * data.config.broken_fraction) as usize;
    assert!(
        earner.finished,
        "the earned-tool run never restored the store"
    );
    assert_eq!(
        earner.shelved, data.config.toy_count,
        "not every toy reached a display"
    );
    assert_eq!(
        earner.repaired, expected_repairs,
        "not every broken toy was repaired"
    );
    assert_eq!(
        earner.still_loose, 0,
        "the finished run left usable toys loose"
    );
    assert_eq!(
        earner.tools_owned,
        data.upgrades.len(),
        "the complete run did not earn and buy the full tool ladder"
    );
    assert!(
        earner.minutes < limit,
        "a closer that buys tools as it earns them cannot clear the shop before \
         opening: {:.1} min against a {limit:.0} min shift. Either shift_seconds \
         is too tight or the tools are too weak.",
        earner.minutes
    );
    assert!(
        earner.minutes < reports[0].1.minutes,
        "buying tools did not beat staying bare-handed"
    );
    let headroom = (limit - earner.minutes) / limit;
    assert!(
        headroom >= 0.15,
        "the deterministic closer keeps only {:.1}% deadline headroom; it walks \
         direct lines and cannot justify calling this comfortable",
        headroom * 100.0
    );
}
#[test]
fn a_display_stays_fillable_to_its_back_row() {
    let data = GameData::load().unwrap();

    for scenario in [BEGINNER, FULLY_EQUIPPED] {
        let report = run(&scenario, &data, data.config.toy_count * 4);
        println!("{}", report.line(scenario.name));
        assert!(
            report.shelved > 0,
            "{} shelved nothing at all",
            scenario.name
        );
        assert!(
            report.place_whiffs * 4 < report.shelved,
            "{} was turned away from a shelf {} times against {} toys shelved. \
             Slots are five to a row, so this is what a display too deep for the \
             crosshair looks like: the back rows are shadowed by the front ones.",
            scenario.name,
            report.place_whiffs,
            report.shelved
        );
    }
}
#[test]
fn a_wrong_shelf_costs_about_one_toys_worth_of_time() {
    let data = GameData::load().unwrap();
    let report = run(&BEGINNER, &data, data.config.toy_count * 4);

    let seconds_per_toy = report.minutes * 60.0 / report.shelved as f32;
    let penalty = data.config.mistake_penalty_seconds;
    let ratio = penalty / seconds_per_toy;

    println!(
        "{:.1}s per toy, {penalty:.1}s penalty, ratio {ratio:.2}",
        seconds_per_toy
    );
    assert!(
        (0.5..=2.0).contains(&ratio),
        "a wrong shelf costs {ratio:.2} toys' worth of time ({penalty:.1}s against \
         {seconds_per_toy:.1}s per toy). Below 0.5 the penalty is not felt; above \
         2.0 a single slip outweighs the work it interrupted."
    );
}
#[test]
fn tools_pay_for_themselves_over_the_same_work() {
    let data = GameData::load().unwrap();
    let budget = 400;

    let reports: Vec<(&str, RunReport)> = [BEGINNER, MID_UPGRADE, FULLY_EQUIPPED]
        .iter()
        .map(|scenario| (scenario.name, run(scenario, &data, budget)))
        .collect();

    for (name, report) in &reports {
        println!("{}", report.line(name));
    }

    for (name, report) in &reports {
        assert_eq!(
            report.mistakes, 0,
            "{name} mis-shelved something: the closer only ever shelves a toy on \
             its own display, so a mistake here is a bug in placement or matching"
        );
    }

    let beginner = reports[0].1;
    let equipped = reports[2].1;
    assert!(
        equipped.minutes < beginner.minutes,
        "a full toolset should finish the same {budget} actions faster: \
         {:.1} min equipped vs {:.1} min bare-handed",
        equipped.minutes,
        beginner.minutes
    );
}
#[test]
#[ignore = "full-shop replay: minutes to run, not part of the normal suite"]
fn a_full_shift_completes_the_shop() {
    let data = GameData::load().unwrap();
    for scenario in [BEGINNER, FULLY_EQUIPPED, EARNER] {
        let report = run(&scenario, &data, data.config.toy_count * 4);
        println!("{}", report.line(scenario.name));
        if scenario.strategy == Strategy::Earner {
            assert!(report.finished);
            assert_eq!(report.shelved, data.config.toy_count);
            assert_eq!(report.still_loose, 0);
        }
    }
}
#[test]
#[ignore = "shop-scaling sweep: for retuning toy_count, not part of the normal suite"]
fn shop_scale_sets_the_length_of_a_shift() {
    let base = GameData::load().unwrap();

    for per_display in [8usize, 12, 20, 40, 100, 200] {
        let mut data = base.clone();
        for display in &mut data.displays {
            display.capacity = per_display;
        }
        data.config.toy_count = per_display * data.displays.len();

        for scenario in [BEGINNER, FULLY_EQUIPPED] {
            let report = run(&scenario, &data, data.config.toy_count * 4);
            println!(
                "cap {per_display:>3} / {:>4} toys  {}",
                data.config.toy_count,
                report.line(scenario.name)
            );
        }
    }
}
#[test]
fn the_crosshair_hands_over_the_toy_you_aimed_at() {
    let data = GameData::load().unwrap();

    for scenario in [BEGINNER, FULLY_EQUIPPED] {
        let report = run(&scenario, &data, data.config.toy_count * 4);
        println!("{}", report.line(scenario.name));
        assert!(report.shelved > 0, "{} shelved nothing", scenario.name);

        assert!(
            report.whiffs * 20 < report.shelved,
            "{}: the crosshair delivered nothing {} times against {} toys \
             shelved. Bare-handed this should be zero — a nonzero rate means \
             the pick-up cone is not finding what is in front of it.",
            scenario.name,
            report.whiffs,
            report.shelved
        );
        assert!(
            report.grabbed_neighbour * 5 < report.shelved,
            "{}: the crosshair handed over a different toy than the one aimed \
             at {} times against {} toys shelved. At 4000 toys this ran near \
             45% and was called pile texture; at the shipped shop size it is a \
             few per run, so this rate climbing back means the floor got denser \
             or the targeting got worse.",
            scenario.name,
            report.grabbed_neighbour,
            report.shelved
        );
    }
}
