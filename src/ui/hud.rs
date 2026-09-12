//! Gameplay HUD panels inspired by the warm toy-store reference.

use super::hud_chrome::{
    brass, draw_hud_panel, draw_identity_token, draw_keycap, draw_notice_dot, draw_progress_bar,
    draw_prompt_status_icon, draw_toy_badge, parchment, warm_card, warm_panel, NoticeRow,
    NoticeTone, PromptTone, PromptVisual,
};
use super::hud_icons::{draw_icon, draw_open_box_icon, draw_stopwatch_icon, IconKind};
use crate::state::{CounterpartLocation, GamePhase, InteractionPreview, ToyState, ZoneProgress};
use crate::ui::widgets::draw_fitted_text;
use crate::ui::{UiContext, LOGICAL_HEIGHT, LOGICAL_WIDTH};
use macroquad::prelude::*;
use macroquad_toolkit::prelude::*;
use macroquad_toolkit::ui::{draw_ui_text_ex, format_mmss, measure_ui_text};

pub(super) fn draw_game_hud(ctx: &UiContext<'_>) {
    draw_status_panel(ctx);
    draw_notice_panel(ctx);
    draw_carried_card(ctx);
    draw_context_prompt(ctx);
    draw_crosshair(ctx);
}

fn draw_status_panel(ctx: &UiContext<'_>) {
    let rect = status_panel_rect();
    draw_hud_panel(rect, warm_panel(0.90), hud_border());

    // A timed shift counts down to opening, because what the player needs to
    // know is how much is left, not how much is gone. A relaxed run has nothing
    // to count down to, so it shows elapsed time instead.
    let counts_down = ctx.session.shift_mode.shows_countdown();
    let remaining = ctx.session.shift_remaining(ctx.data);
    let (time, clock_color) = if counts_down {
        // Amber under five minutes, red under one.
        let colour = if remaining <= 60.0 {
            Color::new(0.98, 0.42, 0.36, 1.0)
        } else if remaining <= 300.0 {
            Color::new(0.99, 0.76, 0.34, 1.0)
        } else {
            dark::TEXT_BRIGHT
        };
        (format_mmss(remaining), colour)
    } else {
        (
            format_mmss(ctx.session.player.elapsed_seconds),
            dark::TEXT_BRIGHT,
        )
    };
    draw_stopwatch_icon(vec2(rect.x + 34.0, rect.y + 55.0), 15.0, clock_color);
    draw_ui_text_ex(
        &time,
        rect.x + 58.0,
        rect.y + 67.0,
        TextStyle::new(31.0, clock_color).params(),
    );

    // Say which way the number runs. The two modes drew the identical panel —
    // same stopwatch, same colour, same place — so "07:11" meant seven minutes
    // gone in a relaxed run and seven minutes left in a timed one, with nothing
    // on screen to tell them apart. The mode was named once, in a notification
    // at the start of the run, which fades; and now that quitting saves, a
    // player can resume a shift days later having forgotten which it was.
    //
    // Right-aligned off the measured width rather than placed after the digits,
    // since the countdown is five characters and the elapsed clock passes six
    // at an hour.
    let caption = format!("TIME {}", ctx.session.shift_mode.clock_caption());
    let caption_size = 11.0;
    let caption_width = measure_ui_text(&caption, None, caption_size as u16, 1.0).width;
    draw_ui_text_ex(
        &caption,
        rect.x + 151.0 - caption_width,
        rect.y + 88.0,
        TextStyle::new(caption_size, parchment(0.68)).params(),
    );

    for x in [rect.x + 166.0, rect.x + 292.0, rect.x + 392.0] {
        draw_line(x, rect.y + 18.0, x, rect.bottom() - 16.0, 1.0, brass(0.28));
    }

    let placed = ctx.session.total_placed_toys();
    let toy_count = ctx.data.config.toy_count.max(1);
    draw_compact_stat(
        vec2(rect.x + 180.0, rect.y + 19.0),
        100.0,
        IconKind::Star,
        "PUT AWAY",
        &format!("{placed} / {toy_count}"),
        Some(placed as f32 / toy_count as f32),
        Color::new(1.0, 0.72, 0.16, 1.0),
    );

    let carry_limit = ctx.session.carry_limit(ctx.data).max(1);
    let carried = ctx.session.player.carried_toy_ids.len();
    draw_compact_stat(
        vec2(rect.x + 306.0, rect.y + 19.0),
        74.0,
        IconKind::Crate,
        "CARRY",
        &format!("{carried} / {carry_limit}"),
        Some(carried as f32 / carry_limit as f32),
        Color::new(0.93, 0.48, 0.18, 1.0),
    );

    draw_zone_stat(ctx, vec2(rect.x + 406.0, rect.y + 19.0), 166.0);
}

