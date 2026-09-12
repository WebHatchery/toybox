//! Procedural toy library and per-identity renderers.

use crate::data::ToyCategory;
use crate::state::ToyState;
use library::ToyIdentity;
use macroquad::prelude::*;

mod antenna_bot;
mod arch_blocks;
mod bear;
mod boxy_bot;
mod bridge_blocks;
mod card_deck;
mod cart_blocks;
mod castle_blocks;
mod castle_quest;
mod cat;
mod chess_set;
mod claw_bot;
mod crab_bot;
mod crescent_dragon;
mod curled_dragon;
mod dice_tower;
mod dome_bot;
mod duck;
mod elephant;
mod fin_dragon;
mod hatchling_dragon;
mod horned_dragon;
mod house_blocks;
pub mod library;
mod longtail_dragon;
mod maze_box;
mod octopus;
mod owl;
pub mod part_accents;
mod penguin;
mod planet_race;
mod primitives;
mod pudgy_dragon;
mod puppy;
mod puzzle_cube;
mod pyramid_blocks;
mod rabbit;
mod rainbow_blocks;
mod repair_parts;
mod rocket_bot;
mod roller_bot;
mod screen_bot;
mod spike_dragon;
mod spinner_game;
mod spiral_blocks;
mod tower_blocks;
mod train_blocks;
mod tread_bot;
mod treasure_map;
mod tripod_bot;
mod turtle;
mod twin_dragon;
mod word_tiles;
mod wyrm_dragon;

pub use library::{spawn_pose_for_toy, toy_color, toy_name, toy_profile, ToySpawnPose};
pub use primitives::brighten;

pub fn draw_loose_toy_3d(toy: &ToyState, center: Vec3, color: Color, scale: f32) {
    let matrix = glam::Mat4::from_translation(center)
        * glam::Mat4::from_rotation_y(toy.spawn_pose.yaw)
        * glam::Mat4::from_rotation_z(toy.spawn_pose.roll)
        * glam::Mat4::from_rotation_x(toy.spawn_pose.pitch)
        * glam::Mat4::from_translation(-center);

    unsafe {
        get_internal_gl().quad_gl.push_model_matrix(matrix);
    }
    draw_toy_3d(toy, center, color, scale);
    unsafe {
        get_internal_gl().quad_gl.pop_model_matrix();
    }
}

/// Distant stand-in: a single cube in the toy's color, proportioned to the
/// toy's category so the swap to full detail near the player stays subtle.
pub fn draw_toy_lod_3d(toy: &ToyState, center: Vec3, color: Color, scale: f32) {
    let (lift, size) = if toy.is_repair_part() {
        (0.14, vec3(0.34, 0.26, 0.30))
    } else {
        match toy.category {
            ToyCategory::Plushies => (0.15, vec3(0.34, 0.34, 0.34)),
            ToyCategory::TinyDragons => (0.10, vec3(0.38, 0.22, 0.42)),
            ToyCategory::ActionFigures => (0.20, vec3(0.28, 0.42, 0.26)),
            ToyCategory::BoardGames => (0.08, vec3(0.42, 0.16, 0.40)),
            ToyCategory::BuildingBlocks => (0.15, vec3(0.34, 0.32, 0.34)),
        }
    };
    draw_cube(
        center + vec3(0.0, lift, 0.0) * scale,
        size * scale,
        None,
        color,
    );
}

