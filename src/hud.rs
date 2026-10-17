use hecs::{Entity, World};
use macroquad::{color, prelude::*};

use crate::database::Database;
use crate::health::Health;
use crate::inventory::{Equipment, GroundItem, Inventory, ItemRef};
use crate::movement::Position;
use crate::player::Player;
use crate::utils;

const INVENTORY_WIDTH: f32 = 420.0;
const INVENTORY_PADDING: f32 = 12.0;
const INVENTORY_HEADER_HEIGHT: f32 = 76.0;
const INVENTORY_ROW_HEIGHT: f32 = 46.0;
const INVENTORY_MAX_VISIBLE_ROWS: usize = 9;

pub struct UIInventory {
    pub rect : Rect,
    pub bg_color : Color,
    pub is_handling : bool,
    pub drag_offset: Vec2,
    pub dragged_item: Option<ItemRef>,
    pub drag_source_slot: Option<usize>,
    pub scroll: usize,
}

pub struct UIButton{
    pub is_clicked : bool,
    pub id : &'static str
}


fn draw_round_rect(x: f32, y: f32, w: f32, h: f32, r: f32, color: Color) {
    // Centro
    draw_rectangle(x + r, y, w - 2.0 * r, h, color);

    // Laterais
    draw_rectangle(x, y + r, r, h - 2.0 * r, color);
    draw_rectangle(x + w - r, y + r, r, h - 2.0 * r, color);

    // Cantos
    draw_circle(x + r, y + r, r, color);
    draw_circle(x + w - r, y + r, r, color);
    draw_circle(x + r, y + h - r, r, color);
    draw_circle(x + w - r, y + h - r, r, color);
}

pub fn update_ui(world: &mut World, database: &Database) -> bool {
    if player_inventory_entity(world).is_none() {
        return false;
    }

    handle_spawn_ui(world);
    update_inventory_ui(world, database);
    world.query::<&UIInventory>().iter().next().is_some()
}

pub fn ensure_inventory_ui(world: &mut World) {
    if world.query::<&UIInventory>().iter().next().is_some() {
        return;
    }

    let window_size = inventory_window_size();
    world.spawn((UIInventory {
        bg_color: Color::from_rgba(32, 32, 32, 255),
        rect: Rect::new(
            (screen_width() - window_size.x) * 0.5,
            (screen_height() - window_size.y) * 0.5,
            window_size.x,
            window_size.y,
        ),
        is_handling: false,
        drag_offset: vec2(0.0, 0.0),
        dragged_item: None,
        drag_source_slot: None,
        scroll: 0,
    },));
}

pub fn inventory_slot_at_position(world: &World, point: Vec2) -> Option<usize> {
    let (rect, scroll) = world
        .query::<&UIInventory>()
        .iter()
        .next()
        .map(|ui| (ui.rect, ui.scroll))?;
    let inventory_entity = player_inventory_entity(world)?;
    let slot_count = world.get::<&Inventory>(inventory_entity).ok()?.items.len();
    slot_at_position(rect, scroll, slot_count, point)
}

fn player_inventory_entity(world: &World) -> Option<Entity> {
    world
        .query::<(Entity, &Player, &Inventory)>()
        .iter()
        .next()
        .map(|(entity, _, _)| entity)
}

pub fn handle_spawn_ui(world: &mut World) {

    if is_key_pressed(KeyCode::I) {
        let inventory_ui = {
            let mut query = world.query::<(Entity, &UIInventory)>();
            query.iter().next().map(|(entity, ui)| {
                (
                    entity,
                    ui.drag_source_slot,
                    ui.dragged_item.clone(),
                )
            })
        };

        if let Some((entity, source_slot, dragged_item)) = inventory_ui {
            if let (Some(source_slot), Some(dragged_item)) = (source_slot, dragged_item) {
                if let Some(entity) = player_inventory_entity(world) {
                    if let Ok(mut inventory) = world.get::<&mut Inventory>(entity) {
                    if let Some(slot) = inventory.items.get_mut(source_slot) {
                        *slot = Some(dragged_item);
                    }
                    }
                }
            }
            world.despawn(entity).unwrap();
        } else {
            ensure_inventory_ui(world);
        }
    }
}

fn inventory_window_size() -> Vec2 {
    let maximum_height = (screen_height() - 24.0)
        .max(INVENTORY_HEADER_HEIGHT + INVENTORY_ROW_HEIGHT + INVENTORY_PADDING * 2.0);
    let visible_rows = ((maximum_height - INVENTORY_HEADER_HEIGHT - INVENTORY_PADDING * 2.0)
        / INVENTORY_ROW_HEIGHT)
        .floor()
        .clamp(1.0, INVENTORY_MAX_VISIBLE_ROWS as f32);
    vec2(
        INVENTORY_WIDTH.min((screen_width() - 24.0).max(180.0)),
        INVENTORY_HEADER_HEIGHT + visible_rows * INVENTORY_ROW_HEIGHT + INVENTORY_PADDING * 2.0,
    )
}