/// Completion of the aisle the player is standing in. At 4000 toys the overall
/// count barely moves, so the zone figure is what tells them the last hour was
/// worth anything — and which aisle to work next.
fn draw_zone_stat(ctx: &UiContext<'_>, origin: Vec2, width: f32) {
    let Some(zone_index) = ctx.session.current_zone_index(ctx.data) else {
        return;
    };
    let progress = ctx.session.zone_progress(ctx.data);
    let zone = &ctx.data.layout.zones[zone_index];
    let here: ZoneProgress = progress[zone_index];
    let accent = Color::new(zone.accent[0], zone.accent[1], zone.accent[2], 1.0);

    let value = if here.has_displays() {
        format!(
            "{} / {}  ·  {:.0}%",
            here.placed,
            here.capacity,
            here.fraction() * 100.0
        )
    } else {
        "no shelves".to_owned()
    };
    // The percentage alone stalls short of 100 with no explanation once every
    // whole toy in the aisle is shelved. The count rides on the label rather
    // than the value: the label is the smaller font and has the room, and
    // "90% +5 to mend" ran straight off the edge of the panel.
    let label = if here.broken > 0 {
        format!("{} - {} to mend", zone.name, here.broken)
    } else {
        zone.name.clone()
    };
    // No bar for a zone with no shelves. `fraction()` reports 1.0 there so that
    // aggregates do not count an empty zone as outstanding work — but rendered
    // as a full meter beside the words "no shelves" it reads as a finished
    // aisle, which is a claim about work that does not exist.
    draw_compact_stat(
        origin,
        width,
        IconKind::Star,
        &label.to_uppercase(),
        &value,
        here.has_displays().then(|| here.fraction()),
        accent,
    );
}

fn draw_compact_stat(
    origin: Vec2,
    width: f32,
    icon: IconKind,
    label: &str,
    value: &str,
    progress: Option<f32>,
    accent: Color,
) {
    draw_icon(icon, vec2(origin.x + 10.0, origin.y + 9.0), 10.0, accent);
    // Fitted, not free-drawn: zone names are data and a longer one added later
    // would silently spill out of the panel again. Truncating is the failure
    // that stays inside the box.
    draw_fitted_text(
        label,
        origin.x + 26.0,
        origin.y + 13.0,
        width - 26.0,
        11.0,
        parchment(0.82),
    );
    draw_ui_text_ex(
        value,
        origin.x,
        origin.y + 43.0,
        TextStyle::new(18.0, dark::TEXT_BRIGHT).params(),
    );
    // Underline rather than sit beside the value: at 4000 toys the counter
    // reads "1105 / 4000" and ran straight into a bar placed to its right.
    if let Some(progress) = progress {
        draw_progress_bar(
            Rect::new(origin.x, origin.y + 54.0, width, 5.0),
            progress,
            accent,
        );
    }
}

