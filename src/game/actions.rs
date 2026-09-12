//! Player actions and the notifications that acknowledge them.

use super::*;

impl Game {
    /// `R` restarts, and it must not silently move a relaxed player onto the
    /// clock — so it re-runs whichever mode is already in play.
    pub(super) fn start_shift(&mut self, mode: ShiftMode) {
        let seed = match mode {
            ShiftMode::Timed => CLOSING_SHIFT_SEED,
            ShiftMode::Relaxed => self.next_relaxed_seed(),
        };
        self.start_shift_with_seed(mode, seed);
    }

    fn start_shift_with_seed(&mut self, mode: ShiftMode, seed: u64) {
        self.session = GameSession::new_with_seed(&self.data, seed);
        self.session.shift_mode = mode;
        self.recorded_run = false;
        self.beat_record = false;
        self.tutorial = TutorialProgress::new(!self.preferences.tutorial_complete);
        self.warned_five_minutes = false;
        self.warned_one_minute = false;
        self.audio.start_ambience();
        self.screen = GameScreen::Playing;
        match mode {
            ShiftMode::Timed => self.notifications.info(format!(
                "Closing shift: {} until opening",
                format_mmss(self.data.config.shift_seconds)
            )),
            ShiftMode::Relaxed => self.notifications.info(format!(
                "Relaxed run: no deadline - layout {}",
                ui::shift_seed_code(seed)
            )),
        }
    }

