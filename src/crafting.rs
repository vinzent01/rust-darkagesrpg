use hecs::{Entity, World};
use macroquad::prelude::*;

use crate::database::Database;
use crate::inventory::{self, Inventory};
use crate::player::Player;

const WINDOW_PADDING: f32 = 16.0;
const LIST_ROW_HEIGHT: f32 = 38.0;

pub struct CraftingUI {
    rect: Rect,
    selected: usize,
    scroll: usize,
    status: String,
}

pub fn update(world: &mut World, database: &Database) -> bool {
    if is_key_pressed(KeyCode::C) {
        let existing_ui = world
            .query::<Entity>()
            .iter()
            .find(|entity| world.get::<&CraftingUI>(*entity).is_ok());
        if let Some(entity) = existing_ui {
            world.despawn(entity).expect("crafting UI disappeared");
        } else {
            let size = vec2(
                (screen_width() - 32.0).min(800.0),
                (screen_height() - 32.0).min(520.0),
            );
            world.spawn((CraftingUI {
                rect: Rect::new(
                    (screen_width() - size.x) * 0.5,
                    (screen_height() - size.y) * 0.5,
                    size.x,
                    size.y,
                ),
                selected: 0,
                scroll: 0,
                status: String::new(),
            },));
        }
    }

    let Some(ui_entity) = world
        .query::<Entity>()
        .iter()
        .find(|entity| world.get::<&CraftingUI>(*entity).is_ok())
    else {
        return false;
    };

    if is_key_pressed(KeyCode::Escape) {
        world
            .despawn(ui_entity)
            .expect("crafting UI disappeared while closing");
        return false;
    }

    let recipe_ids = recipe_ids(database);
    let mut craft_item_id = None;
    if let Ok(mut ui) = world.get::<&mut CraftingUI>(ui_entity) {
        let visible_rows = visible_rows(ui.rect);
        let list_rect = recipe_list_rect(ui.rect);
        if list_rect.contains(mouse_position().into()) {
            let (_, wheel_y) = mouse_wheel();
            if wheel_y != 0.0 {
                if wheel_y < 0.0 {
                    ui.scroll = ui.scroll.saturating_add(1);
                } else {
                    ui.scroll = ui.scroll.saturating_sub(1);
                }
                ui.scroll = ui.scroll.min(recipe_ids.len().saturating_sub(visible_rows));
            }
        }

        if is_mouse_button_pressed(MouseButton::Left) && list_rect.contains(mouse_position().into())
        {
            let mouse = mouse_position();
            let row = ((mouse.1 - list_rect.y) / LIST_ROW_HEIGHT).floor() as usize;
            let selected = ui.scroll + row;
            if selected < recipe_ids.len() {
                ui.selected = selected;
                ui.status.clear();
            }
        }

        let craft_button = craft_button_rect(ui.rect);
        if is_mouse_button_pressed(MouseButton::Left)
            && craft_button.contains(mouse_position().into())
        {
            craft_item_id = recipe_ids.get(ui.selected).cloned();
        }
    }

    if let Some(item_id) = craft_item_id {
        let status = inventory::craft_item(world, database, &item_id)
            .map(|()| "Crafting concluido.".to_string())
            .unwrap_or_else(|error| error);
        if let Ok(mut ui) = world.get::<&mut CraftingUI>(ui_entity) {
            ui.status = status;
        }
    }

    true
}

