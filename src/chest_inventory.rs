use hecs::{Entity, World};
use macroquad::prelude::*;

use crate::database::Database;
use crate::hud::{self, UIInventory};
use crate::inventory::{self, GroundItem, Inventory, ItemRef};
use crate::movement::Position;
use crate::player::Player;

const PANEL_WIDTH: f32 = 320.0;
const PANEL_PADDING: f32 = 12.0;
const PANEL_HEADER_HEIGHT: f32 = 52.0;
const ROW_HEIGHT: f32 = 46.0;
const MAX_VISIBLE_ROWS: usize = 9;

pub struct ChestInventoryUI {
    pub chest_entity: Entity,
    pub rect: Rect,
    pub scroll: usize,
    pub dragged_item: Option<ItemRef>,
    pub drag_source_slot: Option<usize>,
}

pub fn is_open(world: &World) -> bool {
    world.query::<&ChestInventoryUI>().iter().next().is_some()
}

pub fn open(world: &mut World, chest_entity: Entity) {
    if is_open(world) {
        return;
    }

    hud::ensure_inventory_ui(world);
    let player_rect = world
        .query::<&UIInventory>()
        .iter()
        .next()
        .expect("player inventory UI was not created")
        .rect;
    let rect = panel_rect(player_rect);
    world.spawn((ChestInventoryUI {
        chest_entity,
        rect,
        scroll: 0,
        dragged_item: None,
        drag_source_slot: None,
    },));
}