fn inventory_list_top(rect: Rect) -> f32 {
    rect.y + INVENTORY_PADDING + INVENTORY_HEADER_HEIGHT
}

fn inventory_visible_rows(rect: Rect) -> usize {
    ((rect.h - INVENTORY_PADDING * 2.0 - INVENTORY_HEADER_HEIGHT) / INVENTORY_ROW_HEIGHT)
        .floor()
        .max(1.0) as usize
}

fn slot_at_position(rect: Rect, scroll: usize, slot_count: usize, point: Vec2) -> Option<usize> {
    let list_top = inventory_list_top(rect);
    let relative_y = point.y - list_top;
    if point.x < rect.x + INVENTORY_PADDING
        || point.x >= rect.x + rect.w - INVENTORY_PADDING
        || relative_y < 0.0
        || relative_y >= inventory_visible_rows(rect) as f32 * INVENTORY_ROW_HEIGHT
    {
        return None;
    }

    let index = scroll + (relative_y / INVENTORY_ROW_HEIGHT) as usize;
    (index < slot_count).then_some(index)
}

pub fn draw_inventory_ui(world : &World, database : &Database){
    let Some(inventory_entity) = player_inventory_entity(world) else {
        return;
    };
    let inventory = world
        .get::<&Inventory>(inventory_entity)
        .expect("player inventory disappeared");
    let items: Vec<Option<(String, i32)>> = inventory
        .items
        .iter()
        .map(|item| item.as_ref().map(|item| (item.id.clone(), item.quantity)))
        .collect();
    let weight = inventory.weight;
    let max_weight = inventory.max_weight;
    let health = world
        .get::<&Health>(inventory_entity)
        .map(|health| format!("Vida: {}/{}", health.current, health.maximum))
        .ok();
    let equipped_items: Vec<String> = world
        .get::<&Equipment>(inventory_entity)
        .expect("player equipment disappeared")
        .items
        .values()
        .cloned()
        .collect();

    for inv in world.query::<&UIInventory>().iter() {
        let rect = inv.rect;
        draw_round_rect(
            rect.x,
            rect.y,
            rect.w,
            rect.h,
            12.0,
            inv.bg_color,
        );
        utils::draw_ui_text(
            "Inventario",
            rect.x + INVENTORY_PADDING,
            rect.y + 30.0,
            24.0,
            color::WHITE,
        );
        if let Some(health) = &health {
            let font_size = 16;
            let text_width = utils::measure_ui_text(health, font_size).width;
            utils::draw_ui_text(
                health,
                rect.x + rect.w - INVENTORY_PADDING - text_width,
                rect.y + 29.0,
                font_size as f32,
                color::WHITE,
            );
        }
        utils::draw_ui_text(
            &format!("Peso: {:.2} / {:.2} kg", weight, max_weight),
            rect.x + INVENTORY_PADDING,
            rect.y + 56.0,
            18.0,
            if weight >= max_weight { RED } else { LIGHTGRAY },
        );
        utils::draw_ui_text(
            "Botao direito em uma peca para equipar",
            rect.x + INVENTORY_PADDING,
            rect.y + 75.0,
            12.0,
            LIGHTGRAY,
        );

        let visible_rows = inventory_visible_rows(rect);
        for row in 0..visible_rows {
            let item_index = inv.scroll + row;
            if item_index >= items.len() {
                break;
            }
            let row_y = inventory_list_top(rect) + row as f32 * INVENTORY_ROW_HEIGHT;
            let item_is_equipped = items[item_index]
                .as_ref()
                .is_some_and(|(item_id, _)| equipped_items.contains(item_id));
            draw_rectangle(
                rect.x + INVENTORY_PADDING,
                row_y,
                rect.w - INVENTORY_PADDING * 2.0,
                INVENTORY_ROW_HEIGHT - 3.0,
                if item_is_equipped {
                    Color::from_rgba(42, 75, 48, 255)
                } else {
                    Color::from_rgba(55, 55, 55, 255)
                },
            );
            if item_is_equipped {
                draw_rectangle_lines(
                    rect.x + INVENTORY_PADDING,
                    row_y,
                    rect.w - INVENTORY_PADDING * 2.0,
                    INVENTORY_ROW_HEIGHT - 3.0,
                    1.0,
                    GREEN,
                );
            }

            if let Some((item_id, quantity)) = &items[item_index] {
                let item = database
                    .items
                    .get(item_id)
                    .expect("inventory item definition not found");
                let texture = database
                    .textures
                    .get(&item.texture_path)
                    .expect("inventory item texture not found");
                draw_texture_ex(
                    texture,
                    rect.x + INVENTORY_PADDING + 6.0,
                    row_y + 7.0,
                    color::WHITE,
                    DrawTextureParams {
                        dest_size: Some(vec2(30.0, 30.0)),
                        ..Default::default()
                    },
                );
                utils::draw_ui_text(&item.name, rect.x + 56.0, row_y + 19.0, 17.0, color::WHITE);
                if item_is_equipped {
                    utils::draw_ui_text(
                        "EQUIPADO",
                        rect.x + rect.w - 124.0,
                        row_y + 18.0,
                        12.0,
                        GREEN,
                    );
                }
                utils::draw_ui_text(
                    &format!(
                        "{} x {:.2} kg (total {:.2} kg)",
                        quantity,
                        item.weight,
                        item.weight * *quantity as f32
                    ),
                    rect.x + 56.0,
                    row_y + 37.0,
                    14.0,
                    LIGHTGRAY,
                );
                utils::draw_ui_text(
                    &format!("#{}", item_index + 1),
                    rect.x + rect.w - 46.0,
                    row_y + 27.0,
                    14.0,
                    GRAY,
                );
            } else {
                utils::draw_ui_text(
                    &format!("Espaco vazio #{}", item_index + 1),
                    rect.x + INVENTORY_PADDING + 8.0,
                    row_y + 27.0,
                    15.0,
                    GRAY,
                );
            }
        }

        let mouse = mouse_position();
        if let Some(item_ref) = &inv.dragged_item {
            let item = database
                .items
                .get(&item_ref.id)
                .expect("inventory item definition not found");
            let texture = database
                .textures
                .get(&item.texture_path)
                .expect("inventory item texture not found");
            draw_texture_ex(
                texture,
                mouse.0 - 16.0,
                mouse.1 - 16.0,
                color::WHITE,
                DrawTextureParams {
                    dest_size: Some(vec2(32.0, 32.0)),
                    ..Default::default()
                },
            );
        }
    }
}

