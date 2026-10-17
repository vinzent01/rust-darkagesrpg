use hecs::{Entity, World};
use macroquad::prelude::*;

use crate::body::{Appearence, get_all_hair_colors, get_all_skin_colors};
use crate::database::Database;
use crate::player::{self, Player};
use crate::utils;

const SKIN_NAMES: [&str; 3] = ["Clara", "Morena", "Escura"];
const HAIR_COLOR_NAMES: [&str; 4] = ["Ruivo", "Branco", "Loiro", "Moreno"];

pub struct CharacterCreatorUI {
    rect: Rect,
    appearance: Appearence,
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
    confirm: Rect,
}

pub fn open_for_creation(world: &mut World) {
    open(world, player::default_appearance());
}

pub fn update(world: &mut World, database: &Database) -> bool {
    if is_key_pressed(KeyCode::P) && player_appearance(world).is_some() {
        if let Some(entity) = creator_entity(world) {
            finish_creation(world, entity);
            return true;
        }
        open(
            world,
            player_appearance(world).expect("player appearance disappeared"),
        );
    }

    let Some(ui_entity) = creator_entity(world) else {
        return false;
    };
    let (rect, initial_creation) = world
        .get::<&CharacterCreatorUI>(ui_entity)
        .map(|ui| (ui.rect, player_appearance(world).is_none()))
        .expect("character creator disappeared");
    if is_key_pressed(KeyCode::Escape) && !initial_creation {
        finish_creation(world, ui_entity);
        return true;
    }

    let layout = layout(rect);
    if is_mouse_button_pressed(MouseButton::Left) {
        let mouse = mouse_position().into();
        if layout.confirm.contains(mouse) || (layout.close.contains(mouse) && !initial_creation) {
            finish_creation(world, ui_entity);
            return true;
        }

        let mut ui = world
            .get::<&mut CharacterCreatorUI>(ui_entity)
            .expect("character creator disappeared");
        let appearance = &mut ui.appearance;
        if layout.male.contains(mouse) {
            appearance.body.genre = true;
        } else if layout.female.contains(mouse) {
            appearance.body.genre = false;
        } else if let Some(index) = layout.skin.iter().position(|button| button.contains(mouse)) {
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
            appearance.beard = next_beard(appearance.beard, database.beards.len());
        } else if let Some(index) = layout
            .hair_colors
            .iter()
            .position(|button| button.contains(mouse))
        {
            appearance.hair_color = get_all_hair_colors()[index];
        }
    }

    true
}

fn open(world: &mut World, appearance: Appearence) {
    let width = (screen_width() - 32.0).min(900.0);
    let height = (screen_height() - 32.0).min(620.0);
    world.spawn((CharacterCreatorUI {
        rect: Rect::new(
            (screen_width() - width) * 0.5,
            (screen_height() - height) * 0.5,
            width,
            height,
        ),
        appearance,
    },));
}

fn player_appearance(world: &World) -> Option<Appearence> {
    world
        .query::<(&Player, &Appearence)>()
        .iter()
        .next()
        .map(|(_, appearance)| appearance.clone())
}

fn finish_creation(world: &mut World, ui_entity: Entity) {
    let appearance = world
        .get::<&CharacterCreatorUI>(ui_entity)
        .expect("character creator disappeared")
        .appearance
        .clone();

    if let Some((player_entity, _)) = world.query::<(Entity, &Player)>().iter().next() {
        *world
            .get::<&mut Appearence>(player_entity)
            .expect("player appearance disappeared") = appearance;
    } else {
        player::spawn_player(world, appearance);
    }

    world
        .despawn(ui_entity)
        .expect("character creator disappeared while closing");
}

pub fn draw(world: &World, database: &Database) {
    let Some(ui_entity) = creator_entity(world) else {
        return;
    };
    let ui = world
        .get::<&CharacterCreatorUI>(ui_entity)
        .expect("character creator disappeared");
    let rect = ui.rect;
    let initial_creation = player_appearance(world).is_none();
    let appearance = &ui.appearance;
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
    utils::draw_ui_text(
        "Criador de personagem",
        rect.x + 20.0,
        rect.y + 36.0,
        26.0,
        WHITE,
    );
    if !initial_creation {
        draw_button(layout.close, "X", false);
    }
    draw_preview(database, layout.preview, appearance);
    draw_controls(database, rect, layout, appearance);
    draw_button(
        layout.confirm,
        if initial_creation {
            "Criar personagem"
        } else {
            "Concluir"
        },
        false,
    );
    utils::draw_ui_text(
        if initial_creation {
            "Confirme para entrar no mundo"
        } else {
            "P ou Esc para fechar"
        },
        rect.x + rect.w - 190.0,
        rect.y + rect.h - 14.0,
        16.0,
        LIGHTGRAY,
    );
}

