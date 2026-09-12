use toybox_after_hours::data::*;

#[test]
fn embedded_data_loads() {
    let data = GameData::load().unwrap();
    let total_capacity: usize = data.displays.iter().map(|display| display.capacity).sum();

    assert_eq!(data.config.game_name, "toybox_after_hours");
    assert_eq!(data.displays.len(), 20);
    assert_eq!(total_capacity, data.config.toy_count);
}

/// A tool nobody can reach is dead content, and a duplicate id is bought
/// once but owned twice — `has_upgrade` matches by id.
#[test]
fn every_tool_is_reachable_and_uniquely_named() {
    /// Roughly two wrapped lines at the shop's 14px description size.
    const DESCRIPTION_BUDGET: usize = 125;

    let data = GameData::load().unwrap();
    assert!(!data.upgrades.is_empty());

    let mut seen = std::collections::HashSet::new();
    let mut total_cost = 0;
    for upgrade in &data.upgrades {
        assert!(
            seen.insert(upgrade.id.as_str()),
            "duplicate id {}",
            upgrade.id
        );
        assert!(
            upgrade.unlock_completed_displays <= data.displays.len(),
            "{} unlocks at {} completed displays but only {} exist",
            upgrade.id,
            upgrade.unlock_completed_displays,
            data.displays.len()
        );
        // The shop row gives a description two wrapped lines. Rendered width
        // cannot be measured without a GL context, so this guards the
        // proxy — but it is the measure that broke: four of five tools once
        // ended mid-word, and the Sorting Trolley lost the half explaining
        // how to load it, which the shop is the only place to learn.
        // Re-capture `tool_shop` after changing any of these.
        assert!(
            upgrade.description.len() <= DESCRIPTION_BUDGET,
            "{} has a {}-character description; the shop row fits about {}",
            upgrade.id,
            upgrade.description.len(),
            DESCRIPTION_BUDGET
        );
        total_cost += upgrade.cost;
    }

    // Credits come one per completed display, so the whole shop has to be
    // affordable inside a single run or the last tools never sell.
    assert!(
        total_cost <= data.displays.len(),
        "tools cost {total_cost} credits but a run yields at most {}",
        data.displays.len()
    );
}

#[test]
fn touch_first_tutorial_copy_has_every_step_and_action_key() {
    let data = GameData::load().unwrap();

    assert_eq!(data.ui_copy.tutorial.len(), 6);
    assert!(data
        .ui_copy
        .tutorial
        .iter()
        .all(|step| !step.title.is_empty() && !step.body.is_empty() && !step.keys.is_empty()));
}

#[test]
fn preference_ranges_are_positive_and_ordered() {
    let config = &GameData::load().unwrap().config;

    assert!(config.fov_min_degrees < config.fov_max_degrees);
    assert!(config.sensitivity_min < config.sensitivity_max);
    assert!(config.ui_scale_min < config.ui_scale_max);
    assert!(config.fov_step_degrees > 0.0);
    assert!(config.sensitivity_step > 0.0);
    assert!(config.ui_scale_step > 0.0);
}

#[test]
fn layout_references_are_unique_and_inside_the_room() {
    let data = GameData::load().unwrap();
    let bench_ids: std::collections::HashSet<_> = data
        .layout
        .benches
        .iter()
        .map(|bench| bench.id.as_str())
        .collect();
    let zone_names: std::collections::HashSet<_> = data
        .layout
        .zones
        .iter()
        .map(|zone| zone.name.as_str())
        .collect();

    assert_eq!(bench_ids.len(), data.layout.benches.len());
    assert_eq!(zone_names.len(), data.layout.zones.len());
    assert!(data.layout.benches.iter().all(|bench| {
        bench.x >= 0.0
            && bench.y >= 0.0
            && bench.x + bench.w <= data.config.room_width
            && bench.y + bench.h <= data.config.room_height
    }));
}

#[test]
fn game_page_declares_touch_first_recovery_controls() {
    let page: serde_json::Value =
        serde_json::from_str(include_str!("../game_page.json")).expect("valid game page JSON");

    assert_eq!(page["input"]["touch_first"], true);
    assert_eq!(page["input"]["pointer_lock_required"], false);
    let controls = page["controls"].as_array().expect("controls array");
    assert!(controls.iter().any(|control| {
        control["key"] == "Tap controls"
            && control["desc"]
                .as_str()
                .is_some_and(|text| text.contains("recovery"))
    }));
    assert!(controls
        .iter()
        .any(|control| { control["key"] == "NEW SHIFT / REPLAY LAYOUT buttons" }));
}
