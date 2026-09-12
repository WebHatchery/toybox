//! Runtime toy-store state, save data, and save migration helpers.

use crate::data::{DisplayDef, GameData, ToyCategory};
use crate::toys::ToySpawnPose;
use macroquad::prelude::*;
use macroquad_toolkit::rng::SeededRng;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

mod collision;
mod interactions;
mod persistence;
mod progress;
mod records;
mod repair;
mod results;
mod spatial;
mod spawn;
mod tools;

use collision::{keep_off_fixtures, position_blocked};
use spawn::build_toys;

pub use persistence::{migrate_save_value, SaveData};
pub use progress::{ShiftSummary, ZoneProgress};
pub use records::{BestRuns, ShiftRecord};
pub use repair::{BenchStage, BenchStatus, CounterpartLocation};
pub use results::{DisplaySlotTarget, InteractionPreview, InteractionResult, ToolPurchaseResult};
pub use spatial::ToySpatialGrid;
pub use tools::{
    STOCKROOM_SPOTLIGHT_COST, STOCKROOM_SPOTLIGHT_MAX_SECONDS, STOCKROOM_SPOTLIGHT_NAME,
    STOCKROOM_SPOTLIGHT_SECONDS,
};

/// The catalog-comparable Closing Shift layout. Its derived variation is zero,
/// preserving the scatter used by every balance replay before seeds existed.
pub const CLOSING_SHIFT_SEED: u64 = 0xC105_1A6F_7EED_0001;

pub(super) const fn default_shift_seed() -> u64 {
    CLOSING_SHIFT_SEED
}

