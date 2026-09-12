//! F3-toggled debug overlay: frame timing, draw counts, and player pose.
//!
//! Wraps the shared `macroquad_toolkit::debug::DebugOverlay` (toggle state
//! and smoothed FPS/frame-time) with toybox's own panel layout, since it
//! sits beside the HUD status panel rather than the toolkit's default
//! top-left corner.

use crate::ui::scene3d::SceneStats;
use crate::ui::UiContext;
use macroquad::prelude::*;
use macroquad_toolkit::debug::DebugOverlay as ToolkitDebugOverlay;
use macroquad_toolkit::prelude::*;
use macroquad_toolkit::ui::draw_ui_text_ex;

pub struct DebugOverlay {
    inner: ToolkitDebugOverlay,
}

impl DebugOverlay {
    pub fn new() -> Self {
        Self {
            inner: ToolkitDebugOverlay::new(),
        }
    }

    pub fn toggle(&mut self) {
        self.inner.toggle();
    }

    pub fn record_frame(&mut self, dt: f32) {
        self.inner.record_frame(dt);
    }

    pub fn draw(&self, ctx: &UiContext<'_>, stats: &SceneStats) {
        if !self.inner.visible {
            return;
        }

        let fps = self.inner.fps();
        let frame_ms = self.inner.frame_ms();
        let player = &ctx.session.player;
        let zone = ctx
            .data
            .layout
            .zone_name_at(player.position.x, player.position.y)
            .unwrap_or("Aisles");
        let lines = [
            format!("{:.0} FPS  {:.2} ms", fps, frame_ms),
            format!(
                "Toys drawn {} / {}",
                stats.drawn_toys,
                ctx.session.toys.len()
            ),
            format!(
                "Pos {:.1}, {:.1}  Yaw {:.0}",
                player.position.x,
                player.position.y,
                player.yaw.to_degrees().rem_euclid(360.0)
            ),
            format!("Zone {}", zone),
        ];

        // Sits to the right of the HUD status panel (18,16,190x166).
        let panel = Rect::new(222.0, 16.0, 236.0, 16.0 + lines.len() as f32 * 22.0);
        draw_surface(
            panel,
            &SurfaceStyle::new(Color::new(0.02, 0.03, 0.04, 0.82))
                .with_border(1.0, Color::new(0.55, 0.64, 0.72, 0.55)),
        );
        for (index, line) in lines.iter().enumerate() {
            draw_ui_text_ex(
                line,
                panel.x + 12.0,
                panel.y + 24.0 + index as f32 * 22.0,
                TextStyle::new(15.0, dark::TEXT_BRIGHT).params(),
            );
        }
    }
}

impl Default for DebugOverlay {
    fn default() -> Self {
        Self::new()
    }
}
