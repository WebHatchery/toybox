//! Shared deterministic toy generation and rendering helpers.

use crate::data::{DisplayDef, ToyCategory};
use crate::state::ToyState;
use macroquad::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ToyIdentity {
    Bear,
    Duck,
    Rabbit,
    Cat,
    Puppy,
    Elephant,
    Owl,
    Turtle,
    Penguin,
    Octopus,
    CrescentDragon,
    HornedDragon,
    FinDragon,
    SpikeDragon,
    LongtailDragon,
    WyrmDragon,
    PudgyDragon,
    TwinDragon,
    HatchlingDragon,
    CurledDragon,
    AntennaBot,
    ClawBot,
    TreadBot,
    ScreenBot,
    TripodBot,
    DomeBot,
    BoxyBot,
    RollerBot,
    CrabBot,
    RocketBot,
    MazeBox,
    CastleQuest,
    PlanetRace,
    WordTiles,
    TreasureMap,
    DiceTower,
    CardDeck,
    SpinnerGame,
    ChessSet,
    PuzzleCube,
    TowerBlocks,
    ArchBlocks,
    BridgeBlocks,
    CastleBlocks,
    TrainBlocks,
    PyramidBlocks,
    RainbowBlocks,
    HouseBlocks,
    SpiralBlocks,
    CartBlocks,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ToyProfile {
    pub identity: ToyIdentity,
    pub label: &'static str,
    pub short_code: &'static str,
    pub detail_index: usize,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct ToySpawnPose {
    pub yaw: f32,
    pub pitch: f32,
    pub roll: f32,
    pub floor_lift: f32,
}

impl ToySpawnPose {
    pub fn is_uninitialized(self) -> bool {
        self.yaw == 0.0 && self.pitch == 0.0 && self.roll == 0.0 && self.floor_lift == 0.0
    }

    pub fn is_tumbled(self) -> bool {
        self.pitch.abs() > 0.4 || self.roll.abs() > 0.4
    }
}

impl Default for ToySpawnPose {
    fn default() -> Self {
        Self {
            yaw: 0.0,
            pitch: 0.0,
            roll: 0.0,
            floor_lift: 0.0,
        }
    }
}

#[derive(Debug, Clone, Copy)]
struct IdentityDef {
    identity: ToyIdentity,
    label: &'static str,
    short_code: &'static str,
}

const PLUSH_IDENTITIES: [IdentityDef; 10] = [
    IdentityDef {
        identity: ToyIdentity::Bear,
        label: "Bear",
        short_code: "BR",
    },
    IdentityDef {
        identity: ToyIdentity::Duck,
        label: "Duck",
        short_code: "DK",
    },
    IdentityDef {
        identity: ToyIdentity::Rabbit,
        label: "Rabbit",
        short_code: "RB",
    },
    IdentityDef {
        identity: ToyIdentity::Cat,
        label: "Cat",
        short_code: "CT",
    },
    IdentityDef {
        identity: ToyIdentity::Puppy,
        label: "Puppy",
        short_code: "PP",
    },
    IdentityDef {
        identity: ToyIdentity::Elephant,
        label: "Elephant",
        short_code: "EL",
    },
    IdentityDef {
        identity: ToyIdentity::Owl,
        label: "Owl",
        short_code: "OW",
    },
    IdentityDef {
        identity: ToyIdentity::Turtle,
        label: "Turtle",
        short_code: "TU",
    },
    IdentityDef {
        identity: ToyIdentity::Penguin,
        label: "Penguin",
        short_code: "PG",
    },
    IdentityDef {
        identity: ToyIdentity::Octopus,
        label: "Octopus",
        short_code: "OC",
    },
];

const DRAGON_IDENTITIES: [IdentityDef; 10] = [
    IdentityDef {
        identity: ToyIdentity::CrescentDragon,
        label: "Crescent Dragon",
        short_code: "CR",
    },
    IdentityDef {
        identity: ToyIdentity::HornedDragon,
        label: "Horned Dragon",
        short_code: "HN",
    },
    IdentityDef {
        identity: ToyIdentity::FinDragon,
        label: "Fin Dragon",
        short_code: "FN",
    },
    IdentityDef {
        identity: ToyIdentity::SpikeDragon,
        label: "Spike Dragon",
        short_code: "SP",
    },
    IdentityDef {
        identity: ToyIdentity::LongtailDragon,
        label: "Longtail Dragon",
        short_code: "LT",
    },
    IdentityDef {
        identity: ToyIdentity::WyrmDragon,
        label: "Wyrm Dragon",
        short_code: "WY",
    },
    IdentityDef {
        identity: ToyIdentity::PudgyDragon,
        label: "Pudgy Dragon",
        short_code: "PD",
    },
    IdentityDef {
        identity: ToyIdentity::TwinDragon,
        label: "Twin Dragon",
        short_code: "TD",
    },
    IdentityDef {
        identity: ToyIdentity::HatchlingDragon,
        label: "Hatchling Dragon",
        short_code: "HD",
    },
    IdentityDef {
        identity: ToyIdentity::CurledDragon,
        label: "Curled Dragon",
        short_code: "CD",
    },
];

const ROBOT_IDENTITIES: [IdentityDef; 10] = [
    IdentityDef {
        identity: ToyIdentity::AntennaBot,
        label: "Antenna Bot",
        short_code: "AN",
    },
    IdentityDef {
        identity: ToyIdentity::ClawBot,
        label: "Claw Bot",
        short_code: "CL",
    },
    IdentityDef {
        identity: ToyIdentity::TreadBot,
        label: "Tread Bot",
        short_code: "TR",
    },
    IdentityDef {
        identity: ToyIdentity::ScreenBot,
        label: "Screen Bot",
        short_code: "SC",
    },
    IdentityDef {
        identity: ToyIdentity::TripodBot,
        label: "Tripod Bot",
        short_code: "TP",
    },
    IdentityDef {
        identity: ToyIdentity::DomeBot,
        label: "Dome Bot",
        short_code: "DM",
    },
    IdentityDef {
        identity: ToyIdentity::BoxyBot,
        label: "Boxy Bot",
        short_code: "BX",
    },
    IdentityDef {
        identity: ToyIdentity::RollerBot,
        label: "Roller Bot",
        short_code: "RL",
    },
    IdentityDef {
        identity: ToyIdentity::CrabBot,
        label: "Crab Bot",
        short_code: "CB",
    },
    IdentityDef {
        identity: ToyIdentity::RocketBot,
        label: "Rocket Bot",
        short_code: "RK",
    },
];

const BOARD_GAME_IDENTITIES: [IdentityDef; 10] = [
    IdentityDef {
        identity: ToyIdentity::MazeBox,
        label: "Maze Box",
        short_code: "MZ",
    },
    IdentityDef {
        identity: ToyIdentity::CastleQuest,
        label: "Castle Quest",
        short_code: "CQ",
    },
    IdentityDef {
        identity: ToyIdentity::PlanetRace,
        label: "Planet Race",
        short_code: "PR",
    },
    IdentityDef {
        identity: ToyIdentity::WordTiles,
        label: "Word Tiles",
        short_code: "WT",
    },
    IdentityDef {
        identity: ToyIdentity::TreasureMap,
        label: "Treasure Map",
        short_code: "TM",
    },
    IdentityDef {
        identity: ToyIdentity::DiceTower,
        label: "Dice Tower",
        short_code: "DT",
    },
    IdentityDef {
        identity: ToyIdentity::CardDeck,
        label: "Card Deck",
        short_code: "CC",
    },
    IdentityDef {
        identity: ToyIdentity::SpinnerGame,
        label: "Spinner Game",
        short_code: "SG",
    },
    IdentityDef {
        identity: ToyIdentity::ChessSet,
        label: "Chess Set",
        short_code: "CH",
    },
    IdentityDef {
        identity: ToyIdentity::PuzzleCube,
        label: "Puzzle Cube",
        short_code: "PC",
    },
];

const BLOCK_IDENTITIES: [IdentityDef; 10] = [
    IdentityDef {
        identity: ToyIdentity::TowerBlocks,
        label: "Tower Blocks",
        short_code: "TW",
    },
    IdentityDef {
        identity: ToyIdentity::ArchBlocks,
        label: "Arch Blocks",
        short_code: "AR",
    },
    IdentityDef {
        identity: ToyIdentity::BridgeBlocks,
        label: "Bridge Blocks",
        short_code: "BG",
    },
    IdentityDef {
        identity: ToyIdentity::CastleBlocks,
        label: "Castle Blocks",
        short_code: "CA",
    },
    IdentityDef {
        identity: ToyIdentity::TrainBlocks,
        label: "Train Blocks",
        short_code: "TN",
    },
    IdentityDef {
        identity: ToyIdentity::PyramidBlocks,
        label: "Pyramid Blocks",
        short_code: "PY",
    },
    IdentityDef {
        identity: ToyIdentity::RainbowBlocks,
        label: "Rainbow Blocks",
        short_code: "RW",
    },
    IdentityDef {
        identity: ToyIdentity::HouseBlocks,
        label: "House Blocks",
        short_code: "HS",
    },
    IdentityDef {
        identity: ToyIdentity::SpiralBlocks,
        label: "Spiral Blocks",
        short_code: "SL",
    },
    IdentityDef {
        identity: ToyIdentity::CartBlocks,
        label: "Cart Blocks",
        short_code: "KT",
    },
];

pub fn toy_profile(category: ToyCategory, slot_number: usize) -> ToyProfile {
    let identities: &[IdentityDef] = match category {
        ToyCategory::Plushies => &PLUSH_IDENTITIES,
        ToyCategory::TinyDragons => &DRAGON_IDENTITIES,
        ToyCategory::ActionFigures => &ROBOT_IDENTITIES,
        ToyCategory::BoardGames => &BOARD_GAME_IDENTITIES,
        ToyCategory::BuildingBlocks => &BLOCK_IDENTITIES,
    };
    let stable_index = slot_number.saturating_sub(1);
    let identity = identities[stable_index % identities.len()];

    ToyProfile {
        identity: identity.identity,
        label: identity.label,
        short_code: identity.short_code,
        detail_index: stable_index / identities.len(),
    }
}

pub fn toy_name(display: &DisplayDef, slot_number: usize) -> String {
    let profile = toy_profile(display.category, slot_number);
    format!("{} {} #{slot_number:02}", display.theme, profile.label)
}

pub fn spawn_pose_for_toy(
    toy_index: usize,
    display_index: usize,
    slot_index: usize,
) -> ToySpawnPose {
    let yaw_seed = (toy_index * 53 + display_index * 17 + slot_index * 29) % 360;
    let yaw = (yaw_seed as f32 + 7.0).to_radians();
    let lean = (((toy_index * 31 + slot_index * 11) % 21) as f32 - 10.0).to_radians();

    // Slot coefficient must keep the selector odd-capable: with per-display
    // capacity C, toy_index = C*display + slot, and an all-even reduction
    // (e.g. C=25 with slot*3) locks out half the poses.
    match (toy_index * 7 + display_index * 5 + slot_index * 4) % 8 {
        0 => ToySpawnPose {
            yaw,
            pitch: lean * 0.4,
            roll: 0.22,
            floor_lift: 0.03,
        },
        1 => ToySpawnPose {
            yaw,
            pitch: lean,
            roll: std::f32::consts::FRAC_PI_2,
            floor_lift: 0.17,
        },
        2 => ToySpawnPose {
            yaw,
            pitch: -lean,
            roll: -std::f32::consts::FRAC_PI_2,
            floor_lift: 0.17,
        },
        3 => ToySpawnPose {
            yaw,
            pitch: lean * 0.5,
            roll: std::f32::consts::PI,
            floor_lift: 0.48,
        },
        4 => ToySpawnPose {
            yaw,
            pitch: std::f32::consts::FRAC_PI_2,
            roll: lean,
            floor_lift: 0.24,
        },
        5 => ToySpawnPose {
            yaw,
            pitch: -std::f32::consts::FRAC_PI_2,
            roll: -lean,
            floor_lift: 0.24,
        },
        6 => ToySpawnPose {
            yaw,
            pitch: 0.38 + lean * 0.3,
            roll: std::f32::consts::FRAC_PI_2 * 1.35,
            floor_lift: 0.30,
        },
        _ => ToySpawnPose {
            yaw,
            pitch: -0.30 + lean * 0.3,
            roll: -std::f32::consts::FRAC_PI_2 * 1.35,
            floor_lift: 0.30,
        },
    }
}

/// Candy-shop palette: saturated but soft toy tones spanning the wheel plus
/// cream and chocolate neutrals so shelves don't read as pure rainbow.
const TOY_PALETTE: [Color; 14] = [
    Color::new(0.82, 0.24, 0.20, 1.0), // cherry
    Color::new(0.95, 0.56, 0.20, 1.0), // tangerine
    Color::new(0.96, 0.78, 0.26, 1.0), // sunflower
    Color::new(0.62, 0.80, 0.30, 1.0), // lime
    Color::new(0.28, 0.64, 0.42, 1.0), // forest
    Color::new(0.20, 0.68, 0.68, 1.0), // teal
    Color::new(0.42, 0.72, 0.92, 1.0), // sky
    Color::new(0.26, 0.46, 0.86, 1.0), // royal
    Color::new(0.62, 0.54, 0.88, 1.0), // lavender
    Color::new(0.56, 0.34, 0.72, 1.0), // plum
    Color::new(0.86, 0.36, 0.62, 1.0), // magenta
    Color::new(0.94, 0.62, 0.72, 1.0), // bubblegum
    Color::new(0.92, 0.86, 0.70, 1.0), // cream
    Color::new(0.55, 0.38, 0.26, 1.0), // chocolate
];

pub fn toy_color(toy: &ToyState) -> Color {
    let index = toy.slot_number + toy.color_index * 3;
    let base = TOY_PALETTE[index % TOY_PALETTE.len()];
    // Two independent warm/cool nudges give same-hue siblings subtle variety
    // instead of one flat lightness shift.
    let warm = ((toy.slot_number * 7 + toy.color_index * 5) % 7) as f32 * 0.016 - 0.048;
    let cool = ((toy.slot_number * 11 + toy.color_index * 3) % 5) as f32 * 0.014 - 0.028;
    Color::new(
        (base.r + warm).clamp(0.08, 0.98),
        (base.g + warm * 0.6 + cool * 0.4).clamp(0.08, 0.98),
        (base.b + cool).clamp(0.08, 0.98),
        1.0,
    )
}