    fn next_relaxed_seed(&mut self) -> u64 {
        // SplitMix64 is used as a one-way mixer, not a gameplay RNG. The clock
        // starts the nonce and the increment guarantees fresh successive runs.
        self.relaxed_seed_nonce = self.relaxed_seed_nonce.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut value = self.relaxed_seed_nonce;
        value = (value ^ (value >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        value = (value ^ (value >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        value ^= value >> 31;
        if value == CLOSING_SHIFT_SEED {
            value ^ 0xA5A5_A5A5_A5A5_A5A5
        } else {
            value
        }
    }

    pub(super) fn apply_action(&mut self, action: UiAction) {
        match action {
            UiAction::NewGame => self.start_shift(ShiftMode::Timed),
            UiAction::NewRelaxedGame => self.start_shift(ShiftMode::Relaxed),
            UiAction::ReplayShiftSeed => {
                let mode = self.session.shift_mode;
                let seed = self.session.shift_seed;
                self.start_shift_with_seed(mode, seed);
                self.notifications
                    .success(format!("Replaying layout {}", ui::shift_seed_code(seed)));
            }
            UiAction::Continue => {
                if self.load_game() {
                    self.sync_closing_warnings();
                    self.audio.start_ambience();
                    self.screen = GameScreen::Playing;
                }
            }
            UiAction::Settings => {
                self.settings_from_game = self.screen == GameScreen::Playing;
                self.screen = GameScreen::Settings;
            }
            UiAction::CloseSettings => {
                self.screen = if self.settings_from_game {
                    GameScreen::Playing
                } else {
                    GameScreen::Title
                };
                self.settings_from_game = false;
            }
            UiAction::BackToTitle => {
                self.save_game();
                self.audio.stop_ambience();
                self.settings_from_game = false;
                self.screen = GameScreen::Title;
                self.has_save_file =
                    slot_exists(&self.data.config.game_name, &self.data.config.save_slot);
            }
            UiAction::OpenToolShop => {
                self.tutorial.opened_tools();
                self.screen = GameScreen::ToolShop;
            }
            UiAction::MoveForward => self.touch_movement.y += 1.0,
            UiAction::MoveBackward => self.touch_movement.y -= 1.0,
            UiAction::MoveLeft => self.touch_movement.x -= 1.0,
            UiAction::MoveRight => self.touch_movement.x += 1.0,
            UiAction::LookUp => self.touch_look.y += 1.0,
            UiAction::LookDown => self.touch_look.y -= 1.0,
            UiAction::LookLeft => self.touch_look.x -= 1.0,
            UiAction::LookRight => self.touch_look.x += 1.0,
            UiAction::CloseToolShop => self.screen = GameScreen::Playing,
            UiAction::ToggleFullscreen => {
                self.settings.toggle_fullscreen();
                if let Err(err) = self.settings.save(&self.data.config.game_name) {
                    eprintln!("Failed to save settings: {err}");
                }
            }
            UiAction::FovIncrease => {
                self.preferences.fov_degrees =
                    (self.preferences.fov_degrees + self.data.config.fov_step_degrees).clamp(
                        self.data.config.fov_min_degrees,
                        self.data.config.fov_max_degrees,
                    );
                self.save_preferences();
            }
            UiAction::FovDecrease => {
                self.preferences.fov_degrees =
                    (self.preferences.fov_degrees - self.data.config.fov_step_degrees).clamp(
                        self.data.config.fov_min_degrees,
                        self.data.config.fov_max_degrees,
                    );
                self.save_preferences();
            }
            UiAction::SensitivityIncrease => {
                self.preferences.mouse_sensitivity =
                    (self.preferences.mouse_sensitivity + self.data.config.sensitivity_step).clamp(
                        self.data.config.sensitivity_min,
                        self.data.config.sensitivity_max,
                    );
                self.save_preferences();
            }
            UiAction::SensitivityDecrease => {
                self.preferences.mouse_sensitivity =
                    (self.preferences.mouse_sensitivity - self.data.config.sensitivity_step).clamp(
                        self.data.config.sensitivity_min,
                        self.data.config.sensitivity_max,
                    );
                self.save_preferences();
            }
            UiAction::UiScaleIncrease => {
                self.settings.ui_text_scale = (self.settings.ui_text_scale
                    + self.data.config.ui_scale_step)
                    .clamp(self.data.config.ui_scale_min, self.data.config.ui_scale_max);
                self.save_shared_settings();
            }
            UiAction::UiScaleDecrease => {
                self.settings.ui_text_scale = (self.settings.ui_text_scale
                    - self.data.config.ui_scale_step)
                    .clamp(self.data.config.ui_scale_min, self.data.config.ui_scale_max);
                self.save_shared_settings();
            }
            UiAction::ToggleHighContrast => {
                self.preferences.high_contrast = !self.preferences.high_contrast;
                self.save_preferences();
            }
            UiAction::MasterVolumeIncrease => {
                self.settings.master_volume = (self.settings.master_volume + 0.1).clamp(0.0, 1.0);
                self.apply_audio_volumes();
                self.save_shared_settings();
            }
            UiAction::MasterVolumeDecrease => {
                self.settings.master_volume = (self.settings.master_volume - 0.1).clamp(0.0, 1.0);
                self.apply_audio_volumes();
                self.save_shared_settings();
            }
            UiAction::EffectsVolumeIncrease => {
                self.settings.sfx_volume = (self.settings.sfx_volume + 0.1).clamp(0.0, 1.0);
                self.apply_audio_volumes();
                self.save_shared_settings();
                self.audio.play(Cue::Pickup);
            }
            UiAction::EffectsVolumeDecrease => {
                self.settings.sfx_volume = (self.settings.sfx_volume - 0.1).clamp(0.0, 1.0);
                self.apply_audio_volumes();
                self.save_shared_settings();
                self.audio.play(Cue::Pickup);
            }
            UiAction::AmbienceVolumeIncrease => {
                self.settings.music_volume = (self.settings.music_volume + 0.1).clamp(0.0, 1.0);
                self.apply_audio_volumes();
                self.save_shared_settings();
            }
            UiAction::AmbienceVolumeDecrease => {
                self.settings.music_volume = (self.settings.music_volume - 0.1).clamp(0.0, 1.0);
                self.apply_audio_volumes();
                self.save_shared_settings();
            }
            UiAction::OpenHelp => self.screen = GameScreen::Help,
            UiAction::CloseHelp => self.screen = GameScreen::Settings,
            UiAction::ReplayTutorial => {
                self.preferences.tutorial_complete = false;
                self.tutorial = TutorialProgress::new(true);
                self.save_preferences();
                self.notifications
                    .success("First-shift guide will appear when you resume play");
            }
            UiAction::QuitGame => {
                self.audio.stop_ambience();
                quit();
            }
            UiAction::Save => self.save_game(),
            UiAction::Load => {
                if self.load_game() {
                    self.sync_closing_warnings();
                }
            }
            UiAction::Interact => self.handle_interaction(),
            UiAction::CycleCarry => {
                let had_multiple = self.session.player.carried_toy_ids.len() > 1;
                self.session.cycle_carried();
                self.tutorial.cycled_trolley(had_multiple);
                if had_multiple {
                    self.audio.play_at(Cue::Pickup, 0.45);
                }
            }
            UiAction::DropActive => {
                if let Some(toy_name) = self.session.drop_active(&self.data) {
                    self.audio.play(Cue::Drop);
                    self.notifications
                        .info(format!("Placed {} on the floor", toy_name));
                }
            }
            UiAction::SkipTutorial => {
                self.tutorial.skip();
                self.preferences.tutorial_complete = true;
                self.save_preferences();
                self.notifications
                    .info("First-shift guide hidden. Replay it from Settings");
            }
            UiAction::BuyTool(tool_id) => self.handle_tool_purchase(&tool_id),
            UiAction::BuyStockroomSpotlight => self.handle_stockroom_spotlight_purchase(),
        }
    }

    fn handle_interaction(&mut self) {
        let result = self.session.interact(&self.data);
        self.tutorial.observe_interaction(&result);
        match result {
            InteractionResult::PickedUp { toy_name } => {
                self.audio.play(Cue::Pickup);
                self.notifications.info(format!("Picked up {}", toy_name));
            }
            InteractionResult::Dropped { toy_name } => {
                self.audio.play(Cue::Drop);
                self.notifications
                    .info(format!("Placed {} on the floor", toy_name));
            }
            InteractionResult::Placed {
                toy_name,
                display_name,
                was_wrong,
                completed_display,
                completed_zone,
                available_tools,
                finished,
            } => {
                if finished {
                    self.audio.play(Cue::Restored);
                } else if completed_display.is_some() {
                    self.audio.play(Cue::DisplayComplete);
                } else if was_wrong {
                    self.audio.play(Cue::ShelfWrong);
                } else {
                    self.audio.play(Cue::ShelfCorrect);
                }
                if was_wrong {
                    self.notifications
                        .warning(format!("{} does not belong in {}", toy_name, display_name));
                } else {
                    self.notifications
                        .success(format!("Placed {} in {}", toy_name, display_name));
                }
                if let Some(name) = completed_display {
                    self.notifications.success(format!("Completed {}", name));
                }
                if let Some(name) = completed_zone {
                    self.notifications
                        .success(format!("{} is fully restored", name));
                }
                for tool_name in available_tools {
                    self.notifications
                        .info(format!("Tool available: {} (open TOOLS)", tool_name));
                }
                if finished {
                    self.notifications.success("Store restored before opening");
                }
            }
            InteractionResult::PlacementPrevented {
                toy_name,
                display_name,
                guards_remaining,
            } => {
                self.audio.play(Cue::ShelfWrong);
                self.notifications.warning(format!(
                    "Manager stopped {} at {}. {} checks left",
                    toy_name, display_name, guards_remaining
                ));
            }
            InteractionResult::PlacedOnRepairBench { toy_name } => {
                self.audio.play(Cue::Drop);
                self.notifications
                    .info(format!("Placed {} on the repair bench", toy_name));
            }
            InteractionResult::Repaired { toy_name } => {
                self.audio.play(Cue::Repair);
                self.notifications
                    .success(format!("Repaired {}. Ready for display", toy_name));
            }
            InteractionResult::NeedsRepair { toy_name } => self
                .notifications
                .warning(format!("Repair {} before shelving it", toy_name)),
            InteractionResult::NeedsRepairParts { toy_name } => self
                .notifications
                .warning(format!("Find the matching part for {}", toy_name)),
            InteractionResult::InventoryFull => {
                let message = match self.session.carry_tool_name(&self.data) {
                    Some(tool) => format!("{} is full", tool),
                    None => "Hands full".to_owned(),
                };
                self.notifications.warning(message);
            }
            InteractionResult::RepairBenchFull => {
                self.notifications.warning("Repair bench is full")
            }
            InteractionResult::RepairMismatch => self
                .notifications
                .warning("Those repair parts do not match"),
            InteractionResult::ShelfFull => self.notifications.warning("That shelf is full"),
            InteractionResult::ShelfSlotUnavailable => {
                self.notifications.warning("Look at an empty shelf spot")
            }
            InteractionResult::NothingNearby => {
                self.notifications.info("Move closer to a toy or display");
            }
        }
    }

    fn handle_tool_purchase(&mut self, tool_id: &str) {
        match self.session.purchase_tool(&self.data, tool_id) {
            ToolPurchaseResult::Purchased {
                tool_name,
                remaining_credits,
            } => {
                self.audio.play(Cue::ToolPurchase);
                self.notifications.success(format!(
                    "Purchased {}. Tool credits left: {}",
                    tool_name, remaining_credits
                ));
            }
            ToolPurchaseResult::NeedMoreCredits {
                tool_name,
                cost,
                available_credits,
            } => self.notifications.warning(format!(
                "{} needs {}. You have {}",
                tool_name,
                ui::credits_phrase(cost),
                available_credits
            )),
            ToolPurchaseResult::AlreadyOwned { tool_name } => self
                .notifications
                .info(format!("{} already owned", tool_name)),
            ToolPurchaseResult::Locked {
                tool_name,
                required_displays,
                completed_displays,
            } => self.notifications.warning(format!(
                "{} unlocks at {}/{} restored displays",
                tool_name, completed_displays, required_displays
            )),
            ToolPurchaseResult::NoToolsAvailable => {
                self.notifications.info("No tools available to buy")
            }
            ToolPurchaseResult::ServicePurchased { .. }
            | ToolPurchaseResult::ServiceAtCapacity { .. } => unreachable!("permanent purchase"),
        }
    }

    fn handle_stockroom_spotlight_purchase(&mut self) {
        match self.session.purchase_stockroom_spotlight(&self.data) {
            ToolPurchaseResult::ServicePurchased {
                service_name,
                seconds_active,
                remaining_credits,
            } => {
                self.audio.play(Cue::ToolPurchase);
                self.notifications.success(format!(
                    "{} active for {:.0}s. Tool credits left: {}",
                    service_name, seconds_active, remaining_credits
                ));
            }
            ToolPurchaseResult::NeedMoreCredits {
                cost,
                available_credits,
                ..
            } => self.notifications.warning(format!(
                "Spotlight needs {}. You have {}",
                ui::credits_phrase(cost),
                available_credits
            )),
            ToolPurchaseResult::ServiceAtCapacity {
                service_name,
                seconds_active,
            } => self.notifications.info(format!(
                "{} already has {:.0}s queued",
                service_name, seconds_active
            )),
            _ => self
                .notifications
                .info("Buy every shift tool before calling the spotlight"),
        }
    }
}
