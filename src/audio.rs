//! Procedural shop audio. The store already draws every toy from primitives;
//! its sound follows the same rule and renders a small deterministic cue set at
//! startup rather than shipping sampled files.

use macroquad::audio::PlaySoundParams;
use macroquad_toolkit::audio::SoundManager;
use macroquad_toolkit::synth::{render_wav, SynthConfig, Voice, Wave};
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Cue {
    Footstep,
    Pickup,
    Drop,
    ShelfCorrect,
    ShelfWrong,
    Repair,
    ToolPurchase,
    DisplayComplete,
    ClosingWarning,
    Restored,
}

impl Cue {
    pub const ALL: [Cue; 10] = [
        Cue::Footstep,
        Cue::Pickup,
        Cue::Drop,
        Cue::ShelfCorrect,
        Cue::ShelfWrong,
        Cue::Repair,
        Cue::ToolPurchase,
        Cue::DisplayComplete,
        Cue::ClosingWarning,
        Cue::Restored,
    ];

    pub fn cooldown(self) -> f32 {
        match self {
            Cue::Footstep => 0.34,
            Cue::Pickup | Cue::Drop | Cue::ShelfCorrect | Cue::ShelfWrong => 0.08,
            Cue::Repair | Cue::ToolPurchase => 0.18,
            Cue::DisplayComplete => 0.45,
            Cue::ClosingWarning => 2.0,
            Cue::Restored => 4.0,
        }
    }
}

pub fn config() -> SynthConfig {
    SynthConfig {
        sample_rate: 22_050,
        master_gain: 0.28,
    }
}

pub fn voices_for(cue: Cue) -> Vec<Voice> {
    match cue {
        Cue::Footstep => vec![
            Voice::tone(0.0, 0.075, 120.0, 0.16)
                .wave(Wave::Noise)
                .attack(0.01),
            Voice::tone(0.0, 0.10, 86.0, 0.10).wave(Wave::Sine),
        ],
        Cue::Pickup => vec![
            Voice::tone(0.0, 0.09, 440.0, 0.20)
                .glide(660.0)
                .wave(Wave::Triangle),
            Voice::tone(0.0, 0.035, 1800.0, 0.08).wave(Wave::Noise),
        ],
        Cue::Drop => vec![
            Voice::tone(0.0, 0.08, 160.0, 0.20)
                .wave(Wave::Noise)
                .attack(0.01),
            Voice::tone(0.0, 0.13, 105.0, 0.12).wave(Wave::Sine),
        ],
        Cue::ShelfCorrect => vec![
            Voice::tone(0.0, 0.11, 659.0, 0.18).wave(Wave::Triangle),
            Voice::tone(0.07, 0.16, 880.0, 0.16).wave(Wave::Sine),
        ],
        Cue::ShelfWrong => vec![
            Voice::tone(0.0, 0.23, 240.0, 0.20)
                .glide(135.0)
                .wave(Wave::Square),
            Voice::tone(0.0, 0.10, 105.0, 0.13).wave(Wave::Sine),
        ],
        Cue::Repair => vec![
            Voice::tone(0.0, 0.24, 523.0, 0.15).wave(Wave::Sine),
            Voice::tone(0.07, 0.27, 659.0, 0.15).wave(Wave::Triangle),
            Voice::tone(0.14, 0.34, 784.0, 0.14).wave(Wave::Sine),
        ],
        Cue::ToolPurchase => vec![
            Voice::tone(0.0, 0.06, 1046.0, 0.14).wave(Wave::Triangle),
            Voice::tone(0.05, 0.07, 1318.0, 0.13).wave(Wave::Triangle),
            Voice::tone(0.11, 0.18, 1568.0, 0.12).wave(Wave::Sine),
        ],
        Cue::DisplayComplete => vec![
            Voice::tone(0.0, 0.12, 659.0, 0.16).wave(Wave::Triangle),
            Voice::tone(0.10, 0.14, 880.0, 0.17).wave(Wave::Triangle),
            Voice::tone(0.22, 0.30, 1046.0, 0.15).wave(Wave::Sine),
        ],
        Cue::ClosingWarning => vec![
            Voice::tone(0.0, 0.22, 440.0, 0.18).wave(Wave::Triangle),
            Voice::tone(0.28, 0.28, 330.0, 0.18).wave(Wave::Triangle),
        ],
        Cue::Restored => vec![
            Voice::tone(0.0, 0.24, 523.0, 0.16).wave(Wave::Triangle),
            Voice::tone(0.16, 0.28, 659.0, 0.16).wave(Wave::Triangle),
            Voice::tone(0.34, 0.34, 784.0, 0.16).wave(Wave::Triangle),
            Voice::tone(0.56, 0.58, 1046.0, 0.15).wave(Wave::Sine),
        ],
    }
}

