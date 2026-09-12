use macroquad_toolkit::synth::render_waveform;
use toybox_after_hours::audio::{ambience_voices, config, voices_for, AudioDirector, Cue};

#[test]
fn every_cue_renders_an_audible_non_clipping_wave() {
    for (index, cue) in Cue::ALL.into_iter().enumerate() {
        let wave = render_waveform(&voices_for(cue), &config(), index as u64);
        let peak = wave
            .iter()
            .fold(0.0f32, |peak, sample| peak.max(sample.abs()));
        assert!(peak > 0.01, "{cue:?} rendered inaudibly at {peak}");
        assert!(peak <= 1.0, "{cue:?} clipped at {peak}");
    }
}

#[test]
fn common_sorting_cues_are_short_and_rate_limited() {
    for cue in [Cue::Footstep, Cue::Pickup, Cue::Drop, Cue::ShelfCorrect] {
        let end = voices_for(cue)
            .iter()
            .map(|voice| voice.start() + voice.duration())
            .fold(0.0f32, f32::max);
        assert!(end <= 0.25, "{cue:?} lasts {end}s");
        assert!(cue.cooldown() > 0.0);
    }
}

#[test]
fn ambience_is_quiet_and_loops_at_eight_seconds() {
    let voices = ambience_voices();
    let end = voices
        .iter()
        .map(|voice| voice.start() + voice.duration())
        .fold(0.0f32, f32::max);
    assert_eq!(end, 8.0);
    let peak = render_waveform(&voices, &config(), 7)
        .iter()
        .fold(0.0f32, |peak, sample| peak.max(sample.abs()));
    assert!(peak > 0.005 && peak < 0.30, "ambient peak is {peak}");
}

#[test]
fn a_silent_director_fails_softly() {
    let mut audio = AudioDirector::silent();
    audio.play(Cue::Restored);
    audio.start_ambience();
    audio.set_volumes(1.0, 1.0);
    assert!(!audio.is_enabled());
}

#[test]
fn zero_effects_volume_never_starts_a_cue_cooldown() {
    let mut audio = AudioDirector::silent();
    audio.set_enabled(true);
    audio.set_volumes(0.0, 1.0);
    assert!(!audio.channel_allows(Cue::ShelfCorrect));
    audio.play(Cue::ShelfCorrect);
    assert_eq!(audio.cooldown_count(), 0);
}