fn update_inventory_ui(world: &mut World, database: &Database) {
    let mouse = mouse_position();
    let mouse_pos = vec2(mouse.0, mouse.1);
    let window_size = inventory_window_size();
    let ui_entity = world
        .query::<(Entity, &UIInventory)>()
        .iter()
        .next()
        .map(|(entity, _)| entity);
    let Some(ui_entity) = ui_entity else {
        return;
    };

    if let Ok(mut ui) = world.get::<&mut UIInventory>(ui_entity) {
        ui.rect.w = window_size.x;
        ui.rect.h = window_size.y;
    }

    let (rect, scroll) = world
        .get::<&UIInventory>(ui_entity)
        .map(|ui| (ui.rect, ui.scroll))
        .expect("inventory UI disappeared");
    let item_count = world
        .get::<&Inventory>(
            player_inventory_entity(world).expect("player inventory must be initialized"),
        )
        .expect("player inventory disappeared")
        .items
        .len();

    if rect.contains(mouse_pos) && mouse_pos.y >= inventory_list_top(rect) {
        let (_, scroll_y) = mouse_wheel();
        if scroll_y != 0.0 {
            if let Ok(mut ui) = world.get::<&mut UIInventory>(ui_entity) {
                if scroll_y < 0.0 {
                    ui.scroll = ui.scroll.saturating_add(1);
                } else {
                    ui.scroll = ui.scroll.saturating_sub(1);
                }
                ui.scroll = ui
                    .scroll
                    .min(item_count.saturating_sub(inventory_visible_rows(rect)));
            }
        }
    }
    let scroll = world
        .get::<&UIInventory>(ui_entity)
        .map(|ui| ui.scroll)
        .unwrap_or(scroll);

    if is_mouse_button_pressed(MouseButton::Right) {
        if let Some(slot_index) = slot_at_position(rect, scroll, item_count, mouse_pos) {
            let item = world
                .get::<&Inventory>(
                    player_inventory_entity(world).expect("player inventory must be initialized"),
                )
                .ok()
                .and_then(|inventory| inventory.items.get(slot_index).cloned())
                .flatten();

            if let Some(item) = item {
                let definition = database
                    .items
                    .get(&item.id)
                    .expect("inventory item definition not found");
                if let Some(item_equipment) = &definition.equipment {
                    let inventory_entity =
                        player_inventory_entity(world).expect("player inventory must be initialized");
                    let mut equipment = world
                        .get::<&mut Equipment>(inventory_entity)
                        .expect("player equipment disappeared");
                    if equipment
                        .items
                        .get(&item_equipment.slot)
                        .is_some_and(|equipped_item_id| equipped_item_id == &item.id)
                    {
                        equipment.items.remove(&item_equipment.slot);
                    } else {
                        equipment
                            .items
                            .insert(item_equipment.slot, item.id);
                    }
                }
            }
        }
    }

    if is_mouse_button_pressed(MouseButton::Left) {
        if let Some(slot_index) = slot_at_position(rect, scroll, item_count, mouse_pos) {
            let item = world
                .get::<&Inventory>(
                    player_inventory_entity(world).expect("player inventory must be initialized"),
                )
                .ok()
                .and_then(|inventory| inventory.items.get(slot_index).cloned())
                .flatten();

            if let Some(item) = item {
                if let Some(entity) = player_inventory_entity(world) {
                    if let Ok(mut inventory) = world.get::<&mut Inventory>(entity) {
                    if slot_index < inventory.items.len() {
                        inventory.items[slot_index] = None;
                    }
                    }
                }
                if let Ok(mut ui) = world.get::<&mut UIInventory>(ui_entity) {
                    ui.dragged_item = Some(item);
                    ui.drag_source_slot = Some(slot_index);
                }
            }
        } else if rect.contains(mouse_pos)
            && mouse_pos.y < rect.y + INVENTORY_PADDING + INVENTORY_HEADER_HEIGHT
        {
            if let Ok(mut ui) = world.get::<&mut UIInventory>(ui_entity) {
                ui.is_handling = true;
                ui.drag_offset = mouse_pos - vec2(rect.x, rect.y);
            }
        }
    }

    if let Ok(mut ui) = world.get::<&mut UIInventory>(ui_entity) {
        if ui.is_handling && is_mouse_button_down(MouseButton::Left) {
            ui.rect.x = mouse_pos.x - ui.drag_offset.x;
            ui.rect.y = mouse_pos.y - ui.drag_offset.y;
        }
        if is_mouse_button_released(MouseButton::Left) {
            ui.is_handling = false;
        }
    }

    if !is_mouse_button_released(MouseButton::Left) {
        return;
    }

    let drag = world
        .get::<&UIInventory>(ui_entity)
        .ok()
        .and_then(|ui| {
            Some((
                ui.drag_source_slot?,
                ui.dragged_item.clone()?,
                ui.rect,
            ))
        });
    let Some((source_slot, dragged_item, rect)) = drag else {
        return;
    };

    if let Some(target_slot) = slot_at_position(rect, scroll, item_count, mouse_pos) {
        if let Some(entity) = player_inventory_entity(world) {
            if let Ok(mut inventory) = world.get::<&mut Inventory>(entity) {
            if target_slot < inventory.items.len() && source_slot < inventory.items.len() {
                if target_slot == source_slot {
                    inventory.items[source_slot] = Some(dragged_item);
                } else if inventory.items[target_slot]
                    .as_ref()
                    .is_some_and(|item| item.id == dragged_item.id)
                {
                    if let Some(target) = inventory.items[target_slot].as_mut() {
                        target.quantity += dragged_item.quantity;
                    }
                } else {
                    let displaced_item = inventory.items[target_slot].take();
                    inventory.items[target_slot] = Some(dragged_item);
                    inventory.items[source_slot] = displaced_item;
                }
            }
            }
        }
    } else if rect.contains(mouse_pos) {
        if let Some(entity) = player_inventory_entity(world) {
            if let Ok(mut inventory) = world.get::<&mut Inventory>(entity) {
            inventory.items[source_slot] = Some(dragged_item);
            }
        }
    } else {
        let player_position = world
        .query::<(&Player, &Position)>()
        .iter()
        .next()
        .map(|(_, position)| position.value);
        if let Some(player_position) = player_position {
            if let Some(drop_position) =
                crate::inventory::find_drop_position(world, player_position)
            {
                let item_weight = database
                    .items
                    .get(&dragged_item.id)
                    .expect("inventory item definition not found")
                    .weight
                    * dragged_item.quantity as f32;
                if let Some(entity) = player_inventory_entity(world) {
                    if let Ok(mut inventory) = world.get::<&mut Inventory>(entity) {
                        inventory.weight -= item_weight;
                    }
                }
                for _ in 0..dragged_item.quantity {
                    world.spawn((GroundItem {
                        item: ItemRef {
                            id: dragged_item.id.clone(),
                            quantity: 1,
                        },
                        position: drop_position,
                    },));
                }
            } else if let Some(entity) = player_inventory_entity(world) {
                if let Ok(mut inventory) = world.get::<&mut Inventory>(entity) {
                    inventory.items[source_slot] = Some(dragged_item);
                }
            }
        } else if let Some(entity) = player_inventory_entity(world) {
            if let Ok(mut inventory) = world.get::<&mut Inventory>(entity) {
                inventory.items[source_slot] = Some(dragged_item);
            }
        }
    }

    if let Ok(mut ui) = world.get::<&mut UIInventory>(ui_entity) {
        ui.dragged_item = None;
        ui.drag_source_slot = None;
    }
}