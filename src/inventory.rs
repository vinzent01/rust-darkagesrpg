use hecs::{Entity, World};
use macroquad::prelude::*;
use std::collections::HashMap;

use crate::database::Database;
use crate::chest_inventory;
use crate::hud::{self, UIInventory};
use crate::input;
use crate::movement::Position;
use crate::player::Player;
use crate::tilemap::{MAP_HEIGHT, MAP_WIDTH};
use crate::utils::TILE_SIZE;

pub const INVENTORY_SLOT_COUNT: usize = 25;
pub const MAX_INVENTORY_WEIGHT_KG: f32 = 20.0;

#[derive(Clone)]
pub struct ItemRef {
    pub id : String,
    pub quantity : i32
}

pub struct Inventory{
    pub items : Vec<Option<ItemRef>>,
    pub weight : f32,
    pub max_weight : f32,
    pub rect : Rect
}

pub struct Item{
    pub id : String,
    pub name : String,
    pub weight : f32,
    pub texture_path: String,
    pub equipment: Option<ItemEquipment>,
}

#[derive(Clone, Copy, Eq, Hash, PartialEq)]
pub enum EquipmentSlot {
    Feet,
    Legs,
    Torso,
    Head,
    MainHand,
    OffHand,
}

pub struct ItemEquipment {
    pub slot: EquipmentSlot,
    pub male_texture_path: String,
    pub female_texture_path: String,
    pub offset: Vec2,
}

#[derive(Default)]
pub struct Equipment {
    pub items: HashMap<EquipmentSlot, String>,
}

pub struct GroundItem {
    pub item: ItemRef,
    pub position: Vec2,
}

pub struct GroundStackUI {
    pub rect: Rect,
    pub position: Vec2,
    pub scroll: usize,
    pub dragged_entity: Option<Entity>,
}

pub fn get_item<'a>(item_ref: &ItemRef, database: &'a Database) -> Option<&'a Item> {
    database.items.get(&item_ref.id)
}

fn world_cell(position: Vec2) -> Vec2 {
    (position / TILE_SIZE).round() * TILE_SIZE
}

pub fn find_drop_position(world: &World, preferred: Vec2) -> Option<Vec2> {
    let origin = world_cell(preferred);
    for radius in 1_i32..MAP_WIDTH.max(MAP_HEIGHT) as i32 {
        for y in -radius..=radius {
            for x in -radius..=radius {
                if x.abs() != radius && y.abs() != radius {
                    continue;
                }

                let candidate = origin + vec2(x as f32, y as f32) * TILE_SIZE;
                if !input::is_blocked(world, candidate) {
                    return Some(candidate);
                }
            }
        }
    }
    None
}

fn ground_items_at(world: &World, position: Vec2) -> Vec<(Entity, ItemRef)> {
    world
        .query::<(Entity, &GroundItem)>()
        .iter()
        .filter(|(_, item)| world_cell(item.position) == position)
        .map(|(entity, item)| (entity, item.item.clone()))
        .collect()
}

fn player_inventory_entity(world: &World) -> Option<Entity> {
    world
        .query::<(Entity, &Player, &Inventory)>()
        .iter()
        .next()
        .map(|(entity, _, _)| entity)
}

