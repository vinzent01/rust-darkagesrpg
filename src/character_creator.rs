use hecs::{Entity, World};
use macroquad::prelude::*;

use crate::body::{get_all_hair_colors, get_all_skin_colors, Appearence};
use crate::database::Database;
use crate::player::Player;

const SKIN_NAMES: [&str; 3] = ["Clara", "Morena", "Escura"];
const HAIR_COLOR_NAMES: [&str; 4] = ["Ruivo", "Branco", "Loiro", "Moreno"];

pub struct CharacterCreatorUI {
    rect: Rect,
}

#[derive(Clone, Copy)]
struct Layout {
    preview: Rect,
    male: Rect,
    female: Rect,
    skin: [Rect; 3],
    hair_previous: Rect,
    hair_next: Rect,
    beard_previous: Rect,
    beard_next: Rect,
    hair_colors: [Rect; 4],
    close: Rect,
}

pub fn update(world: &mut World, database: &Database) -> bool {
    if is_key_pressed(KeyCode::P) {
        if let Some(entity) = creator_entity(world) {
            world
                .despawn(entity)
                .expect("character creator disappeared");
        } else {
            let width = (screen_width() - 32.0).min(900.0);
            let height = (screen_height() - 32.0).min(620.0);
            world.spawn((CharacterCreatorUI {
                rect: Rect::new(
                    (screen_width() - width) * 0.5,
                    (screen_height() - height) * 0.5,
                    width,
                    height,
                ),
            },));
        }
    }

    let Some(ui_entity) = creator_entity(world) else {
        return false;
    };
    let rect = world
        .get::<&CharacterCreatorUI>(ui_entity)
        .expect("character creator disappeared")
        .rect;
    if is_key_pressed(KeyCode::Escape) {
        world
            .despawn(ui_entity)
            .expect("character creator disappeared while closing");
        return false;
    }

    let layout = layout(rect);
    if is_mouse_button_pressed(MouseButton::Left) {
        let mouse = mouse_position().into();
        if layout.close.contains(mouse) {
            world
                .despawn(ui_entity)
                .expect("character creator disappeared while closing");
            return false;
        }

        let mut player_query = world.query::<(&Player, &mut Appearence)>();
        if let Some((_, appearance)) = player_query.iter().next() {
            if layout.male.contains(mouse) {
                appearance.body.genre = true;
            } else if layout.female.contains(mouse) {
                appearance.body.genre = false;
            } else if let Some(index) = layout.skin.iter().position(|button| button.contains(mouse))
            {
                appearance.skin_color = get_all_skin_colors()[index];
            } else if layout.hair_previous.contains(mouse) && !database.hairs.is_empty() {
                appearance.hair = Some(
                    appearance
                        .hair
                        .unwrap_or(0)
                        .checked_add(database.hairs.len() - 1)
                        .unwrap_or(0)
                        % database.hairs.len(),
                );
            } else if layout.hair_next.contains(mouse) && !database.hairs.is_empty() {
                appearance.hair =
                    Some(appearance.hair.unwrap_or(0).wrapping_add(1) % database.hairs.len());
            } else if layout.beard_previous.contains(mouse) && !database.beards.is_empty() {
                let selected = appearance.beard.map_or(0, |index| index + 1);
                let previous = if selected == 0 {
                    database.beards.len()
                } else {
                    selected - 1
                };
                appearance.beard = previous.checked_sub(1);
            } else if layout.beard_next.contains(mouse) && !database.beards.is_empty() {
                let selected = appearance.beard.map_or(0, |index| index + 1);
                let next = (selected + 1) % (database.beards.len() + 1);
                appearance.beard = (next > 0).then_some(next - 1);
            } else if let Some(index) = layout
                .hair_colors
                .iter()
                .position(|button| button.contains(mouse))
            {
                appearance.hair_color = get_all_hair_colors()[index];
            }
        }
    }

    true
}

pub fn draw(world: &World, database: &Database) {
    let Some(ui_entity) = creator_entity(world) else {
        return;
    };
    let ui = world
        .get::<&CharacterCreatorUI>(ui_entity)
        .expect("character creator disappeared");
    let rect = ui.rect;
    let layout = layout(rect);

    draw_rectangle(
        0.0,
        0.0,
        screen_width(),
        screen_height(),
        Color::from_rgba(0, 0, 0, 150),
    );
    draw_rectangle(
        rect.x,
        rect.y,
        rect.w,
        rect.h,
        Color::from_rgba(28, 30, 34, 250),
    );
    draw_rectangle_lines(rect.x, rect.y, rect.w, rect.h, 2.0, LIGHTGRAY);
    draw_text(
        "Criador de personagem",
        rect.x + 20.0,
        rect.y + 36.0,
        26.0,
        WHITE,
    );
    draw_button(layout.close, "X", false);
    draw_preview(world, database, layout.preview);
    draw_controls(world, database, rect, layout);
    draw_text(
        "P ou Esc para fechar",
        rect.x + rect.w - 190.0,
        rect.y + rect.h - 14.0,
        16.0,
        LIGHTGRAY,
    );
}