pub fn ambience_voices() -> Vec<Voice> {
    let mut voices = Vec::new();
    for (start, root, bell) in [
        (0.0, 196.0, 523.0),
        (2.0, 220.0, 587.0),
        (4.0, 174.6, 493.9),
        (6.0, 196.0, 659.0),
    ] {
        voices.push(Voice::tone(start, 2.0, root, 0.10).wave(Wave::Sine));
        voices.push(
            Voice::tone(start + 0.22, 1.45, bell, 0.035)
                .wave(Wave::Triangle)
                .attack(0.08),
        );
    }
    voices
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
enum SoundKey {
    Cue(Cue),
    Ambience,
}

pub struct AudioDirector {
    cues: SoundManager<SoundKey>,
    cooldowns: HashMap<Cue, f32>,
    ambience: bool,
    effects_volume: f32,
    ambience_volume: f32,
    ambience_started: bool,
    enabled: bool,
}

impl AudioDirector {
    pub fn silent() -> Self {
        Self {
            cues: SoundManager::new(),
            cooldowns: HashMap::new(),
            ambience: false,
            effects_volume: 0.0,
            ambience_volume: 0.0,
            ambience_started: false,
            enabled: false,
        }
    }

    pub async fn load(effects_volume: f32, ambience_volume: f32) -> Self {
        let mut cues = SoundManager::new();
        for (index, cue) in Cue::ALL.into_iter().enumerate() {
            let bytes = render_wav(&voices_for(cue), &config(), 0x70B0_0000 + index as u64);
            let _ = cues.load_sound_bytes(SoundKey::Cue(cue), &bytes).await;
        }
        let ambience_bytes = render_wav(&ambience_voices(), &config(), 0x70B0_A11C);
        let ambience = cues
            .load_sound_bytes(SoundKey::Ambience, &ambience_bytes)
            .await
            .is_ok();
        Self {
            cues,
            cooldowns: HashMap::new(),
            ambience,
            effects_volume: effects_volume.clamp(0.0, 1.0),
            ambience_volume: ambience_volume.clamp(0.0, 1.0),
            ambience_started: false,
            enabled: true,
        }
    }

    pub fn update(&mut self, dt: f32) {
        for seconds in self.cooldowns.values_mut() {
            *seconds = (*seconds - dt).max(0.0);
        }
    }

    pub fn set_volumes(&mut self, effects: f32, ambience: f32) {
        self.effects_volume = effects.clamp(0.0, 1.0);
        self.ambience_volume = ambience.clamp(0.0, 1.0);
        if self.ambience {
            self.cues
                .set_raw_volume(SoundKey::Ambience, self.ambience_volume);
        }
    }

    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    pub fn start_ambience(&mut self) {
        if !self.enabled || self.ambience_started {
            return;
        }
        if !self.ambience {
            return;
        }
        self.cues.play_raw(
            SoundKey::Ambience,
            PlaySoundParams {
                looped: true,
                volume: self.ambience_volume,
            },
        );
        self.ambience_started = true;
    }

    pub fn stop_ambience(&mut self) {
        if self.ambience {
            self.cues.stop_raw(SoundKey::Ambience);
        }
        self.ambience_started = false;
    }

    pub fn play(&mut self, cue: Cue) {
        self.play_at(cue, 1.0);
    }

    pub fn play_at(&mut self, cue: Cue, gain: f32) {
        if !self.channel_allows(cue) {
            return;
        }
        if !self.cues.has_sound(SoundKey::Cue(cue)) {
            return;
        }
        self.cues.play_raw(
            SoundKey::Cue(cue),
            PlaySoundParams {
                looped: false,
                volume: (self.effects_volume * gain).clamp(0.0, 1.0),
            },
        );
        self.cooldowns.insert(cue, cue.cooldown());
    }

    pub fn channel_allows(&self, cue: Cue) -> bool {
        self.enabled
            && self.effects_volume > 0.0
            && self.cooldowns.get(&cue).copied().unwrap_or(0.0) <= 0.0
    }

    pub fn cooldown_count(&self) -> usize {
        self.cooldowns.len()
    }

    pub fn is_enabled(&self) -> bool {
        self.enabled
    }
}
