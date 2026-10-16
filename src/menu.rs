use macroquad::prelude::*;

use crate::utils;

pub struct Menu {
    pub is_open: bool,
}

impl Menu {
    pub fn new() -> Self {
        Self { is_open: false }
    }
}

pub fn update(menu: &mut Menu) {
    if is_key_pressed(KeyCode::M) {
        menu.is_open = !menu.is_open;
    }
}

pub fn draw(menu: &Menu) {
    if !menu.is_open {
        return;
    }

    let panel_size = vec2(360.0, 240.0);
    let panel_position = vec2(
        (screen_width() - panel_size.x) * 0.5,
        (screen_height() - panel_size.y) * 0.5,
    );

    draw_rectangle(
        0.0,
        0.0,
        screen_width(),
        screen_height(),
        Color::from_rgba(0, 0, 0, 150),
    );
    draw_rectangle(
        panel_position.x,
        panel_position.y,
        panel_size.x,
        panel_size.y,
        Color::from_rgba(32, 32, 32, 245),
    );
    draw_rectangle_lines(
        panel_position.x,
        panel_position.y,
        panel_size.x,
        panel_size.y,
        2.0,
        Color::from_rgba(180, 180, 180, 255),
    );

    let title = "Menu";
    let title_size = utils::measure_ui_text(title, 32);
    utils::draw_ui_text(
        title,
        panel_position.x + (panel_size.x - title_size.width) * 0.5,
        panel_position.y + 55.0,
        32.0,
        WHITE,
    );
    utils::draw_ui_text(
        "Pressione M para fechar",
        panel_position.x + 70.0,
        panel_position.y + 130.0,
        20.0,
        LIGHTGRAY,
    );
}