pub fn draw_toy_3d(toy: &ToyState, center: Vec3, color: Color, scale: f32) {
    let profile = toy_profile(toy.category, toy.slot_number);
    if let Some(part) = toy.repair_part_kind() {
        // Parts keep the whole toy's category and slot number, so the identity
        // that shaped the intact toy is still available to shape its halves.
        repair_parts::draw(toy.category, profile.identity, part, center, color, scale);
        return;
    }

    match profile.identity {
        ToyIdentity::Bear => bear::draw(center, color, scale),
        ToyIdentity::Duck => duck::draw(center, color, scale),
        ToyIdentity::Rabbit => rabbit::draw(center, color, scale),
        ToyIdentity::Cat => cat::draw(center, color, scale),
        ToyIdentity::Puppy => puppy::draw(center, color, scale),
        ToyIdentity::Elephant => elephant::draw(center, color, scale),
        ToyIdentity::Owl => owl::draw(center, color, scale),
        ToyIdentity::Turtle => turtle::draw(center, color, scale),
        ToyIdentity::Penguin => penguin::draw(center, color, scale),
        ToyIdentity::Octopus => octopus::draw(center, color, scale),
        ToyIdentity::CrescentDragon => crescent_dragon::draw(center, color, scale),
        ToyIdentity::HornedDragon => horned_dragon::draw(center, color, scale),
        ToyIdentity::FinDragon => fin_dragon::draw(center, color, scale),
        ToyIdentity::SpikeDragon => spike_dragon::draw(center, color, scale),
        ToyIdentity::LongtailDragon => longtail_dragon::draw(center, color, scale),
        ToyIdentity::WyrmDragon => wyrm_dragon::draw(center, color, scale),
        ToyIdentity::PudgyDragon => pudgy_dragon::draw(center, color, scale),
        ToyIdentity::TwinDragon => twin_dragon::draw(center, color, scale),
        ToyIdentity::HatchlingDragon => hatchling_dragon::draw(center, color, scale),
        ToyIdentity::CurledDragon => curled_dragon::draw(center, color, scale),
        ToyIdentity::AntennaBot => antenna_bot::draw(center, color, scale),
        ToyIdentity::ClawBot => claw_bot::draw(center, color, scale),
        ToyIdentity::TreadBot => tread_bot::draw(center, color, scale),
        ToyIdentity::ScreenBot => screen_bot::draw(center, color, scale),
        ToyIdentity::TripodBot => tripod_bot::draw(center, color, scale),
        ToyIdentity::DomeBot => dome_bot::draw(center, color, scale),
        ToyIdentity::BoxyBot => boxy_bot::draw(center, color, scale),
        ToyIdentity::RollerBot => roller_bot::draw(center, color, scale),
        ToyIdentity::CrabBot => crab_bot::draw(center, color, scale),
        ToyIdentity::RocketBot => rocket_bot::draw(center, color, scale),
        ToyIdentity::MazeBox => maze_box::draw(center, color, scale),
        ToyIdentity::CastleQuest => castle_quest::draw(center, color, scale),
        ToyIdentity::PlanetRace => planet_race::draw(center, color, scale),
        ToyIdentity::WordTiles => word_tiles::draw(center, color, scale),
        ToyIdentity::TreasureMap => treasure_map::draw(center, color, scale),
        ToyIdentity::DiceTower => dice_tower::draw(center, color, scale),
        ToyIdentity::CardDeck => card_deck::draw(center, color, scale),
        ToyIdentity::SpinnerGame => spinner_game::draw(center, color, scale),
        ToyIdentity::ChessSet => chess_set::draw(center, color, scale),
        ToyIdentity::PuzzleCube => puzzle_cube::draw(center, color, scale),
        ToyIdentity::TowerBlocks => tower_blocks::draw(center, color, scale),
        ToyIdentity::ArchBlocks => arch_blocks::draw(center, color, scale),
        ToyIdentity::BridgeBlocks => bridge_blocks::draw(center, color, scale),
        ToyIdentity::CastleBlocks => castle_blocks::draw(center, color, scale),
        ToyIdentity::TrainBlocks => train_blocks::draw(center, color, scale),
        ToyIdentity::PyramidBlocks => pyramid_blocks::draw(center, color, scale),
        ToyIdentity::RainbowBlocks => rainbow_blocks::draw(center, color, scale),
        ToyIdentity::HouseBlocks => house_blocks::draw(center, color, scale),
        ToyIdentity::SpiralBlocks => spiral_blocks::draw(center, color, scale),
        ToyIdentity::CartBlocks => cart_blocks::draw(center, color, scale),
    }
}
