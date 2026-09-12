//! Static in-world stock signs with procedural sign-face textures.

use crate::data::{DisplayDef, GameData, PosterDef, ToyCategory, ZoneDef};
use crate::ui::fixtures::{display_style, DisplayStyle};
use macroquad::prelude::*;
use macroquad_toolkit::raster::fill_rect;
use std::cell::RefCell;
use std::collections::HashMap;

mod font;
use font::{draw_pixel_text_centered, pixel_text_width};

thread_local! {
    static SIGN_TEXTURES: RefCell<HashMap<String, Texture2D>> = RefCell::new(HashMap::new());
}

/// Which side a textured cube face is read from. macroquad's `draw_cube`
/// reuses the front-face UVs on every face, so the back face mirrors
/// horizontally and the ±x faces flip vertically; each view needs its own
/// baked orientation for text to read correctly.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum FaceView {
    FromPosZ,
    FromNegZ,
    FromPosX,
    FromNegX,
}

impl FaceView {
    fn tag(self) -> &'static str {
        match self {
            FaceView::FromPosZ => "pz",
            FaceView::FromNegZ => "nz",
            FaceView::FromPosX => "px",
            FaceView::FromNegX => "nx",
        }
    }
}

/// Orient an authored image (row 0 = visual top) for the given view and
/// upload it.
fn finalize_texture(mut image: Image, view: FaceView) -> Texture2D {
    match view {
        FaceView::FromPosZ => flip_image_vertical(&mut image),
        FaceView::FromNegZ => {
            flip_image_vertical(&mut image);
            flip_image_horizontal(&mut image);
        }
        FaceView::FromPosX => flip_image_horizontal(&mut image),
        FaceView::FromNegX => {}
    }
    let texture = Texture2D::from_image(&image);
    texture.set_filter(FilterMode::Nearest);
    texture
}

#[derive(Debug, Clone, Copy)]
struct SignLayout {
    panel_center: Vec3,
    panel_size: Vec3,
    face_z_sign: f32,
    hanging: bool,
}

pub fn draw_stock_sign(display: &DisplayDef, accent: Color) {
    let layout = sign_layout(display);
    let board = Color::new(0.12, 0.09, 0.06, 1.0);

    if layout.hanging {
        draw_hanging_sign_wires(layout, accent);
        draw_cube(
            layout.panel_center,
            layout.panel_size,
            None,
            Color::new(0.10, 0.075, 0.055, 1.0),
        );
        // Readable panel on both approaches.
        for (offset, view) in [(-0.048, FaceView::FromNegZ), (0.048, FaceView::FromPosZ)] {
            draw_textured_face(
                display,
                accent,
                layout.panel_center + vec3(0.0, 0.0, offset),
                vec3(
                    layout.panel_size.x * 0.90,
                    layout.panel_size.y * 0.64,
                    0.030,
                ),
                view,
            );
        }
    } else {
        draw_cube(layout.panel_center, layout.panel_size, None, board);
        let view = if layout.face_z_sign > 0.0 {
            FaceView::FromPosZ
        } else {
            FaceView::FromNegZ
        };
        draw_textured_face(
            display,
            accent,
            layout.panel_center + vec3(0.0, 0.0, layout.face_z_sign * 0.067),
            vec3(
                layout.panel_size.x * 0.90,
                layout.panel_size.y * 0.66,
                0.030,
            ),
            view,
        );
        draw_cube_wires(layout.panel_center, layout.panel_size, accent);
        draw_cube(
            layout.panel_center + vec3(-layout.panel_size.x * 0.42, -0.28, 0.0),
            vec3(0.055, 0.48, 0.055),
            None,
            board,
        );
        draw_cube(
            layout.panel_center + vec3(layout.panel_size.x * 0.42, -0.28, 0.0),
            vec3(0.055, 0.48, 0.055),
            None,
            board,
        );
    }
}