pub fn update(world: &mut World, database: &Database) -> bool {
    let ui_entity = world
        .query::<(Entity, &ChestInventoryUI)>()
        .iter()
        .next()
        .map(|(entity, _)| entity);
    let Some(ui_entity) = ui_entity else {
        return false;
    };

    let (chest_entity, scroll) = world
        .get::<&ChestInventoryUI>(ui_entity)
        .map(|ui| (ui.chest_entity, ui.scroll))
        .expect("chest inventory UI disappeared");
    let chest_position = world
        .get::<&Position>(chest_entity)
        .map(|position| position.value)
        .expect("chest position disappeared");
    if world.query::<&UIInventory>().iter().next().is_none() {
        hud::ensure_inventory_ui(world);
    }
    let player_rect = world
        .query::<&UIInventory>()
        .iter()
        .next()
        .expect("player inventory UI was not created")
        .rect;
    let rect = panel_rect(player_rect);
    let slot_count = world
        .get::<&Inventory>(chest_entity)
        .expect("chest inventory disappeared")
        .items
        .len();
    let visible_rows = visible_rows(rect);
    if let Ok(mut ui) = world.get::<&mut ChestInventoryUI>(ui_entity) {
        ui.rect = rect;
        ui.scroll = ui.scroll.min(slot_count.saturating_sub(visible_rows));
    }

    let mouse = mouse_position();
    let point = vec2(mouse.0, mouse.1);

    if is_key_pressed(KeyCode::Escape) || is_key_pressed(KeyCode::I) {
        if let Some((source_slot, item)) = world
            .get::<&ChestInventoryUI>(ui_entity)
            .ok()
            .and_then(|ui| Some((ui.drag_source_slot?, ui.dragged_item.clone()?)))
        {
            restore_chest_drag(world, chest_entity, ui_entity, source_slot, item);
        }
        close(world, ui_entity, chest_entity);
        return true;
    }

    let close_rect = Rect::new(rect.x + rect.w - 32.0, rect.y + 6.0, 24.0, 24.0);
    if is_mouse_button_pressed(MouseButton::Left) && close_rect.contains(point) {
        if let Some((source_slot, item)) = world
            .get::<&ChestInventoryUI>(ui_entity)
            .ok()
            .and_then(|ui| Some((ui.drag_source_slot?, ui.dragged_item.clone()?)))
        {
            restore_chest_drag(world, chest_entity, ui_entity, source_slot, item);
        }
        close(world, ui_entity, chest_entity);
        return true;
    }

    if rect.contains(point) {
        let (_, scroll_y) = mouse_wheel();
        if scroll_y != 0.0 {
            if let Ok(mut ui) = world.get::<&mut ChestInventoryUI>(ui_entity) {
                if scroll_y < 0.0 {
                    ui.scroll = ui.scroll.saturating_add(1);
                } else {
                    ui.scroll = ui.scroll.saturating_sub(1);
                }
                ui.scroll = ui.scroll.min(slot_count.saturating_sub(visible_rows));
            }
        }
    }

    if is_mouse_button_released(MouseButton::Left) {
        let player_drag = world
            .query::<&UIInventory>()
            .iter()
            .next()
            .and_then(|ui| Some((ui.drag_source_slot?, ui.dragged_item.clone()?)));
        if let Some((source_slot, item)) = player_drag {
            if let Some(target_slot) = chest_slot_at(rect, scroll, slot_count, point) {
                if store_in_inventory(world, chest_entity, target_slot, &item, database) {
                    remove_player_weight(world, item_weight(&item, database));
                    clear_player_drag(world);
                } else {
                    restore_player_drag(world, source_slot, item);
                }
            } else if rect.contains(point) {
                restore_player_drag(world, source_slot, item);
            }
        }
    }

    let (scroll, chest_dragged_item, chest_drag_source) = world
        .get::<&ChestInventoryUI>(ui_entity)
        .map(|ui| (ui.scroll, ui.dragged_item.clone(), ui.drag_source_slot))
        .expect("chest inventory UI disappeared");

    if is_mouse_button_pressed(MouseButton::Left) && chest_dragged_item.is_none() {
        if let Some(slot_index) = chest_slot_at(rect, scroll, slot_count, point) {
            let item = world
                .get::<&Inventory>(chest_entity)
                .ok()
                .and_then(|inventory| inventory.items.get(slot_index).cloned())
                .flatten();
            if let Some(item) = item {
                if let Ok(mut inventory) = world.get::<&mut Inventory>(chest_entity) {
                    inventory.items[slot_index] = None;
                }
                if let Ok(mut ui) = world.get::<&mut ChestInventoryUI>(ui_entity) {
                    ui.dragged_item = Some(item);
                    ui.drag_source_slot = Some(slot_index);
                }
            }
        }
    }

    if is_mouse_button_released(MouseButton::Left) {
        if let (Some(source_slot), Some(item)) = (chest_drag_source, chest_dragged_item) {
            if let Some(target_slot) = hud::inventory_slot_at_position(world, point) {
                if store_in_inventory(world, player_entity(world), target_slot, &item, database) {
                    let weight = item_weight(&item, database);
                    adjust_chest_weight(world, chest_entity, -weight);
                    clear_chest_drag(world, ui_entity);
                } else {
                    restore_chest_drag(world, chest_entity, ui_entity, source_slot, item);
                }
            } else if let Some(target_slot) = chest_slot_at(rect, scroll, slot_count, point) {
                if store_in_inventory(world, chest_entity, target_slot, &item, database) {
                    clear_chest_drag(world, ui_entity);
                } else {
                    restore_chest_drag(world, chest_entity, ui_entity, source_slot, item);
                }
            } else if rect.contains(point) || player_panel_contains(world, point) {
                restore_chest_drag(world, chest_entity, ui_entity, source_slot, item);
            } else {
                if let Some(drop_position) =
                    inventory::find_drop_position(world, chest_position)
                {
                    for _ in 0..item.quantity {
                        world.spawn((GroundItem {
                            item: ItemRef {
                                id: item.id.clone(),
                                quantity: 1,
                            },
                            position: drop_position,
                        },));
                    }
                    adjust_chest_weight(world, chest_entity, -item_weight(&item, database));
                    clear_chest_drag(world, ui_entity);
                } else {
                    restore_chest_drag(world, chest_entity, ui_entity, source_slot, item);
                }
            }
        }
    }

    true
}

