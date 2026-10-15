use macroquad::prelude::*;
use hecs::{Entity, World};

use crate::database::Database;
use crate::movement::Position;
use crate::objects::{OnOffSprite, Tree};
use crate::tilemap::*;
use crate::utils;
use crate::body::{Appearence, Sprite, SpriteOverlay};
use crate::inventory::{Equipment, EquipmentSlot};

pub fn draw_entities(world: &World, database: &Database, camera: &Camera2D) {
    let camera_bounds = camera_world_bounds(camera);
    let mut render_order: Vec<(f32, Entity)> = world
        .query::<(Entity, &Position)>()
        .iter()
        .filter(|(entity, _)| {
            let is_renderable = world.get::<&Sprite>(*entity).is_ok()
                || world.get::<&SpriteOverlay>(*entity).is_ok()
                || world.get::<&Appearence>(*entity).is_ok()
                || world.get::<&OnOffSprite>(*entity).is_ok();
            let visible_tree = world.get::<&Tree>(*entity).is_err()
                || tree_is_visible(world, *entity, camera_bounds);
            is_renderable && visible_tree
        })
        .map(|(entity, position)| (position.value.y, entity))
        .collect();
    render_order.sort_by(|(left_y, _), (right_y, _)| left_y.total_cmp(right_y));

    for (_, entity) in render_order {
        let position = world
            .get::<&Position>(entity)
            .expect("render entity position disappeared");

        if let Ok(sprite) = world.get::<&Sprite>(entity) {
            draw_sprite(&sprite, &position);
        }
        if let Ok(overlay) = world.get::<&SpriteOverlay>(entity) {
            draw_sprite(&overlay.sprite, &position);
        }

        if let Ok(sprite) = world.get::<&OnOffSprite>(entity) {
            let sprite = if sprite.is_on { &sprite.on } else { &sprite.off };
            draw_sprite(sprite, &position);
        }

        if let Ok(appearence) = world.get::<&Appearence>(entity) {
            let equipment = world
                .get::<&Equipment>(entity)
                .expect("character equipment disappeared");
            draw_appearence(&position, &appearence, &equipment, database);
        }
    }
}

fn camera_world_bounds(camera: &Camera2D) -> Rect {
    let corners = [
        camera.screen_to_world(vec2(0.0, 0.0)),
        camera.screen_to_world(vec2(screen_width(), 0.0)),
        camera.screen_to_world(vec2(0.0, screen_height())),
        camera.screen_to_world(vec2(screen_width(), screen_height())),
    ];
    let min = corners.iter().copied().reduce(Vec2::min).unwrap();
    let max = corners.iter().copied().reduce(Vec2::max).unwrap();
    Rect::new(min.x, min.y, max.x - min.x, max.y - min.y)
}

fn tree_is_visible(world: &World, entity: Entity, camera_bounds: Rect) -> bool {
    let Ok(position) = world.get::<&Position>(entity) else {
        return false;
    };
    let sprite_is_visible = |sprite: &Sprite| {
        let left = position.value.x + position.offset.x + sprite.offset.x;
        let top = position.value.y + position.offset.y + sprite.offset.y;
        left < camera_bounds.x + camera_bounds.w
            && left + sprite.texture.width() > camera_bounds.x
            && top < camera_bounds.y + camera_bounds.h
            && top + sprite.texture.height() > camera_bounds.y
    };

    world
        .get::<&Sprite>(entity)
        .is_ok_and(|sprite| sprite_is_visible(&sprite))
        || world
            .get::<&SpriteOverlay>(entity)
            .is_ok_and(|overlay| sprite_is_visible(&overlay.sprite))
}

fn draw_sprite(sprite: &Sprite, position: &Position) {
    draw_texture(
        &sprite.texture,
        position.value.x + position.offset.x + sprite.offset.x,
        position.value.y + position.offset.y + sprite.offset.y,
        sprite.color,
    );
}

