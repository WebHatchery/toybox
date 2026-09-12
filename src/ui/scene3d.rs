//! Macroquad 3D renderer for the tiny toy shop.

use crate::data::DisplayDef;
use crate::state::{display_slot_position, ToyState, WorldPoint};
use crate::toys::{brighten, draw_loose_toy_3d, draw_toy_3d, draw_toy_lod_3d, toy_color};
use crate::ui::benches::draw_repair_benches;
use crate::ui::environment::draw_shop_environment;
use crate::ui::fixtures::{
    draw_aisle_shelving, draw_checkout_counters, draw_displays, placed_height_for_slot,
};
use crate::ui::scanner::{draw_counterpart_beacon, draw_stockroom_spotlight};
use crate::ui::signs::{draw_wall_posters, draw_zone_sign};
use crate::ui::UiContext;
use macroquad::prelude::*;

/// Per-frame render statistics surfaced on the debug overlay.
pub struct SceneStats {
    pub drawn_toys: usize,
}

pub fn draw_shop_scene(ctx: &UiContext<'_>) -> SceneStats {
    let eye = player_eye(ctx.session.player.position);
    let (front, _, up) = camera_basis(ctx.session.player.yaw, ctx.session.player.pitch);
    let camera = Camera3D {
        position: eye,
        target: eye + front,
        up,
        fovy: ctx.fov_degrees.to_radians(),
        projection: Projection::Perspective,
        aspect: Some(screen_width() / screen_height().max(1.0)),
        ..Default::default()
    };

    set_camera(&camera);
    draw_shop_environment(ctx.data);
    draw_displays(ctx);
    draw_aisle_shelving(ctx);
    draw_checkout_counters(ctx);
    for zone in &ctx.data.layout.zones {
        draw_zone_sign(zone);
    }
    draw_wall_posters(ctx.data);
    draw_repair_benches(ctx);
    let drawn_toys = draw_loose_toys(ctx) + draw_placed_toys(ctx);
    draw_counterpart_beacon(ctx);
    draw_stockroom_spotlight(ctx);
    draw_placement_preview(ctx);
    draw_player_presence(ctx);
    SceneStats { drawn_toys }
}

fn draw_loose_toys(ctx: &UiContext<'_>) -> usize {
    let config = &ctx.data.config;
    let player = ctx.session.player.position.to_vec2();
    let forward = vec2(ctx.session.player.yaw.cos(), ctx.session.player.yaw.sin());
    let mut drawn = 0;
    for index in ctx
        .session
        .spatial()
        .indices_near(player, config.toy_render_distance)
    {
        let toy = &ctx.session.toys[index];
        if toy.is_held || toy.placed_display_id.is_some() || toy.is_consumed_repair_part() {
            continue;
        }
        let to_toy = toy.position.to_vec2() - player;
        let distance = to_toy.length();
        if !loose_toy_is_visible(to_toy, distance, forward, config) {
            continue;
        }
        let layer = (index % 7) as f32;
        let base_height = if toy.bench_slot_index.is_some() {
            0.88
        } else {
            0.20 + layer * 0.020
        };
        let scale = toy_draw_scale(index, toy.bench_slot_index.is_some());
        if distance > config.toy_lod_distance {
            // Stand-ins stay grounded: floor_lift only compensates for the
            // tumbled pose, and applying it to upright draws leaves distant
            // toys hovering and dropping as the player approaches.
            draw_toy_lod_3d(
                toy,
                world_point(toy.position, base_height),
                toy_color(toy),
                scale,
            );
        } else if distance > config.toy_pose_distance {
            // Full detail, but upright: the spawn-pose model matrix forces a
            // render-batch flush per toy, so it is reserved for close range.
            draw_toy_3d(
                toy,
                world_point(toy.position, base_height),
                toy_color(toy),
                scale,
            );
        } else {
            let lift = if toy.bench_slot_index.is_some() {
                0.0
            } else {
                toy.spawn_pose.floor_lift
            };
            draw_loose_toy_3d(
                toy,
                world_point(toy.position, base_height + lift),
                toy_color(toy),
                scale,
            );
        }
        drawn += 1;
    }
    drawn
}

fn loose_toy_is_visible(
    to_toy: Vec2,
    distance: f32,
    forward: Vec2,
    config: &crate::data::GameConfig,
) -> bool {
    if distance > config.toy_render_distance {
        return false;
    }
    // Nearby toys always draw so turning in place never pops them in late.
    if distance <= config.toy_always_draw_radius {
        return true;
    }
    to_toy.dot(forward) / distance >= config.toy_view_cull_min_dot
}

fn draw_placed_toys(ctx: &UiContext<'_>) -> usize {
    let player = ctx.session.player.position.to_vec2();
    let lod_distance = ctx.data.config.toy_lod_distance;
    let mut drawn = 0;
    for display in &ctx.data.displays {
        for toy in ctx.session.placed_toys_for_display(&display.id) {
            let height = placed_height(display, toy);
            let center = world_point(toy.position, height);
            if toy.position.to_vec2().distance(player) > lod_distance {
                draw_toy_lod_3d(toy, center, toy_color(toy), 0.92);
            } else {
                draw_toy_3d(toy, center, toy_color(toy), 0.92);
            }
            if toy.wrong_marker_seconds > 0.0 {
                draw_wrong_placement_marker(center);
            }
            drawn += 1;
        }
    }
    drawn
}