pub fn draw(world: &World, database: &Database) {
    let Some((chest_entity, rect, scroll, dragged_item)) = world
        .query::<&ChestInventoryUI>()
        .iter()
        .next()
        .map(|ui| (ui.chest_entity, ui.rect, ui.scroll, ui.dragged_item.clone()))
    else {
        return;
    };

    draw_rectangle(
        rect.x,
        rect.y,
        rect.w,
        rect.h,
        Color::from_rgba(32, 32, 32, 250),
    );
    draw_rectangle_lines(rect.x, rect.y, rect.w, rect.h, 2.0, LIGHTGRAY);
    draw_text("Bau", rect.x + PANEL_PADDING, rect.y + 26.0, 20.0, WHITE);
    draw_text("X", rect.x + rect.w - 27.0, rect.y + 24.0, 20.0, LIGHTGRAY);

    let Ok(inventory) = world.get::<&Inventory>(chest_entity) else {
        return;
    };
    draw_text(
        &format!("Peso guardado: {:.2} kg", inventory.weight),
        rect.x + PANEL_PADDING,
        rect.y + 46.0,
        14.0,
        LIGHTGRAY,
    );
    for row in 0..visible_rows(rect) {
        let item_index = scroll + row;
        if item_index >= inventory.items.len() {
            break;
        }
        let row_y = list_top(rect) + row as f32 * ROW_HEIGHT;
        draw_rectangle(
            rect.x + PANEL_PADDING,
            row_y,
            rect.w - PANEL_PADDING * 2.0,
            ROW_HEIGHT - 3.0,
            Color::from_rgba(55, 55, 55, 255),
        );
        if let Some(item_ref) = &inventory.items[item_index] {
            let item = database
                .items
                .get(&item_ref.id)
                .expect("chest item definition not found");
            let texture = database
                .textures
                .get(&item.texture_path)
                .expect("chest item texture not found");
            draw_texture_ex(
                texture,
                rect.x + PANEL_PADDING + 6.0,
                row_y + 7.0,
                WHITE,
                DrawTextureParams {
                    dest_size: Some(vec2(30.0, 30.0)),
                    ..Default::default()
                },
            );
            draw_text(&item.name, rect.x + 56.0, row_y + 19.0, 16.0, WHITE);
            draw_text(
                &format!(
                    "{} x {:.2} kg (total {:.2} kg)",
                    item_ref.quantity,
                    item.weight,
                    item.weight * item_ref.quantity as f32
                ),
                rect.x + 56.0,
                row_y + 37.0,
                13.0,
                LIGHTGRAY,
            );
        } else {
            draw_text(
                &format!("Espaco vazio #{}", item_index + 1),
                rect.x + PANEL_PADDING + 8.0,
                row_y + 27.0,
                15.0,
                GRAY,
            );
        }
    }

    if let Some(item_ref) = dragged_item {
        let item = database
            .items
            .get(&item_ref.id)
            .expect("dragged chest item definition not found");
        let texture = database
            .textures
            .get(&item.texture_path)
            .expect("dragged chest item texture not found");
        let mouse = mouse_position();
        draw_texture_ex(
            texture,
            mouse.0 - 16.0,
            mouse.1 - 16.0,
            WHITE,
            DrawTextureParams {
                dest_size: Some(vec2(32.0, 32.0)),
                ..Default::default()
            },
        );
    }
}

pub fn store_ground_item_at_position(
    world: &mut World,
    database: &Database,
    point: Vec2,
    ground_entity: Entity,
) -> bool {
    let Some((ui, chest_entity, scroll, slot_count)) = world
        .query::<&ChestInventoryUI>()
        .iter()
        .next()
        .and_then(|ui| {
            let slot_count = world
                .get::<&Inventory>(ui.chest_entity)
                .ok()
                .map(|inventory| inventory.items.len())?;
            Some((ui.rect, ui.chest_entity, ui.scroll, slot_count))
        })
    else {
        return false;
    };
    let Some(target_slot) = chest_slot_at(ui, scroll, slot_count, point) else {
        return false;
    };
    let Some(item) = world
        .get::<&GroundItem>(ground_entity)
        .ok()
        .map(|ground_item| ground_item.item.clone())
    else {
        return false;
    };
    if !store_in_inventory(world, chest_entity, target_slot, &item, database) {
        return false;
    }
    world
        .despawn(ground_entity)
        .expect("ground item disappeared before chest pickup");
    true
}

fn close(world: &mut World, ui_entity: Entity, chest_entity: Entity) {
    let _ = world.despawn(ui_entity);
    if let Ok(mut sprite) = world.get::<&mut crate::objects::OnOffSprite>(chest_entity) {
        sprite.is_on = false;
    }
}

fn panel_rect(player_rect: Rect) -> Rect {
    let width = PANEL_WIDTH.min((screen_width() - 24.0).max(180.0));
    let maximum_height = (screen_height() - 24.0).max(PANEL_HEADER_HEIGHT + ROW_HEIGHT + 24.0);
    let rows = ((maximum_height - PANEL_HEADER_HEIGHT - PANEL_PADDING * 2.0) / ROW_HEIGHT)
        .floor()
        .clamp(1.0, MAX_VISIBLE_ROWS as f32);
    let height = PANEL_HEADER_HEIGHT + rows * ROW_HEIGHT + PANEL_PADDING * 2.0;
    let right = player_rect.x + player_rect.w + 12.0;
    let x = if right + width <= screen_width() - 12.0 {
        right
    } else {
        (player_rect.x - width - 12.0).max(12.0)
    };
    let y = player_rect
        .y
        .max(12.0)
        .min((screen_height() - height - 12.0).max(12.0));
    Rect::new(x, y, width, height)
}