fn draw_appearence(
    position: &Position,
    appearence: &Appearence,
    equipment: &Equipment,
    database: &Database,
) {
    let race = database
        .races
        .get(appearence.body.race)
        .expect("race not found");
    let body_sprite = if appearence.body.genre {
        &race.male_sprite
    } else {
        &race.female_sprite
    };

    draw_texture_ex(
        &body_sprite.texture,
        position.value.x + position.offset.x,
        position.value.y + position.offset.y,
        race.default_skin_color,
        DrawTextureParams {
            flip_x: appearence.is_flipped,
            ..Default::default()
        },
    );

    for slot in [
        EquipmentSlot::Legs,
        EquipmentSlot::Torso,
        EquipmentSlot::Feet,
        EquipmentSlot::Head,
    ] {
        let Some(item_id) = equipment.items.get(&slot) else {
            continue;
        };
        let item = database
            .items
            .get(item_id)
            .expect("equipped item definition not found");
        let Some(item_equipment) = &item.equipment else {
            continue;
        };
        let texture_path = if appearence.body.genre {
            &item_equipment.male_texture_path
        } else {
            &item_equipment.female_texture_path
        };
        let texture = database
            .textures
            .get(texture_path)
            .expect("equipped item texture not found");
        let offset_x = if appearence.is_flipped {
            -item_equipment.offset.x
        } else {
            item_equipment.offset.x
        };
        draw_texture_ex(
            texture,
            position.value.x + position.offset.x + offset_x,
            position.value.y + position.offset.y + item_equipment.offset.y,
            WHITE,
            DrawTextureParams {
                flip_x: appearence.is_flipped,
                ..Default::default()
            },
        );
    }

    let body_left = position.value.x + position.offset.x;
    let body_top = position.value.y + position.offset.y;
    let body_width = body_sprite.texture.width();
    for (slot, is_main_hand) in [
        (EquipmentSlot::OffHand, false),
        (EquipmentSlot::MainHand, true),
    ] {
        let Some(item_id) = equipment.items.get(&slot) else {
            continue;
        };
        let item = database
            .items
            .get(item_id)
            .expect("equipped item definition not found");
        let Some(item_equipment) = &item.equipment else {
            continue;
        };
        let texture_path = if appearence.body.genre {
            &item_equipment.male_texture_path
        } else {
            &item_equipment.female_texture_path
        };
        let texture = database
            .textures
            .get(texture_path)
            .expect("equipped item texture not found");
        let is_on_right = is_main_hand != appearence.is_flipped;
        let item_x = if is_on_right {
            body_left + body_width
        } else {
            body_left - texture.width()
        };
        let offset_x = if appearence.is_flipped {
            -item_equipment.offset.x
        } else {
            item_equipment.offset.x
        };
        draw_texture_ex(
            texture,
            item_x + offset_x,
            body_top + item_equipment.offset.y,
            WHITE,
            DrawTextureParams {
                flip_x: appearence.is_flipped,
                ..Default::default()
            },
        );
    }

    if let Some(beard_id) = appearence.beard {
        let beard = &database.hairs[beard_id];
        draw_texture_ex(
            &beard.texture,
            position.value.x,
            position.value.y,
            beard.color,
            DrawTextureParams {
                flip_x: appearence.is_flipped,
                ..Default::default()
            },
        );
    }

    if let Some(hair_id) = appearence.hair {
        let hair = &database.hairs[hair_id];
        let body_width = body_sprite.texture.width();
        let hair_x = if appearence.is_flipped {
            position.value.x + body_width - hair.offset.x - hair.texture.width()
        } else {
            position.value.x + hair.offset.x
        };

        draw_texture_ex(
            &hair.texture,
            hair_x,
            position.value.y + hair.offset.y,
            hair.color,
            DrawTextureParams {
                flip_x: appearence.is_flipped,
                ..Default::default()
            },
        );
    }
}

pub fn draw_tilemap(world: &World, database : &Database, camera : &Camera2D) {
    for tilemap in world.query::<&TileMap>().iter() {
        let top_left = camera.screen_to_world(vec2(0.0, 0.0));
        let bottom_right = camera.screen_to_world(vec2(screen_width(), screen_height()));

        let left = top_left.x;
        let top = top_left.y;
        let right = bottom_right.x;
        let bottom = bottom_right.y;

        let tile_size = utils::TILE_SIZE;

        for layer in &tilemap.layers {
            let start_x = (left / tile_size).floor().max(0.0) as usize;
            let end_x = (right / tile_size).ceil().min(layer.width as f32) as usize;

            let start_y = (top / tile_size).floor().max(0.0) as usize;
            let end_y = (bottom / tile_size).ceil().min(layer.height as f32) as usize;

            for y in start_y..end_y {
                for x in start_x..end_x {
                    let index = y * layer.width + x;
                    let tile_id = layer.tiles[index];

                    if tile_id == 0 {
                        continue;
                    }

                    if let Some(tile) = database.tiles.get(&tile_id) {
                        draw_texture(
                            &tile.texture,
                            x as f32 * tile_size,
                            y as f32 * tile_size,
                            WHITE,
                        );
                    }
                }
            }
        }
    }
}