/// A deterministic offset for one part of seeded generation. The competitive
/// seed intentionally returns zero so adding seeds cannot move its old layout.
pub(super) fn seeded_offset(shift_seed: u64, salt: u64, upper: usize) -> usize {
    let variation = shift_seed ^ CLOSING_SHIFT_SEED;
    if variation == 0 || upper == 0 {
        0
    } else {
        SeededRng::new(variation ^ salt).below(upper)
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct WorldPoint {
    pub x: f32,
    pub y: f32,
}

impl WorldPoint {
    pub fn from_vec2(position: Vec2) -> Self {
        Self {
            x: position.x,
            y: position.y,
        }
    }

    /// Staging helper for the screenshot scenes in `capture_scenes`.
    pub(crate) fn from_vec2_for_capture(position: Vec2) -> Self {
        Self::from_vec2(position)
    }

    pub fn to_vec2(self) -> Vec2 {
        vec2(self.x, self.y)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerState {
    pub position: WorldPoint,
    #[serde(default = "default_player_yaw")]
    pub yaw: f32,
    #[serde(default)]
    pub pitch: f32,
    pub carried_toy_ids: Vec<String>,
    pub active_carry_index: usize,
    pub mistakes: u32,
    /// Broken toys rejoined at a bench. Saves predating the score screen never
    /// counted them, so they load as zero rather than refusing to load.
    #[serde(default)]
    pub repairs: u32,
    /// Protected wrong placements still count, but the manager stops the toy
    /// before it leaves the player's hands. Kept as remaining uses so buying
    /// the tool after earlier mistakes still grants the advertised full count.
    #[serde(default)]
    pub mistake_guards_remaining: u32,
    /// Distinguishes an old save with no guard field from a current save whose
    /// guards were legitimately used up.
    #[serde(default)]
    pub mistake_guards_initialized: bool,
    /// Credits spent on repeatable shop services rather than permanent tools.
    #[serde(default)]
    pub service_credits_spent: usize,
    /// Remaining seconds of the nearest-loose-toy beacon.
    #[serde(default)]
    pub stockroom_spotlight_seconds: f32,
    pub elapsed_seconds: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToyState {
    pub id: String,
    pub name: String,
    pub category: ToyCategory,
    pub theme: String,
    pub slot_number: usize,
    pub color_index: usize,
    pub position: WorldPoint,
    #[serde(default)]
    pub spawn_pose: ToySpawnPose,
    pub is_held: bool,
    pub placed_display_id: Option<String>,
    #[serde(default)]
    pub placed_slot_index: Option<usize>,
    #[serde(default)]
    pub bench_slot_index: Option<usize>,
    #[serde(default)]
    pub bench_id: Option<String>,
    #[serde(default)]
    pub wrong_marker_seconds: f32,
    #[serde(default)]
    pub repair_state: RepairState,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "state", rename_all = "snake_case")]
#[derive(Default)]
pub enum RepairState {
    #[default]
    Whole,
    BrokenPart {
        repair_id: String,
        part: RepairPartKind,
        repaired_name: String,
    },
    ConsumedPart {
        repair_id: String,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RepairPartKind {
    Head,
    Body,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DisplayState {
    pub id: String,
    pub placed_toy_ids: Vec<String>,
    pub is_complete: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum GamePhase {
    Playing,
    /// The shop was fully restored — every display stocked, every break mended.
    Finished,
    /// The doors opened with work still on the floor. A separate variant rather
    /// than a flag on `Finished` so the existing `phase != Playing` guards stop
    /// movement and interaction for free, and so a score screen can tell the
    /// two endings apart without a second field to keep in sync.
    TimeUp,
}

impl GamePhase {
    pub fn is_over(self) -> bool {
        !matches!(self, GamePhase::Playing)
    }

    /// Whether leaving for the title should write this shift to the save slot.
    ///
    /// A run still in progress has to survive being quit — that button is the
    /// only way out of a shift from the UI. A finished one must *not* be
    /// written, because the title's "Continue" would then resume onto a score
    /// screen for a run that is already over and already recorded, and would
    /// have overwritten whatever resumable shift was in the slot before it.
    pub fn should_save_on_leaving(self) -> bool {
        !self.is_over()
    }
}

impl ShiftMode {
    pub fn shows_countdown(self) -> bool {
        matches!(self, ShiftMode::Timed)
    }

    /// What the HUD clock is measuring. Lives beside `shows_countdown` so the
    /// two cannot drift: the caption exists precisely because the panel is
    /// otherwise identical in both modes, and a caption that disagreed with the
    /// direction would be worse than none.
    pub fn clock_caption(self) -> &'static str {
        if self.shows_countdown() {
            "LEFT"
        } else {
            "ELAPSED"
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            ShiftMode::Timed => "Closing Shift",
            ShiftMode::Relaxed => "Relaxed Run",
        }
    }
}

/// Whether the shift runs against the clock.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum ShiftMode {
    /// Opening time is a deadline; the run ends when it arrives.
    #[default]
    Timed,
    /// No deadline. The clock still counts, so a relaxed run can be compared
    /// against a timed one, but it never ends the shift.
    Relaxed,
}

#[derive(Debug, Clone)]
pub struct GameSession {
    pub player: PlayerState,
    pub toys: Vec<ToyState>,
    pub displays: Vec<DisplayState>,
    pub unlocked_upgrade_ids: Vec<String>,
    pub phase: GamePhase,
    pub shift_mode: ShiftMode,
    /// Reproduces the initial scatter, tumble poses, and broken-pair layout.
    /// The live toy state is still saved in full so continuing never rerolls.
    pub shift_seed: u64,
    spatial: ToySpatialGrid,
}

impl GameSession {
    pub const MAX_LOOK_PITCH: f32 = 1.18;
    pub const WRONG_MARKER_SECONDS: f32 = 2.5;

    pub fn new(data: &GameData) -> Self {
        Self::new_with_seed(data, CLOSING_SHIFT_SEED)
    }

    pub fn new_with_seed(data: &GameData, shift_seed: u64) -> Self {
        let config = &data.config;
        let toys = build_toys(data, shift_seed);
        let mut spatial = ToySpatialGrid::new(
            config.room_width,
            config.room_height,
            config.spatial_cell_size,
        );
        spatial.rebuild(&toys);
        let displays = data
            .displays
            .iter()
            .map(|display| DisplayState {
                id: display.id.clone(),
                placed_toy_ids: Vec::new(),
                is_complete: false,
            })
            .collect();

        Self {
            player: PlayerState {
                position: WorldPoint {
                    x: config.room_width * 0.63,
                    y: config.room_height * 0.72,
                },
                yaw: default_player_yaw(),
                pitch: -0.12,
                carried_toy_ids: Vec::new(),
                active_carry_index: 0,
                mistakes: 0,
                repairs: 0,
                mistake_guards_remaining: 0,
                mistake_guards_initialized: true,
                service_credits_spent: 0,
                stockroom_spotlight_seconds: 0.0,
                elapsed_seconds: 0.0,
            },
            toys,
            displays,
            unlocked_upgrade_ids: Vec::new(),
            phase: GamePhase::Playing,
            shift_mode: ShiftMode::default(),
            shift_seed,
            spatial,
        }
    }

    pub fn spatial(&self) -> &ToySpatialGrid {
        &self.spatial
    }

    /// Rebuild the spatial query index after a test or editor changes toy positions.
    pub fn rebuild_spatial_index(&mut self) {
        self.spatial.rebuild(&self.toys);
    }

    /// Advance the shift clock. Returns true on the frame the doors open, so
    /// the caller can announce it once rather than every frame afterwards.
    pub fn update_timer(&mut self, dt: f32, data: &GameData) -> bool {
        let mut just_ran_out = false;
        if self.phase == GamePhase::Playing {
            self.player.elapsed_seconds += dt;
            if self.shift_mode == ShiftMode::Timed && self.shift_remaining(data) <= 0.0 {
                self.phase = GamePhase::TimeUp;
                just_ran_out = true;
            }
        }
        for toy in &mut self.toys {
            toy.wrong_marker_seconds = (toy.wrong_marker_seconds - dt).max(0.0);
        }
        if self.phase == GamePhase::Playing {
            self.player.stockroom_spotlight_seconds =
                (self.player.stockroom_spotlight_seconds - dt).max(0.0);
        }
        just_ran_out
    }

    /// Seconds left before opening, clamped at zero. Always zero in a relaxed
    /// run, where callers should show elapsed time instead — see
    /// `ShiftMode::shows_countdown`.
    pub fn shift_remaining(&self, data: &GameData) -> f32 {
        (data.config.shift_seconds - self.player.elapsed_seconds).max(0.0)
    }

    pub fn update_player_look(&mut self, yaw_delta: f32, pitch_delta: f32) {
        if self.phase != GamePhase::Playing {
            return;
        }

        self.player.yaw += yaw_delta;
        self.player.pitch =
            (self.player.pitch + pitch_delta).clamp(-Self::MAX_LOOK_PITCH, Self::MAX_LOOK_PITCH);
    }

    pub fn move_player(&mut self, direction: Vec2, data: &GameData, dt: f32) {
        if self.phase != GamePhase::Playing || direction.length_squared() == 0.0 {
            return;
        }

        let config = &data.config;
        let forward = vec2(self.player.yaw.cos(), self.player.yaw.sin());
        let right = vec2(-forward.y, forward.x);
        let world_direction = right * direction.x + forward * direction.y;
        if world_direction.length_squared() == 0.0 {
            return;
        }

        let step =
            world_direction.normalize() * config.player_speed * self.speed_multiplier(data) * dt;
        let current = self.player.position.to_vec2();
        let clamp_x = |x: f32| x.clamp(0.45, config.room_width - 0.45);
        let clamp_y = |y: f32| y.clamp(0.45, config.room_height - 0.45);

        // Legacy saves can wake up inside a newly added fixture: let them
        // walk freely until clear instead of pinning them in place.
        if position_blocked(current, data) {
            self.player.position = WorldPoint::from_vec2(vec2(
                clamp_x(current.x + step.x),
                clamp_y(current.y + step.y),
            ));
            return;
        }

        // Axis-separated moves so the player slides along fixture edges.
        let mut next = current;
        let candidate_x = vec2(clamp_x(current.x + step.x), next.y);
        if !position_blocked(candidate_x, data) {
            next.x = candidate_x.x;
        }
        let candidate_y = vec2(next.x, clamp_y(current.y + step.y));
        if !position_blocked(candidate_y, data) {
            next.y = candidate_y.y;
        }
        self.player.position = WorldPoint::from_vec2(next);
    }

    pub fn active_toy(&self) -> Option<&ToyState> {
        let toy_id = self
            .player
            .carried_toy_ids
            .get(self.player.active_carry_index)?;
        self.toys.iter().find(|toy| &toy.id == toy_id)
    }

    pub fn cycle_carried(&mut self) {
        if self.player.carried_toy_ids.is_empty() {
            self.player.active_carry_index = 0;
        } else {
            self.player.active_carry_index =
                (self.player.active_carry_index + 1) % self.player.carried_toy_ids.len();
        }
    }

    pub fn drop_active(&mut self, data: &GameData) -> Option<String> {
        let toy_id = self.active_toy()?.id.clone();
        let toy_index = self.toys.iter().position(|toy| toy.id == toy_id)?;
        let toy_name = self.toys[toy_index].name.clone();
        let drop_position = self.floor_drop_position(data);

        self.toys[toy_index].is_held = false;
        self.toys[toy_index].placed_display_id = None;
        self.toys[toy_index].placed_slot_index = None;
        self.toys[toy_index].bench_slot_index = None;
        self.toys[toy_index].bench_id = None;
        self.toys[toy_index].wrong_marker_seconds = 0.0;
        self.toys[toy_index].position = drop_position;
        self.spatial.sync_toy(toy_index, &self.toys[toy_index]);
        self.player.carried_toy_ids.retain(|id| id != &toy_id);
        self.normalize_active_carry(self.carry_limit(data));

        Some(toy_name)
    }

    fn floor_drop_position(&self, data: &GameData) -> WorldPoint {
        let config = &data.config;
        let forward = vec2(self.player.yaw.cos(), self.player.yaw.sin()).normalize_or_zero();
        let offset = if forward.length_squared() > f32::EPSILON {
            forward * 0.82
        } else {
            Vec2::ZERO
        };
        let position = self.player.position.to_vec2() + offset;
        let clamped = vec2(
            position.x.clamp(0.65, config.room_width - 0.65),
            position.y.clamp(0.65, config.room_height - 0.65),
        );
        let off_fixtures = keep_off_fixtures(clamped, data);

        WorldPoint {
            x: off_fixtures.x.clamp(0.65, config.room_width - 0.65),
            y: off_fixtures.y.clamp(0.65, config.room_height - 0.65),
        }
    }

    pub fn completed_display_count(&self) -> usize {
        self.displays
            .iter()
            .filter(|display| display.is_complete)
            .count()
    }

    pub fn total_placed_toys(&self) -> usize {
        self.displays
            .iter()
            .map(|display| display.placed_toy_ids.len())
            .sum()
    }

    pub fn is_display_complete(&self, display_id: &str) -> bool {
        self.displays
            .iter()
            .find(|display| display.id == display_id)
            .map(|display| display.is_complete)
            .unwrap_or(false)
    }

    /// Recompute which displays are full. Runs after every placement.
    ///
    /// One pass over the toys, bucketed by display. The obvious shape — rescan
    /// every toy once per display, and again once per slot to ask "is anything
    /// in this slot" — is O(displays x capacity x toys): at this shop's 20
    /// displays, 200 slots each and 4500 toys that was ~18 million iterations
    /// and thousands of String clones for every single toy shelved.
    fn refresh_display_completion(&mut self, data: &GameData) {
        let mut index_of: HashMap<&str, usize> = HashMap::with_capacity(data.displays.len());
        for (display_index, display) in data.displays.iter().enumerate() {
            index_of.insert(display.id.as_str(), display_index);
        }

        let mut placed: Vec<Vec<String>> = vec![Vec::new(); data.displays.len()];
        let mut matched_slots: Vec<Vec<bool>> = data
            .displays
            .iter()
            .map(|display| vec![false; display.capacity])
            .collect();

        for toy in &self.toys {
            let Some(display_id) = toy.placed_display_id.as_deref() else {
                continue;
            };
            let Some(&display_index) = index_of.get(display_id) else {
                continue;
            };
            placed[display_index].push(toy.id.clone());

            let display = &data.displays[display_index];
            // A mis-shelved toy occupies the slot without completing it.
            if let Some(slot_index) = toy.placed_slot_index {
                if slot_index < display.capacity && toy_matches_display(toy, display) {
                    matched_slots[display_index][slot_index] = true;
                }
            }
        }

        for display_state in &mut self.displays {
            // A display in the save that the data no longer defines keeps
            // whatever it had, exactly as the per-display lookup used to.
            let Some(&display_index) = index_of.get(display_state.id.as_str()) else {
                continue;
            };
            display_state.placed_toy_ids = std::mem::take(&mut placed[display_index]);
            display_state.is_complete = matched_slots[display_index].iter().all(|filled| *filled);
        }
    }

    fn normalize_active_carry(&mut self, carry_limit: usize) {
        if self.player.carried_toy_ids.len() > carry_limit {
            let kept_id = self
                .player
                .carried_toy_ids
                .get(self.player.active_carry_index)
                .cloned()
                .or_else(|| self.player.carried_toy_ids.first().cloned());
            self.player.carried_toy_ids.retain(|toy_id| {
                kept_id
                    .as_ref()
                    .is_some_and(|kept_toy_id| toy_id == kept_toy_id)
            });
            for toy in &mut self.toys {
                if toy.is_held && Some(&toy.id) != kept_id.as_ref() {
                    toy.is_held = false;
                    toy.placed_display_id = None;
                    toy.placed_slot_index = None;
                    toy.bench_slot_index = None;
                    toy.bench_id = None;
                    toy.wrong_marker_seconds = 0.0;
                    toy.position = self.player.position;
                }
            }
            self.spatial.rebuild(&self.toys);
        }

        if self.player.carried_toy_ids.is_empty() {
            self.player.active_carry_index = 0;
        } else if self.player.active_carry_index >= self.player.carried_toy_ids.len() {
            self.player.active_carry_index = self.player.carried_toy_ids.len() - 1;
        }
    }

    fn repair_player_view(&mut self) {
        if !self.player.yaw.is_finite() {
            self.player.yaw = default_player_yaw();
        }
        if !self.player.pitch.is_finite() {
            self.player.pitch = 0.0;
        }
        self.player.pitch = self
            .player
            .pitch
            .clamp(-Self::MAX_LOOK_PITCH, Self::MAX_LOOK_PITCH);
    }
}

pub fn toy_matches_display(toy: &ToyState, display: &DisplayDef) -> bool {
    matches!(toy.repair_state, RepairState::Whole)
        && toy.category == display.category
        && toy.theme == display.theme
}

fn default_player_yaw() -> f32 {
    -std::f32::consts::FRAC_PI_2
}

pub fn display_slot_position(
    display: &DisplayDef,
    placed_index: usize,
    room_width: f32,
) -> WorldPoint {
    let columns = display.capacity.clamp(1, 5);
    let row = placed_index / columns;
    let column = placed_index % columns;
    let spacing_x = display.w / (columns as f32 + 1.0);
    let row_count = display.capacity.div_ceil(columns).max(1);
    let spacing_y = display.h / (row_count as f32 + 1.0);
    let x = display.x + spacing_x * (column as f32 + 1.0);
    let y = display.y + spacing_y * (row as f32 + 1.0);

    WorldPoint {
        x: x.clamp(0.4, room_width - 0.4),
        y,
    }
}
