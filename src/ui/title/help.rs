//! Help and controls screen.

use super::*;

pub(crate) fn draw_help_screen(title_texture: Option<&Texture2D>) -> Vec<UiAction> {
    draw_title_background(title_texture);
    draw_title_scrim();
    let pointer = super::super::logical_pointer();
    let mut actions = Vec::new();
    let panel = Rect::new(224.0, 82.0, 832.0, 570.0);
    draw_surface(
        panel,
        &SurfaceStyle::new(Color::new(0.09, 0.045, 0.022, 0.97))
            .with_border(2.0, Color::new(0.86, 0.62, 0.25, 0.82))
            .with_inner_border(7.0, 1.0, Color::new(1.0, 0.84, 0.48, 0.14)),
    );
    draw_text_centered_in_box(
        "Controls & How to Play",
        panel.x,
        panel.y + 22.0,
        panel.w,
        40.0,
        28.0,
        title_parchment(),
    );
    draw_text_centered_in_box(
        "Restore every display before the doors open — or take your time in Relaxed Run.",
        panel.x,
        panel.y + 62.0,
        panel.w,
        26.0,
        14.0,
        Color::new(0.92, 0.78, 0.56, 0.84),
    );

    draw_help_column(
        panel.x + 48.0,
        panel.y + 116.0,
        "CONTROLS",
        &[
            ("WASD", "Walk the shop floor"),
            ("View", "Drag or tap the arrow buttons to look"),
            ("ACT", "Pick up · shelf · repair"),
            ("CARRY", "Cycle the Sorting Trolley"),
            ("DROP", "Put the active toy down"),
            ("TOOLS", "Open the tool rack"),
            ("PAUSE", "Pause and open settings"),
            ("F5", "Replay this exact layout"),
        ],
    );
    draw_help_column(
        panel.x + 432.0,
        panel.y + 116.0,
        "THE CLOSING ROUTINE",
        &[
            ("1", "Match toy category to display"),
            ("2", "Rejoin broken pairs at a bench"),
            ("3", "Finish displays to earn credits"),
            ("4", "Buy tools that speed the shift"),
            ("5", "Use the map for aisle progress"),
            ("6", "Shelve all 240 toys to finish"),
        ],
    );

    let y = panel.bottom() - 62.0;
    if title_button(
        Rect::new(panel.x + 184.0, y, 216.0, 40.0),
        "Replay First-Shift Guide",
        true,
        ButtonTone::Primary,
        pointer,
    ) {
        actions.push(UiAction::ReplayTutorial);
    }
    if title_button(
        Rect::new(panel.x + 432.0, y, 216.0, 40.0),
        "Back to Settings",
        true,
        ButtonTone::Muted,
        pointer,
    ) {
        actions.push(UiAction::CloseHelp);
    }
    actions
}

fn draw_help_column(x: f32, y: f32, heading: &str, rows: &[(&str, &str)]) {
    draw_ui_text_ex(
        heading,
        x,
        y,
        TextStyle::new(14.0, Color::new(1.0, 0.70, 0.24, 0.94)).params(),
    );
    for (index, (key, text)) in rows.iter().enumerate() {
        let row_y = y + 34.0 + index as f32 * 43.0;
        draw_plaque(
            Rect::new(x, row_y - 21.0, 78.0, 29.0),
            &title_plaque_style(),
            &title_button_palette(ButtonTone::Muted),
            PlaqueState::idle(true),
        );
        draw_text_centered_in_box(key, x, row_y - 22.0, 78.0, 29.0, 12.0, title_parchment());
        draw_ui_text_ex(
            text,
            x + 92.0,
            row_y,
            TextStyle::new(14.0, Color::new(0.94, 0.88, 0.76, 0.92)).params(),
        );
    }
}