pub fn update_ground_stack_ui(
    world: &mut World,
    database: &Database,
    camera: &Camera2D,
) -> bool {
    let mouse = mouse_position();
    let mouse_position = vec2(mouse.0, mouse.1);
    let ui_entity = world
        .query::<(Entity, &GroundStackUI)>()
        .iter()
        .next()
        .map(|(entity, _)| entity);

    let Some(ui_entity) = ui_entity else {
        if !is_mouse_button_pressed(MouseButton::Left) {
            return false;
        }

        let clicked_position = camera.screen_to_world(mouse_position);
        let clicked_cell = world_cell(clicked_position);
        let player_position = world
            .query::<(&Player, &Position)>()
            .iter()
            .next()
            .map(|(_, position)| position.value);
        let Some(player_position) = player_position else {
            return false;
        };

        if player_position.distance(clicked_cell) > TILE_SIZE * 2.0
            || ground_items_at(world, clicked_cell).len() < 2
        {
            return false;
        }

        hud::ensure_inventory_ui(world);
        let inventory_rect = {
            let mut query = world.query::<&UIInventory>();
            query
                .iter()
                .next()
                .expect("inventory UI must be open for a ground stack")
                .rect
        };
        let rect = ground_stack_rect(world, inventory_rect, 1);
        world.spawn((GroundStackUI {
            rect,
            position: clicked_cell,
            scroll: 0,
            dragged_entity: None,
        },));
        return true;
    };

    let stack_position = world
        .get::<&GroundStackUI>(ui_entity)
        .expect("ground stack UI disappeared")
        .position;
    let items = ground_items_at(world, stack_position);

    if world.query::<&UIInventory>().iter().next().is_none() {
        hud::ensure_inventory_ui(world);
    }
    let inventory_rect = world
        .query::<&UIInventory>()
        .iter()
        .next()
        .expect("inventory UI must be open for a ground stack")
        .rect;

    let item_count = items.len();
    if item_count == 0 {
        let _ = world.despawn(ui_entity);
        return true;
    }

    let rect = ground_stack_rect(world, inventory_rect, item_count);
    if let Ok(mut ui) = world.get::<&mut GroundStackUI>(ui_entity) {
        ui.rect = rect;
        let visible_rows = ground_stack_visible_rows(rect);
        ui.scroll = ui.scroll.min(item_count.saturating_sub(visible_rows));
    }

    if is_key_pressed(KeyCode::Escape) {
        let _ = world.despawn(ui_entity);
        return true;
    }

    let close_rect = Rect::new(rect.x + rect.w - 32.0, rect.y + 6.0, 24.0, 24.0);
    if is_mouse_button_pressed(MouseButton::Left) && close_rect.contains(mouse_position) {
        let _ = world.despawn(ui_entity);
        return true;
    }

    if rect.contains(mouse_position) {
        let (_, scroll_y) = mouse_wheel();
        if scroll_y != 0.0 {
            if let Ok(mut ui) = world.get::<&mut GroundStackUI>(ui_entity) {
                if scroll_y < 0.0 {
                    ui.scroll = ui.scroll.saturating_add(1);
                } else {
                    ui.scroll = ui.scroll.saturating_sub(1);
                }
            }
        }
    }

    let (scroll, dragged_entity) = world
        .get::<&GroundStackUI>(ui_entity)
        .map(|ui| (ui.scroll, ui.dragged_entity))
        .expect("ground stack UI disappeared");
    let row_height = 40.0;
    let list_top = rect.y + 38.0;

    if is_mouse_button_pressed(MouseButton::Left) && dragged_entity.is_none() {
        let row = ((mouse_position.y - list_top) / row_height).floor();
        if mouse_position.x >= rect.x + 8.0
            && mouse_position.x < rect.x + rect.w - 8.0
            && row >= 0.0
            && row < ground_stack_visible_rows(rect) as f32
        {
            let item_index = scroll + row as usize;
            if let Some((entity, _)) = items.get(item_index) {
                if let Ok(mut ui) = world.get::<&mut GroundStackUI>(ui_entity) {
                    ui.dragged_entity = Some(*entity);
                }
            }
        }
    }

    if is_mouse_button_released(MouseButton::Left) {
        let dragged_entity = world
            .get::<&GroundStackUI>(ui_entity)
            .ok()
            .and_then(|ui| ui.dragged_entity);
        if let Some(ground_entity) = dragged_entity {
            let stored_in_chest = chest_inventory::store_ground_item_at_position(
                world,
                database,
                mouse_position,
                ground_entity,
            );
            if !stored_in_chest {
                if let Some(target_slot) = hud::inventory_slot_at_position(world, mouse_position) {
                let ground_item = world
                    .get::<&GroundItem>(ground_entity)
                    .ok()
                    .map(|item| item.item.clone());
                if let Some(ground_item) = ground_item {
                    let item_weight = database
                        .items
                        .get(&ground_item.id)
                        .expect("ground item definition not found")
                        .weight
                        * ground_item.quantity as f32;
                    let mut stored_item = false;

                    let inventory_entity = player_inventory_entity(world)
                        .expect("player inventory must be initialized");
                    if let Ok(mut inventory) = world.get::<&mut Inventory>(inventory_entity) {
                        if inventory.weight + item_weight <= inventory.max_weight {
                            if let Some(slot) = inventory.items.get_mut(target_slot) {
                            match slot {
                                Some(existing_item) if existing_item.id == ground_item.id => {
                                    existing_item.quantity += ground_item.quantity;
                                    stored_item = true;
                                }
                                None => {
                                    *slot = Some(ground_item);
                                    stored_item = true;
                                }
                                Some(_) => {}
                            }
                            }
                            if stored_item {
                                inventory.weight += item_weight;
                            }
                        }
                    }

                    if stored_item {
                        world
                            .despawn(ground_entity)
                            .expect("ground item disappeared before pickup");
                    }
                }
                }
            }

            if let Ok(mut ui) = world.get::<&mut GroundStackUI>(ui_entity) {
                ui.dragged_entity = None;
            }
        }
    }

    true
}

