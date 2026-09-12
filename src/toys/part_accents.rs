//! Per-identity silhouette cues for broken toy halves.
//!
//! A split toy is drawn by one of ten category renderers, so before this every
//! plush head was the same head: a broken Bear and a broken Octopus were the
//! same object in two colours, and the only way to tell a pair apart on the
//! floor was to walk up and read the name.
//!
//! Giving each of the fifty identities its own head and body model would mean a
//! hundred more renderers to write and keep in step with the fifty whole-toy
//! ones. That is a great deal of work for an object the player sees for the
//! length of one errand. Instead each identity contributes a handful of numbers
//! — what sits on top, how far the face juts out, how wide the eyes sit — and
//! the ten renderers read them. Fifty table rows, one drawing helper, and a
//! Rabbit head no longer looks like an Octopus head.

use super::library::ToyIdentity;
use super::primitives::{brighten, draw_cube_with_edges, draw_toy_sphere};
use macroquad::prelude::*;

/// What tops a head. Deliberately a small set: these are read at a glance, from
/// across an aisle, on an object the size of a fist.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Crest {
    /// Nothing on top — smooth domes, lids, plain blocks.
    Bare,
    /// Two round blobs, high and wide. Bears, most bots.
    RoundEars,
    /// Two long uprights. Rabbits, antenna bots, spires.
    TallEars,
    /// Two triangular points, lower and closer in. Cats, foxes, fins.
    PointedEars,
    /// A ring of short stubs — tentacles, fringes, many-legged things.
    Fringe,
    /// A pair of hard angular horns.
    Horns,
    /// A single central spike or aerial.
    Spike,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PartAccent {
    pub crest: Crest,
    /// Multiplier on the identity's accent geometry: crest size on a head,
    /// limb size on a body.
    pub accent_scale: f32,
    /// How far the face juts forward. 0.0 is flat, 1.0 a long snout or beak.
    pub muzzle: f32,
    /// Multiplier on the default eye separation.
    pub eye_spread: f32,
}

impl PartAccent {
    const fn new(crest: Crest, accent_scale: f32, muzzle: f32, eye_spread: f32) -> Self {
        Self {
            crest,
            accent_scale,
            muzzle,
            eye_spread,
        }
    }
}

/// The cues for one identity. Every arm is chosen so that the ten identities
/// sharing a category renderer differ from *each other* — matching the whole
/// toy is secondary, because the player never sees both at once.
pub fn accent_for(identity: ToyIdentity) -> PartAccent {
    use Crest::*;
    use ToyIdentity as T;

    match identity {
        // Plushies: ears and snouts do all the work.
        T::Bear => PartAccent::new(RoundEars, 1.0, 0.35, 1.0),
        T::Duck => PartAccent::new(Bare, 1.0, 0.95, 1.15),
        T::Rabbit => PartAccent::new(TallEars, 1.15, 0.30, 0.95),
        T::Cat => PartAccent::new(PointedEars, 1.0, 0.25, 1.05),
        T::Puppy => PartAccent::new(RoundEars, 1.25, 0.60, 1.0),
        T::Elephant => PartAccent::new(RoundEars, 1.6, 1.0, 1.2),
        T::Owl => PartAccent::new(PointedEars, 0.75, 0.40, 1.45),
        T::Turtle => PartAccent::new(Bare, 1.0, 0.55, 0.9),
        T::Penguin => PartAccent::new(Bare, 1.0, 0.75, 0.8),
        T::Octopus => PartAccent::new(Fringe, 1.0, 0.0, 1.3),

        // Dragons: horn counts and muzzle lengths.
        T::CrescentDragon => PartAccent::new(Horns, 1.25, 0.70, 1.0),
        T::HornedDragon => PartAccent::new(Horns, 1.6, 0.55, 0.95),
        T::FinDragon => PartAccent::new(PointedEars, 1.3, 0.60, 1.0),
        T::SpikeDragon => PartAccent::new(Spike, 1.35, 0.55, 0.9),
        T::LongtailDragon => PartAccent::new(Horns, 0.85, 0.95, 0.85),
        T::WyrmDragon => PartAccent::new(Bare, 1.0, 1.0, 0.8),
        T::PudgyDragon => PartAccent::new(RoundEars, 1.1, 0.30, 1.2),
        T::TwinDragon => PartAccent::new(Horns, 1.0, 0.45, 1.5),
        T::HatchlingDragon => PartAccent::new(Spike, 0.7, 0.35, 1.25),
        T::CurledDragon => PartAccent::new(Fringe, 0.9, 0.50, 1.0),

        // Robots: aerials, vents and sensor spreads. Nearly all of them carry
        // *something* on top — a robot head is a flat-topped cube, so a muzzle
        // or an eye spacing reads far weaker on it than on a plush sphere, and
        // a first pass that left three of these `Bare` produced a lineup where
        // half the machines were the same box in different colours.
        T::AntennaBot => PartAccent::new(TallEars, 1.0, 0.20, 1.0),
        T::ClawBot => PartAccent::new(PointedEars, 1.1, 0.35, 1.1),
        T::TreadBot => PartAccent::new(Horns, 0.9, 0.30, 1.25),
        T::ScreenBot => PartAccent::new(Bare, 1.0, 0.15, 1.55),
        T::TripodBot => PartAccent::new(Spike, 1.2, 0.25, 0.85),
        T::DomeBot => PartAccent::new(Fringe, 0.6, 0.10, 0.95),
        T::BoxyBot => PartAccent::new(RoundEars, 0.8, 0.25, 1.15),
        T::RollerBot => PartAccent::new(Fringe, 0.9, 0.20, 1.0),
        T::CrabBot => PartAccent::new(Fringe, 1.3, 0.30, 1.4),
        T::RocketBot => PartAccent::new(Spike, 1.5, 0.45, 0.8),

        // Board games and blocks: no faces, so the crest carries everything.
        T::MazeBox => PartAccent::new(Bare, 1.0, 0.0, 1.0),
        T::CastleQuest => PartAccent::new(TallEars, 0.9, 0.0, 1.0),
        T::PlanetRace => PartAccent::new(Spike, 0.9, 0.0, 1.0),
        T::WordTiles => PartAccent::new(RoundEars, 0.7, 0.0, 1.0),
        T::TreasureMap => PartAccent::new(PointedEars, 0.9, 0.0, 1.0),
        T::DiceTower => PartAccent::new(TallEars, 1.3, 0.0, 1.0),
        T::CardDeck => PartAccent::new(RoundEars, 1.15, 0.0, 1.0),
        T::SpinnerGame => PartAccent::new(Spike, 1.2, 0.0, 1.0),
        T::ChessSet => PartAccent::new(Horns, 0.9, 0.0, 1.0),
        T::PuzzleCube => PartAccent::new(Fringe, 0.7, 0.0, 1.0),

        T::TowerBlocks => PartAccent::new(TallEars, 1.2, 0.0, 1.0),
        T::ArchBlocks => PartAccent::new(RoundEars, 1.0, 0.0, 1.0),
        T::BridgeBlocks => PartAccent::new(Horns, 1.1, 0.0, 1.0),
        T::CastleBlocks => PartAccent::new(PointedEars, 1.2, 0.0, 1.0),
        T::TrainBlocks => PartAccent::new(Horns, 0.8, 0.0, 1.0),
        T::PyramidBlocks => PartAccent::new(Spike, 1.0, 0.0, 1.0),
        T::RainbowBlocks => PartAccent::new(Fringe, 1.0, 0.0, 1.0),
        T::HouseBlocks => PartAccent::new(PointedEars, 0.8, 0.0, 1.0),
        T::SpiralBlocks => PartAccent::new(Spike, 0.7, 0.0, 1.0),
        T::CartBlocks => PartAccent::new(RoundEars, 1.3, 0.0, 1.0),
    }
}

