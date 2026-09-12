//! The end-of-shift score screen.
//!
//! Two endings arrive here and they are not the same news: the shop restored,
//! or the doors opening on work still on the floor. The panel reports the same
//! figures either way — what changes is the heading, the accent, and the line
//! telling the player what to do next.

use super::hud_chrome::{brass, draw_hud_panel, parchment, warm_card, warm_panel};
use super::{shift_seed_code, UiAction, UiContext, LOGICAL_HEIGHT, LOGICAL_WIDTH};
use crate::state::{GamePhase, ShiftRecord, ShiftSummary, ZoneProgress};
use macroquad::prelude::*;
use macroquad_toolkit::prelude::*;
use macroquad_toolkit::ui::{draw_ui_text_ex, format_mmss};

const PANEL: Rect = Rect {
    x: 270.0,
    y: 72.0,
    w: 740.0,
    h: 576.0,
};

pub(super) fn draw_score_screen(ctx: &UiContext<'_>) -> Vec<UiAction> {
    let mut actions = Vec::new();
    let summary = ctx.session.shift_summary(ctx.data);
    let restored = ctx.session.phase == GamePhase::Finished;
    let accent = if restored {
        Color::new(0.62, 0.92, 0.68, 1.0)
    } else {
        Color::new(0.98, 0.62, 0.42, 1.0)
    };

    draw_rectangle(
        0.0,
        0.0,
        LOGICAL_WIDTH,
        LOGICAL_HEIGHT,
        Color::new(0.04, 0.018, 0.008, 0.76),
    );
    draw_hud_panel(
        PANEL,
        warm_panel(0.985),
        Color::new(accent.r, accent.g, accent.b, 0.78),
    );
    draw_rectangle(
        PANEL.x + 8.0,
        PANEL.y + 12.0,
        PANEL.w - 16.0,
        92.0,
        Color::new(accent.r, accent.g, accent.b, 0.075),
    );

    draw_heading(
        &summary,
        ctx.session.shift_mode.label(),
        ctx.session.shift_seed,
        restored,
        accent,
    );
    draw_grade_badge(&summary, accent);

    let stats_y = PANEL.y + 132.0;
    draw_stat_row(
        stats_y,
        &[
            (
                "Toys shelved",
                format!("{} / {}", summary.toys_shelved, summary.toy_count),
            ),
            (
                "Repairs",
                format!("{} / {}", summary.repairs, summary.breaks_total),
            ),
            ("Wrong shelves", summary.mistakes.to_string()),
            ("Time", format_mmss(summary.elapsed_seconds)),
        ],
    );

    draw_zone_table(&summary, PANEL.y + 212.0, accent);
    draw_best_run(ctx.best_run, ctx.beat_record, accent);
    draw_footer(restored);

    draw_score_actions(ctx, &mut actions);
    actions
}

fn draw_heading(
    summary: &ShiftSummary,
    mode_label: &str,
    shift_seed: u64,
    restored: bool,
    accent: Color,
) {
    let heading = if restored {
        "Store Restored"
    } else {
        "Doors Open"
    };
    draw_text_centered_in_box(
        heading,
        PANEL.x + 24.0,
        PANEL.y + 24.0,
        PANEL.w - 168.0,
        40.0,
        32.0,
        parchment(1.0),
    );

    draw_text_centered_in_box(
        &format!(
            "{mode_label} - layout {} - {} of {} aisles restored",
            shift_seed_code(shift_seed),
            summary.zones_restored,
            summary.zones_with_shelves
        ),
        PANEL.x + 24.0,
        PANEL.y + 68.0,
        PANEL.w - 168.0,
        26.0,
        16.0,
        Color::new(accent.r, accent.g, accent.b, 0.90),
    );
}

