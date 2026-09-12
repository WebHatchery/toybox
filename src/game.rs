//! High-level game loop, state transitions, and toolkit integration.

use crate::audio::{AudioDirector, Cue};
use crate::capture_scenes;
use crate::data::GameData;
use crate::gallery::GalleryScene;
use crate::preferences::ToyboxPreferences;
use crate::state::{
    migrate_save_value, BestRuns, GamePhase, GameSession, InteractionResult, SaveData, ShiftMode,
    ShiftRecord, ToolPurchaseResult, CLOSING_SHIFT_SEED,
};
use crate::tutorial::TutorialProgress;
use crate::ui::{self, DebugOverlay, UiAction, UiContext};
use macroquad::miniquad::window::quit;
use macroquad::prelude::*;
use macroquad_toolkit::assets::{load_texture_from_pack_or_file, AssetPack};
use macroquad_toolkit::events::EventBus;
use macroquad_toolkit::notifications::{
    NotificationAnchor, NotificationManager, NotificationRenderConfig,
};
use macroquad_toolkit::persistence::{
    load_from_slot_with_migration, save_to_slot_with_version, slot_exists,
};
use macroquad_toolkit::prelude::dark;
use macroquad_toolkit::settings::GameSettings;
use macroquad_toolkit::ui::format_mmss;

mod actions;
mod update;

const TITLE_TEXTURE_PATH: &str = "assets/toybox_title.png";
const ASSET_PACK_PATH: &str = "assets.zip";

pub struct Game {
    data: GameData,
    session: GameSession,
    title_texture: Option<Texture2D>,
    notifications: NotificationManager,
    events: EventBus<UiAction>,
    screen: GameScreen,
    has_save_file: bool,
    settings: GameSettings,
    preferences: ToyboxPreferences,
    tutorial: TutorialProgress,
    audio: AudioDirector,
    warned_five_minutes: bool,
    warned_one_minute: bool,
    settings_from_game: bool,
    debug_overlay: DebugOverlay,
    bench: Option<BenchMode>,
    gallery: Option<GalleryScene>,
    best_runs: BestRuns,
    /// Whether the run now on screen has already been submitted. A finished
    /// shift keeps drawing every frame, so without this the record would be
    /// resubmitted sixty times a second.
    recorded_run: bool,
    /// Set when the finished run beat its mode's record, for the score screen.
    beat_record: bool,
    /// Advances independently of simulation state so two fresh Relaxed Runs
    /// cannot receive the same seed even when started in one clock tick.
    relaxed_seed_nonce: u64,
    touch_movement: Vec2,
    touch_look: Vec2,
}

/// How long the perf probe should run, if it was asked for.
///
/// Read here *and* by `window_conf`, which uncaps vsync for a bench run —
/// otherwise every measurement comes back at the refresh rate and the probe
/// reports the display rather than the game.
pub fn bench_seconds() -> Option<f32> {
    std::env::var("TOYBOX_BENCH_SECONDS")
        .ok()
        .and_then(|raw| raw.parse::<f32>().ok())
        .filter(|seconds| *seconds > 0.0)
}

/// Headless-ish perf probe: set TOYBOX_BENCH_SECONDS=<n> to boot straight
/// into a fresh run, sweep the view for n seconds, print frame stats to
/// stdout, and exit. Used by the roadmap perf gates.
struct BenchMode {
    duration_seconds: f32,
    elapsed_seconds: f32,
    frames: u32,
    worst_frame_seconds: f32,
    slow_frames: u32,
    measured_frames: u32,
}

/// Frames to let settle before the stutter counters start.
///
/// Shader compilation, texture upload and the first draw of every model land in
/// the opening frames. They are real costs, but they are startup, not stutter —
/// and one 90ms first frame otherwise sets `worst_frame_ms` for the whole run
/// and hides everything that happens afterwards.
const BENCH_WARMUP_FRAMES: u32 = 60;

/// A frame slower than this is one the player can feel at 60Hz.
const BENCH_SLOW_FRAME_SECONDS: f32 = 1.0 / 60.0;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum GameScreen {
    Title,
    Settings,
    Help,
    ToolShop,
    Playing,
}