/// Double-sided hanging sign with the zone name, suspended over the zone
/// center on accent-colored wires.
pub fn draw_zone_sign(zone: &ZoneDef) {
    let accent = Color::new(
        zone.accent[0],
        zone.accent[1],
        zone.accent[2],
        zone.accent[3],
    );
    let center = vec3(zone.x + zone.w * 0.5, 2.02, zone.y + zone.h * 0.5);
    let panel_size = vec3(2.6, 0.5, 0.14);

    for x in [-1.0_f32, 1.0] {
        draw_line_3d(
            center + vec3(x * 1.1, 0.26, 0.0),
            vec3(center.x + x * 1.1, 2.30, center.z),
            Color::new(accent.r, accent.g, accent.b, 0.82),
        );
    }
    draw_cube(
        center,
        panel_size,
        None,
        Color::new(0.10, 0.075, 0.055, 1.0),
    );
    draw_cube_wires(
        center,
        panel_size,
        Color::new(accent.r, accent.g, accent.b, 0.55),
    );

    for (z_sign, view) in [(-1.0_f32, FaceView::FromNegZ), (1.0, FaceView::FromPosZ)] {
        draw_zone_face(
            zone,
            accent,
            center + vec3(0.0, 0.0, z_sign * 0.085),
            vec3(panel_size.x * 0.92, panel_size.y * 0.72, 0.030),
            view,
        );
    }
}

/// Framed pixel-text posters pinned to the perimeter walls, defined in
/// layout.json (wall + offset + text + accent).
pub fn draw_wall_posters(data: &GameData) {
    let frame = Color::new(0.12, 0.09, 0.06, 1.0);
    let room_w = data.config.room_width;
    let room_h = data.config.room_height;

    for poster in &data.layout.posters {
        let height = poster.width * 0.42;
        // (center, frame size, face size, face inset direction, reading side)
        let (center, frame_size, face_size, face_push, view) = match poster.wall.as_str() {
            "front" => (
                vec3(poster.offset, poster.center_y, 0.05),
                vec3(poster.width + 0.10, height + 0.10, 0.03),
                vec3(poster.width, height, 0.03),
                vec3(0.0, 0.0, 0.02),
                FaceView::FromPosZ,
            ),
            "back" => (
                vec3(poster.offset, poster.center_y, room_h - 0.05),
                vec3(poster.width + 0.10, height + 0.10, 0.03),
                vec3(poster.width, height, 0.03),
                vec3(0.0, 0.0, -0.02),
                FaceView::FromNegZ,
            ),
            "west" => (
                vec3(0.05, poster.center_y, poster.offset),
                vec3(0.03, height + 0.10, poster.width + 0.10),
                vec3(0.03, height, poster.width),
                vec3(0.02, 0.0, 0.0),
                FaceView::FromPosX,
            ),
            _ => (
                vec3(room_w - 0.05, poster.center_y, poster.offset),
                vec3(0.03, height + 0.10, poster.width + 0.10),
                vec3(0.03, height, poster.width),
                vec3(-0.02, 0.0, 0.0),
                FaceView::FromNegX,
            ),
        };

        draw_cube(center, frame_size, None, frame);
        let accent = Color::new(
            poster.accent[0],
            poster.accent[1],
            poster.accent[2],
            poster.accent[3],
        );
        SIGN_TEXTURES.with(|textures| {
            let mut textures = textures.borrow_mut();
            let texture = textures
                .entry(format!("poster:{}:{}", poster.text, view.tag()))
                .or_insert_with(|| finalize_texture(build_poster_image(poster, accent), view));
            draw_cube(center + face_push, face_size, Some(texture), WHITE);
        });
    }
}