fn draw_wrong_placement_marker(center: Vec3) {
    let color = Color::new(0.98, 0.18, 0.14, 0.92);
    draw_cube_wires(center + vec3(0.0, 0.04, 0.0), vec3(0.72, 0.72, 0.72), color);
    draw_line_3d(
        center + vec3(-0.42, 0.48, -0.42),
        center + vec3(0.42, 0.48, 0.42),
        color,
    );
    draw_line_3d(
        center + vec3(-0.42, 0.48, 0.42),
        center + vec3(0.42, 0.48, -0.42),
        color,
    );
}

fn draw_placement_preview(ctx: &UiContext<'_>) {
    let Some(toy) = ctx.session.active_toy() else {
        return;
    };
    if toy.is_repair_part() {
        return;
    }
    let Some(target) = ctx.session.targeted_empty_display_slot(ctx.data) else {
        return;
    };
    let display = &ctx.data.displays[target.display_index];

    let accent = Color::new(0.92, 0.72, 0.34, 1.0);
    let slot = display_slot_position(display, target.slot_index, ctx.data.config.room_width);
    let height = placed_height_for_slot(display, target.slot_index + 1) - 0.24;
    draw_target_brackets(world_point(slot, height), accent);
}

fn draw_player_presence(ctx: &UiContext<'_>) {
    let pos = world_point(ctx.session.player.position, 0.0);
    draw_cube_wires(
        pos + vec3(0.0, 0.05, 0.0),
        vec3(
            ctx.data.config.interaction_radius * 2.0,
            0.05,
            ctx.data.config.interaction_radius * 2.0,
        ),
        Color::new(0.95, 0.78, 0.30, 0.25),
    );

    if let Some(toy) = ctx.session.active_toy() {
        let eye = player_eye(ctx.session.player.position);
        let (front, right, up) = camera_basis(ctx.session.player.yaw, ctx.session.player.pitch);
        draw_toy_3d(
            toy,
            eye + front * 0.82 + right * 0.30 - up * 0.22,
            brighten(toy_color(toy), 0.08),
            0.58,
        );
    }
}

fn placed_height(display: &DisplayDef, toy: &ToyState) -> f32 {
    placed_height_for_slot(
        display,
        toy.placed_slot_index
            .map(|slot_index| slot_index + 1)
            .unwrap_or(toy.slot_number),
    )
}

fn draw_target_brackets(center: Vec3, color: Color) {
    let half = 0.25;
    let tick = 0.11;
    let y = center.y.max(0.055);
    let corners = [
        (
            vec3(-half, 0.0, -half),
            vec3(-half + tick, 0.0, -half),
            vec3(-half, 0.0, -half + tick),
        ),
        (
            vec3(half, 0.0, -half),
            vec3(half - tick, 0.0, -half),
            vec3(half, 0.0, -half + tick),
        ),
        (
            vec3(-half, 0.0, half),
            vec3(-half + tick, 0.0, half),
            vec3(-half, 0.0, half - tick),
        ),
        (
            vec3(half, 0.0, half),
            vec3(half - tick, 0.0, half),
            vec3(half, 0.0, half - tick),
        ),
    ];

    for (corner, along_x, along_z) in corners {
        let start = vec3(center.x, y, center.z) + corner;
        draw_line_3d(start, vec3(center.x, y, center.z) + along_x, color);
        draw_line_3d(start, vec3(center.x, y, center.z) + along_z, color);
    }
}

fn world_point(point: WorldPoint, height: f32) -> Vec3 {
    vec3(point.x, height, point.y)
}

fn player_eye(point: WorldPoint) -> Vec3 {
    world_point(point, 1.08)
}

fn camera_basis(yaw: f32, pitch: f32) -> (Vec3, Vec3, Vec3) {
    let world_up = vec3(0.0, 1.0, 0.0);
    let front = vec3(
        yaw.cos() * pitch.cos(),
        pitch.sin(),
        yaw.sin() * pitch.cos(),
    )
    .normalize();
    let right = front.cross(world_up).normalize_or_zero();
    let up = right.cross(front).normalize_or_zero();
    (front, right, up)
}

/// How big to draw a loose toy, and why a benched part is exempt.
///
/// Toys on the floor get a little size variety off their index, so a shop
/// buried in them does not read as a grid of identical objects. A part waiting
/// on the repair bench must not: the two halves of one toy sit side by side
/// there, at the only range in the game where a player studies them closely,
/// and the jitter drew them up to 23% apart. A real pair measured 0.88 against
/// 1.03 — a bear whose head is visibly too big for its own body, moments before
/// you tap ACT to join them.
///
/// Same shape of exemption as `floor_lift` just below: what makes a tumbled
/// floor look alive makes a workbench look wrong.
pub fn toy_draw_scale(index: usize, on_bench: bool) -> f32 {
    if on_bench {
        1.0
    } else {
        0.88 + ((index * 13) % 9) as f32 * 0.025
    }
}
