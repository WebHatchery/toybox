//! 3D shop scene orchestration and immediate-mode HUD for Toybox After Hours.

use crate::data::{GameData, UpgradeDef};
use crate::state::{
    GameSession, ShiftRecord, STOCKROOM_SPOTLIGHT_COST, STOCKROOM_SPOTLIGHT_MAX_SECONDS,
    STOCKROOM_SPOTLIGHT_NAME,
};
use macroquad::prelude::*;
use macroquad_toolkit::prelude::*;
use macroquad_toolkit::ui::draw_ui_text_ex;

mod ambience;
mod benches;
mod debug_overlay;
mod environment;
mod fixtures;
mod hud;
mod hud_chrome;
mod hud_icons;
mod minimap;
mod scanner;
pub mod scene3d;
mod score;
mod signs;
mod space;
mod title;
mod touch_controls;
mod tutorial;
mod widgets;
mod wood;

pub use debug_overlay::DebugOverlay;
use hud::draw_game_hud;
pub(crate) use hud_chrome::set_high_contrast;
use hud_chrome::{brass, draw_hud_panel, parchment, warm_card, warm_panel};
use scene3d::draw_shop_scene;
pub use space::{begin_ui_frame, end_ui_frame, set_ui_camera};
pub(crate) use title::{draw_help_screen, draw_settings_screen, draw_title_screen, SettingsView};
use widgets::{draw_fitted_text, draw_wrapped_text, WrapStyle};

pub const LOGICAL_WIDTH: f32 = 1280.0;
pub const LOGICAL_HEIGHT: f32 = 720.0;

thread_local! {
    static ANIMATION_SECONDS: std::cell::Cell<f32> = const { std::cell::Cell::new(0.0) };
}

/// Advance the view layer's animation clock by one frame.
///
/// Ambient animation — lamp breathing, dust motes, completion sparkles, the
/// scanner beacon pulse — used to read `get_time()`, macroquad's wall clock.
/// The capture harness simulates a fixed number of frames at a fixed timestep
/// *precisely* so a screenshot is reproducible, and the wall clock defeated
/// that: the same scene captured twice came out with different bytes, so no
/// drift check could ever distinguish a stale reference image from a fresh one.
/// Driving it from the same `dt` the simulation gets makes captures repeatable
/// and costs normal play nothing, since there `dt` is the frame time anyway.
pub fn advance_animation_clock(dt: f32) {
    ANIMATION_SECONDS.with(|clock| clock.set(clock.get() + dt));
}

/// Seconds of animation elapsed. View-only: gameplay state never reads this.
pub(crate) fn animation_seconds() -> f32 {
    ANIMATION_SECONDS.with(|clock| clock.get())
}

