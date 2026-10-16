use crate::body::{Appearence, Sprite, SpriteOverlay};
use crate::database::Database;
use crate::inventory::{Equipment, EquipmentSlot};
use crate::movement::{BezierMovement, Position, RandomWalker};
use crate::objects::OnOffSprite;
use crate::objects::{self, Gatherable, Solid, Tree};
use crate::player::Player;
use crate::tilemap::{MAP_HEIGHT, MAP_WIDTH};
use crate::utils;
use hecs::{Entity, World};
use macroquad::prelude::*;

pub struct Input;

pub fn attack_mode_active() -> bool {
    is_key_down(KeyCode::LeftShift) || is_key_down(KeyCode::RightShift)
}

pub fn attack_input(world: &World, database: &Database, camera: &Camera2D) -> Option<Entity> {
    if !attack_mode_active() || !is_mouse_button_pressed(MouseButton::Left) {
        return None;
    }

    let target_entity = attack_target_at_cursor(world, database, camera)?;
    if let Ok(tree) = world.get::<&Tree>(target_entity) {
        if !tree.can_be_attacked(player_has_stone_axe(world)) {
            return None;
        }
    }
    let Ok(target_position) = world.get::<&Position>(target_entity) else {
        return None;
    };
    crate::movement::start_attack(world, target_entity, target_position.value)
        .then_some(target_entity)
}

pub fn gather_input(world: &mut World, database: &Database, camera: &Camera2D) -> bool {
    if !is_mouse_button_pressed(MouseButton::Left) {
        return false;
    }

    let Some(target_entity) = attack_target_at_cursor(world, database, camera) else {
        return false;
    };
    if world.get::<&Gatherable>(target_entity).is_err() {
        return false;
    }

    objects::collect_gatherable(world, database, target_entity)
}

fn player_has_stone_axe(world: &World) -> bool {
    world
        .query::<(&Player, &Equipment)>()
        .iter()
        .any(|(_, equipment)| {
            equipment
                .items
                .get(&EquipmentSlot::MainHand)
                .is_some_and(|item_id| item_id == "stone_axe")
        })
}

pub fn attack_target_at_cursor(
    world: &World,
    database: &Database,
    camera: &Camera2D,
) -> Option<Entity> {
    let mouse = mouse_position();
    let mouse_world = camera.screen_to_world(vec2(mouse.0, mouse.1));
    let mut selected = None;
    for (entity, position) in world.query::<(Entity, &Position)>().iter() {
        if selected.is_some_and(|(selected_y, _)| {
            position.value.y.total_cmp(&selected_y) != std::cmp::Ordering::Greater
        }) || !entity_hit_at(world, database, entity, &position, mouse_world)
        {
            continue;
        }

        selected = Some((position.value.y, entity));
    }

    selected.map(|(_, entity)| entity)
}

pub fn attack_target_at_cursor_visible(
    world: &World,
    database: &Database,
    camera: &Camera2D,
    visible_entities: &[(f32, Entity)],
) -> Option<Entity> {
    let mouse = mouse_position();
    let mouse_world = camera.screen_to_world(vec2(mouse.0, mouse.1));

    visible_entities.iter().rev().find_map(|&(_, entity)| {
        let position = world.get::<&Position>(entity).ok()?;
        entity_hit_at(world, database, entity, &position, mouse_world).then_some(entity)
    })
}

fn entity_hit_at(
    world: &World,
    database: &Database,
    entity: Entity,
    position: &Position,
    mouse_world: Vec2,
) -> bool {
    if world.get::<&Player>(entity).is_ok()
        || (world.get::<&Sprite>(entity).is_err()
            && world.get::<&SpriteOverlay>(entity).is_err()
            && world.get::<&OnOffSprite>(entity).is_err()
            && world.get::<&Appearence>(entity).is_err())
    {
        return false;
    }

    let tree_overlay_hidden = world
        .get::<&Tree>(entity)
        .is_ok_and(|tree| tree.fall.is_some() || tree.felled);
    let sprite_hit = world.get::<&Sprite>(entity).is_ok_and(|sprite| {
        texture_hit_at(
            database,
            &sprite.texture,
            position.value + position.offset + sprite.offset,
            mouse_world,
            false,
        )
    }) || (!tree_overlay_hidden
        && world.get::<&SpriteOverlay>(entity).is_ok_and(|overlay| {
            texture_hit_at(
                database,
                &overlay.sprite.texture,
                position.value + position.offset + overlay.sprite.offset,
                mouse_world,
                false,
            )
        }))
        || world.get::<&OnOffSprite>(entity).is_ok_and(|sprite| {
            let sprite = if sprite.is_on {
                &sprite.on
            } else {
                &sprite.off
            };
            texture_hit_at(
                database,
                &sprite.texture,
                position.value + position.offset + sprite.offset,
                mouse_world,
                false,
            )
        });
    let appearance_hit = world.get::<&Appearence>(entity).is_ok_and(|appearance| {
        let equipment = world
            .get::<&Equipment>(entity)
            .expect("character equipment disappeared");
        appearance_hit_at(database, position, &appearance, &equipment, mouse_world)
    });

    sprite_hit || appearance_hit
}