fn draw_preview(world: &World, database: &Database, rect: Rect) {
    draw_rectangle(
        rect.x,
        rect.y,
        rect.w,
        rect.h,
        Color::from_rgba(18, 20, 23, 255),
    );
    draw_rectangle_lines(rect.x, rect.y, rect.w, rect.h, 1.0, DARKGRAY);

    let mut player_query = world.query::<(&Player, &Appearence)>();
    let Some((_, appearance)) = player_query.iter().next() else {
        draw_text(
            "Jogador indisponivel",
            rect.x + 12.0,
            rect.y + 30.0,
            16.0,
            WHITE,
        );
        return;
    };
    let Some(race) = database.races.get(appearance.body.race) else {
        draw_text(
            "Corpo indisponivel",
            rect.x + 12.0,
            rect.y + 30.0,
            16.0,
            WHITE,
        );
        return;
    };
    let body = if appearance.body.genre {
        &race.male_sprite
    } else {
        &race.female_sprite
    };
    let scale = ((rect.w - 28.0) / body.texture.width())
        .min((rect.h - 100.0) / body.texture.height())
        .max(1.0);
    let body_x = rect.x + (rect.w - body.texture.width() * scale) * 0.5;
    let body_y = rect.y + 48.0 + (rect.h - 100.0 - body.texture.height() * scale) * 0.5;

    draw_texture_ex(
        &body.texture,
        body_x,
        body_y,
        appearance.skin_color,
        DrawTextureParams {
            dest_size: Some(vec2(
                body.texture.width() * scale,
                body.texture.height() * scale,
            )),
            ..Default::default()
        },
    );

    if let Some(beard_id) = appearance.beard {
        if let Some(beard) = database.beards.get(beard_id) {
            let offset = beard.offset_for(appearance.body.genre);
            draw_texture_ex(
                &beard.texture,
                body_x + offset.x * scale,
                body_y + offset.y * scale,
                appearance.hair_color,
                DrawTextureParams {
                    dest_size: Some(vec2(
                        beard.texture.width() * scale,
                        beard.texture.height() * scale,
                    )),
                    ..Default::default()
                },
            );
        }
    }
    if let Some(hair_id) = appearance.hair {
        if let Some(hair) = database.hairs.get(hair_id) {
            let offset = hair.offset_for(appearance.body.genre);
            draw_texture_ex(
                &hair.texture,
                body_x + offset.x * scale,
                body_y + offset.y * scale,
                appearance.hair_color,
                DrawTextureParams {
                    dest_size: Some(vec2(
                        hair.texture.width() * scale,
                        hair.texture.height() * scale,
                    )),
                    ..Default::default()
                },
            );
        }
    }

    let gender = if appearance.body.genre {
        "Masculino"
    } else {
        "Feminino"
    };
    let text_width = measure_text(gender, None, 18, 1.0).width;
    draw_text(
        gender,
        rect.x + (rect.w - text_width) * 0.5,
        rect.y + rect.h - 18.0,
        18.0,
        WHITE,
    );
}

fn draw_controls(world: &World, database: &Database, rect: Rect, layout: Layout) {
    let mut player_query = world.query::<(&Player, &Appearence)>();
    let Some((_, appearance)) = player_query.iter().next() else {
        return;
    };

    draw_text("Genero", layout.male.x, layout.male.y - 8.0, 18.0, WHITE);
    draw_button(layout.male, "Masculino", appearance.body.genre);
    draw_button(layout.female, "Feminino", !appearance.body.genre);

    draw_text(
        "Tom de pele",
        layout.skin[0].x,
        layout.skin[0].y - 8.0,
        18.0,
        WHITE,
    );
    for (index, button) in layout.skin.iter().enumerate() {
        let colors = get_all_skin_colors();
        draw_swatch(
            *button,
            SKIN_NAMES[index],
            colors[index],
            colors[index] == appearance.skin_color,
        );
    }

    let hair_name = appearance
        .hair
        .and_then(|index| database.hairs.get(index))
        .map_or("Nenhum", |hair| hair.name);
    draw_text(
        "Cabelo",
        layout.hair_previous.x,
        layout.hair_previous.y - 8.0,
        18.0,
        WHITE,
    );
    draw_button(layout.hair_previous, "<", false);
    draw_button(layout.hair_next, ">", false);
    draw_centered_text(
        hair_name,
        Rect::new(
            layout.hair_previous.x + layout.hair_previous.w + 8.0,
            layout.hair_previous.y,
            layout.hair_next.x - layout.hair_previous.x - layout.hair_previous.w - 16.0,
            layout.hair_previous.h,
        ),
        18,
        WHITE,
    );

    let beard_name = appearance
        .beard
        .and_then(|index| database.beards.get(index))
        .map_or("Sem barba", |beard| beard.name);
    draw_text(
        "Barba",
        layout.beard_previous.x,
        layout.beard_previous.y - 8.0,
        18.0,
        WHITE,
    );
    draw_button(layout.beard_previous, "<", false);
    draw_button(layout.beard_next, ">", false);
    draw_centered_text(
        beard_name,
        Rect::new(
            layout.beard_previous.x + layout.beard_previous.w + 8.0,
            layout.beard_previous.y,
            layout.beard_next.x - layout.beard_previous.x - layout.beard_previous.w - 16.0,
            layout.beard_previous.h,
        ),
        18,
        WHITE,
    );

    draw_text(
        "Cor dos cabelos e da barba",
        layout.hair_colors[0].x,
        layout.hair_colors[0].y - 8.0,
        18.0,
        WHITE,
    );
    let colors = get_all_hair_colors();
    for (index, button) in layout.hair_colors.iter().enumerate() {
        draw_swatch(
            *button,
            HAIR_COLOR_NAMES[index],
            colors[index],
            colors[index] == appearance.hair_color,
        );
    }

    if rect.w < 500.0 {
        draw_text(
            "Ajuste a janela para ver todos os controles.",
            rect.x + 18.0,
            rect.y + rect.h - 40.0,
            14.0,
            LIGHTGRAY,
        );
    }
}