fn ground_stack_visible_rows(rect: Rect) -> usize {
    ((rect.h - 46.0) / 40.0).floor().max(1.0) as usize
}

fn ground_stack_rect(world: &World, inventory_rect: Rect, item_count: usize) -> Rect {
    let width = 280.0;
    let height = (46.0 + item_count.min(ground_stack_visible_rows(Rect::new(
        0.0,
        0.0,
        width,
        (screen_height() - 40.0).max(86.0),
    ))) as f32 * 40.0)
        .min((screen_height() - 40.0).max(86.0));
    let y = inventory_rect
        .y
        .max(12.0)
        .min((screen_height() - height - 12.0).max(12.0));

    if let Some(chest_rect) = world
        .query::<&chest_inventory::ChestInventoryUI>()
        .iter()
        .next()
        .map(|ui| ui.rect)
    {
        let chest_is_right = chest_rect.x >= inventory_rect.x + inventory_rect.w;
        let preferred_x = if chest_is_right {
            inventory_rect.x - width - 12.0
        } else {
            inventory_rect.x + inventory_rect.w + 12.0
        };
        let preferred = Rect::new(preferred_x, y, width, height);
        if preferred.x >= 12.0
            && preferred.x + width <= screen_width() - 12.0
            && !rects_overlap(preferred, inventory_rect)
            && !rects_overlap(preferred, chest_rect)
        {
            return preferred;
        }

        for candidate_y in [
            chest_rect.y + chest_rect.h + 12.0,
            inventory_rect.y - height - 12.0,
            chest_rect.y - height - 12.0,
        ] {
            let candidate_y = candidate_y
                .max(12.0)
                .min((screen_height() - height - 12.0).max(12.0));
            let candidate = Rect::new(
                inventory_rect.x.max(12.0).min(
                    (screen_width() - width - 12.0).max(12.0),
                ),
                candidate_y,
                width,
                height,
            );
            if !rects_overlap(candidate, inventory_rect) && !rects_overlap(candidate, chest_rect) {
                return candidate;
            }
        }
    }

    let right_x = inventory_rect.x + inventory_rect.w + 12.0;
    let x = if right_x + width <= screen_width() - 12.0 {
        right_x
    } else {
        (inventory_rect.x - width - 12.0).max(12.0)
    };
    Rect::new(x, y, width, height)
}

fn rects_overlap(first: Rect, second: Rect) -> bool {
    first.x < second.x + second.w
        && first.x + first.w > second.x
        && first.y < second.y + second.h
        && first.y + first.h > second.y
}

pub fn draw_ground_stack_ui(world: &World, database: &Database) {
    let Some((rect, position, scroll, dragged_entity)) = world
        .query::<&GroundStackUI>()
        .iter()
        .next()
        .map(|ui| (ui.rect, ui.position, ui.scroll, ui.dragged_entity))
    else {
        return;
    };

    draw_rectangle(rect.x, rect.y, rect.w, rect.h, Color::from_rgba(32, 32, 32, 250));
    draw_rectangle_lines(rect.x, rect.y, rect.w, rect.h, 2.0, LIGHTGRAY);
    draw_text("Itens no chao", rect.x + 12.0, rect.y + 26.0, 20.0, WHITE);
    draw_text("X", rect.x + rect.w - 27.0, rect.y + 24.0, 20.0, LIGHTGRAY);

    let items = ground_items_at(world, position);
    let visible_rows = ground_stack_visible_rows(rect);
    let item_texture = |item: &ItemRef| {
        let definition = database
            .items
            .get(&item.id)
            .expect("ground item definition not found");
        database
            .textures
            .get(&definition.texture_path)
            .expect("ground item texture not found")
    };

    for (row, (_, item)) in items
        .iter()
        .skip(scroll)
        .take(visible_rows)
        .enumerate()
    {
        let y = rect.y + 38.0 + row as f32 * 40.0;
        draw_rectangle(rect.x + 8.0, y, rect.w - 16.0, 36.0, Color::from_rgba(55, 55, 55, 255));
        draw_texture_ex(
            item_texture(item),
            rect.x + 12.0,
            y + 4.0,
            WHITE,
            DrawTextureParams {
                dest_size: Some(vec2(28.0, 28.0)),
                ..Default::default()
            },
        );
        let name = &database
            .items
            .get(&item.id)
            .expect("ground item definition not found")
            .name;
        draw_text(name, rect.x + 50.0, y + 23.0, 18.0, WHITE);
    }

    if let Some(entity) = dragged_entity {
        if let Ok(item) = world.get::<&GroundItem>(entity) {
            draw_texture_ex(
                item_texture(&item.item),
                mouse_position().0 - 16.0,
                mouse_position().1 - 16.0,
                WHITE,
                DrawTextureParams {
                    dest_size: Some(vec2(32.0, 32.0)),
                    ..Default::default()
                },
            );
        }
    }
}

