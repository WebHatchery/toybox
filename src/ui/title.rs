use super::{shift_seed_code, UiAction, LOGICAL_HEIGHT, LOGICAL_WIDTH};
use crate::state::{BestRuns, ShiftMode};
use macroquad::prelude::*;
use macroquad_toolkit::prelude::*;

mod help;
pub(crate) use help::draw_help_screen;

pub(crate) fn draw_title_screen(
    title_texture: Option<&Texture2D>,
    continue_enabled: bool,
    best_runs: &BestRuns,
) -> Vec<UiAction> {
    draw_title_background(title_texture);
    draw_title_scrim();

    let mut actions = Vec::new();
    let pointer = super::logical_pointer();
    let button_w = 148.0;
    let button_h = 38.0;
    let button_gap = 14.0;
    let buttons: [(&str, UiAction, bool, ButtonTone); 5] = [
        (
            "Closing Shift",
            UiAction::NewGame,
            true,
            ButtonTone::Primary,
        ),
        (
            "Relaxed Run",
            UiAction::NewRelaxedGame,
            true,
            ButtonTone::Muted,
        ),
        (
            "Continue",
            UiAction::Continue,
            continue_enabled,
            ButtonTone::Positive,
        ),
        ("Settings", UiAction::Settings, true, ButtonTone::Muted),
        ("Quit Game", UiAction::QuitGame, true, ButtonTone::Danger),
    ];
    let count = buttons.len() as f32;
    let total_w = button_w * count + button_gap * (count - 1.0);
    let x = (LOGICAL_WIDTH - total_w) * 0.5;
    let y = 614.0;

    for (index, (label, action, enabled, tone)) in buttons.into_iter().enumerate() {
        let slot_x = x + (button_w + button_gap) * index as f32;
        if title_button(
            Rect::new(slot_x, y, button_w, button_h),
            label,
            enabled,
            tone,
            pointer,
        ) {
            actions.push(action);
        }
    }

    // The two ways to start differ only in whether the clock can end the run,
    // which no button label can carry on its own.
    draw_title_caption(
        "Closing Shift keeps the fixed record layout. Relaxed Run has no deadline and a fresh scatter.",
        y + button_h + 22.0,
    );
    draw_best_runs(best_runs, y - 18.0);

    actions
}

/// The player's best run for each mode, above the buttons that pick one.
///
/// The score screen already reports a record once a shift ends, but by then the
/// choice has been made. Sitting at the title deciding between a timed run and
/// a relaxed one, what you are chasing is the thing worth knowing — and with
/// nothing carrying between shifts, the record is the only reason to pick the
/// clock at all.
fn draw_best_runs(best: &BestRuns, y: f32) {
    let parts: Vec<String> = [ShiftMode::Timed, ShiftMode::Relaxed]
        .into_iter()
        .filter_map(|mode| {
            best.best_for(mode)
                .map(|record| format!("{}: {} toys", mode.label(), record.toys_shelved))
        })
        .collect();

    if parts.is_empty() {
        return;
    }
    draw_title_caption(&format!("Best - {}", parts.join("   ")), y);
}

/// A centred line of small text over the title art, on a plate.
///
/// The art behind it is a lit shop full of bright toys, so unbacked 15px text
/// lands on whatever colour happens to be there and is barely readable over the
/// pale ones. The plate is the same idiom as the HUD panels: dark, mostly
/// transparent, sized to the text rather than the screen.
fn draw_title_caption(text: &str, y: f32) {
    let size = 15.0_f32;
    let width = measure_ui_text(text, None, size as u16, 1.0).width;
    let x = (LOGICAL_WIDTH - width) * 0.5;

    draw_rectangle(
        x - 14.0,
        y - size - 4.0,
        width + 28.0,
        size + 12.0,
        Color::new(0.02, 0.025, 0.03, 0.62),
    );
    let style = TextStyle::new(size, Color::new(0.90, 0.91, 0.94, 0.95));
    draw_ui_text_ex(text, x, y, style.params());
}