fn list_top(rect: Rect) -> f32 {
    rect.y + PANEL_PADDING + PANEL_HEADER_HEIGHT
}

fn visible_rows(rect: Rect) -> usize {
    ((rect.h - PANEL_PADDING * 2.0 - PANEL_HEADER_HEIGHT) / ROW_HEIGHT)
        .floor()
        .max(1.0) as usize
}

fn chest_slot_at(rect: Rect, scroll: usize, slot_count: usize, point: Vec2) -> Option<usize> {
    let relative_y = point.y - list_top(rect);
    if point.x < rect.x + PANEL_PADDING
        || point.x >= rect.x + rect.w - PANEL_PADDING
        || relative_y < 0.0
        || relative_y >= visible_rows(rect) as f32 * ROW_HEIGHT
    {
        return None;
    }
    let index = scroll + (relative_y / ROW_HEIGHT) as usize;
    (index < slot_count).then_some(index)
}

fn player_entity(world: &World) -> Entity {
    world
        .query::<(Entity, &Player, &Inventory)>()
        .iter()
        .next()
        .map(|(entity, _, _)| entity)
        .expect("player inventory must be initialized")
}

fn player_panel_contains(world: &World, point: Vec2) -> bool {
    world
        .query::<&UIInventory>()
        .iter()
        .next()
        .is_some_and(|ui| ui.rect.contains(point))
}

fn store_in_inventory(
    world: &mut World,
    inventory_entity: Entity,
    target_slot: usize,
    item: &ItemRef,
    database: &Database,
) -> bool {
    let weight = item_weight(item, database);
    let Ok(mut inventory) = world.get::<&mut Inventory>(inventory_entity) else {
        return false;
    };
    if inventory.weight + weight > inventory.max_weight {
        return false;
    }
    let Some(slot) = inventory.items.get_mut(target_slot) else {
        return false;
    };
    match slot {
        Some(existing) if existing.id == item.id => existing.quantity += item.quantity,
        None => *slot = Some(item.clone()),
        Some(_) => return false,
    }
    inventory.weight += weight;
    true
}

fn item_weight(item: &ItemRef, database: &Database) -> f32 {
    database
        .items
        .get(&item.id)
        .expect("item definition not found")
        .weight
        * item.quantity as f32
}

fn remove_player_weight(world: &mut World, weight: f32) {
    let entity = player_entity(world);
    if let Ok(mut inventory) = world.get::<&mut Inventory>(entity) {
        inventory.weight -= weight;
    }
}

fn adjust_chest_weight(world: &mut World, chest_entity: Entity, weight: f32) {
    if let Ok(mut inventory) = world.get::<&mut Inventory>(chest_entity) {
        inventory.weight += weight;
    }
}

fn clear_player_drag(world: &mut World) {
    let ui_entity = {
        world
            .query::<(Entity, &UIInventory)>()
            .iter()
            .next()
            .map(|(entity, _)| entity)
    };
    if let Some(entity) = ui_entity {
        if let Ok(mut ui) = world.get::<&mut UIInventory>(entity) {
            ui.dragged_item = None;
            ui.drag_source_slot = None;
        }
    }
}

fn restore_player_drag(world: &mut World, source_slot: usize, item: ItemRef) {
    let ui_entity = {
        world
            .query::<(Entity, &UIInventory)>()
            .iter()
            .next()
            .map(|(entity, _)| entity)
    };
    if let Some(entity) = ui_entity {
        if let Ok(mut ui) = world.get::<&mut UIInventory>(entity) {
            ui.dragged_item = None;
            ui.drag_source_slot = None;
        }
    }
    let player = player_entity(world);
    if let Ok(mut inventory) = world.get::<&mut Inventory>(player) {
        if let Some(slot) = inventory.items.get_mut(source_slot) {
            *slot = Some(item);
        }
    }
}

fn clear_chest_drag(world: &mut World, ui_entity: Entity) {
    if let Ok(mut ui) = world.get::<&mut ChestInventoryUI>(ui_entity) {
        ui.dragged_item = None;
        ui.drag_source_slot = None;
    }
}

fn restore_chest_drag(
    world: &mut World,
    chest_entity: Entity,
    ui_entity: Entity,
    source_slot: usize,
    item: ItemRef,
) {
    if let Ok(mut inventory) = world.get::<&mut Inventory>(chest_entity) {
        if let Some(slot) = inventory.items.get_mut(source_slot) {
            *slot = Some(item);
        }
    }
    clear_chest_drag(world, ui_entity);
}
