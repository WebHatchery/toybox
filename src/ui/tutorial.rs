//! First-shift guidance card. Gameplay prompts remain the authority for the
//! immediate action; this card explains the larger reason for that action.

use super::hud_chrome::{brass, draw_hud_panel, draw_keycap, parchment, warm_panel};
use crate::tutorial::TutorialHint;
use crate::ui::widgets::{draw_wrapped_text, WrapStyle};
use crate::ui::UiAction;
use macroquad::prelude::*;
use macroquad_toolkit::prelude::{
    draw_surface, draw_text_centered_in_box, SurfaceStyle, TextStyle,
};
use macroquad_toolkit::ui::draw_ui_text_ex;

pub(super) fn draw_tutorial_hint(hint: &TutorialHint, actions: &mut Vec<UiAction>) {
    let rect = Rect::new(630.0, 16.0, 364.0, 124.0);
    draw_hud_panel(rect, warm_panel(0.94), brass(0.72));
    draw_ui_text_ex(
        &hint.eyebrow,
        rect.x + 18.0,
        rect.y + 25.0,
        TextStyle::new(10.0, Color::new(1.0, 0.66, 0.20, 0.88)).params(),
    );
    draw_ui_text_ex(
        &hint.title,
        rect.x + 18.0,
        rect.y + 49.0,
        TextStyle::new(17.0, parchment(1.0)).params(),
    );
    draw_wrapped_text(
        &hint.body,
        rect.x + 18.0,
        rect.y + 69.0,
        rect.w - 36.0,
        WrapStyle {
            size: 12.0,
            line_height: 16.0,
            max_lines: 2,
            color: parchment(0.68),
        },
    );

    let mut x = rect.x + 18.0;
    for key in &hint.keys {
        let width = if key.len() > 2 { 48.0 } else { 26.0 };
        draw_keycap(Rect::new(x, rect.bottom() - 25.0, width, 19.0), key, false);
        x += width + 7.0;
    }
    let skip = Rect::new(rect.right() - 124.0, rect.bottom() - 50.0, 108.0, 44.0);
    let pointer = super::logical_pointer();
    draw_surface(
        skip,
        &SurfaceStyle::new(if pointer.pressing(skip) {
            brass(0.38)
        } else {
            brass(0.20)
        })
        .with_border(1.0, brass(0.60)),
    );
    draw_text_centered_in_box(
        "SKIP GUIDE",
        skip.x,
        skip.y,
        skip.w,
        skip.h,
        11.0,
        parchment(0.86),
    );
    if pointer.released_on(skip) {
        actions.push(UiAction::SkipTutorial);
    }
}