pub(crate) fn draw_settings_screen(
    title_texture: Option<&Texture2D>,
    view: SettingsView,
) -> Vec<UiAction> {
    draw_title_background(title_texture);
    draw_title_scrim();

    let mut actions = Vec::new();
    let pointer = super::logical_pointer();
    let button_w = 224.0;
    let button_h = 38.0;
    let settings_panel = Rect::new(322.0, 188.0, 636.0, 500.0);
    draw_rectangle(
        settings_panel.x + 6.0,
        settings_panel.y + 7.0,
        settings_panel.w,
        settings_panel.h,
        Color::new(0.0, 0.0, 0.0, 0.34),
    );
    draw_surface(
        settings_panel,
        &SurfaceStyle::new(Color::new(0.105, 0.055, 0.026, 0.91))
            .with_border(2.0, Color::new(0.80, 0.57, 0.24, 0.78))
            .with_inner_border(6.0, 1.0, Color::new(1.0, 0.82, 0.44, 0.14)),
    );
    draw_rectangle(
        settings_panel.x + 8.0,
        settings_panel.y + 8.0,
        settings_panel.w - 16.0,
        4.0,
        Color::new(0.34, 0.16, 0.055, 0.72),
    );

    draw_text_centered_in_box(
        if view.from_game {
            "Paused & Settings"
        } else {
            "Settings"
        },
        settings_panel.x,
        settings_panel.y + 22.0,
        settings_panel.w,
        38.0,
        25.0,
        title_parchment(),
    );
    if view.from_game {
        let layout = match view.shift_mode {
            ShiftMode::Timed => format!(
                "Closing Shift - fixed layout {}",
                shift_seed_code(view.shift_seed)
            ),
            ShiftMode::Relaxed => {
                format!("Relaxed Run - layout {}", shift_seed_code(view.shift_seed))
            }
        };
        draw_text_centered_in_box(
            &layout,
            settings_panel.x,
            settings_panel.y + 60.0,
            settings_panel.w,
            18.0,
            12.0,
            Color::new(0.92, 0.72, 0.40, 0.78),
        );
    }

    let left = settings_panel.x + 52.0;
    let right = settings_panel.x + 360.0;
    let row1_y = settings_panel.y + 96.0;
    let fullscreen_label = if view.fullscreen_enabled {
        "Fullscreen: On"
    } else {
        "Fullscreen: Off"
    };
    if title_button(
        Rect::new(left, row1_y, button_w, button_h),
        fullscreen_label,
        true,
        ButtonTone::Primary,
        pointer,
    ) {
        actions.push(UiAction::ToggleFullscreen);
    }

    draw_settings_stepper(
        "FIELD OF VIEW",
        &format!("{}°", view.fov_degrees.round() as i32),
        vec2(right, row1_y),
        UiAction::FovDecrease,
        UiAction::FovIncrease,
        pointer,
        &mut actions,
    );

    let row2_y = settings_panel.y + 162.0;
    draw_settings_stepper(
        "LOOK SENSITIVITY",
        &format!("{:.1}×", view.mouse_sensitivity),
        vec2(left, row2_y),
        UiAction::SensitivityDecrease,
        UiAction::SensitivityIncrease,
        pointer,
        &mut actions,
    );
    draw_settings_stepper(
        "UI TEXT SIZE",
        &format!("{:.0}%", view.ui_scale * 100.0),
        vec2(right, row2_y),
        UiAction::UiScaleDecrease,
        UiAction::UiScaleIncrease,
        pointer,
        &mut actions,
    );

    let row3_y = settings_panel.y + 236.0;
    draw_settings_stepper(
        "MASTER VOLUME",
        &format!("{:.0}%", view.master_volume * 100.0),
        vec2(left, row3_y),
        UiAction::MasterVolumeDecrease,
        UiAction::MasterVolumeIncrease,
        pointer,
        &mut actions,
    );
    draw_settings_stepper(
        "EFFECTS VOLUME",
        &format!("{:.0}%", view.effects_volume * 100.0),
        vec2(right, row3_y),
        UiAction::EffectsVolumeDecrease,
        UiAction::EffectsVolumeIncrease,
        pointer,
        &mut actions,
    );

    let row4_y = settings_panel.y + 310.0;
    draw_settings_stepper(
        "AMBIENCE VOLUME",
        &format!("{:.0}%", view.ambience_volume * 100.0),
        vec2(left, row4_y),
        UiAction::AmbienceVolumeDecrease,
        UiAction::AmbienceVolumeIncrease,
        pointer,
        &mut actions,
    );
    if title_button(
        Rect::new(right, row4_y, button_w, button_h),
        if view.high_contrast {
            "High Contrast: On"
        } else {
            "High Contrast: Off"
        },
        true,
        if view.high_contrast {
            ButtonTone::Positive
        } else {
            ButtonTone::Muted
        },
        pointer,
    ) {
        actions.push(UiAction::ToggleHighContrast);
    }

    let row5_y = settings_panel.y + 380.0;
    if title_button(
        Rect::new((LOGICAL_WIDTH - button_w) * 0.5, row5_y, button_w, button_h),
        "Controls & How to Play",
        true,
        ButtonTone::Primary,
        pointer,
    ) {
        actions.push(UiAction::OpenHelp);
    }

    let bottom_y = settings_panel.bottom() - 54.0;
    let bottom_w = if view.from_game {
        button_w * 2.0 + 16.0
    } else {
        button_w
    };
    let bottom_x = (LOGICAL_WIDTH - bottom_w) * 0.5;

    if title_button(
        Rect::new(bottom_x, bottom_y, button_w, button_h),
        if view.from_game { "Resume" } else { "Back" },
        true,
        if view.from_game {
            ButtonTone::Positive
        } else {
            ButtonTone::Muted
        },
        pointer,
    ) {
        actions.push(UiAction::CloseSettings);
    }

    // Named for what it does rather than what it used to: leaving writes the
    // shift to the save slot, so the title's Continue picks it back up. A
    // Danger tone said the opposite — that the run was about to be thrown
    // away, which is exactly what used to happen.
    if view.from_game
        && title_button(
            Rect::new(bottom_x + button_w + 16.0, bottom_y, button_w, button_h),
            "Save & Quit",
            true,
            ButtonTone::Muted,
            pointer,
        )
    {
        actions.push(UiAction::BackToTitle);
    }

    actions
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct SettingsView {
    pub fullscreen_enabled: bool,
    pub fov_degrees: f32,
    pub mouse_sensitivity: f32,
    pub ui_scale: f32,
    pub high_contrast: bool,
    pub master_volume: f32,
    pub effects_volume: f32,
    pub ambience_volume: f32,
    pub from_game: bool,
    pub shift_mode: ShiftMode,
    pub shift_seed: u64,
}

fn draw_settings_stepper(
    label: &str,
    value: &str,
    origin: Vec2,
    decrease: UiAction,
    increase: UiAction,
    pointer: Pointer,
    actions: &mut Vec<UiAction>,
) {
    let height = 38.0;
    draw_text_centered_in_box(
        label,
        origin.x,
        origin.y - 20.0,
        224.0,
        18.0,
        11.0,
        Color::new(0.88, 0.72, 0.48, 0.86),
    );
    if title_button(
        Rect::new(origin.x, origin.y, 44.0, height),
        "-",
        true,
        ButtonTone::Muted,
        pointer,
    ) {
        actions.push(decrease);
    }
    draw_plaque(
        Rect::new(origin.x + 50.0, origin.y, 124.0, height),
        &title_plaque_style(),
        &title_button_palette(ButtonTone::Muted),
        PlaqueState::idle(true),
    );
    draw_text_centered_in_box(
        value,
        origin.x + 50.0,
        origin.y - 1.0,
        124.0,
        height,
        15.0,
        title_parchment(),
    );
    if title_button(
        Rect::new(origin.x + 180.0, origin.y, 44.0, height),
        "+",
        true,
        ButtonTone::Muted,
        pointer,
    ) {
        actions.push(increase);
    }
}

fn draw_title_background(texture: Option<&Texture2D>) {
    if let Some(texture) = texture {
        let texture_size = texture.size();
        let scale = (LOGICAL_WIDTH / texture_size.x).max(LOGICAL_HEIGHT / texture_size.y);
        let dest_size = texture_size * scale;
        let x = (LOGICAL_WIDTH - dest_size.x) * 0.5;
        let y = (LOGICAL_HEIGHT - dest_size.y) * 0.5;

        draw_texture_ex(
            texture,
            x,
            y,
            WHITE,
            DrawTextureParams {
                dest_size: Some(dest_size),
                ..Default::default()
            },
        );
    } else {
        clear_background(Color::new(0.05, 0.045, 0.055, 1.0));
        draw_text_centered_in_box(
            "Toybox After Hours",
            0.0,
            230.0,
            LOGICAL_WIDTH,
            70.0,
            54.0,
            dark::TEXT_BRIGHT,
        );
    }
}

fn draw_title_scrim() {
    draw_rectangle(
        0.0,
        0.0,
        LOGICAL_WIDTH,
        LOGICAL_HEIGHT,
        Color::new(0.02, 0.014, 0.010, 0.16),
    );
}

fn title_button(rect: Rect, text: &str, enabled: bool, tone: ButtonTone, pointer: Pointer) -> bool {
    let style = title_plaque_style();
    let palette = title_button_palette(tone);
    let hovered = enabled && pointer.hovering_over(rect);
    let pressed = enabled && pointer.pressing(rect);
    let activated = enabled && pointer.released_on(rect);
    draw_plaque(
        rect,
        &style,
        &palette,
        PlaqueState {
            enabled,
            hovered,
            pressed,
            selected: false,
        },
    );
    let text_color = if enabled {
        palette.text
    } else {
        style.disabled_text
    };
    let nudge = if pressed { style.press_nudge } else { 0.0 };
    draw_text_centered_in_box_ex(
        text,
        rect.x + 8.0,
        rect.y + nudge - 1.0,
        rect.w - 16.0,
        rect.h,
        TextStyle::new(style.label_size(rect.h), text_color),
    );
    activated
}

/// Toolkit plaque defaults were extracted from these exact title buttons;
/// only the fixed label size and disabled-border alpha differ.
fn title_plaque_style() -> PlaqueStyle {
    PlaqueStyle {
        font_size: Some(15.0),
        disabled_border: Color::new(0.30, 0.28, 0.24, 0.78),
        ..PlaqueStyle::default()
    }
}

fn title_button_palette(tone: ButtonTone) -> PlaquePalette {
    match tone {
        ButtonTone::Primary => PlaquePalette {
            normal: Color::new(0.12, 0.20, 0.31, 0.92),
            hovered: Color::new(0.18, 0.29, 0.44, 0.96),
            pressed: Color::new(0.08, 0.14, 0.23, 0.98),
            disabled: Color::new(0.08, 0.10, 0.13, 0.72),
            border: Color::new(0.58, 0.64, 0.74, 0.86),
            text: title_parchment(),
        },
        ButtonTone::Positive => PlaquePalette {
            normal: Color::new(0.11, 0.28, 0.17, 0.92),
            hovered: Color::new(0.17, 0.38, 0.23, 0.96),
            pressed: Color::new(0.07, 0.20, 0.12, 0.98),
            disabled: Color::new(0.07, 0.10, 0.08, 0.68),
            border: Color::new(0.55, 0.72, 0.42, 0.84),
            text: title_parchment(),
        },
        ButtonTone::Danger => PlaquePalette {
            normal: Color::new(0.31, 0.12, 0.10, 0.92),
            hovered: Color::new(0.45, 0.17, 0.14, 0.96),
            pressed: Color::new(0.22, 0.08, 0.07, 0.98),
            disabled: Color::new(0.14, 0.08, 0.08, 0.70),
            border: Color::new(0.78, 0.42, 0.34, 0.80),
            text: title_parchment(),
        },
        ButtonTone::Muted | ButtonTone::Secondary | ButtonTone::Warning => PlaquePalette {
            normal: Color::new(0.080, 0.083, 0.090, 0.90),
            hovered: Color::new(0.13, 0.135, 0.145, 0.96),
            pressed: Color::new(0.055, 0.058, 0.064, 0.98),
            disabled: Color::new(0.055, 0.055, 0.060, 0.68),
            border: Color::new(0.54, 0.48, 0.38, 0.72),
            text: title_parchment(),
        },
    }
}

fn title_parchment() -> Color {
    Color::new(0.92, 0.82, 0.62, 1.0)
}