fn draw_button(rect: Rect, label: &str, selected: bool) {
    draw_rectangle(
        rect.x,
        rect.y,
        rect.w,
        rect.h,
        if selected {
            Color::from_rgba(72, 83, 95, 255)
        } else {
            Color::from_rgba(42, 45, 50, 255)
        },
    );
    draw_rectangle_lines(
        rect.x,
        rect.y,
        rect.w,
        rect.h,
        1.0,
        if selected { GOLD } else { LIGHTGRAY },
    );
    draw_centered_text(label, rect, 16, WHITE);
}

fn draw_swatch(rect: Rect, label: &str, color: Color, selected: bool) {
    draw_rectangle(rect.x, rect.y, rect.w, rect.h, color);
    draw_rectangle_lines(
        rect.x,
        rect.y,
        rect.w,
        rect.h,
        if selected { 3.0 } else { 1.0 },
        if selected { GOLD } else { LIGHTGRAY },
    );
    let luminance = color.r * 0.299 + color.g * 0.587 + color.b * 0.114;
    let text_color = if luminance > 0.5 { BLACK } else { WHITE };
    draw_centered_text(label, rect, 15, text_color);
}

fn draw_centered_text(text: &str, rect: Rect, font_size: u16, color: Color) {
    let dimensions = measure_text(text, None, font_size, 1.0);
    draw_text(
        text,
        rect.x + (rect.w - dimensions.width) * 0.5,
        rect.y + (rect.h + dimensions.height) * 0.5,
        f32::from(font_size),
        color,
    );
}

fn layout(rect: Rect) -> Layout {
    let preview = Rect::new(rect.x + 18.0, rect.y + 62.0, 210.0, rect.h - 82.0);
    let controls_x = rect.x + 252.0;
    let controls_width = (rect.x + rect.w - 20.0 - controls_x).max(150.0);
    let button_gap = 8.0;
    let gender_width = (controls_width - button_gap) * 0.5;
    let gender_y = rect.y + 90.0;
    let skin_y = rect.y + 168.0;
    let selector_height = 34.0;
    let selector_width = 38.0;
    let hair_y = rect.y + 250.0;
    let beard_y = rect.y + 332.0;
    let colors_y = rect.y + 420.0;
    let swatch_gap = 8.0;
    let skin_width = (controls_width - swatch_gap * 2.0) / 3.0;
    let hair_color_width = (controls_width - swatch_gap * 3.0) / 4.0;

    Layout {
        preview,
        male: Rect::new(controls_x, gender_y, gender_width, selector_height),
        female: Rect::new(
            controls_x + gender_width + button_gap,
            gender_y,
            gender_width,
            selector_height,
        ),
        skin: std::array::from_fn(|index| {
            Rect::new(
                controls_x + index as f32 * (skin_width + swatch_gap),
                skin_y,
                skin_width,
                selector_height,
            )
        }),
        hair_previous: Rect::new(controls_x, hair_y, selector_width, selector_height),
        hair_next: Rect::new(
            controls_x + controls_width - selector_width,
            hair_y,
            selector_width,
            selector_height,
        ),
        beard_previous: Rect::new(controls_x, beard_y, selector_width, selector_height),
        beard_next: Rect::new(
            controls_x + controls_width - selector_width,
            beard_y,
            selector_width,
            selector_height,
        ),
        hair_colors: std::array::from_fn(|index| {
            Rect::new(
                controls_x + index as f32 * (hair_color_width + swatch_gap),
                colors_y,
                hair_color_width,
                selector_height,
            )
        }),
        close: Rect::new(rect.x + rect.w - 42.0, rect.y + 10.0, 30.0, 30.0),
    }
}

fn creator_entity(world: &World) -> Option<Entity> {
    world
        .query::<Entity>()
        .iter()
        .find(|entity| world.get::<&CharacterCreatorUI>(*entity).is_ok())
}
