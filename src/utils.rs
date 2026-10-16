use macroquad::prelude::*;

pub const TILE_SIZE: f32 = 32.0;

pub fn draw_ui_text(text: &str, x: f32, y: f32, font_size: f32, color: Color) {
    let font = get_default_font();
    let (font_size, font_scale, font_scale_aspect) = camera_font_scale(font_size);
    draw_text_ex(
        text,
        x,
        y,
        TextParams {
            font: Some(&font),
            font_size,
            font_scale,
            font_scale_aspect,
            color,
            ..Default::default()
        },
    );
}

pub fn measure_ui_text(text: &str, font_size: u16) -> TextDimensions {
    let font = get_default_font();
    measure_text(text, Some(&font), font_size, 1.0)
}

pub fn bezier(p0: Vec2, p1: Vec2, p2: Vec2, t: f32) -> Vec2 {
    let x =
        (1.0 - t).powi(2) * p0.x +
        2.0 * (1.0 - t) * t * p1.x +
        t.powi(2) * p2.x;

    let y =
        (1.0 - t).powi(2) * p0.y +
        2.0 * (1.0 - t) * t * p1.y +
        t.powi(2) * p2.y;

    vec2(x, y)
}

pub fn cell_to_world(x: i32, y: i32) -> Vec2 {
    vec2(
        x as f32 * TILE_SIZE,
        y as f32 * TILE_SIZE,
    )
}

pub fn draw_grid() {
    for x in 0..30 {
        draw_line(
            x as f32 * TILE_SIZE,
            0.0,
            x as f32 * TILE_SIZE,
            screen_height(),
            1.0,
            DARKGRAY,
        );
    }

    for y in 0..20 {
        draw_line(
            0.0,
            y as f32 * TILE_SIZE,
            screen_width(),
            y as f32 * TILE_SIZE,
            1.0,
            DARKGRAY,
        );
    }
}



pub fn max_f(value1 : f32, value2: f32) -> f32{
    if value1 > value2{
        return value1;
    }
    else {
        return value2;
    }
}