fn build_poster_image(poster: &PosterDef, accent: Color) -> Image {
    let paper = Color::new(0.91, 0.87, 0.78, 1.0);
    let mut image = Image::gen_image_color(160, 64, paper);

    // Accent bands top and bottom, plus corner pin dots.
    fill_rect(&mut image, 0, 0, 160, 7, accent);
    fill_rect(&mut image, 0, 57, 160, 7, accent);
    for &(x, y) in &[(6_i32, 12_i32), (150, 12), (6, 48), (150, 48)] {
        fill_rect(&mut image, x, y, 4, 4, Color::new(0.30, 0.26, 0.22, 1.0));
    }

    let text = poster.text.to_ascii_uppercase();
    let scale = if pixel_text_width(&text, 2) <= 146 {
        2
    } else {
        1
    };
    let text_y = (64 - 7 * scale) / 2;
    draw_pixel_text_centered(
        &mut image,
        &text,
        text_y,
        scale,
        Color::new(0.16, 0.13, 0.10, 1.0),
    );

    image
}

fn draw_zone_face(zone: &ZoneDef, accent: Color, center: Vec3, size: Vec3, view: FaceView) {
    SIGN_TEXTURES.with(|textures| {
        let mut textures = textures.borrow_mut();
        let texture = textures
            .entry(format!("zone:{}:{}", zone.name, view.tag()))
            .or_insert_with(|| finalize_texture(build_zone_sign_image(zone, accent), view));
        draw_cube(center, size, Some(texture), WHITE);
    });
}

fn build_zone_sign_image(zone: &ZoneDef, accent: Color) -> Image {
    let face = Color::new(
        (accent.r * 0.42).clamp(0.0, 1.0),
        (accent.g * 0.42).clamp(0.0, 1.0),
        (accent.b * 0.42).clamp(0.0, 1.0),
        1.0,
    );
    let mut image = Image::gen_image_color(160, 40, Color::new(0.055, 0.038, 0.025, 1.0));
    fill_rect(&mut image, 3, 3, 154, 34, face);

    let name = zone.name.to_ascii_uppercase();
    let scale = if pixel_text_width(&name, 2) <= 148 {
        2
    } else {
        1
    };
    let text_y = (40 - 7 * scale) / 2;
    draw_pixel_text_centered(
        &mut image,
        &name,
        text_y,
        scale,
        Color::new(0.98, 0.94, 0.76, 1.0),
    );

    image
}

fn draw_textured_face(
    display: &DisplayDef,
    accent: Color,
    center: Vec3,
    size: Vec3,
    view: FaceView,
) {
    SIGN_TEXTURES.with(|textures| {
        let mut textures = textures.borrow_mut();
        let texture = textures
            .entry(format!("{}:{}", display.id, view.tag()))
            .or_insert_with(|| finalize_texture(build_sign_image(display, accent), view));
        draw_cube(center, size, Some(texture), WHITE);
    });
}

fn build_sign_image(display: &DisplayDef, accent: Color) -> Image {
    let face = Color::new(
        (accent.r * 0.58).clamp(0.0, 1.0),
        (accent.g * 0.58).clamp(0.0, 1.0),
        (accent.b * 0.58).clamp(0.0, 1.0),
        1.0,
    );
    let mut image = Image::gen_image_color(128, 64, Color::new(0.055, 0.038, 0.025, 1.0));
    fill_rect(&mut image, 4, 4, 120, 56, face);
    fill_rect(
        &mut image,
        8,
        8,
        112,
        48,
        Color::new(
            (face.r + 0.06).clamp(0.0, 1.0),
            (face.g + 0.05).clamp(0.0, 1.0),
            (face.b + 0.04).clamp(0.0, 1.0),
            1.0,
        ),
    );
    draw_border(&mut image, Color::new(0.04, 0.030, 0.020, 1.0));

    let (title, theme) = stock_label(display);
    draw_pixel_text_centered(&mut image, title, 15, 2, Color::new(0.98, 0.94, 0.76, 1.0));
    draw_pixel_text_centered(
        &mut image,
        &theme.to_ascii_uppercase(),
        43,
        1,
        Color::new(0.05, 0.045, 0.038, 1.0),
    );

    image
}

fn flip_image_horizontal(image: &mut Image) {
    let width = image.width() as u32;
    let height = image.height() as u32;
    for y in 0..height {
        for x in 0..width / 2 {
            let opposite_x = width - 1 - x;
            let left = image.get_pixel(x, y);
            let right = image.get_pixel(opposite_x, y);
            image.set_pixel(x, y, right);
            image.set_pixel(opposite_x, y, left);
        }
    }
}