fn draw_grade_badge(summary: &ShiftSummary, accent: Color) {
    let badge = Rect::new(PANEL.x + PANEL.w - 126.0, PANEL.y + 18.0, 96.0, 96.0);
    let center = vec2(badge.x + badge.w * 0.5, badge.y + badge.h * 0.5);
    draw_circle(
        center.x + 3.0,
        center.y + 4.0,
        46.0,
        Color::new(0.0, 0.0, 0.0, 0.32),
    );
    draw_circle(center.x, center.y, 46.0, warm_card(1.0));
    draw_circle_lines(
        center.x,
        center.y,
        45.0,
        3.0,
        Color::new(accent.r, accent.g, accent.b, 0.88),
    );
    draw_circle_lines(center.x, center.y, 39.0, 1.0, brass(0.44));
    draw_text_centered_in_box(
        summary.grade(),
        badge.x,
        badge.y + 9.0,
        badge.w,
        52.0,
        46.0,
        accent,
    );
    draw_text_centered_in_box(
        &format!("{:.0}%", summary.completion() * 100.0),
        badge.x,
        badge.y + 63.0,
        badge.w,
        20.0,
        15.0,
        dark::TEXT_DIM,
    );
}

fn draw_stat_row(y: f32, cells: &[(&str, String)]) {
    let inner_x = PANEL.x + 28.0;
    let inner_w = PANEL.w - 56.0;
    let cell_w = inner_w / cells.len() as f32;

    for (index, (label, value)) in cells.iter().enumerate() {
        let x = inner_x + cell_w * index as f32;
        let card = Rect::new(x + 4.0, y, cell_w - 10.0, 72.0);
        draw_surface(
            card,
            &SurfaceStyle::new(warm_card(0.94))
                .with_border(1.0, brass(0.30))
                .with_top_highlight(2.0, brass(0.08)),
        );
        draw_text_centered_in_box(
            &label.to_uppercase(),
            card.x,
            card.y + 9.0,
            card.w,
            18.0,
            11.0,
            parchment(0.56),
        );
        draw_text_centered_in_box(
            value,
            card.x,
            card.y + 28.0,
            card.w,
            32.0,
            22.0,
            parchment(1.0),
        );
    }
}

fn draw_zone_table(summary: &ShiftSummary, top: f32, accent: Color) {
    draw_ui_text_ex(
        "Aisles",
        PANEL.x + 28.0,
        top,
        TextStyle::new(14.0, parchment(0.70)).params(),
    );

    for (index, (name, zone)) in summary.zones.iter().enumerate() {
        draw_zone_row(name, *zone, top + 22.0 + index as f32 * 34.0, accent);
    }
}

fn draw_zone_row(name: &str, zone: ZoneProgress, y: f32, accent: Color) {
    let x = PANEL.x + 28.0;
    let width = PANEL.w - 56.0;
    let restored = zone.is_restored();
    let label_color = if restored { accent } else { parchment(0.92) };

    if ((y / 34.0) as i32) % 2 == 0 {
        draw_rectangle(x - 8.0, y - 3.0, width + 16.0, 29.0, warm_card(0.42));
    }

    draw_ui_text_ex(
        name,
        x,
        y + 14.0,
        TextStyle::new(15.0, label_color).params(),
    );
    // "43 / 48" alone sends a player who shelved the whole aisle back out to
    // hunt five toys that are lying around in halves. Split the shortfall into
    // the two jobs it actually is — and show both, because an aisle can want
    // more searching *and* more mending, and reporting only one of those is how
    // the bare shortfall misled in the first place.
    let mut shortfall = Vec::new();
    if zone.still_to_find() > 0 {
        shortfall.push(format!("{} to find", zone.still_to_find()));
    }
    if zone.broken > 0 {
        shortfall.push(format!("{} to mend", zone.broken));
    }
    if !shortfall.is_empty() {
        draw_ui_text_ex(
            &shortfall.join("  -  "),
            x + 168.0,
            y + 14.0,
            TextStyle::new(13.0, Color::new(0.88, 0.70, 0.94, 1.0)).params(),
        );
    }

    draw_ui_text_ex(
        &format!("{} / {}", zone.placed, zone.capacity),
        x + width - 128.0,
        y + 14.0,
        TextStyle::new(14.0, parchment(0.58)).params(),
    );

    // A bar rather than a bare percentage: an aisle one toy short of restored
    // and an aisle half done should not read the same at a glance.
    let bar = Rect::new(x + width - 72.0, y + 4.0, 72.0, 12.0);
    draw_rectangle(
        bar.x,
        bar.y,
        bar.w,
        bar.h,
        Color::new(0.12, 0.07, 0.04, 0.96),
    );
    draw_rectangle(
        bar.x,
        bar.y,
        bar.w * zone.fraction().clamp(0.0, 1.0),
        bar.h,
        if restored {
            accent
        } else {
            Color::new(0.96, 0.74, 0.38, 0.86)
        },
    );
}