fn draw_notice_panel(ctx: &UiContext<'_>) {
    let mut rows = Vec::new();
    let credits = ctx.session.available_tool_credits(ctx.data);

    if let Some(upgrade) = ctx.session.next_available_upgrade(ctx.data) {
        let row = if credits >= upgrade.cost {
            NoticeRow {
                key: Some("T"),
                text: format!("Open tools: {}", upgrade.name),
                tone: NoticeTone::Tool,
            }
        } else {
            NoticeRow {
                key: None,
                text: format!(
                    "{} needs {}",
                    upgrade.name,
                    crate::ui::credits_phrase(upgrade.cost)
                ),
                tone: NoticeTone::Warning,
            }
        };
        rows.push(row);
    }

    let scanner = ctx.session.scanner_enabled(ctx.data);
    if let Some(active_toy) = ctx.session.active_toy() {
        // A carried half always names the aisle its counterpart landed in, tool
        // or not. Without any signal the errand is a sweep of the whole shop
        // for one object among hundreds, which is not a search so much as a
        // wall — and the scatter is meant to be a journey, not a lockout. The
        // scanner still earns its price: it adds the distance and the beacon
        // that turn "somewhere in the Robot Lab" into "exactly there".
        let (text, tone) = if active_toy.is_repair_part() {
            (counterpart_text(ctx, scanner), NoticeTone::Scanner)
        } else if scanner {
            let named = ctx
                .session
                .recommended_display_index(ctx.data, active_toy)
                .and_then(|index| ctx.data.displays.get(index))
                .map(|display| format!("Scanner: {} - {}", display.name, display.theme))
                .unwrap_or_default();
            (named, NoticeTone::Scanner)
        } else {
            (String::new(), NoticeTone::Scanner)
        };

        if !text.is_empty() {
            rows.push(NoticeRow {
                key: None,
                text,
                tone,
            });
        }
    }

    if ctx.session.stockroom_spotlight_active() {
        rows.push(NoticeRow {
            key: None,
            text: format!(
                "Spotlight: nearest loose toy, {:.0}s",
                ctx.session.player.stockroom_spotlight_seconds.ceil()
            ),
            tone: NoticeTone::Warning,
        });
    } else if ctx.session.all_tools_owned(ctx.data) && credits > 0 {
        rows.push(NoticeRow {
            key: Some("T"),
            text: "Call the Stockroom Spotlight".to_owned(),
            tone: NoticeTone::Warning,
        });
    }

    if rows.is_empty() {
        return;
    }

    let row_height = 34.0;
    let rect = Rect::new(
        18.0,
        status_panel_rect().bottom() + 10.0,
        342.0,
        14.0 + row_height * rows.len() as f32,
    );
    draw_hud_panel(rect, warm_card(0.88), subtle_border());

    for (index, row) in rows.iter().enumerate() {
        let y = rect.y + 9.0 + index as f32 * row_height;
        let accent = row.tone.accent();
        if let Some(key) = row.key {
            draw_keycap(Rect::new(rect.x + 13.0, y + 4.0, 27.0, 23.0), key, false);
            draw_fitted_text(
                &row.text,
                rect.x + 50.0,
                y + 21.0,
                rect.w - 64.0,
                14.0,
                row.tone.text_color(),
            );
        } else {
            draw_notice_dot(vec2(rect.x + 27.0, y + 16.0), accent);
            draw_fitted_text(
                &row.text,
                rect.x + 50.0,
                y + 21.0,
                rect.w - 64.0,
                14.0,
                row.tone.text_color(),
            );
        }
    }
}

/// Scanner readout for a carried repair part: where its other half is, in
/// zone-and-distance terms. Falls back to the bench hint when the counterpart
/// has already been consumed by an earlier repair.
fn counterpart_text(ctx: &UiContext<'_>, scanner: bool) -> String {
    let Some(counterpart) = ctx.session.carried_counterpart() else {
        return if scanner {
            "Scanner: Repair Bench".to_owned()
        } else {
            "Take it to a repair bench".to_owned()
        };
    };
    describe_counterpart(ctx, counterpart, scanner)
}

/// Where the other half is. Unaided this names the aisle and stops there: the
/// player still has to walk it and pick the part out of whatever is lying
/// around. The scanner adds the metres, and pairs with the beacon column that
/// marks the exact spot.
fn describe_counterpart(
    ctx: &UiContext<'_>,
    counterpart: CounterpartLocation,
    scanner: bool,
) -> String {
    let zone = ctx
        .data
        .layout
        .zone_name_at(counterpart.position.x, counterpart.position.y);
    // One phrase for both tiers, carrying its own preposition. Naming the zone
    // without one produced "head Checkout, 17m", and bolting "in" on in front
    // of the bench case would produce "in on a bench".
    let place = if counterpart.on_bench {
        "on a bench".to_owned()
    } else {
        zone.map(|zone| format!("in {zone}"))
            .unwrap_or_else(|| "on the shop floor".to_owned())
    };
    let part = counterpart.part.label();

    if !scanner {
        return format!("Other half: {part} {place}");
    }

    let distance = ctx
        .session
        .player
        .position
        .to_vec2()
        .distance(counterpart.position.to_vec2());
    format!("Scanner: {part} {place}, {distance:.0}m")
}

