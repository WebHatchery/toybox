//! Crosshair targeting and spatial queries used by interaction handling.

use super::{display_slot_position, GameSession, ToyState};
use crate::data::{DisplayDef, GameConfig, GameData};
use crate::state::DisplaySlotTarget;
use macroquad::prelude::*;

const SLOT_LOOK_MIN_DOT: f32 = 0.42;
const SLOT_LOOK_MAX_LATERAL: f32 = 1.15;
const LOOSE_TOY_LOOK_RADIUS: f32 = 0.58;
const PLAYER_EYE_HEIGHT: f32 = 1.08;
const SLOT_OCCUPANCY_RADIUS: f32 = 0.25;

impl GameSession {
    /// The best free slot in view, for a player holding something to shelve.
    pub fn targeted_empty_display_slot(&self, data: &GameData) -> Option<DisplaySlotTarget> {
        self.targeted_slot(data, true)
    }

    /// The best slot in view regardless of what is in it. This is the retrieval
    /// target: empty-handed, the player reaching at a shelf means the toy they
    /// can see, so an occupied slot is exactly what they want.
    pub fn targeted_display_slot(&self, data: &GameData) -> Option<DisplaySlotTarget> {
        self.targeted_slot(data, false)
    }

    fn targeted_slot(&self, data: &GameData, free_only: bool) -> Option<DisplaySlotTarget> {
        let player = self.player.position.to_vec2();
        let forward = vec2(self.player.yaw.cos(), self.player.yaw.sin()).normalize_or_zero();
        if forward.length_squared() <= f32::EPSILON {
            return None;
        }

        data.displays
            .iter()
            .enumerate()
            .filter(|(_, display)| self.display_is_near_player(display, &data.config))
            .flat_map(|(display_index, display)| {
                (0..display.capacity).filter_map(move |slot_index| {
                    if free_only
                        && self
                            .toy_index_at_display_slot(display, slot_index, data.config.room_width)
                            .is_some()
                    {
                        return None;
                    }

                    let slot = display_slot_position(display, slot_index, data.config.room_width)
                        .to_vec2();
                    let to_slot = slot - player;
                    let distance = to_slot.length();
                    let direction = if distance > 0.05 {
                        to_slot / distance
                    } else {
                        forward
                    };
                    let dot = direction.dot(forward);
                    if dot < SLOT_LOOK_MIN_DOT {
                        return None;
                    }

                    let lateral = (to_slot.x * forward.y - to_slot.y * forward.x).abs();
                    if lateral > SLOT_LOOK_MAX_LATERAL && distance > 0.55 {
                        return None;
                    }

                    let score = lateral * 3.6 + (1.0 - dot) * 1.25 + distance * 0.04;
                    Some((
                        DisplaySlotTarget {
                            display_index,
                            slot_index,
                        },
                        score,
                    ))
                })
            })
            .min_by(|(_, left), (_, right)| left.total_cmp(right))
            .map(|(target, _)| target)
    }

    pub(super) fn toy_at_display_slot(
        &self,
        display: &DisplayDef,
        slot_index: usize,
        room_width: f32,
    ) -> Option<&ToyState> {
        self.toy_index_at_display_slot(display, slot_index, room_width)
            .map(|toy_index| &self.toys[toy_index])
    }

    pub(super) fn toy_index_at_display_slot(
        &self,
        display: &DisplayDef,
        slot_index: usize,
        room_width: f32,
    ) -> Option<usize> {
        let slot = display_slot_position(display, slot_index, room_width).to_vec2();
        self.spatial
            .indices_near(slot, SLOT_OCCUPANCY_RADIUS)
            .into_iter()
            .find(|&toy_index| {
                let toy = &self.toys[toy_index];
                toy.placed_display_id.as_deref() == Some(display.id.as_str())
                    && toy.placed_slot_index == Some(slot_index)
            })
    }

    /// The loose toy the crosshair would *add* to an armful that still has room.
    pub(super) fn targeted_pickup_addition(&self, data: &GameData) -> Option<usize> {
        if self.player.carried_toy_ids.len() >= self.carry_limit(data) {
            return None;
        }
        self.targeted_loose_toy_index(data)
    }

    pub(super) fn targeted_loose_toy_index(&self, data: &GameData) -> Option<usize> {
        let reach = self.interaction_reach(data);
        let player_2d = self.player.position.to_vec2();
        let eye = vec3(player_2d.x, PLAYER_EYE_HEIGHT, player_2d.y);
        let forward = self.look_forward_3d();
        if forward.length_squared() <= f32::EPSILON {
            return None;
        }

        self.spatial
            .indices_near(player_2d, reach)
            .into_iter()
            .map(|index| (index, &self.toys[index]))
            .filter(|(_, toy)| {
                !toy.is_held && toy.placed_display_id.is_none() && !toy.is_consumed_repair_part()
            })
            .filter_map(|(index, toy)| {
                let horizontal_distance = toy.position.to_vec2().distance(player_2d);
                if horizontal_distance > reach {
                    return None;
                }

                let center = loose_toy_aim_center(index, toy);
                let to_toy = center - eye;
                let along_ray = to_toy.dot(forward);
                if along_ray <= 0.10 {
                    return None;
                }

                let lateral = (to_toy - forward * along_ray).length();
                if lateral > LOOSE_TOY_LOOK_RADIUS {
                    return None;
                }

                let score = lateral * 5.0 + along_ray * 0.08 + horizontal_distance * 0.05;
                Some((index, score))
            })
            .min_by(|(_, left), (_, right)| left.total_cmp(right))
            .map(|(index, _)| index)
    }

    pub(super) fn is_near_display(&self, data: &GameData) -> bool {
        data.displays
            .iter()
            .any(|display| self.display_is_near_player(display, &data.config))
    }

    pub(super) fn display_is_full(&self, display: &DisplayDef, room_width: f32) -> bool {
        (0..display.capacity).all(|slot_index| {
            self.toy_index_at_display_slot(display, slot_index, room_width)
                .is_some()
        })
    }

    fn display_is_near_player(&self, display: &DisplayDef, config: &GameConfig) -> bool {
        let player = self.player.position.to_vec2();
        let nearest_point = vec2(
            player.x.clamp(display.x, display.x + display.w),
            player.y.clamp(display.y, display.y + display.h),
        );
        let max_distance_sq = config.interaction_radius * config.interaction_radius;
        nearest_point.distance_squared(player) <= max_distance_sq
    }

    fn look_forward_3d(&self) -> Vec3 {
        vec3(
            self.player.yaw.cos() * self.player.pitch.cos(),
            self.player.pitch.sin(),
            self.player.yaw.sin() * self.player.pitch.cos(),
        )
        .normalize_or_zero()
    }
}

fn loose_toy_aim_center(index: usize, toy: &ToyState) -> Vec3 {
    let layer = (index % 7) as f32;
    let height = if toy.bench_slot_index.is_some() {
        0.88
    } else {
        0.32 + layer * 0.020 + toy.spawn_pose.floor_lift
    };
    vec3(toy.position.x, height, toy.position.y)
}
