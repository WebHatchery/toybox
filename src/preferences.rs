//! Toybox-specific preferences that do not belong in the shared display and
//! audio settings model.

use crate::data::GameConfig;
use macroquad_toolkit::persistence::{load_json_key, save_json_key};
use serde::{Deserialize, Serialize};

const PREFERENCES_KEY: &str = "toybox_preferences";

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct ToyboxPreferences {
    pub fov_degrees: f32,
    pub mouse_sensitivity: f32,
    pub high_contrast: bool,
    pub tutorial_complete: bool,
}

impl Default for ToyboxPreferences {
    fn default() -> Self {
        Self {
            fov_degrees: 85.0,
            mouse_sensitivity: 1.0,
            high_contrast: false,
            tutorial_complete: false,
        }
    }
}

impl ToyboxPreferences {
    pub fn load(game_name: &str, config: &GameConfig) -> Self {
        let mut preferences: Self = load_json_key(game_name, PREFERENCES_KEY).unwrap_or_default();
        preferences.sanitize_with_config(config);
        preferences
    }

    pub fn save(&self, game_name: &str) -> Result<(), String> {
        save_json_key(game_name, PREFERENCES_KEY, self)
    }

    pub fn sanitize_with_config(&mut self, config: &GameConfig) {
        self.fov_degrees = self
            .fov_degrees
            .clamp(config.fov_min_degrees, config.fov_max_degrees);
        self.mouse_sensitivity = self
            .mouse_sensitivity
            .clamp(config.sensitivity_min, config.sensitivity_max);
    }
}