fn draw_carried_card(ctx: &UiContext<'_>) {
    let rect = carried_card_rect();
    draw_hud_panel(rect, warm_panel(0.90), subtle_border());

    if ctx.session.player.carried_toy_ids.is_empty() {
        draw_empty_hands_card(rect);
        return;
    }

    let Some(active_toy) = ctx.session.active_toy() else {
        return;
    };

    draw_toy_badge(
        Rect::new(rect.x + 15.0, rect.y + 17.0, 64.0, 64.0),
        active_toy,
    );
    // Name on its own line above the controls. The pips used to sit beside it
    // and the two overlapped; with a full trolley the name ran straight through
    // three pips and the cycle key.
    draw_fitted_text(
        &active_toy.name,
        rect.x + 94.0,
        rect.y + 42.0,
        rect.w - 112.0,
        20.0,
        dark::TEXT_BRIGHT,
    );

    draw_keycap(
        Rect::new(rect.right() - 76.0, rect.y + 62.0, 25.0, 22.0),
        "G",
        false,
    );
    draw_ui_text_ex(
        "Drop",
        rect.right() - 38.0,
        rect.y + 78.0,
        TextStyle::new(12.0, dark::TEXT_DIM).params(),
    );

    draw_carry_pips(ctx, rect, active_toy);
}

fn draw_empty_hands_card(rect: Rect) {
    let icon_rect = Rect::new(rect.x + 18.0, rect.y + 20.0, 58.0, 58.0);
    draw_surface(
        icon_rect,
        &SurfaceStyle::new(warm_card(0.94)).with_border(1.0, brass(0.48)),
    );
    draw_open_box_icon(
        vec2(
            icon_rect.x + icon_rect.w * 0.5,
            icon_rect.y + icon_rect.h * 0.53,
        ),
        19.0,
        Color::new(0.70, 0.60, 0.46, 0.82),
    );
    draw_ui_text_ex(
        "Hands empty",
        rect.x + 94.0,
        rect.y + 43.0,
        TextStyle::new(20.0, dark::TEXT_BRIGHT).params(),
    );
    draw_ui_text_ex(
        "Ready to sort",
        rect.x + 96.0,
        rect.y + 66.0,
        TextStyle::new(13.0, dark::TEXT_DIM).params(),
    );
}

fn draw_carry_pips(ctx: &UiContext<'_>, rect: Rect, active_toy: &ToyState) {
    let carried = ctx.session.player.carried_toy_ids.len();
    // The controls row, left of the drop hint: [Q] then one pip per toy.
    let mut x = rect.x + 94.0;
    let y = rect.y + 66.0;

    // The key that moves the ring, next to the ring it moves. Only once there
    // is more than one toy to cycle between: bare-handed the carry limit is
    // one, and a dead key on screen is worse than no key. Until now `Q` was
    // named exactly once, in the Sorting Trolley's shop description, which the
    // player reads at the moment of purchase and never again — the same shape
    // of gap as the trolley having no input path at all.
    if carried > 1 {
        draw_keycap(Rect::new(x, y - 3.0, 23.0, 21.0), "Q", false);
        x += 32.0;
    }

    for (index, toy_id) in ctx.session.player.carried_toy_ids.iter().enumerate() {
        let Some(toy) = ctx
            .session
            .toys
            .iter()
            .find(|candidate| &candidate.id == toy_id)
        else {
            continue;
        };

        let pip = Rect::new(x, y, 18.0, 18.0);
        draw_identity_token(pip, toy);
        if toy.id == active_toy.id || index == ctx.session.player.active_carry_index {
            draw_rectangle_lines(
                pip.x - 3.0,
                pip.y - 3.0,
                pip.w + 6.0,
                pip.h + 6.0,
                1.5,
                Color::new(1.0, 0.73, 0.22, 0.95),
            );
        }
        x += 25.0;
    }
}

fn draw_context_prompt(ctx: &UiContext<'_>) {
    let Some(prompt) = prompt_for_interaction(ctx) else {
        return;
    };

    let rect = prompt_rect();
    let border = prompt.tone.border();
    draw_hud_panel(rect, warm_panel(0.92), border);

    let text_x = if let Some(key) = prompt.key {
        draw_keycap(
            Rect::new(rect.x + 22.0, rect.y + 10.0, 34.0, 32.0),
            key,
            true,
        );
        rect.x + 78.0
    } else {
        draw_prompt_status_icon(vec2(rect.x + 39.0, rect.y + 26.0), prompt.tone);
        rect.x + 68.0
    };

    let badge_width = if let Some(toy) = ctx.session.active_toy() {
        if prompt.tone == PromptTone::Action || prompt.tone == PromptTone::Good {
            draw_identity_token(
                Rect::new(rect.right() - 50.0, rect.y + 15.0, 24.0, 24.0),
                toy,
            );
            54.0
        } else {
            20.0
        }
    } else {
        20.0
    };

    draw_fitted_text(
        &prompt.message,
        text_x,
        rect.y + 32.0,
        rect.right() - text_x - badge_width,
        20.0,
        prompt.tone.text_color(),
    );
}