/// The line that turns a grade into something to beat.
///
/// Nothing carries between shifts by design — tools are earned and lost inside
/// one run — so without this the game scores you and forgets. A record is the
/// only thread between runs, which is why it is worth the separate save slot.
fn draw_best_run(best: Option<ShiftRecord>, beat_record: bool, accent: Color) {
    let y = PANEL.y + PANEL.h - 112.0;
    let text = match best {
        Some(record) if beat_record => format!(
            "New best: {} toys in {}",
            record.toys_shelved,
            format_mmss(record.elapsed_seconds)
        ),
        Some(record) => format!(
            "Best so far: {} toys, {} wrong, {}",
            record.toys_shelved,
            record.mistakes,
            format_mmss(record.elapsed_seconds)
        ),
        None => "No record kept for this mode yet.".to_owned(),
    };
    let colour = if beat_record { accent } else { parchment(0.62) };
    draw_text_centered_in_box(&text, PANEL.x, y, PANEL.w, 24.0, 16.0, colour);
}

fn draw_footer(restored: bool) {
    draw_text_centered_in_box(
        if restored {
            // This read "The snow globe shop is ready for sunrise" — stray text
            // from another game, and the only occurrence of "snow" in the repo.
            // It sat on the *win* screen, so a perfect twenty-five minute shift
            // was congratulated on tidying a shop that does not exist here.
            "Every toy is home before the doors open."
        } else {
            "Opening time caught up with you."
        },
        PANEL.x,
        PANEL.y + PANEL.h - 80.0,
        PANEL.w,
        24.0,
        16.0,
        parchment(0.62),
    );
}

fn draw_score_actions(ctx: &UiContext<'_>, actions: &mut Vec<UiAction>) {
    let pointer = super::logical_pointer();
    let y = PANEL.y + PANEL.h - 48.0;
    let buttons = [
        (
            Rect::new(PANEL.x + 28.0, y, 208.0, 44.0),
            "NEW SHIFT",
            match ctx.session.shift_mode {
                crate::state::ShiftMode::Timed => UiAction::NewGame,
                crate::state::ShiftMode::Relaxed => UiAction::NewRelaxedGame,
            },
        ),
        (
            Rect::new(PANEL.x + 266.0, y, 208.0, 44.0),
            "REPLAY LAYOUT",
            UiAction::ReplayShiftSeed,
        ),
        (
            Rect::new(PANEL.x + 504.0, y, 208.0, 44.0),
            "MENU",
            UiAction::BackToTitle,
        ),
    ];

    for (rect, label, action) in buttons {
        let hovered = pointer.hovering_over(rect);
        let pressed = pointer.pressing(rect);
        draw_surface(
            rect,
            &SurfaceStyle::new(if pressed {
                Color::new(0.46, 0.27, 0.09, 0.98)
            } else if hovered {
                Color::new(0.36, 0.21, 0.07, 0.98)
            } else {
                Color::new(0.23, 0.13, 0.05, 0.98)
            })
            .with_border(1.0, brass(0.72)),
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
        if pointer.released_on(rect) {
            actions.push(action);
        }
    }
}