impl Game {
    pub async fn new() -> Self {
        let data = GameData::load().unwrap_or_else(|err| {
            panic!("Toybox embedded data failed to load: {}", err);
        });
        let asset_pack = AssetPack::load(ASSET_PACK_PATH).await.ok();
        let title_texture = match load_texture_from_pack_or_file(
            asset_pack.as_ref(),
            TITLE_TEXTURE_PATH,
            FilterMode::Linear,
        )
        .await
        {
            Ok(texture) => Some(texture),
            Err(err) => {
                eprintln!("Failed to load title texture '{TITLE_TEXTURE_PATH}': {err}");
                None
            }
        };
        let has_save_file = slot_exists(&data.config.game_name, &data.config.save_slot);
        let notifications = NotificationManager::new();
        let mut settings = GameSettings::load(&data.config.game_name);
        settings.sanitize();
        settings.apply_display();
        let preferences = ToyboxPreferences::load(&data.config.game_name, &data.config);
        let audio_disabled =
            macroquad_toolkit::capture::capture_requested("TOYBOX") || bench_seconds().is_some();
        let audio = if audio_disabled {
            AudioDirector::silent()
        } else {
            AudioDirector::load(
                settings.effective_sfx_volume(),
                settings.effective_music_volume(),
            )
            .await
        };

        let bench = bench_seconds().map(|duration_seconds| BenchMode {
            duration_seconds,
            elapsed_seconds: 0.0,
            frames: 0,
            worst_frame_seconds: 0.0,
            slow_frames: 0,
            measured_frames: 0,
        });
        let screen = if bench.is_some() {
            GameScreen::Playing
        } else {
            GameScreen::Title
        };

        let best_runs = BestRuns::load(
            &data.config.game_name,
            &data.config.records_slot,
            &data.config.version,
        );
        let session = GameSession::new(&data);
        let tutorial = TutorialProgress::new(!preferences.tutorial_complete);
        let relaxed_seed_nonce = macroquad::miniquad::date::now().to_bits();
        Self {
            data,
            session,
            title_texture,
            notifications,
            events: EventBus::new(),
            screen,
            has_save_file,
            settings,
            preferences,
            tutorial,
            audio,
            warned_five_minutes: false,
            warned_one_minute: false,
            settings_from_game: false,
            debug_overlay: DebugOverlay::new(),
            bench,
            gallery: None,
            best_runs,
            recorded_run: false,
            beat_record: false,
            relaxed_seed_nonce,
            touch_movement: Vec2::ZERO,
            touch_look: Vec2::ZERO,
        }
    }