pub fn draw(world: &World, database: &Database) {
    let recipe_ids = recipe_ids(database);
    for ui in world.query::<&CraftingUI>().iter() {
        let rect = ui.rect;
        draw_rectangle(
            rect.x,
            rect.y,
            rect.w,
            rect.h,
            Color::from_rgba(28, 30, 34, 250),
        );
        draw_rectangle_lines(rect.x, rect.y, rect.w, rect.h, 2.0, LIGHTGRAY);
        draw_text("Crafting", rect.x + WINDOW_PADDING, rect.y + 30.0, 24.0, WHITE);
        draw_text(
            "C ou Esc para fechar",
            rect.x + rect.w - 190.0,
            rect.y + 28.0,
            16.0,
            LIGHTGRAY,
        );

        let list_rect = recipe_list_rect(rect);
        draw_rectangle(
            list_rect.x,
            list_rect.y,
            list_rect.w,
            list_rect.h,
            Color::from_rgba(18, 20, 23, 255),
        );
        for row in 0..visible_rows(rect) {
            let index = ui.scroll + row;
            let Some(item_id) = recipe_ids.get(index) else {
                break;
            };
            let item = &database.items[item_id];
            let row_y = list_rect.y + row as f32 * LIST_ROW_HEIGHT;
            let selected = index == ui.selected;
            draw_rectangle(
                list_rect.x + 4.0,
                row_y + 2.0,
                list_rect.w - 8.0,
                LIST_ROW_HEIGHT - 4.0,
                if selected {
                    Color::from_rgba(72, 83, 95, 255)
                } else {
                    Color::from_rgba(42, 45, 50, 255)
                },
            );
            draw_text(
                &item.name,
                list_rect.x + 14.0,
                row_y + 25.0,
                18.0,
                WHITE,
            );
        }

        let detail_x = list_rect.x + list_rect.w + WINDOW_PADDING;
        let detail_width = rect.x + rect.w - WINDOW_PADDING - detail_x;
        if let Some(item_id) = recipe_ids.get(ui.selected) {
            let item = &database.items[item_id];
            draw_text(&item.name, detail_x, list_rect.y + 25.0, 22.0, WHITE);
            if let Some(texture) = database.textures.get(&item.texture_path) {
                draw_texture_ex(
                    texture,
                    detail_x,
                    list_rect.y + 42.0,
                    WHITE,
                    DrawTextureParams {
                        dest_size: Some(vec2(64.0, 64.0)),
                        ..Default::default()
                    },
                );
            }
            draw_wrapped_text(
                &item.description,
                detail_x,
                list_rect.y + 126.0,
                detail_width,
                17.0,
                LIGHTGRAY,
            );

            draw_text("Materiais necessarios:", detail_x, list_rect.y + 208.0, 18.0, WHITE);
            if let Some(recipe) = &item.recipe {
                for (index, ingredient) in recipe.ingredients.iter().enumerate() {
                    let ingredient_name = database
                        .items
                        .get(&ingredient.item_id)
                        .map(|item| item.name.as_str())
                        .unwrap_or(&ingredient.item_id);
                    let available = player_item_quantity(world, &ingredient.item_id);
                    let enough = available >= ingredient.quantity;
                    draw_text(
                        &format!(
                            "{ingredient_name}: {available}/{}",
                            ingredient.quantity
                        ),
                        detail_x,
                        list_rect.y + 236.0 + index as f32 * 24.0,
                        16.0,
                        if enough { GREEN } else { RED },
                    );
                }
                draw_text(
                    &format!("Produz: {}x {}", recipe.output_quantity, item.name),
                    detail_x,
                    list_rect.y + 236.0 + recipe.ingredients.len() as f32 * 24.0,
                    16.0,
                    LIGHTGRAY,
                );
            }

            let can_craft = can_craft(world, database, item_id);
            let button = craft_button_rect(rect);
            draw_rectangle(
                button.x,
                button.y,
                button.w,
                button.h,
                if can_craft {
                    Color::from_rgba(43, 105, 58, 255)
                } else {
                    Color::from_rgba(65, 65, 65, 255)
                },
            );
            draw_rectangle_lines(button.x, button.y, button.w, button.h, 1.0, LIGHTGRAY);
            let label = "Craftar";
            let label_width = measure_text(label, None, 18, 1.0).width;
            draw_text(
                label,
                button.x + (button.w - label_width) * 0.5,
                button.y + 25.0,
                18.0,
                WHITE,
            );
            if !ui.status.is_empty() {
                draw_text(
                    &ui.status,
                    detail_x,
                    button.y + 25.0,
                    14.0,
                    if ui.status == "Crafting concluido." {
                        GREEN
                    } else {
                        RED
                    },
                );
            }
        } else {
            draw_text("Nenhuma receita cadastrada.", detail_x, list_rect.y + 28.0, 18.0, LIGHTGRAY);
        }
    }
}

fn can_craft(world: &World, database: &Database, item_id: &str) -> bool {
    let Some(item) = database.items.get(item_id) else {
        return false;
    };
    let Some(recipe) = &item.recipe else {
        return false;
    };
    let mut player_query = world.query::<(Entity, &Player, &Inventory)>();
    let Some((_, _, inventory)) = player_query.iter().next() else {
        return false;
    };
    recipe.ingredients.iter().all(|ingredient| {
        inventory
            .items
            .iter()
            .flatten()
            .filter(|item| item.id == ingredient.item_id)
            .map(|item| item.quantity)
            .sum::<i32>()
            >= ingredient.quantity
    })
}

fn player_item_quantity(world: &World, item_id: &str) -> i32 {
    world
        .query::<(&Player, &Inventory)>()
        .iter()
        .next()
        .map(|(_, inventory)| {
            inventory
                .items
                .iter()
                .flatten()
                .filter(|item| item.id == item_id)
                .map(|item| item.quantity)
                .sum()
        })
        .unwrap_or(0)
}

fn recipe_ids(database: &Database) -> Vec<String> {
    let mut ids: Vec<String> = database
        .items
        .values()
        .filter(|item| item.recipe.is_some())
        .map(|item| item.id.clone())
        .collect();
    ids.sort();
    ids
}

fn recipe_list_rect(rect: Rect) -> Rect {
    let list_x = rect.x + WINDOW_PADDING;
    let list_y = rect.y + 52.0;
    let list_width = ((rect.w - WINDOW_PADDING * 3.0) * 0.42).max(160.0);
    Rect::new(
        list_x,
        list_y,
        list_width,
        (rect.h - 116.0).max(LIST_ROW_HEIGHT),
    )
}

fn visible_rows(rect: Rect) -> usize {
    (recipe_list_rect(rect).h / LIST_ROW_HEIGHT).floor().max(1.0) as usize
}

fn craft_button_rect(rect: Rect) -> Rect {
    Rect::new(rect.x + rect.w - 132.0, rect.y + rect.h - 48.0, 112.0, 32.0)
}

fn draw_wrapped_text(text: &str, x: f32, y: f32, max_width: f32, line_height: f32, color: Color) {
    let mut line = String::new();
    let mut line_y = y;
    for word in text.split_whitespace() {
        let candidate = if line.is_empty() {
            word.to_string()
        } else {
            format!("{line} {word}")
        };
        if !line.is_empty() && measure_text(&candidate, None, 16, 1.0).width > max_width {
            draw_text(&line, x, line_y, 16.0, color);
            line.clear();
            line_y += line_height;
        }
        if !line.is_empty() {
            line.push(' ');
        }
        line.push_str(word);
    }
    if !line.is_empty() {
        draw_text(&line, x, line_y, 16.0, color);
    }
}
