//! Embedded game data and semantic validation for Toybox After Hours.

use macroquad_toolkit::data_loader::load_embedded_json_labeled;
use serde::{Deserialize, Serialize};

const GAME_CONFIG_JSON: &str =
    macroquad_toolkit::include_json_str!("../assets/data/game_config.json");
const DISPLAYS_JSON: &str = macroquad_toolkit::include_json_str!("../assets/data/displays.json");
const UPGRADES_JSON: &str = macroquad_toolkit::include_json_str!("../assets/data/upgrades.json");
const LAYOUT_JSON: &str = macroquad_toolkit::include_json_str!("../assets/data/layout.json");
const UI_COPY_JSON: &str = macroquad_toolkit::include_json_str!("../assets/data/ui_copy.json");

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
    pub fov_min_degrees: f32,
    pub fov_max_degrees: f32,
    pub fov_step_degrees: f32,
    pub sensitivity_min: f32,
    pub sensitivity_max: f32,
    pub sensitivity_step: f32,
    pub ui_scale_min: f32,
    pub ui_scale_max: f32,
    pub ui_scale_step: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiCopy {
    pub tutorial: [TutorialStepCopy; 6],
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TutorialStepCopy {
    pub eyebrow: String,
    pub title: String,
    pub body: String,
    pub keys: Vec<String>,
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
    pub ui_copy: UiCopy,
    pub displays: Vec<DisplayDef>,
    pub upgrades: Vec<UpgradeDef>,
    pub layout: LayoutData,
}

impl GameData {
    pub fn load() -> Result<Self, String> {
        let config = load_embedded_json_labeled("game_config", GAME_CONFIG_JSON)?;
        let ui_copy = load_embedded_json_labeled("ui_copy", UI_COPY_JSON)?;
        let displays = load_embedded_json_labeled("displays", DISPLAYS_JSON)?;
        let upgrades = load_embedded_json_labeled("upgrades", UPGRADES_JSON)?;
        let layout: LayoutData = load_embedded_json_labeled("layout", LAYOUT_JSON)?;

        let data = Self {
            config,
            ui_copy,
            displays,
            upgrades,
            layout,
        };
        data.validate()?;
        Ok(data)
    }

    fn validate(&self) -> Result<(), String> {
        validate_config(&self.config)?;
        validate_ui_copy(&self.ui_copy)?;
        validate_displays(&self.displays, &self.config)?;
        validate_upgrades(&self.upgrades, self.displays.len())?;
        validate_layout(&self.layout, &self.config)?;
        Ok(())
    }

    pub fn display_by_id(&self, id: &str) -> Option<&DisplayDef> {
        self.displays.iter().find(|display| display.id == id)
    }

    /// The bench used by all single-bench logic until multi-bench lands.
    pub fn primary_bench(&self) -> &BenchDef {
        &self.layout.benches[0]
    }
}

fn validate_config(config: &GameConfig) -> Result<(), String> {
    for (name, value) in [
        ("room_width", config.room_width),
        ("room_height", config.room_height),
        ("shift_seconds", config.shift_seconds),
        ("player_speed", config.player_speed),
        ("interaction_radius", config.interaction_radius),
        ("mistake_penalty_seconds", config.mistake_penalty_seconds),
        ("spatial_cell_size", config.spatial_cell_size),
        ("toy_render_distance", config.toy_render_distance),
        ("toy_lod_distance", config.toy_lod_distance),
        ("toy_pose_distance", config.toy_pose_distance),
        ("toy_always_draw_radius", config.toy_always_draw_radius),
        ("fov_min_degrees", config.fov_min_degrees),
        ("fov_max_degrees", config.fov_max_degrees),
        ("fov_step_degrees", config.fov_step_degrees),
        ("sensitivity_min", config.sensitivity_min),
        ("sensitivity_max", config.sensitivity_max),
        ("sensitivity_step", config.sensitivity_step),
        ("ui_scale_min", config.ui_scale_min),
        ("ui_scale_max", config.ui_scale_max),
        ("ui_scale_step", config.ui_scale_step),
    ] {
        if !value.is_finite() || value <= 0.0 {
            return Err(format!("game_config.json {name} must be positive"));
        }
    }
    if config.game_name.trim().is_empty()
        || config.display_name.trim().is_empty()
        || config.save_slot.trim().is_empty()
        || config.records_slot.trim().is_empty()
        || config.version.trim().is_empty()
    {
        return Err("game_config.json names, slots, and version must not be empty".to_owned());
    }
    if config.toy_count == 0 || config.starting_carry_limit == 0 {
        return Err(
            "game_config.json toy_count and starting_carry_limit must be positive".to_owned(),
        );
    }
    if !(0.0..=1.0).contains(&config.broken_fraction) {
        return Err("game_config.json broken_fraction must be between 0 and 1".to_owned());
    }
    if !(config.fov_min_degrees < config.fov_max_degrees
        && config.sensitivity_min < config.sensitivity_max
        && config.ui_scale_min < config.ui_scale_max)
    {
        return Err(
            "game_config.json ranges must have increasing minimums and maximums".to_owned(),
        );
    }
    if !(config.toy_pose_distance < config.toy_lod_distance
        && config.toy_lod_distance < config.toy_render_distance
        && config.toy_always_draw_radius <= config.toy_render_distance)
    {
        return Err("game_config.json render distances must be ordered".to_owned());
    }
    if config.toy_view_cull_min_dot < -1.0 || config.toy_view_cull_min_dot > 1.0 {
        return Err("game_config.json toy_view_cull_min_dot must be between -1 and 1".to_owned());
    }
    Ok(())
}

fn validate_ui_copy(copy: &UiCopy) -> Result<(), String> {
    for (index, step) in copy.tutorial.iter().enumerate() {
        if step.eyebrow.trim().is_empty()
            || step.title.trim().is_empty()
            || step.body.trim().is_empty()
            || step.keys.is_empty()
            || step.keys.iter().any(|key| key.trim().is_empty())
        {
            return Err(format!(
                "ui_copy.json tutorial step {} is incomplete",
                index + 1
            ));
        }
    }
    Ok(())
}

fn validate_displays(displays: &[DisplayDef], config: &GameConfig) -> Result<(), String> {
    validate_unique_ids(
        displays.iter().map(|display| display.id.as_str()),
        "display",
    )?;
    if displays.is_empty() {
        return Err("displays.json must define at least one display".to_owned());
    }
    for display in displays {
        if display.id.trim().is_empty()
            || display.name.trim().is_empty()
            || display.theme.trim().is_empty()
            || display.symbol.trim().is_empty()
        {
            return Err(
                "displays.json IDs, names, themes, and symbols must not be empty".to_owned(),
            );
        }
        if display.capacity == 0 {
            return Err(format!("display {} capacity must be positive", display.id));
        }
        validate_rect(
            display.x,
            display.y,
            display.w,
            display.h,
            config,
            &format!("display {}", display.id),
        )?;
        validate_accent(display.accent, &format!("display {}", display.id))?;
    }
    let total_capacity: usize = displays.iter().map(|display| display.capacity).sum();
    if total_capacity != config.toy_count {
        return Err(format!(
            "displays.json capacity total {} must equal game_config toy_count {}",
            total_capacity, config.toy_count
        ));
    }
    Ok(())
}

fn validate_upgrades(upgrades: &[UpgradeDef], display_count: usize) -> Result<(), String> {
    validate_unique_ids(
        upgrades.iter().map(|upgrade| upgrade.id.as_str()),
        "upgrade",
    )?;
    for upgrade in upgrades {
        if upgrade.id.trim().is_empty()
            || upgrade.name.trim().is_empty()
            || upgrade.description.trim().is_empty()
        {
            return Err("upgrades.json IDs, names, and descriptions must not be empty".to_owned());
        }
        if upgrade.unlock_completed_displays > display_count || upgrade.cost == 0 {
            return Err(format!(
                "upgrade {} has an invalid unlock or cost",
                upgrade.id
            ));
        }
        match upgrade.effect {
            UpgradeEffect::Scanner => {}
            UpgradeEffect::CarryLimit { toys } if toys > 0 => {}
            UpgradeEffect::Speed { multiplier } | UpgradeEffect::Reach { multiplier }
                if multiplier.is_finite() && multiplier > 0.0 => {}
            UpgradeEffect::MistakeForgiveness { mistakes } if mistakes > 0 => {}
            _ => return Err(format!("upgrade {} has a non-positive effect", upgrade.id)),
        }
    }
    Ok(())
}

fn validate_layout(layout: &LayoutData, config: &GameConfig) -> Result<(), String> {
    if layout.benches.is_empty() {
        return Err("layout.json must define at least one bench".to_owned());
    }
    validate_unique_ids(
        layout.benches.iter().map(|bench| bench.id.as_str()),
        "bench",
    )?;
    validate_unique_ids(layout.zones.iter().map(|zone| zone.name.as_str()), "zone")?;
    for bench in &layout.benches {
        if bench.id.trim().is_empty() || bench.capacity == 0 || bench.radius <= 0.0 {
            return Err(format!(
                "bench {} has invalid identity or capacity",
                bench.id
            ));
        }
        validate_rect(
            bench.x,
            bench.y,
            bench.w,
            bench.h,
            config,
            &format!("bench {}", bench.id),
        )?;
    }
    for zone in &layout.zones {
        if zone.name.trim().is_empty() {
            return Err("layout.json zone names must not be empty".to_owned());
        }
        validate_rect(
            zone.x,
            zone.y,
            zone.w,
            zone.h,
            config,
            &format!("zone {}", zone.name),
        )?;
        validate_accent(zone.accent, &format!("zone {}", zone.name))?;
    }
    for poster in &layout.posters {
        if !matches!(poster.wall.as_str(), "front" | "back" | "west" | "east")
            || poster.text.trim().is_empty()
            || poster.width <= 0.0
        {
            return Err("layout.json posters must have a valid wall, text, and width".to_owned());
        }
        validate_accent(poster.accent, "poster")?;
    }
    Ok(())
}

fn validate_unique_ids<'a>(ids: impl Iterator<Item = &'a str>, kind: &str) -> Result<(), String> {
    let mut seen = std::collections::HashSet::new();
    for id in ids {
        if !seen.insert(id) {
            return Err(format!("duplicate {kind} id or name: {id}"));
        }
    }
    Ok(())
}

fn validate_rect(
    x: f32,
    y: f32,
    w: f32,
    h: f32,
    config: &GameConfig,
    label: &str,
) -> Result<(), String> {
    if ![x, y, w, h].iter().all(|value| value.is_finite()) || w <= 0.0 || h <= 0.0 {
        return Err(format!("{label} must have positive finite dimensions"));
    }
    if x < 0.0 || y < 0.0 || x + w > config.room_width || y + h > config.room_height {
        return Err(format!("{label} must fit inside the configured room"));
    }
    Ok(())
}

fn validate_accent(accent: [f32; 4], label: &str) -> Result<(), String> {
    if !accent
        .iter()
        .all(|value| value.is_finite() && (0.0..=1.0).contains(value))
    {
        return Err(format!(
            "{label} accent must contain channels between 0 and 1"
        ));
    }
    Ok(())
}