pub fn pick_up_ground_item(world: &mut World, database: &Database, camera: &Camera2D) -> bool {
    if !is_mouse_button_pressed(MouseButton::Left) {
        return false;
    }

    let mouse_position = mouse_position();
    let clicked_position = camera.screen_to_world(vec2(mouse_position.0, mouse_position.1));
    let clicked_cell = (clicked_position / TILE_SIZE).round() * TILE_SIZE;
    let player_position = world
        .query::<(&Player, &Position)>()
        .iter()
        .next()
        .map(|(_, position)| position.value);

    let Some(player_position) = player_position else {
        return false;
    };

    if player_position.distance(clicked_cell) > TILE_SIZE * 2.0 {
        return false;
    }

    let clicked_items = ground_items_at(world, clicked_cell);
    if clicked_items.len() > 1 {
        return true;
    }
    let clicked_item = clicked_items
        .into_iter()
        .next()
        .map(|(entity, item)| (entity, item.id, item.quantity));

    let Some((ground_entity, item_id, quantity)) = clicked_item else {
        return false;
    };

    let item_weight = database
        .items
        .get(&item_id)
        .expect("ground item definition not found")
        .weight
        * quantity as f32;
    let mut stored_item = false;

    let inventory_entity = player_inventory_entity(world)
        .expect("player inventory must be initialized");
    if let Ok(mut inventory) = world.get::<&mut Inventory>(inventory_entity) {
        if inventory.weight + item_weight > inventory.max_weight {
            return true;
        }

        if let Some(existing_item) = inventory
            .items
            .iter_mut()
            .flatten()
            .find(|item| item.id == item_id)
        {
            existing_item.quantity += quantity;
            stored_item = true;
        } else if let Some(empty_slot) = inventory.items.iter_mut().find(|slot| slot.is_none()) {
            *empty_slot = Some(ItemRef {
                id: item_id,
                quantity,
            });
            stored_item = true;
        }

        if stored_item {
            inventory.weight += item_weight;
        }
    }

    if stored_item {
        world.despawn(ground_entity).expect("ground item disappeared before pickup");
    }

    true
}

pub fn draw_ground_items(world: &World, database: &Database) {
    let mut groups: Vec<(Vec2, Vec<ItemRef>)> = Vec::new();
    for ground_item in world.query::<&GroundItem>().iter() {
        let cell = world_cell(ground_item.position);
        if let Some((_, items)) = groups.iter_mut().find(|(position, _)| *position == cell) {
            items.push(ground_item.item.clone());
        } else {
            groups.push((cell, vec![ground_item.item.clone()]));
        }
    }

    let stack_texture = database
        .textures
        .get("./content/ItemStack/ItemStack.png")
        .expect("item stack texture not found");
    for (position, items) in groups {
        let texture = if items.len() > 1 {
            stack_texture
        } else {
            let item = database
                .items
                .get(&items[0].id)
                .expect("ground item definition not found");
            database
                .textures
                .get(&item.texture_path)
                .expect("ground item texture not found")
        };
        let icon_size = vec2(TILE_SIZE * 0.7, TILE_SIZE * 0.7);
        draw_texture_ex(
            texture,
            position.x + (TILE_SIZE - icon_size.x) * 0.5,
            position.y + (TILE_SIZE - icon_size.y) * 0.5,
            WHITE,
            DrawTextureParams {
                dest_size: Some(icon_size),
                ..Default::default()
            },
        );
    }
}