/// Draw the crest sitting on a part of half-width `extent`, topped at `top`.
///
/// **Heads only.** Ears, horns, antennae and spikes are things a head has; a
/// body wearing them is a headless torso with rabbit ears growing out of the
/// neck stump, which is both nonsense and ruins the one read that matters while
/// a toy is split — which half am I holding. Bodies vary their own limbs by
/// `accent_scale` instead.
///
/// One helper for all five head renderers: a Rabbit's ears and a Rocket Bot's
/// aerial are the same two calls with different numbers, and keeping them here
/// is what makes the accent table cheap enough to be worth having.
pub(super) fn draw_crest(
    accent: PartAccent,
    center: Vec3,
    top: f32,
    extent: f32,
    color: Color,
    scale: f32,
) {
    let size = accent.accent_scale;
    match accent.crest {
        Crest::Bare => {}
        Crest::RoundEars => {
            for side in [-1.0_f32, 1.0] {
                draw_toy_sphere(
                    center + vec3(side * extent * 0.78, top + 0.02 * size, 0.0) * scale,
                    0.070 * size * scale,
                    None,
                    color,
                );
            }
        }
        Crest::TallEars => {
            for side in [-1.0_f32, 1.0] {
                draw_cube_with_edges(
                    center + vec3(side * extent * 0.46, top + 0.075 * size, 0.0) * scale,
                    vec3(0.045, 0.17 * size, 0.045) * scale,
                    color,
                );
            }
        }
        Crest::PointedEars => {
            for side in [-1.0_f32, 1.0] {
                draw_cube_with_edges(
                    center + vec3(side * extent * 0.58, top + 0.04 * size, 0.0) * scale,
                    vec3(0.055, 0.10 * size, 0.035) * scale,
                    color,
                );
            }
        }
        Crest::Fringe => {
            // Six stubs around the crown, so it reads as many-limbed from any
            // angle rather than only from the front.
            for step in 0..6 {
                let angle = step as f32 * std::f32::consts::TAU / 6.0;
                draw_toy_sphere(
                    center
                        + vec3(
                            angle.cos() * extent * 0.74,
                            top - 0.01,
                            angle.sin() * extent * 0.74,
                        ) * scale,
                    0.042 * size * scale,
                    None,
                    color,
                );
            }
        }
        Crest::Horns => {
            let horn = Color::new(0.96, 0.88, 0.58, 1.0);
            for side in [-1.0_f32, 1.0] {
                draw_cube_with_edges(
                    center + vec3(side * extent * 0.42, top + 0.05 * size, 0.03) * scale,
                    vec3(0.04, 0.11 * size, 0.04) * scale,
                    horn,
                );
            }
        }
        Crest::Spike => {
            draw_cube_with_edges(
                center + vec3(0.0, top + 0.085 * size, 0.0) * scale,
                vec3(0.036, 0.19 * size, 0.036) * scale,
                brighten(color, 0.18),
            );
        }
    }
}