fn prompt_for_interaction(ctx: &UiContext<'_>) -> Option<PromptVisual> {
    let prompt = match ctx.session.interaction_preview(ctx.data) {
        InteractionPreview::PlaceOnShelf => PromptVisual::action("ACT", "Place on shelf"),
        InteractionPreview::PlaceOnRepairBench => PromptVisual::action("ACT", "Place on bench"),
        InteractionPreview::RepairReady { toy_name } => {
            PromptVisual::action("ACT", format!("Repair {toy_name}"))
        }
        InteractionPreview::RepairBenchFull => PromptVisual::warning("Bench full"),
        // Now the common case is carrying a part to a bench already holding
        // someone else's half, so name what is in the way, not just "no".
        InteractionPreview::RepairMismatch => {
            PromptVisual::warning("Bench holds another toy's half")
        }
        InteractionPreview::AwaitingRepairMatch {
            toy_name,
            missing_part,
            // Missing half first: the prompt is width-fitted and long toy names
            // ("Cozy Critters Bear #03") truncate, so the one fact the player
            // needs must survive the ellipsis.
        } => PromptVisual::neutral(format!("Needs the {} - {toy_name}", missing_part.label())),
        InteractionPreview::NeedsRepair => PromptVisual::warning("Repair at the bench first"),
        InteractionPreview::PutDown => PromptVisual::action("ACT", "Place on floor"),
        InteractionPreview::Pickup { toy_name } => {
            PromptVisual::action("ACT", format!("Pick up {toy_name}"))
        }
        InteractionPreview::InventoryFull => PromptVisual::warning("Carry full"),
        InteractionPreview::ShelfFull => PromptVisual::warning("Shelf full"),
        InteractionPreview::LookAtEmptySlot => PromptVisual::neutral("Aim at an empty shelf spot"),
        InteractionPreview::NothingNearby => PromptVisual::neutral("Use the view controls to look"),
        InteractionPreview::Finished => PromptVisual::good("Store restored"),
        InteractionPreview::ShiftOver => PromptVisual::warning("The doors are open"),
    };
    Some(prompt)
}

fn draw_crosshair(ctx: &UiContext<'_>) {
    if ctx.session.phase != GamePhase::Playing {
        return;
    }

    let center = vec2(LOGICAL_WIDTH * 0.5, LOGICAL_HEIGHT * 0.5);
    let color = Color::new(1.0, 0.75, 0.24, 0.90);
    draw_circle_lines(center.x, center.y, 11.0, 1.2, color);
    draw_circle(
        center.x,
        center.y,
        2.0,
        Color::new(color.r, color.g, color.b, color.a * 0.86),
    );
    draw_line(
        center.x - 17.0,
        center.y,
        center.x - 7.0,
        center.y,
        1.2,
        color,
    );
    draw_line(
        center.x + 7.0,
        center.y,
        center.x + 17.0,
        center.y,
        1.2,
        color,
    );
    draw_line(
        center.x,
        center.y - 17.0,
        center.x,
        center.y - 7.0,
        1.2,
        color,
    );
    draw_line(
        center.x,
        center.y + 7.0,
        center.x,
        center.y + 17.0,
        1.2,
        color,
    );
}

fn status_panel_rect() -> Rect {
    // A shelf-like header keeps the centre of the shop visible while grouping
    // clock, whole-store progress, trolley load, and the current aisle.
    Rect::new(18.0, 16.0, 594.0, 104.0)
}

fn carried_card_rect() -> Rect {
    Rect::new(18.0, LOGICAL_HEIGHT - 120.0, 360.0, 102.0)
}

fn prompt_rect() -> Rect {
    Rect::new(
        (LOGICAL_WIDTH - 454.0) * 0.5,
        LOGICAL_HEIGHT - 82.0,
        454.0,
        58.0,
    )
}

fn hud_border() -> Color {
    brass(0.72)
}

fn subtle_border() -> Color {
    brass(0.50)
}
