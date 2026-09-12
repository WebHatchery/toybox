//! Visible touch-first controls for gameplay.

use super::hud_chrome::{brass, draw_hud_panel, parchment, warm_panel};
use crate::ui::UiAction;
use macroquad::prelude::*;
use macroquad_toolkit::prelude::*;
use macroquad_toolkit::ui::draw_ui_text_ex;

pub(super) fn draw_touch_controls(actions: &mut Vec<UiAction>) {
    let pointer = super::logical_pointer();
    let panel = Rect::new(1008.0, 492.0, 254.0, 212.0);
    draw_hud_panel(panel, warm_panel(0.92), brass(0.64));
    draw_ui_text_ex(
        "TOUCH CONTROLS",
        panel.x + 14.0,
        panel.y + 19.0,
        TextStyle::new(11.0, parchment(0.74)).params(),
    );

    for (rect, label, action) in [
        (
            Rect::new(1022.0, 514.0, 48.0, 44.0),
            "◀",
            UiAction::LookLeft,
        ),
        (Rect::new(1076.0, 514.0, 48.0, 44.0), "▲", UiAction::LookUp),
        (
            Rect::new(1130.0, 514.0, 48.0, 44.0),
            "▼",
            UiAction::LookDown,
        ),
        (
            Rect::new(1184.0, 514.0, 48.0, 44.0),
            "▶",
            UiAction::LookRight,
        ),
        (
            Rect::new(1022.0, 562.0, 48.0, 44.0),
            "W",
            UiAction::MoveForward,
        ),
        (
            Rect::new(1076.0, 562.0, 48.0, 44.0),
            "A",
            UiAction::MoveLeft,
        ),
        (
            Rect::new(1130.0, 562.0, 48.0, 44.0),
            "S",
            UiAction::MoveBackward,
        ),
        (
            Rect::new(1184.0, 562.0, 48.0, 44.0),
            "D",
            UiAction::MoveRight,
        ),
    ] {
        if touch_button(rect, label, pointer, true, true) {
            actions.push(action);
        }
    }
    for (rect, label, action) in [
        (
            Rect::new(1022.0, 610.0, 66.0, 44.0),
            "ACT",
            UiAction::Interact,
        ),
        (
            Rect::new(1097.0, 610.0, 66.0, 44.0),
            "CARRY",
            UiAction::CycleCarry,
        ),
        (
            Rect::new(1172.0, 610.0, 60.0, 44.0),
            "DROP",
            UiAction::DropActive,
        ),
        (
            Rect::new(1022.0, 658.0, 66.0, 44.0),
            "TOOLS",
            UiAction::OpenToolShop,
        ),
        (
            Rect::new(1097.0, 658.0, 66.0, 44.0),
            "PAUSE",
            UiAction::Settings,
        ),
    ] {
        if touch_button(rect, label, pointer, true, false) {
            actions.push(action);
        }
    }
}

fn touch_button(rect: Rect, label: &str, pointer: Pointer, enabled: bool, repeat: bool) -> bool {
    let hovered = enabled && pointer.hovering_over(rect);
    let pressed = enabled && pointer.pressing(rect);
    let released = enabled && pointer.released_on(rect);
    draw_surface(
        rect,
        &SurfaceStyle::new(if pressed {
            Color::new(0.44, 0.25, 0.08, 0.98)
        } else if hovered {
            Color::new(0.34, 0.20, 0.07, 0.98)
        } else {
            Color::new(0.22, 0.12, 0.045, 0.96)
        })
        .with_border(1.0, brass(0.70)),
    );
    draw_text_centered_in_box(
        label,
        rect.x,
        rect.y + 1.0,
        rect.w,
        rect.h,
        13.0,
        parchment(1.0),
    );
    if repeat {
        pressed
    } else {
        released
    }
}
