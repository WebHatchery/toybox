use super::*;
use std::collections::HashSet;
use toybox_after_hours::toys::{toy_color, toy_profile};

#[test]
fn new_session_generates_requested_toys() {
    let data = GameData::load().unwrap();
    let session = GameSession::new(&data);
    let final_toy_count = session
        .toys
        .iter()
        .filter(|toy| {
            !matches!(
                toy.repair_state,
                RepairState::BrokenPart {
                    part: RepairPartKind::Head,
                    ..
                } | RepairState::ConsumedPart { .. }
            )
        })
        .count();

    let head_count = session
        .toys
        .iter()
        .filter(|toy| toy.repair_part_kind() == Some(RepairPartKind::Head))
        .count();

    assert_eq!(final_toy_count, data.config.toy_count);
    assert_eq!(session.toys.len(), data.config.toy_count + head_count);
    assert_eq!(session.displays.len(), data.displays.len());
    assert_eq!(session.completed_display_count(), 0);
}

#[test]
fn new_session_breaks_a_deterministic_fraction_into_cross_zone_pairs() {
    let data = GameData::load().unwrap();
    let session = GameSession::new(&data);

    let mut bodies: std::collections::HashMap<&str, &ToyState> = std::collections::HashMap::new();
    let mut heads: std::collections::HashMap<&str, &ToyState> = std::collections::HashMap::new();
    for toy in &session.toys {
        if let RepairState::BrokenPart {
            repair_id, part, ..
        } = &toy.repair_state
        {
            match part {
                RepairPartKind::Body => bodies.insert(repair_id.as_str(), toy),
                RepairPartKind::Head => heads.insert(repair_id.as_str(), toy),
            };
            assert!(!data
                .displays
                .iter()
                .any(|display| toy_matches_display(toy, display)));
        }
    }

    // Roughly broken_fraction of the store, every body with exactly one head.
    let expected = (data.config.toy_count as f32 * data.config.broken_fraction) as usize;
    assert_eq!(bodies.len(), heads.len());
    assert!(
        bodies.len() * 5 >= expected * 4 && bodies.len() * 5 <= expected * 6,
        "broken count {} far from expected {expected}",
        bodies.len()
    );

    for (repair_id, body) in &bodies {
        let head = heads
            .get(repair_id)
            .unwrap_or_else(|| panic!("no head for {repair_id}"));
        let body_zone = data.layout.zone_name_at(body.position.x, body.position.y);
        let head_zone = data.layout.zone_name_at(head.position.x, head.position.y);
        assert_ne!(
            body_zone, head_zone,
            "parts of {repair_id} landed in the same zone"
        );
    }
}

#[test]
fn new_session_generates_identity_variety_per_display() {
    let data = GameData::load().unwrap();
    let session = GameSession::new(&data);

    for display in &data.displays {
        let labels: HashSet<&str> = session
            .toys
            .iter()
            .filter(|toy| toy_matches_display(toy, display))
            .map(|toy| toy_profile(toy.category, toy.slot_number).label)
            .collect();

        assert!(labels.len() >= 5, "{} identities: {:?}", display.id, labels);
    }
}

#[test]
fn new_session_gives_loose_toys_varied_spawn_poses() {
    let data = GameData::load().unwrap();
    let session = GameSession::new(&data);
    let tumbled_count = session
        .toys
        .iter()
        .filter(|toy| toy.spawn_pose.is_tumbled())
        .count();
    let upside_down_count = session
        .toys
        .iter()
        .filter(|toy| (toy.spawn_pose.roll.abs() - std::f32::consts::PI).abs() < 0.01)
        .count();
    let side_count = session
        .toys
        .iter()
        .filter(|toy| {
            (toy.spawn_pose.roll.abs() - std::f32::consts::FRAC_PI_2).abs() < 0.01
                || (toy.spawn_pose.pitch.abs() - std::f32::consts::FRAC_PI_2).abs() < 0.01
        })
        .count();

    assert!(tumbled_count > session.toys.len() / 2);
    assert!(upside_down_count > 0);
    assert!(side_count > 0);
}

#[test]
fn toy_colors_are_not_locked_to_display_category() {
    let data = GameData::load().unwrap();
    let session = GameSession::new(&data);

    for display in &data.displays {
        let colors: HashSet<(u8, u8, u8)> = session
            .toys
            .iter()
            .filter(|toy| toy_matches_display(toy, display))
            .map(|toy| {
                let color = toy_color(toy);
                (
                    (color.r * 255.0) as u8,
                    (color.g * 255.0) as u8,
                    (color.b * 255.0) as u8,
                )
            })
            .collect();

        assert!(
            colors.len() >= 5,
            "{} used only {:?} colors",
            display.id,
            colors
        );
    }
}

#[test]
fn new_session_starts_with_all_toys_on_the_floor() {
    let data = GameData::load().unwrap();
    let session = GameSession::new(&data);

    for toy in &session.toys {
        assert!(!toy.is_held);
        assert!(toy.placed_display_id.is_none());
        assert!(toy.placed_slot_index.is_none());
        for display in &data.displays {
            let on_display = toy.position.x >= display.x
                && toy.position.x <= display.x + display.w
                && toy.position.y >= display.y
                && toy.position.y <= display.y + display.h;
            assert!(
                !on_display,
                "{} started on display {} at {}, {}",
                toy.id, display.id, toy.position.x, toy.position.y
            );
        }
    }
}