    /// Seed a scene for the screenshot harness.
    pub fn begin_capture_scene(&mut self, scene: &str) {
        // Captures stage the UI state they name. A real profile's tutorial
        // preference must never leak an extra card into unrelated references.
        self.settings = GameSettings::default();
        self.preferences = ToyboxPreferences::default();
        self.tutorial = TutorialProgress::new(false);
        match scene {
            scene if scene.starts_with("toy_gallery:") => {
                let slug = scene.strip_prefix("toy_gallery:").unwrap_or("bear");
                self.gallery = Some(GalleryScene::new(slug));
            }
            "gameplay" => {
                self.session = GameSession::new_with_seed(&self.data, CLOSING_SHIFT_SEED);
                self.screen = GameScreen::Playing;
            }
            "tutorial_first_step" => {
                self.session = GameSession::new_with_seed(&self.data, CLOSING_SHIFT_SEED);
                self.tutorial = TutorialProgress::new(true);
                self.screen = GameScreen::Playing;
            }
            // The two start buttons, the caption that tells them apart, and a
            // record to chase. Set in memory, never saved: a capture must not
            // touch a real player's records file.
            "title" => {
                self.has_save_file = true;
                self.best_runs = capture_scenes::previous_best();
                self.screen = GameScreen::Title;
            }
            // The screen every new player actually meets first: no save to
            // continue, no records to chase. `title` forces both on, so the
            // disabled Continue button and the missing best-run line had never
            // been drawn. `BestRuns::default()` explicitly rather than whatever
            // `Game::new` loaded, so the capture does not depend on whether the
            // machine running it has played the game.
            "title_first_run" => {
                self.has_save_file = false;
                self.best_runs = BestRuns::default();
                self.screen = GameScreen::Title;
            }
            "mid_run" => {
                self.session = capture_scenes::mid_run(&self.data);
                self.screen = GameScreen::Playing;
            }
            "shift_over" => {
                self.session = capture_scenes::shift_over(&self.data);
                // A record to beat, so the score screen's best-run line is
                // exercised rather than reading "no record kept yet". Set here
                // rather than saved, so a capture never touches a real player's
                // records file.
                self.best_runs = capture_scenes::previous_best();
                self.recorded_run = true;
                self.screen = GameScreen::Playing;
            }
            "store_restored" => {
                self.session = capture_scenes::store_restored(&self.data);
                self.best_runs = capture_scenes::previous_best();
                // Submit in memory exactly as `record_finished_run` does, so
                // the panel shows "New best" reporting *this* run rather than
                // the record it just beat — in play the submit happens in
                // `update`, before `draw` reads it back. Never saved: a capture
                // must not touch a real player's records file.
                let summary = self.session.shift_summary(&self.data);
                let run = ShiftRecord::from_summary(&summary, true);
                self.beat_record = self.best_runs.submit(self.session.shift_mode, run);
                self.recorded_run = true;
                self.screen = GameScreen::Playing;
            }
            "carrying_armful" => {
                self.session = capture_scenes::carrying_armful(&self.data);
                self.screen = GameScreen::Playing;
            }
            "carrying_a_half" => {
                self.session = capture_scenes::carrying_a_half(&self.data);
                self.screen = GameScreen::Playing;
            }
            "carrying_a_half_scanned" => {
                self.session = capture_scenes::carrying_a_half_scanned(&self.data);
                self.screen = GameScreen::Playing;
            }
            "broken_lineup" => {
                self.session = capture_scenes::broken_lineup(&self.data);
                self.screen = GameScreen::Playing;
            }
            "closing_soon" => {
                self.session = capture_scenes::closing_soon(&self.data);
                self.screen = GameScreen::Playing;
            }
            "relaxed_run" => {
                self.session = capture_scenes::relaxed_run(&self.data);
                self.screen = GameScreen::Playing;
            }
            "tool_shop" => {
                self.session = capture_scenes::tool_shop(&self.data);
                self.screen = GameScreen::ToolShop;
            }
            "tool_shop_early" => {
                self.session = capture_scenes::tool_shop_early(&self.data);
                self.screen = GameScreen::ToolShop;
            }
            "tool_shop_service" => {
                self.session = capture_scenes::tool_shop_service(&self.data);
                self.screen = GameScreen::ToolShop;
            }
            "lamp_contrast" => {
                self.session = capture_scenes::lamp_contrast(&self.data);
                self.screen = GameScreen::Playing;
            }
            "checkout" => {
                self.session = capture_scenes::checkout(&self.data);
                self.screen = GameScreen::Playing;
            }
            "repair_bench" => {
                self.session = capture_scenes::repair_bench(&self.data);
                self.screen = GameScreen::Playing;
            }
            // The settings panel, and the same panel opened mid-shift as the
            // pause menu. They are not one screen with a different heading: the
            // paused form adds Quit to Title and renames Back to Resume, so its
            // second row is laid out to a different width.
            "repair_bench_ready" => {
                self.session = capture_scenes::repair_bench_ready(&self.data);
                self.screen = GameScreen::Playing;
            }
            "settings" => {
                self.settings_from_game = false;
                self.screen = GameScreen::Settings;
            }
            "settings_muted" => {
                self.settings.master_volume = 0.0;
                self.settings_from_game = false;
                self.screen = GameScreen::Settings;
            }
            "controls" => {
                self.settings_from_game = false;
                self.screen = GameScreen::Help;
            }
            "high_contrast" => {
                self.session = capture_scenes::mid_run(&self.data);
                self.preferences.high_contrast = true;
                self.screen = GameScreen::Playing;
            }
            "large_ui" => {
                self.session = capture_scenes::mid_run(&self.data);
                self.settings.ui_text_scale = self.data.config.ui_scale_max;
                self.screen = GameScreen::Playing;
            }
            "paused" => {
                self.session = capture_scenes::relaxed_run(&self.data);
                self.settings_from_game = true;
                self.screen = GameScreen::Settings;
            }
            _ => {}
        }
    }

