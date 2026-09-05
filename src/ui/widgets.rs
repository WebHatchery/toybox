//! Small reusable immediate-mode UI widgets.

use macroquad::prelude::*;
use macroquad_toolkit::prelude::TextStyle;
use macroquad_toolkit::ui::{draw_ui_text_ex, truncate_text_to_width};

pub(super) fn draw_fitted_text(
    text: &str,
    x: f32,
    baseline_y: f32,
    max_width: f32,
    size: f32,
    color: Color,
) {
    let fitted = truncate_text_to_width(text, max_width, size);
    draw_ui_text_ex(&fitted, x, baseline_y, TextStyle::new(size, color).params());
}

/// Wrap on word boundaries and draw up to `max_lines`, truncating only the last
/// line if it still will not fit.
///
/// `draw_fitted_text` cuts at one line, which is right for a HUD notice but
/// wrong for a shop entry the player is reading to decide what to buy: four of
/// the five tool descriptions ended mid-word, and the Sorting Trolley's lost
/// the half that explains how to load it — the only place the game teaches that.
///
/// Returns the baseline of the last line drawn, so callers can lay out whatever
/// follows without assuming how many lines the text took.
/// How a wrapped block is set. Grouped so `draw_wrapped_text` keeps a readable
/// call: the position and width are the caller's layout, these are the type.
#[derive(Debug, Clone, Copy)]
pub(super) struct WrapStyle {
    pub size: f32,
    pub line_height: f32,
    pub max_lines: usize,
    pub color: Color,
}

pub(super) fn draw_wrapped_text(
    text: &str,
    x: f32,
    baseline_y: f32,
    max_width: f32,
    style: WrapStyle,
) -> f32 {
    let WrapStyle {
        size,
        line_height,
        max_lines,
        color,
    } = style;
    let lines = macroquad_toolkit::ui::wrap_text(text, max_width, size);

    let mut last_baseline = baseline_y;
    for (index, line) in lines.iter().take(max_lines).enumerate() {
        last_baseline = baseline_y + index as f32 * line_height;
        draw_fitted_text(line, x, last_baseline, max_width, size, color);
    }
    last_baseline
}