fn flip_image_vertical(image: &mut Image) {
    let width = image.width() as u32;
    let height = image.height() as u32;
    for y in 0..height / 2 {
        let opposite_y = height - 1 - y;
        for x in 0..width {
            let top = image.get_pixel(x, y);
            let bottom = image.get_pixel(x, opposite_y);
            image.set_pixel(x, y, bottom);
            image.set_pixel(x, opposite_y, top);
        }
    }
}

fn draw_border(image: &mut Image, color: Color) {
    fill_rect(image, 4, 4, 120, 3, color);
    fill_rect(image, 4, 57, 120, 3, color);
    fill_rect(image, 4, 4, 3, 56, color);
    fill_rect(image, 121, 4, 3, 56, color);
    for x in (14..114).step_by(12) {
        fill_rect(image, x, 9, 5, 2, Color::new(0.95, 0.82, 0.42, 0.78));
        fill_rect(image, x, 53, 5, 2, Color::new(0.95, 0.82, 0.42, 0.62));
    }
}

fn draw_hanging_sign_wires(layout: SignLayout, accent: Color) {
    let ceiling_y = 2.24;
    let half_x = layout.panel_size.x * 0.42;
    let half_z = layout.panel_size.z * 0.42;
    for x in [-half_x, half_x] {
        for z in [-half_z, half_z] {
            draw_line_3d(
                layout.panel_center + vec3(x, layout.panel_size.y * 0.55, z),
                vec3(
                    layout.panel_center.x + x,
                    ceiling_y,
                    layout.panel_center.z + z,
                ),
                Color::new(accent.r, accent.g, accent.b, 0.82),
            );
        }
    }
}

fn sign_layout(display: &DisplayDef) -> SignLayout {
    let center_x = display.x + display.w * 0.5;
    let center_z = display.y + display.h * 0.5;
    match display_style(display) {
        DisplayStyle::Table => SignLayout {
            panel_center: vec3(center_x, 1.96, center_z),
            panel_size: vec3(display.w * 0.64, 0.46, display.h * 0.28),
            face_z_sign: -1.0,
            hanging: true,
        },
        DisplayStyle::Bin => SignLayout {
            panel_center: vec3(center_x, 1.26, display.y + display.h * 0.54),
            panel_size: vec3(display.w * 0.82, 0.36, 0.10),
            face_z_sign: 1.0,
            hanging: false,
        },
        DisplayStyle::Shelf => SignLayout {
            panel_center: vec3(center_x, 1.90, center_z - 0.48),
            panel_size: vec3(display.w * 0.76, 0.38, 0.10),
            face_z_sign: -1.0,
            hanging: false,
        },
        DisplayStyle::Pegboard => SignLayout {
            panel_center: vec3(center_x, 2.32, display.y + display.h - 0.38),
            panel_size: vec3(display.w * 0.78, 0.38, 0.10),
            face_z_sign: -1.0,
            hanging: false,
        },
        DisplayStyle::Wall => SignLayout {
            panel_center: vec3(center_x, 2.32, display.y + 0.34),
            panel_size: vec3(display.w * 0.82, 0.38, 0.10),
            face_z_sign: 1.0,
            hanging: false,
        },
        DisplayStyle::Generic => SignLayout {
            panel_center: vec3(center_x, 1.70, center_z),
            panel_size: vec3(display.w * 0.70, 0.36, 0.10),
            face_z_sign: -1.0,
            hanging: false,
        },
    }
}

fn stock_label(display: &DisplayDef) -> (&'static str, &str) {
    let title = match display.category {
        ToyCategory::Plushies => "PLUSHIES",
        ToyCategory::TinyDragons => "DRAGONS",
        ToyCategory::ActionFigures => "ROBOTS",
        ToyCategory::BoardGames => "BOARD GAMES",
        ToyCategory::BuildingBlocks => "BLOCK SETS",
    };
    (title, &display.theme)
}