fn draw_preview(database: &Database, rect: Rect, appearance: &Appearence) {
    draw_rectangle(
        rect.x,
        rect.y,
        rect.w,
        rect.h,
        Color::from_rgba(18, 20, 23, 255),
    );
    draw_rectangle_lines(rect.x, rect.y, rect.w, rect.h, 1.0, DARKGRAY);

    let Some(race) = database.races.get(appearance.body.race) else {
        utils::draw_ui_text(
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
    let body_texture = database.texture(&body.texture_path);
    let scale = ((rect.w - 28.0) / body_texture.width())
        .min((rect.h - 100.0) / body_texture.height())
        .max(1.0);
    let body_x = rect.x + (rect.w - body_texture.width() * scale) * 0.5;
    let body_y = rect.y + 48.0 + (rect.h - 100.0 - body_texture.height() * scale) * 0.5;

    draw_texture_ex(
        body_texture,
        body_x,
        body_y,
        appearance.skin_color,
        DrawTextureParams {
            dest_size: Some(vec2(
                body_texture.width() * scale,
                body_texture.height() * scale,
            )),
            ..Default::default()
        },
    );

    if let Some(beard_id) = appearance.beard {
        if let Some(beard) = database.beards.get(beard_id) {
            let offset = beard.offset_for(appearance.body.genre);
            let texture = database.texture(beard.texture_path);
            draw_texture_ex(
                texture,
                body_x + offset.x * scale,
                body_y + offset.y * scale,
                appearance.hair_color,
                DrawTextureParams {
                    dest_size: Some(vec2(
                        texture.width() * scale,
                        texture.height() * scale,
                    )),
                    ..Default::default()
                },
            );
        }
    }
    if let Some(hair_id) = appearance.hair {
        if let Some(hair) = database.hairs.get(hair_id) {
            let offset = hair.offset_for(appearance.body.genre);
            let texture = database.texture(hair.texture_path);
            draw_texture_ex(
                texture,
                body_x + offset.x * scale,
                body_y + offset.y * scale,
                appearance.hair_color,
                DrawTextureParams {
                    dest_size: Some(vec2(
                        texture.width() * scale,
                        texture.height() * scale,
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
    let text_width = utils::measure_ui_text(gender, 18).width;
    utils::draw_ui_text(
        gender,
        rect.x + (rect.w - text_width) * 0.5,
        rect.y + rect.h - 18.0,
        18.0,
        WHITE,
    );
}

fn draw_controls(database: &Database, rect: Rect, layout: Layout, appearance: &Appearence) {
    utils::draw_ui_text("Genero", layout.male.x, layout.male.y - 8.0, 18.0, WHITE);
    draw_button(layout.male, "Masculino", appearance.body.genre);
    draw_button(layout.female, "Feminino", !appearance.body.genre);

    utils::draw_ui_text(
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
    utils::draw_ui_text(
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
    utils::draw_ui_text(
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

    utils::draw_ui_text(
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
        utils::draw_ui_text(
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
    let dimensions = utils::measure_ui_text(text, font_size);
    utils::draw_ui_text(
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
        confirm: Rect::new(rect.x + rect.w - 190.0, rect.y + rect.h - 58.0, 170.0, 36.0),
    }
}

fn creator_entity(world: &World) -> Option<Entity> {
    world
        .query::<Entity>()
        .iter()
        .find(|entity| world.get::<&CharacterCreatorUI>(*entity).is_ok())
}

fn next_beard(current: Option<usize>, beard_count: usize) -> Option<usize> {
    match current {
        None if beard_count > 0 => Some(0),
        Some(index) => index.checked_add(1).filter(|next| *next < beard_count),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::{CharacterCreatorUI, creator_entity, finish_creation, next_beard};
    use crate::movement::Position;
    use crate::player::{self, Player};
    use crate::tilemap::MAP_CENTER_CELL;
    use crate::utils::cell_to_world;
    use hecs::World;
    use macroquad::prelude::Rect;

    #[test]
    fn player_is_spawned_at_map_center_only_after_creation_is_confirmed() {
        let mut world = World::new();
        let creator = world.spawn((CharacterCreatorUI {
            rect: Rect::new(0.0, 0.0, 100.0, 100.0),
            appearance: player::default_appearance(),
        },));

        assert!(world.query::<&Player>().iter().next().is_none());

        finish_creation(&mut world, creator);

        let mut players = world.query::<(&Player, &Position)>();
        let (_, position) = players.iter().next().expect("player was not spawned");
        assert_eq!(
            position.value,
            cell_to_world(MAP_CENTER_CELL.0, MAP_CENTER_CELL.1)
        );
        assert!(creator_entity(&world).is_none());
    }

    #[test]
    fn next_beard_wraps_after_the_last_beard_to_none() {
        assert_eq!(next_beard(Some(4), 5), None);
    }

    #[test]
    fn next_beard_selects_the_first_beard_from_none() {
        assert_eq!(next_beard(None, 5), Some(0));
    }

    #[test]
    fn next_beard_advances_to_the_following_beard() {
        assert_eq!(next_beard(Some(2), 5), Some(3));
    }
}