/// A credit count with the noun agreed, e.g. `1 credit` / `3 credits`.
///
/// Four places quote a price — the shop row, its refusal line, the HUD nudge
/// and the notification when Buy is refused — and all four wrote `credit(s)`.
/// The Toy Scanner costs exactly 1 and is the first tool anyone buys, so
/// `Costs 1 credit(s)` sat on the very first purchase decision a player makes.
/// One helper rather than four fixes, so they cannot drift apart later.
pub fn credits_phrase(count: usize) -> String {
    if count == 1 {
        "1 credit".to_owned()
    } else {
        format!("{count} credits")
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UiAction {
    NewGame,
    /// A shift with the deadline switched off.
    NewRelaxedGame,
    /// Restart the current mode with its exact scatter and broken-pair seed.
    ReplayShiftSeed,
    Continue,
    Settings,
    CloseSettings,
    BackToTitle,
    OpenToolShop,
    CloseToolShop,
    ToggleFullscreen,
    FovIncrease,
    FovDecrease,
    SensitivityIncrease,
    SensitivityDecrease,
    UiScaleIncrease,
    UiScaleDecrease,
    ToggleHighContrast,
    MasterVolumeIncrease,
    MasterVolumeDecrease,
    EffectsVolumeIncrease,
    EffectsVolumeDecrease,
    AmbienceVolumeIncrease,
    AmbienceVolumeDecrease,
    OpenHelp,
    CloseHelp,
    ReplayTutorial,
    QuitGame,
    Save,
    Load,
    Interact,
    CycleCarry,
    DropActive,
    SkipTutorial,
    BuyTool(String),
    BuyStockroomSpotlight,
    MoveForward,
    MoveBackward,
    MoveLeft,
    MoveRight,
    LookUp,
    LookDown,
    LookLeft,
    LookRight,
}

/// A compact but exact human-readable form of the persisted 64-bit seed.
pub(crate) fn shift_seed_code(seed: u64) -> String {
    format!(
        "{:04X}-{:04X}-{:04X}-{:04X}",
        seed >> 48,
        (seed >> 32) & 0xFFFF,
        (seed >> 16) & 0xFFFF,
        seed & 0xFFFF
    )
}

pub struct UiContext<'a> {
    pub data: &'a GameData,
    pub session: &'a GameSession,
    pub fov_degrees: f32,
    /// The best run recorded for the mode being played, if there is one, and
    /// whether the run now finishing beat it. Only the score screen reads them.
    pub best_run: Option<ShiftRecord>,
    pub beat_record: bool,
    pub tutorial_hint: Option<&'a crate::tutorial::TutorialHint>,
}

pub fn draw_game_ui(ctx: UiContext<'_>, overlay: &DebugOverlay) -> Vec<UiAction> {
    let stats = draw_shop_scene(&ctx);
    set_ui_camera();

    let mut actions = Vec::new();

    // The score screen is the whole message once a run ends; leaving the HUD
    // and minimap under it just crowds the panel with numbers it already shows.
    if ctx.session.phase.is_over() {
        actions.extend(score::draw_score_screen(&ctx));
    } else {
        draw_game_hud(&ctx);
        minimap::draw_minimap(&ctx);
        touch_controls::draw_touch_controls(&mut actions);
        if let Some(hint) = ctx.tutorial_hint {
            tutorial::draw_tutorial_hint(hint, &mut actions);
        }
    }
    overlay.draw(&ctx, &stats);

    actions
}

pub(crate) fn draw_tool_shop_screen(ctx: UiContext<'_>) -> Vec<UiAction> {
    draw_shop_scene(&ctx);
    set_ui_camera();

    draw_rectangle(
        0.0,
        0.0,
        LOGICAL_WIDTH,
        LOGICAL_HEIGHT,
        Color::new(0.055, 0.025, 0.012, 0.66),
    );

    let mut actions = Vec::new();
    let pointer = logical_pointer();
    // Tall enough for two description lines per row: at one line, four of the
    // five tools ended mid-word and the player was buying blind.
    let panel = Rect::new(260.0, 34.0, 760.0, 652.0);
    draw_hud_panel(panel, warm_panel(0.985), brass(0.82));

    draw_ui_text_ex(
        "RESTORATION TOOL RACK",
        panel.x + 30.0,
        panel.y + 40.0,
        TextStyle::new(25.0, parchment(1.0)).params(),
    );
    draw_ui_text_ex(
        "Restore displays to earn credits. Every tool lasts this shift.",
        panel.x + 31.0,
        panel.y + 68.0,
        TextStyle::new(14.0, parchment(0.64)).params(),
    );

    let credit_badge = Rect::new(panel.right() - 218.0, panel.y + 22.0, 92.0, 52.0);
    draw_surface(
        credit_badge,
        &SurfaceStyle::new(warm_card(0.98))
            .with_border(1.0, brass(0.62))
            .with_inner_border(3.0, 1.0, brass(0.12)),
    );
    draw_text_centered_in_box(
        &ctx.session.available_tool_credits(ctx.data).to_string(),
        credit_badge.x,
        credit_badge.y + 2.0,
        credit_badge.w,
        28.0,
        24.0,
        Color::new(1.0, 0.78, 0.28, 1.0),
    );
    draw_text_centered_in_box(
        "CREDITS",
        credit_badge.x,
        credit_badge.y + 29.0,
        credit_badge.w,
        17.0,
        10.0,
        parchment(0.62),
    );

    if tool_shop_button(
        Rect::new(panel.right() - 112.0, panel.y + 27.0, 82.0, 42.0),
        "Back",
        true,
        pointer,
    ) {
        actions.push(UiAction::CloseToolShop);
    }

    if ctx.session.all_tools_owned(ctx.data) {
        draw_stockroom_service(panel, &ctx, pointer, &mut actions);
    } else {
        for (index, upgrade) in ctx.data.upgrades.iter().enumerate() {
            let row = Rect::new(
                panel.x + 24.0,
                panel.y + 104.0 + index as f32 * 104.0,
                panel.w - 48.0,
                92.0,
            );
            draw_tool_row(row, upgrade, &ctx, pointer, &mut actions);
        }
    }

    actions
}

fn draw_stockroom_service(
    panel: Rect,
    ctx: &UiContext<'_>,
    pointer: Pointer,
    actions: &mut Vec<UiAction>,
) {
    draw_ui_text_ex(
        "Every shift tool is on the trolley.",
        panel.x + 28.0,
        panel.y + 120.0,
        TextStyle::new(18.0, Color::new(0.66, 0.94, 0.70, 1.0)).params(),
    );
    let owned = ctx
        .data
        .upgrades
        .iter()
        .map(|upgrade| upgrade.name.as_str())
        .collect::<Vec<_>>()
        .join("  -  ");
    draw_wrapped_text(
        &owned,
        panel.x + 28.0,
        panel.y + 151.0,
        panel.w - 56.0,
        WrapStyle {
            size: 14.0,
            line_height: 20.0,
            max_lines: 2,
            color: dark::TEXT_DIM,
        },
    );

    let card = Rect::new(panel.x + 28.0, panel.y + 222.0, panel.w - 56.0, 190.0);
    draw_surface(
        card,
        &SurfaceStyle::new(Color::new(0.14, 0.085, 0.025, 0.94))
            .with_border(2.0, brass(0.86))
            .with_inner_border(5.0, 1.0, brass(0.18)),
    );
    draw_ui_text_ex(
        STOCKROOM_SPOTLIGHT_NAME,
        card.x + 20.0,
        card.y + 34.0,
        TextStyle::new(22.0, Color::new(1.0, 0.84, 0.42, 1.0)).params(),
    );
    draw_wrapped_text(
        "Shines a gold beacon over the nearest loose toy for 60 seconds. You still carry, sort, and repair it.",
        card.x + 20.0,
        card.y + 64.0,
        card.w - 164.0,
        WrapStyle {
            size: 15.0,
            line_height: 21.0,
            max_lines: 3,
            color: dark::TEXT,
        },
    );

    let seconds = ctx.session.player.stockroom_spotlight_seconds;
    let at_capacity = seconds + f32::EPSILON >= STOCKROOM_SPOTLIGHT_MAX_SECONDS;
    let can_buy =
        !at_capacity && ctx.session.available_tool_credits(ctx.data) >= STOCKROOM_SPOTLIGHT_COST;
    let status = if at_capacity {
        "At 3:00 maximum".to_owned()
    } else if seconds > 0.0 {
        format!("Active: {:.0}s  -  Costs 1 credit", seconds.ceil())
    } else {
        "Ready  -  Costs 1 credit".to_owned()
    };
    draw_ui_text_ex(
        &status,
        card.x + 20.0,
        card.bottom() - 24.0,
        TextStyle::new(14.0, Color::new(0.96, 0.78, 0.36, 1.0)).params(),
    );
    let button = Rect::new(card.right() - 122.0, card.y + 62.0, 94.0, 42.0);
    if tool_shop_button(button, "Call", can_buy, pointer) {
        actions.push(UiAction::BuyStockroomSpotlight);
    }
}

pub fn movement_from_keys() -> Vec2 {
    let mut direction = vec2(0.0, 0.0);

    if is_key_down(KeyCode::W) {
        direction.y += 1.0;
    }
    if is_key_down(KeyCode::D) {
        direction.x += 1.0;
    }
    if is_key_down(KeyCode::S) {
        direction.y -= 1.0;
    }
    if is_key_down(KeyCode::A) {
        direction.x -= 1.0;
    }

    direction
}

pub fn continuous_mouse_delta_pixels() -> Vec2 {
    let local_delta = mouse_delta_position();
    vec2(
        -local_delta.x * screen_width() * 0.5,
        -local_delta.y * screen_height() * 0.5,
    )
}

pub fn look_delta_from_input(
    mouse_delta: Vec2,
    dt: f32,
    sensitivity: f32,
    sensitivity_min: f32,
    sensitivity_max: f32,
) -> Vec2 {
    let mut yaw_delta = mouse_delta.x * 0.0032;
    let mut pitch_delta = -mouse_delta.y * 0.0032;

    let keyboard_speed = 1.75 * dt;
    if is_key_down(KeyCode::Left) {
        yaw_delta -= keyboard_speed;
    }
    if is_key_down(KeyCode::Right) {
        yaw_delta += keyboard_speed;
    }
    if is_key_down(KeyCode::Up) {
        pitch_delta += keyboard_speed;
    }
    if is_key_down(KeyCode::Down) {
        pitch_delta -= keyboard_speed;
    }

    vec2(yaw_delta, pitch_delta) * sensitivity.clamp(sensitivity_min, sensitivity_max)
}

fn draw_tool_row(
    rect: Rect,
    upgrade: &UpgradeDef,
    ctx: &UiContext<'_>,
    pointer: Pointer,
    actions: &mut Vec<UiAction>,
) {
    let (status, status_color, can_buy) = tool_status(upgrade, ctx);
    draw_surface(
        rect,
        &SurfaceStyle::new(warm_card(0.94))
            .with_border(
                1.0,
                Color::new(status_color.r, status_color.g, status_color.b, 0.42),
            )
            .with_inner_border(3.0, 1.0, brass(0.08)),
    );
    draw_rectangle(
        rect.x,
        rect.y,
        6.0,
        rect.h,
        Color::new(status_color.r, status_color.g, status_color.b, 0.78),
    );
    draw_circle(
        rect.x + 28.0,
        rect.y + 29.0,
        15.0,
        Color::new(0.22, 0.12, 0.055, 0.98),
    );
    draw_circle_lines(rect.x + 28.0, rect.y + 29.0, 15.0, 1.0, brass(0.62));
    draw_text_centered_in_box(
        &format!("{}", upgrade.unlock_completed_displays),
        rect.x + 13.0,
        rect.y + 14.0,
        30.0,
        30.0,
        14.0,
        parchment(0.92),
    );

    draw_ui_text_ex(
        &upgrade.name,
        rect.x + 54.0,
        rect.y + 26.0,
        TextStyle::new(19.0, parchment(1.0)).params(),
    );
    let last_line = draw_wrapped_text(
        &upgrade.description,
        rect.x + 54.0,
        rect.y + 48.0,
        rect.w - 214.0,
        WrapStyle {
            size: 14.0,
            line_height: 18.0,
            max_lines: 2,
            color: dark::TEXT,
        },
    );

    draw_fitted_text(
        &status,
        rect.x + 54.0,
        last_line + 20.0,
        rect.w - 214.0,
        13.0,
        status_color,
    );

    let button = Rect::new(rect.right() - 132.0, rect.y + 25.0, 108.0, 42.0);
    let button_label = if ctx.session.has_upgrade(&upgrade.id) {
        "Owned"
    } else if can_buy {
        "Buy"
    } else if ctx.session.completed_display_count() < upgrade.unlock_completed_displays {
        "Locked"
    } else {
        "Need credit"
    };
    if tool_shop_button(button, button_label, can_buy, pointer) {
        actions.push(UiAction::BuyTool(upgrade.id.clone()));
    }
}

fn tool_status(upgrade: &UpgradeDef, ctx: &UiContext<'_>) -> (String, Color, bool) {
    if ctx.session.has_upgrade(&upgrade.id) {
        return ("Owned".to_owned(), Color::new(0.60, 0.92, 0.66, 1.0), false);
    }

    let completed = ctx.session.completed_display_count();
    if completed < upgrade.unlock_completed_displays {
        return (
            format!(
                "Locked: {}/{} displays restored",
                completed, upgrade.unlock_completed_displays
            ),
            dark::TEXT_DIM,
            false,
        );
    }

    let credits = ctx.session.available_tool_credits(ctx.data);
    if credits < upgrade.cost {
        return (
            format!(
                "Need {}. You have {}",
                credits_phrase(upgrade.cost),
                credits
            ),
            Color::new(0.95, 0.72, 0.36, 1.0),
            false,
        );
    }

    (
        // "Available: 1 credit(s)" reads as the player's balance, not the
        // price — actively wrong next to a Tool Credits counter showing 9.
        format!("Costs {}", credits_phrase(upgrade.cost)),
        Color::new(0.56, 0.92, 0.92, 1.0),
        true,
    )
}

fn tool_shop_button(rect: Rect, label: &str, enabled: bool, pointer: Pointer) -> bool {
    let hovered = enabled && pointer.hovering_over(rect);
    let pressed = enabled && pointer.pressing(rect);
    let activated = enabled && pointer.released_on(rect);
    let face = if !enabled {
        Color::new(0.085, 0.055, 0.042, 0.90)
    } else if pressed {
        Color::new(0.31, 0.17, 0.055, 0.98)
    } else if hovered {
        Color::new(0.41, 0.24, 0.075, 0.98)
    } else {
        Color::new(0.28, 0.15, 0.060, 0.96)
    };
    draw_surface(
        rect,
        &SurfaceStyle::new(face).with_border(1.0, if enabled { brass(0.86) } else { brass(0.28) }),
    );
    draw_text_centered_in_box(
        label,
        rect.x + 8.0,
        rect.y,
        rect.w - 16.0,
        rect.h,
        14.0,
        if enabled {
            parchment(1.0)
        } else {
            parchment(0.42)
        },
    );
    activated
}

fn logical_pointer() -> Pointer {
    Pointer::read(|position| {
        vec2(
            position.x * LOGICAL_WIDTH / screen_width().max(1.0),
            position.y * LOGICAL_HEIGHT / screen_height().max(1.0),
        )
    })
}