fn texture_hit_at(
    database: &Database,
    texture: &Texture2D,
    origin: Vec2,
    point: Vec2,
    flip_x: bool,
) -> bool {
    let local_x = point.x - origin.x;
    let local_y = point.y - origin.y;
    if local_x < 0.0 || local_y < 0.0 || local_x >= texture.width() || local_y >= texture.height() {
        return false;
    }

    let mut pixel_x = local_x.floor() as u32;
    if flip_x {
        pixel_x = texture.width() as u32 - 1 - pixel_x;
    }
    database.texture_pixel_is_opaque(texture, pixel_x, local_y.floor() as u32)
}

fn appearance_hit_at(
    database: &Database,
    position: &Position,
    appearance: &Appearence,
    equipment: &Equipment,
    point: Vec2,
) -> bool {
    let race = database
        .races
        .get(appearance.body.race)
        .expect("character race not found");
    let body = if appearance.body.genre {
        &race.male_sprite
    } else {
        &race.female_sprite
    };
    let body_origin = position.value + position.offset;
    if texture_hit_at(
        database,
        &body.texture,
        body_origin,
        point,
        appearance.is_flipped,
    ) {
        return true;
    }

    for slot in [
        EquipmentSlot::Legs,
        EquipmentSlot::Torso,
        EquipmentSlot::Feet,
        EquipmentSlot::Head,
        EquipmentSlot::OffHand,
        EquipmentSlot::MainHand,
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
        let texture_path = if appearance.body.genre {
            &item_equipment.male_texture_path
        } else {
            &item_equipment.female_texture_path
        };
        let texture = database
            .textures
            .get(texture_path)
            .expect("equipped item texture not found");
        let offset_x = if appearance.is_flipped {
            -item_equipment.offset.x
        } else {
            item_equipment.offset.x
        };
        let origin = if matches!(slot, EquipmentSlot::MainHand | EquipmentSlot::OffHand) {
            let is_main_hand = slot == EquipmentSlot::MainHand;
            let is_on_right = is_main_hand != appearance.is_flipped;
            let item_x = if is_on_right {
                body_origin.x + body.texture.width()
            } else {
                body_origin.x - texture.width()
            };
            vec2(item_x + offset_x, body_origin.y + item_equipment.offset.y)
        } else {
            body_origin + vec2(offset_x, item_equipment.offset.y)
        };
        if texture_hit_at(database, texture, origin, point, appearance.is_flipped) {
            return true;
        }
    }

    if let Some(beard_id) = appearance.beard {
        let beard = &database.hairs[beard_id];
        if texture_hit_at(
            database,
            &beard.texture,
            position.value,
            point,
            appearance.is_flipped,
        ) {
            return true;
        }
    }

    if let Some(hair_id) = appearance.hair {
        let hair = &database.hairs[hair_id];
        let hair_x = if appearance.is_flipped {
            position.value.x + body.texture.width() - hair.offset.x - hair.texture.width()
        } else {
            position.value.x + hair.offset.x
        };
        if texture_hit_at(
            database,
            &hair.texture,
            vec2(hair_x, position.value.y + hair.offset.y),
            point,
            appearance.is_flipped,
        ) {
            return true;
        }
    }

    false
}

pub fn is_blocked(world: &World, position: Vec2) -> bool {
    let cell = position / utils::TILE_SIZE;
    if cell.x < 0.0 || cell.y < 0.0 || cell.x >= MAP_WIDTH as f32 || cell.y >= MAP_HEIGHT as f32 {
        return true;
    }

    for (pos, _) in world.query::<(&Position, &Solid)>().iter() {
        if pos.value == position {
            return true;
        }
    }

    for (pos, _) in world.query::<(&Position, &RandomWalker)>().iter() {
        if pos.value == position {
            return true;
        }
    }

    false
}

pub fn movement_input(world: &World) -> bool {
    let left = is_key_down(KeyCode::A) || is_key_down(KeyCode::Left);
    let right = is_key_down(KeyCode::D) || is_key_down(KeyCode::Right);
    let up = is_key_down(KeyCode::W) || is_key_down(KeyCode::Up);
    let down = is_key_down(KeyCode::S) || is_key_down(KeyCode::Down);

    let direction = vec2(
        right as i8 as f32 - left as i8 as f32,
        down as i8 as f32 - up as i8 as f32,
    );
    if direction == Vec2::ZERO {
        return false;
    }

    for (bezier, pos, _) in world
        .query::<(&mut BezierMovement, &Position, &Input)>()
        .iter()
    {
        let destination = pos.value + direction * utils::TILE_SIZE;
        if is_blocked(world, destination) {
            return false;
        }

        bezier.p0 = pos.value;
        bezier.p1 = (pos.value + destination) * 0.5 + vec2(0.0, -16.0);
        bezier.p2 = destination;
        bezier.t = 0.0;
        bezier.has_started = true;
        return true;
    }

    false
}

pub fn camera_zoom(camera: &mut Camera2D, zoom: &mut f32, dt: f32) {
    let (_, scroll) = mouse_wheel();
    const ZOOM_FACTOR: f32 = 1.1;

    if scroll != 0.0 {
        *zoom *= ZOOM_FACTOR.powf(scroll * 0.01);
        *zoom = zoom.clamp(0.5, 8.0);

        camera.zoom = vec2(*zoom / screen_width(), *zoom / screen_height());
    }
}
