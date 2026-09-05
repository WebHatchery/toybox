//! Embedded game data and asset manifests for Toybox After Hours.

use macroquad_toolkit::assets::TextureConfig;
use macroquad_toolkit::data_loader::load_embedded_json_labeled;
use serde::{Deserialize, Serialize};

const GAME_CONFIG_JSON: &str =
    macroquad_toolkit::include_json_str!("../assets/data/game_config.json");
const DISPLAYS_JSON: &str = macroquad_toolkit::include_json_str!("../assets/data/displays.json");
const UPGRADES_JSON: &str = macroquad_toolkit::include_json_str!("../assets/data/upgrades.json");
const TEXTURE_MANIFEST_JSON: &str =
    macroquad_toolkit::include_json_str!("../assets/data/texture_manifest.json");
const LAYOUT_JSON: &str = macroquad_toolkit::include_json_str!("../assets/data/layout.json");

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameConfig {
    pub game_name: String,
    pub display_name: String,
    pub save_slot: String,
    /// Best-run records live in their own slot. "New Game" overwrites the
    /// session slot, which is exactly when a record has to survive.
    pub records_slot: String,
    pub version: String,
    pub room_width: f32,
    pub room_height: f32,
    pub toy_count: usize,
    pub starting_carry_limit: usize,
    /// How long a timed shift lasts before the doors open. Tuned against the
    /// replay: a bare-handed closer clears the floor in ~28 minutes, so 30 is a
    /// deadline a careful player meets and a tooled-up one beats comfortably.
    pub shift_seconds: f32,
    pub player_speed: f32,
    pub interaction_radius: f32,
    pub mistake_penalty_seconds: f32,
    pub broken_fraction: f32,
    pub spatial_cell_size: f32,
    pub toy_render_distance: f32,
    pub toy_lod_distance: f32,
    pub toy_pose_distance: f32,
    pub toy_view_cull_min_dot: f32,
    pub toy_always_draw_radius: f32,
    pub debug_overlay_enabled: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ToyCategory {
    Plushies,
    TinyDragons,
    BuildingBlocks,
    ActionFigures,
    BoardGames,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DisplayDef {
    pub id: String,
    pub name: String,
    pub category: ToyCategory,
    pub theme: String,
    pub capacity: usize,
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
    pub accent: [f32; 4],
    pub symbol: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpgradeDef {
    pub id: String,
    pub name: String,
    pub description: String,
    pub unlock_completed_displays: usize,
    #[serde(default = "default_upgrade_cost")]
    pub cost: usize,
    /// What owning this tool actually does. Required: a tool with no declared
    /// effect would sell for credits and change nothing.
    pub effect: UpgradeEffect,
}

/// The mechanical effect of a tool, declared in `upgrades.json` so new tools
/// are content rather than new dispatch arms in `state/`.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum UpgradeEffect {
    /// Names the correct display for the held toy and highlights it in the
    /// shop, and locates the far half of a carried repair part.
    Scanner,
    /// Raises how many toys the player can hold at once. Highest owned wins.
    CarryLimit { toys: usize },
    /// Scales walking speed. Highest owned wins.
    Speed { multiplier: f32 },
    /// Scales how far the player can reach to pick a toy up. Highest wins.
    Reach { multiplier: f32 },
    /// Waives the time penalty for this many mis-shelved toys. Cumulative.
    MistakeForgiveness { mistakes: u32 },
}

fn default_upgrade_cost() -> usize {
    1
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchDef {
    pub id: String,
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
    pub radius: f32,
    pub capacity: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShelfDef {
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WallSpec {
    pub height: f32,
    pub thickness: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WindowSpec {
    pub x: f32,
    pub center_y: f32,
    pub width: f32,
    pub height: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PosterDef {
    /// Which wall the poster hangs on: "front", "back", "west", or "east".
    pub wall: String,
    /// Distance along the wall (x for front/back, y for side walls).
    pub offset: f32,
    pub center_y: f32,
    pub width: f32,
    pub text: String,
    pub accent: [f32; 4],
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ZoneDef {
    pub name: String,
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
    pub accent: [f32; 4],
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LayoutData {
    pub wall: WallSpec,
    pub window: WindowSpec,
    pub skylights: Vec<ShelfDef>,
    pub zones: Vec<ZoneDef>,
    pub shelving: Vec<ShelfDef>,
    pub counters: Vec<ShelfDef>,
    pub benches: Vec<BenchDef>,
    pub posters: Vec<PosterDef>,
}

impl LayoutData {
    pub fn zone_name_at(&self, x: f32, y: f32) -> Option<&str> {
        self.zones
            .iter()
            .find(|zone| x >= zone.x && x <= zone.x + zone.w && y >= zone.y && y <= zone.y + zone.h)
            .map(|zone| zone.name.as_str())
    }
}

#[derive(Debug, Clone)]
pub struct GameData {
    pub config: GameConfig,
    pub displays: Vec<DisplayDef>,
    pub upgrades: Vec<UpgradeDef>,
    pub texture_manifest: Vec<TextureConfig>,
    pub layout: LayoutData,
}

impl GameData {
    pub fn load() -> Result<Self, String> {
        let config = load_embedded_json_labeled("game_config", GAME_CONFIG_JSON)?;
        let displays = load_embedded_json_labeled("displays", DISPLAYS_JSON)?;
        let upgrades = load_embedded_json_labeled("upgrades", UPGRADES_JSON)?;
        let texture_manifest =
            load_embedded_json_labeled("texture_manifest", TEXTURE_MANIFEST_JSON)?;
        let layout: LayoutData = load_embedded_json_labeled("layout", LAYOUT_JSON)?;

        if layout.benches.is_empty() {
            return Err("layout.json must define at least one bench".to_owned());
        }
        if layout.benches.iter().any(|bench| bench.capacity == 0) {
            return Err("layout.json bench capacity must be at least 1".to_owned());
        }

        Ok(Self {
            config,
            displays,
            upgrades,
            texture_manifest,
            layout,
        })
    }

    pub fn display_by_id(&self, id: &str) -> Option<&DisplayDef> {
        self.displays.iter().find(|display| display.id == id)
    }

    /// The bench used by all single-bench logic until multi-bench lands.
    pub fn primary_bench(&self) -> &BenchDef {
        &self.layout.benches[0]
    }
}

#[cfg(test)]
mod tests;