    pub fn draw(&mut self) {
        if let Some(gallery) = &self.gallery {
            gallery.draw();
            return;
        }
        clear_background(dark::BACKGROUND);

        ui::set_high_contrast(self.preferences.high_contrast);
        ui::begin_ui_frame(self.settings.ui_text_scale);
        let tutorial_hint = self.tutorial.hint(&self.session, &self.data);
        let actions = match self.screen {
            GameScreen::Title => ui::draw_title_screen(
                self.title_texture.as_ref(),
                self.has_save_file,
                &self.best_runs,
            ),
            GameScreen::Settings => ui::draw_settings_screen(
                self.title_texture.as_ref(),
                ui::SettingsView {
                    fullscreen_enabled: self.settings.fullscreen,
                    fov_degrees: self.preferences.fov_degrees,
                    mouse_sensitivity: self.preferences.mouse_sensitivity,
                    ui_scale: self.settings.ui_text_scale,
                    high_contrast: self.preferences.high_contrast,
                    master_volume: self.settings.master_volume,
                    effects_volume: self.settings.sfx_volume,
                    ambience_volume: self.settings.music_volume,
                    from_game: self.settings_from_game,
                    shift_mode: self.session.shift_mode,
                    shift_seed: self.session.shift_seed,
                },
            ),
            GameScreen::Help => ui::draw_help_screen(self.title_texture.as_ref()),
            GameScreen::ToolShop => {
                let ctx = UiContext {
                    data: &self.data,
                    session: &self.session,
                    fov_degrees: self.preferences.fov_degrees,
                    best_run: self.best_runs.best_for(self.session.shift_mode),
                    beat_record: self.beat_record,
                    tutorial_hint: None,
                };
                ui::draw_tool_shop_screen(ctx)
            }
            GameScreen::Playing => {
                let ctx = UiContext {
                    data: &self.data,
                    session: &self.session,
                    fov_degrees: self.preferences.fov_degrees,
                    best_run: self.best_runs.best_for(self.session.shift_mode),
                    beat_record: self.beat_record,
                    tutorial_hint: tutorial_hint.as_ref(),
                };
                ui::draw_game_ui(ctx, &self.debug_overlay)
            }
        };
        ui::end_ui_frame();

        for action in actions {
            self.events.push(action);
        }

        self.notifications
            .draw_with_config(&NotificationRenderConfig {
                anchor: NotificationAnchor::TopRight,
                ..Default::default()
            });
    }

    fn save_game(&mut self) {
        // One rule, one place: the slot holds a shift you can come back to.
        // `Ctrl+S` is still live at the score screen, so without this a player
        // could write a finished run over a resumable one, and "Continue" would
        // then drop them onto a score screen for a shift that already ended and
        // was already recorded. Said out loud rather than silently ignored,
        // because this path is an explicit keypress.
        if !self.session.phase.should_save_on_leaving() {
            self.notifications
                .info("The shift is over - nothing left to save");
            return;
        }

        let save = self.session.to_save(&self.data.config.version);
        match save_to_slot_with_version(
            &self.data.config.game_name,
            &self.data.config.save_slot,
            &save,
            &self.data.config.version,
        ) {
            Ok(()) => {
                self.has_save_file = true;
                self.notifications.success("Cleanup saved");
            }
            Err(err) => self.notifications.danger(format!("Save failed: {}", err)),
        }
    }

    fn load_game(&mut self) -> bool {
        let loaded: Result<SaveData, String> = load_from_slot_with_migration(
            &self.data.config.game_name,
            &self.data.config.save_slot,
            &self.data.config.version,
            |version, value| migrate_save_value(version, value, &self.data),
        );

        match loaded {
            Ok(save) => {
                self.session = GameSession::from_save(save, &self.data);
                self.recorded_run = false;
                self.beat_record = false;
                self.has_save_file = true;
                self.notifications.success("Loaded cleanup save");
                true
            }
            Err(err) => {
                self.has_save_file =
                    slot_exists(&self.data.config.game_name, &self.data.config.save_slot);
                self.notifications.warning(format!("Load failed: {}", err));
                false
            }
        }
    }

    fn save_preferences(&mut self) {
        if let Err(err) = self.preferences.save(&self.data.config.game_name) {
            self.notifications
                .warning(format!("Could not save preferences: {err}"));
        }
    }

    fn save_shared_settings(&mut self) {
        if let Err(err) = self.settings.save(&self.data.config.game_name) {
            self.notifications
                .warning(format!("Could not save settings: {err}"));
        }
    }

    fn apply_audio_volumes(&mut self) {
        self.audio.set_volumes(
            self.settings.effective_sfx_volume(),
            self.settings.effective_music_volume(),
        );
    }

    fn sync_closing_warnings(&mut self) {
        let remaining = self.session.shift_remaining(&self.data);
        self.warned_five_minutes = remaining <= 300.0;
        self.warned_one_minute = remaining <= 60.0;
    }

    fn finish_tutorial_if_ready(&mut self) {
        if !self.tutorial.is_active() || !self.tutorial.is_complete() {
            return;
        }
        self.tutorial.skip();
        self.preferences.tutorial_complete = true;
        self.save_preferences();
        self.notifications
            .success("First shift learned. Replay the guide from Settings anytime");
    }
}

fn is_control_down() -> bool {
    is_key_down(KeyCode::LeftControl) || is_key_down(KeyCode::RightControl)
}