/// The three render bands are all reachable.
///
/// `scene3d::draw_loose_toys` picks a stand-in cube beyond `toy_lod_distance`,
/// a full upright model beyond `toy_pose_distance`, and a full posed model
/// closer than that. The middle band is written as an `else if`, so setting the
/// two distances *equal* deletes it silently — which is how the game shipped:
/// both were 5.0, the "full detail, but upright" arm could never run, and every
/// toy past five metres was a coloured box. That LOD was tuned when the shop
/// held 4000 toys; it holds 240 now.
///
/// Measured on the shipped shop with the vsync-uncapped bench, average fps:
/// lod 5 -> ~320, 7 -> 272, 8 -> 245, 9 -> 221, 12 (no cubes at all) -> 113.
/// 8.0 buys nearly all of the visual gain for about 1.3x the frame cost.
#[test]
fn every_toy_render_band_is_reachable() {
    let data = GameData::load().unwrap();
    let config = &data.config;

    assert!(
        config.toy_lod_distance > config.toy_pose_distance,
        "lod {} is not beyond pose {}, so the upright band is dead code",
        config.toy_lod_distance,
        config.toy_pose_distance
    );
    assert!(
        config.toy_render_distance > config.toy_lod_distance,
        "render {} is not beyond lod {}, so no toy is ever a stand-in",
        config.toy_render_distance,
        config.toy_lod_distance
    );
    // Toys inside this radius always draw, so it must not reach past the point
    // where they stop being drawn at all.
    assert!(config.toy_always_draw_radius <= config.toy_render_distance);
}

#[test]
fn closing_shift_keeps_the_original_fixed_layout() {
    let data = GameData::load().unwrap();
    let implicit = GameSession::new(&data);
    let explicit = GameSession::new_with_seed(&data, CLOSING_SHIFT_SEED);

    assert_eq!(implicit.shift_seed, CLOSING_SHIFT_SEED);
    assert_eq!(
        serde_json::to_vec(&implicit.to_save(&data.config.version)).unwrap(),
        serde_json::to_vec(&explicit.to_save(&data.config.version)).unwrap()
    );
}

#[test]
fn equal_seeds_repeat_and_distinct_seeds_reshape_the_floor() {
    let data = GameData::load().unwrap();
    let seed = 0x2E1A_8ED5_CAFE_0240;
    let first = GameSession::new_with_seed(&data, seed);
    let repeated = GameSession::new_with_seed(&data, seed);
    let different = GameSession::new_with_seed(&data, seed ^ 0x55AA_F00D_1234_5678);

    assert_eq!(
        serde_json::to_vec(&first.to_save(&data.config.version)).unwrap(),
        serde_json::to_vec(&repeated.to_save(&data.config.version)).unwrap(),
        "an equal seed must reproduce the whole serialized session"
    );

    let moved_bodies = first
        .toys
        .iter()
        .filter(|toy| !toy.id.ends_with("_head"))
        .filter(|toy| {
            let other = different
                .toys
                .iter()
                .find(|candidate| candidate.id == toy.id)
                .expect("every seed keeps the same 240 body identities");
            toy.position.to_vec2().distance(other.position.to_vec2()) > 0.1
        })
        .count();
    assert!(
        moved_bodies >= data.config.toy_count * 9 / 10,
        "only {moved_bodies} body toys materially moved"
    );
}

#[test]
fn varied_seeds_preserve_safe_cross_zone_repair_layouts() {
    let data = GameData::load().unwrap();
    for seed in [
        0x0000_0000_0000_0001,
        0x2E1A_8ED5_CAFE_0240,
        0xFFFF_0000_A5A5_5A5A,
    ] {
        let session = GameSession::new_with_seed(&data, seed);
        let bodies: Vec<&ToyState> = session
            .toys
            .iter()
            .filter(|toy| !toy.id.ends_with("_head"))
            .collect();
        assert_eq!(bodies.len(), data.config.toy_count);

        for toy in &session.toys {
            assert!(toy.position.x >= 0.8 && toy.position.x <= data.config.room_width - 0.8);
            assert!(toy.position.y >= 0.8 && toy.position.y <= data.config.room_height - 0.8);
            let point = toy.position.to_vec2();
            let inside_display = data.displays.iter().any(|fixture| {
                Rect::new(fixture.x, fixture.y, fixture.w, fixture.h).contains(point)
            });
            let inside_shelf_or_counter = data
                .layout
                .shelving
                .iter()
                .chain(data.layout.counters.iter())
                .any(|fixture| {
                    Rect::new(fixture.x, fixture.y, fixture.w, fixture.h).contains(point)
                });
            let inside_bench = data.layout.benches.iter().any(|bench| {
                Rect::new(
                    bench.x - bench.w * 0.5,
                    bench.y - bench.h * 0.5,
                    bench.w,
                    bench.h,
                )
                .contains(point)
            });
            assert!(
                !inside_display && !inside_shelf_or_counter && !inside_bench,
                "seed {seed:016X} put {} inside a fixture",
                toy.id
            );
        }

        let broken_bodies: Vec<&ToyState> = session
            .toys
            .iter()
            .filter(|toy| toy.repair_part_kind() == Some(RepairPartKind::Body))
            .collect();
        let heads: Vec<&ToyState> = session
            .toys
            .iter()
            .filter(|toy| toy.repair_part_kind() == Some(RepairPartKind::Head))
            .collect();
        assert_eq!(broken_bodies.len(), heads.len());
        for body in broken_bodies {
            let RepairState::BrokenPart { repair_id, .. } = &body.repair_state else {
                unreachable!();
            };
            let head = heads
                .iter()
                .find(|head| {
                    matches!(
                        &head.repair_state,
                        RepairState::BrokenPart { repair_id: candidate, .. } if candidate == repair_id
                    )
                })
                .expect("every broken body keeps one head");
            assert_ne!(
                data.layout.zone_name_at(body.position.x, body.position.y),
                data.layout.zone_name_at(head.position.x, head.position.y),
                "seed {seed:016X} left {repair_id} in one zone"
            );
        }
    }
}
