use hecs::{Entity, World};
use macroquad::prelude::*;
use std::collections::{HashMap, HashSet};

use crate::body::{Appearence, Sprite, SpriteOverlay};
use crate::database::Database;
use crate::health::Health;
use crate::input;
use crate::inventory::{Equipment, EquipmentSlot};
use crate::movement::{BezierMovement, Position};
use crate::objects::{Gatherable, OnOffSprite, Solid, Tree};
use crate::tilemap::*;
use crate::utils::{self, TILE_SIZE};

#[derive(Default)]
pub struct EntityRenderCache {
    static_entities_by_cell: HashMap<(i32, i32), Vec<Entity>>,
    static_index_built: bool,
    render_order: Vec<(f32, Entity)>,
    static_candidates: Vec<Entity>,
    seen_static_entities: HashSet<Entity>,
}

pub fn create_outline_material() -> Result<Material, macroquad::Error> {
    load_material(
        ShaderSource::Glsl {
            vertex: include_str!("../content/shaders/outline.vs"),
            fragment: include_str!("../content/shaders/outline.fs"),
        },
        MaterialParams {
            uniforms: vec![UniformDesc::new("outlineColor", UniformType::Float4)],
            ..Default::default()
        },
    )
}

pub fn draw_entities(
    world: &World,
    database: &Database,
    camera: &Camera2D,
    outline_material: &Material,
    show_sprite_hitboxes: bool,
    render_cache: &mut EntityRenderCache,
) {
    let camera_bounds = camera_world_bounds(camera);
    if !render_cache.static_index_built {
        build_static_entity_index(world, database, render_cache);
    }

    render_cache.render_order.clear();
    collect_static_candidates(camera_bounds, render_cache);
    for &entity in &render_cache.static_candidates {
        let Ok(position) = world.get::<&Position>(entity) else {
            continue;
        };
        if entity_is_visible(world, entity, database, camera_bounds) {
            render_cache.render_order.push((position.value.y, entity));
        }
    }
    for (entity, position, _) in world.query::<(Entity, &Position, &BezierMovement)>().iter() {
        if is_renderable(world, entity) && entity_is_visible(world, entity, database, camera_bounds)
        {
            render_cache.render_order.push((position.value.y, entity));
        }
    }
    let render_order = &mut render_cache.render_order;
    render_order.sort_by(|(left_y, _), (right_y, _)| left_y.total_cmp(right_y));
    let selected_entity =
        input::attack_target_at_cursor_visible(world, database, camera, render_order);
    let outline_color = if input::attack_mode_active() {
        RED
    } else {
        WHITE
    };

    for &(_, entity) in render_order.iter() {
        let position = world
            .get::<&Position>(entity)
            .expect("render entity position disappeared");

        if selected_entity == Some(entity) {
            draw_entity_outline(
                world,
                entity,
                &position,
                database,
                outline_material,
                outline_color,
            );
        }

        if let Ok(sprite) = world.get::<&Sprite>(entity) {
            draw_sprite(&sprite, &position, database);
        }
        if let Ok(overlay) = world.get::<&SpriteOverlay>(entity) {
            if let Ok(tree) = world.get::<&Tree>(entity) {
                if tree.felled {
                    continue;
                }
                if let Some(fall) = &tree.fall {
                    let x = position.value.x + position.offset.x + overlay.sprite.offset.x;
                    let y = position.value.y + position.offset.y + overlay.sprite.offset.y;
                    let rotation_direction = if fall.direction.x != 0.0 {
                        fall.direction.x
                    } else {
                        fall.direction.y
                    };
                    let rotation_progress = (fall.elapsed
                        / crate::objects::TREE_FALL_ROTATION_DURATION)
                        .clamp(0.0, 1.0);
                    let fade_progress = ((fall.elapsed
                        - crate::objects::TREE_FALL_ROTATION_DURATION)
                        / crate::objects::TREE_FALL_FADE_DURATION)
                        .clamp(0.0, 1.0);
                    let mut color = overlay.sprite.color;
                    color.a *= 1.0 - fade_progress;
                    let texture = database.texture(&overlay.sprite.texture_path);
                    draw_texture_ex(
                        texture,
                        x,
                        y,
                        color,
                        DrawTextureParams {
                            rotation: rotation_direction * 1.2 * rotation_progress,
                            pivot: Some(vec2(
                                x + texture.width() * 0.5,
                                y + texture.height(),
                            )),
                            ..Default::default()
                        },
                    );
                } else {
                    draw_sprite(&overlay.sprite, &position, database);
                }
            } else {
                draw_sprite(&overlay.sprite, &position, database);
            }
        }

        if let Ok(sprite) = world.get::<&OnOffSprite>(entity) {
            let sprite = if sprite.is_on {
                &sprite.on
            } else {
                &sprite.off
            };
            draw_sprite(sprite, &position, database);
        }

        if let Ok(appearence) = world.get::<&Appearence>(entity) {
            let equipment = world
                .get::<&Equipment>(entity)
                .expect("character equipment disappeared");
            draw_appearence(&position, &appearence, &equipment, database);
        }
    }

    draw_health_indicators(world, database, render_order);
    if show_sprite_hitboxes {
        draw_sprite_hitboxes(world, database, render_order);
    }
}

fn is_renderable(world: &World, entity: Entity) -> bool {
    world.get::<&Sprite>(entity).is_ok()
        || world.get::<&SpriteOverlay>(entity).is_ok()
        || world.get::<&Appearence>(entity).is_ok()
        || world.get::<&OnOffSprite>(entity).is_ok()
}

fn build_static_entity_index(
    world: &World,
    database: &Database,
    render_cache: &mut EntityRenderCache,
) {
    render_cache.static_entities_by_cell.clear();
    for (entity, position) in world.query::<(Entity, &Position)>().iter() {
        let is_static_world_entity = world.get::<&Tree>(entity).is_ok()
            || world.get::<&Gatherable>(entity).is_ok()
            || world.get::<&Solid>(entity).is_ok();
        if !is_static_world_entity
            || world.get::<&BezierMovement>(entity).is_ok()
            || !is_renderable(world, entity)
        {
            continue;
        }

        let Some(bounds) = static_entity_bounds(world, entity, &position, database) else {
            continue;
        };
        let start_x = (bounds.x / TILE_SIZE).floor() as i32;
        let start_y = (bounds.y / TILE_SIZE).floor() as i32;
        let end_x = ((bounds.x + bounds.w) / TILE_SIZE).floor() as i32;
        let end_y = ((bounds.y + bounds.h) / TILE_SIZE).floor() as i32;
        for y in start_y..=end_y {
            for x in start_x..=end_x {
                render_cache
                    .static_entities_by_cell
                    .entry((x, y))
                    .or_default()
                    .push(entity);
            }
        }
    }
    render_cache.static_index_built = true;
}

fn static_entity_bounds(
    world: &World,
    entity: Entity,
    position: &Position,
    database: &Database,
) -> Option<Rect> {
    let mut bounds: Option<Rect> = None;
    if let Ok(sprite) = world.get::<&Sprite>(entity) {
        include_sprite_bounds(&mut bounds, &sprite, position, database);
    }
    if let Ok(overlay) = world.get::<&SpriteOverlay>(entity) {
        include_sprite_bounds(&mut bounds, &overlay.sprite, position, database);
    }
    if let Ok(on_off) = world.get::<&OnOffSprite>(entity) {
        include_sprite_bounds(&mut bounds, &on_off.on, position, database);
        include_sprite_bounds(&mut bounds, &on_off.off, position, database);
    }
    bounds
}

fn include_sprite_bounds(
    bounds: &mut Option<Rect>,
    sprite: &Sprite,
    position: &Position,
    database: &Database,
) {
    let texture = database.texture(&sprite.texture_path);
    let sprite_bounds = Rect::new(
        position.value.x + position.offset.x + sprite.offset.x,
        position.value.y + position.offset.y + sprite.offset.y,
        texture.width(),
        texture.height(),
    );
    *bounds = Some(match *bounds {
        Some(current) => {
            let left = current.x.min(sprite_bounds.x);
            let top = current.y.min(sprite_bounds.y);
            let right = (current.x + current.w).max(sprite_bounds.x + sprite_bounds.w);
            let bottom = (current.y + current.h).max(sprite_bounds.y + sprite_bounds.h);
            Rect::new(left, top, right - left, bottom - top)
        }
        None => sprite_bounds,
    });
}

fn collect_static_candidates(camera_bounds: Rect, render_cache: &mut EntityRenderCache) {
    render_cache.static_candidates.clear();
    render_cache.seen_static_entities.clear();
    let start_x = (camera_bounds.x / TILE_SIZE).floor() as i32;
    let start_y = (camera_bounds.y / TILE_SIZE).floor() as i32;
    let end_x = ((camera_bounds.x + camera_bounds.w) / TILE_SIZE).floor() as i32;
    let end_y = ((camera_bounds.y + camera_bounds.h) / TILE_SIZE).floor() as i32;
    for y in start_y..=end_y {
        for x in start_x..=end_x {
            if let Some(entities) = render_cache.static_entities_by_cell.get(&(x, y)) {
                for &entity in entities {
                    if render_cache.seen_static_entities.insert(entity) {
                        render_cache.static_candidates.push(entity);
                    }
                }
            }
        }
    }
}

fn draw_entity_outline(
    world: &World,
    entity: Entity,
    position: &Position,
    database: &Database,
    outline_material: &Material,
    outline_color: Color,
) {
    if let Ok(sprite) = world.get::<&Sprite>(entity) {
        draw_sprite_outline(&sprite, position, database, false, outline_material, outline_color);
    }
    let hide_tree_canopy = world
        .get::<&Tree>(entity)
        .is_ok_and(|tree| tree.fall.is_some() || tree.felled);
    if !hide_tree_canopy {
        if let Ok(overlay) = world.get::<&SpriteOverlay>(entity) {
            draw_sprite_outline(
                &overlay.sprite,
                position,
                database,
                false,
                outline_material,
                outline_color,
            );
        }
    }
    if let Ok(sprite) = world.get::<&OnOffSprite>(entity) {
        let sprite = if sprite.is_on {
            &sprite.on
        } else {
            &sprite.off
        };
        draw_sprite_outline(sprite, position, database, false, outline_material, outline_color);
    }
    if let Ok(appearance) = world.get::<&Appearence>(entity) {
        let race = database
            .races
            .get(appearance.body.race)
            .expect("character race not found");
        let body = if appearance.body.genre {
            &race.male_sprite
        } else {
            &race.female_sprite
        };
        let body_texture = database.texture(&body.texture_path);
        draw_texture_outline(
            body_texture,
            position.value.x + position.offset.x,
            position.value.y + position.offset.y,
            appearance.is_flipped,
            outline_material,
            outline_color,
        );
        let base = position.value + position.offset;
        let body_width = body_texture.width();
        if let Some(beard_id) = appearance.beard {
            let beard = &database.beards[beard_id];
            let offset = beard.offset_for(appearance.body.genre);
            let texture = database.texture(beard.texture_path);
            let x = if appearance.is_flipped {
                base.x + body_width - offset.x - texture.width()
            } else {
                base.x + offset.x
            };
            draw_texture_outline(
                texture,
                x,
                base.y + offset.y,
                appearance.is_flipped,
                outline_material,
                outline_color,
            );
        }
        if let Some(hair_id) = appearance.hair {
            let hair = &database.hairs[hair_id];
            let offset = hair.offset_for(appearance.body.genre);
            let texture = database.texture(hair.texture_path);
            let x = if appearance.is_flipped {
                base.x + body_width - offset.x - texture.width()
            } else {
                base.x + offset.x
            };
            draw_texture_outline(
                texture,
                x,
                base.y + offset.y,
                appearance.is_flipped,
                outline_material,
                outline_color,
            );
        }
    }
}

fn draw_sprite_outline(
    sprite: &Sprite,
    position: &Position,
    database: &Database,
    flip_x: bool,
    outline_material: &Material,
    outline_color: Color,
) {
    draw_texture_outline(
        database.texture(&sprite.texture_path),
        position.value.x + position.offset.x + sprite.offset.x,
        position.value.y + position.offset.y + sprite.offset.y,
        flip_x,
        outline_material,
        outline_color,
    );
}

fn draw_texture_outline(
    texture: &Texture2D,
    x: f32,
    y: f32,
    flip_x: bool,
    outline_material: &Material,
    outline_color: Color,
) {
    outline_material.set_uniform("outlineColor", outline_color);
    gl_use_material(outline_material);
    for (dx, dy) in [
        (-1.0, -1.0),
        (0.0, -1.0),
        (1.0, -1.0),
        (-1.0, 0.0),
        (1.0, 0.0),
        (-1.0, 1.0),
        (0.0, 1.0),
        (1.0, 1.0),
    ] {
        draw_texture_ex(
            texture,
            x + dx,
            y + dy,
            WHITE,
            DrawTextureParams {
                flip_x,
                ..Default::default()
            },
        );
    }
    gl_use_default_material();
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

fn entity_is_visible(
    world: &World,
    entity: Entity,
    database: &Database,
    camera_bounds: Rect,
) -> bool {
    let Ok(position) = world.get::<&Position>(entity) else {
        return false;
    };
    let sprite_is_visible = |sprite: &Sprite| {
        let left = position.value.x + position.offset.x + sprite.offset.x;
        let top = position.value.y + position.offset.y + sprite.offset.y;
        texture_is_visible(
            database.texture(&sprite.texture_path),
            left,
            top,
            camera_bounds,
        )
    };

    let sprite_visible = world
        .get::<&Sprite>(entity)
        .is_ok_and(|sprite| sprite_is_visible(&sprite))
        || world
            .get::<&SpriteOverlay>(entity)
            .is_ok_and(|overlay| sprite_is_visible(&overlay.sprite))
        || world.get::<&OnOffSprite>(entity).is_ok_and(|on_off| {
            sprite_is_visible(if on_off.is_on {
                &on_off.on
            } else {
                &on_off.off
            })
        });

    sprite_visible
        || world.get::<&Appearence>(entity).is_ok_and(|appearance| {
            let equipment = world
                .get::<&Equipment>(entity)
                .expect("character equipment disappeared");
            appearance_is_visible(&position, &appearance, &equipment, database, camera_bounds)
        })
}

fn texture_is_visible(texture: &Texture2D, left: f32, top: f32, camera_bounds: Rect) -> bool {
    left < camera_bounds.x + camera_bounds.w
        && left + texture.width() > camera_bounds.x
        && top < camera_bounds.y + camera_bounds.h
        && top + texture.height() > camera_bounds.y
}

fn appearance_is_visible(
    position: &Position,
    appearance: &Appearence,
    equipment: &Equipment,
    database: &Database,
    camera_bounds: Rect,
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
    let body_texture = database.texture(&body.texture_path);
    let base = position.value + position.offset;
    if texture_is_visible(body_texture, base.x, base.y, camera_bounds) {
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
        let (left, top) = if matches!(slot, EquipmentSlot::MainHand | EquipmentSlot::OffHand) {
            let is_main_hand = slot == EquipmentSlot::MainHand;
            let is_on_right = is_main_hand != appearance.is_flipped;
            let item_x = if is_on_right {
                base.x + body_texture.width()
            } else {
                base.x - texture.width()
            };
            (item_x + offset_x, base.y + item_equipment.offset.y)
        } else {
            (base.x + offset_x, base.y + item_equipment.offset.y)
        };
        if texture_is_visible(texture, left, top, camera_bounds) {
            return true;
        }
    }

    if let Some(beard_id) = appearance.beard {
        let beard = &database.beards[beard_id];
        let offset = beard.offset_for(appearance.body.genre);
        let texture = database.texture(beard.texture_path);
        let left = if appearance.is_flipped {
            base.x + body_texture.width() - offset.x - texture.width()
        } else {
            base.x + offset.x
        };
        if texture_is_visible(
            texture,
            left,
            base.y + offset.y,
            camera_bounds,
        ) {
            return true;
        }
    }
    if let Some(hair_id) = appearance.hair {
        let hair = &database.hairs[hair_id];
        let offset = hair.offset_for(appearance.body.genre);
        let texture = database.texture(hair.texture_path);
        let left = if appearance.is_flipped {
            base.x + body_texture.width() - offset.x - texture.width()
        } else {
            base.x + offset.x
        };
        if texture_is_visible(
            texture,
            left,
            base.y + offset.y,
            camera_bounds,
        ) {
            return true;
        }
    }
    false
}

fn draw_sprite(sprite: &Sprite, position: &Position, database: &Database) {
    draw_texture(
        database.texture(&sprite.texture_path),
        position.value.x + position.offset.x + sprite.offset.x,
        position.value.y + position.offset.y + sprite.offset.y,
        sprite.color,
    );
}

fn draw_health_indicators(world: &World, database: &Database, visible_entities: &[(f32, Entity)]) {
    for &(_, entity) in visible_entities {
        let (Ok(position), Ok(health)) =
            (world.get::<&Position>(entity), world.get::<&Health>(entity))
        else {
            continue;
        };
        let base_x = position.value.x + position.offset.x;
        let base_y = position.value.y + position.offset.y;
        let mut left = base_x;
        let mut right = base_x;
        let mut top = base_y;

        let mut include_sprite = |sprite: &Sprite| {
            let sprite_x = base_x + sprite.offset.x;
            let sprite_y = base_y + sprite.offset.y;
            left = left.min(sprite_x);
            right = right.max(
                sprite_x + database.texture(&sprite.texture_path).width(),
            );
            top = top.min(sprite_y);
        };

        if let Ok(sprite) = world.get::<&Sprite>(entity) {
            include_sprite(&sprite);
        }
        if let Ok(overlay) = world.get::<&SpriteOverlay>(entity) {
            include_sprite(&overlay.sprite);
        }
        if let Ok(on_off) = world.get::<&OnOffSprite>(entity) {
            include_sprite(if on_off.is_on {
                &on_off.on
            } else {
                &on_off.off
            });
        }
        if let Ok(appearance) = world.get::<&Appearence>(entity) {
            let race = database
                .races
                .get(appearance.body.race)
                .expect("character race not found");
            let body = if appearance.body.genre {
                &race.male_sprite
            } else {
                &race.female_sprite
            };
            right = right.max(
                base_x + database.texture(&body.texture_path).width(),
            );
            top = top.min(base_y);
        }

        let label = format!("{}/{}", health.current, health.maximum);
        let font_size = 16;
        let dimensions = measure_text(&label, None, font_size, 1.0);
        let x = (left + right - dimensions.width) * 0.5;
        let y = top - 4.0;
        draw_text(&label, x, y, font_size as f32, WHITE);
    }
}

fn draw_sprite_hitboxes(world: &World, database: &Database, visible_entities: &[(f32, Entity)]) {
    for &(_, entity) in visible_entities {
        let Ok(position) = world.get::<&Position>(entity) else {
            continue;
        };
        if let Ok(sprite) = world.get::<&Sprite>(entity) {
            draw_texture_hitbox(
                database.texture(&sprite.texture_path),
                position.value.x + position.offset.x + sprite.offset.x,
                position.value.y + position.offset.y + sprite.offset.y,
            );
        }
        if let Ok(overlay) = world.get::<&SpriteOverlay>(entity) {
            draw_texture_hitbox(
                database.texture(&overlay.sprite.texture_path),
                position.value.x + position.offset.x + overlay.sprite.offset.x,
                position.value.y + position.offset.y + overlay.sprite.offset.y,
            );
        }
        if let Ok(on_off) = world.get::<&OnOffSprite>(entity) {
            let sprite = if on_off.is_on {
                &on_off.on
            } else {
                &on_off.off
            };
            draw_texture_hitbox(
                database.texture(&sprite.texture_path),
                position.value.x + position.offset.x + sprite.offset.x,
                position.value.y + position.offset.y + sprite.offset.y,
            );
        }
        if let Ok(appearance) = world.get::<&Appearence>(entity) {
            let equipment = world
                .get::<&Equipment>(entity)
                .expect("character equipment disappeared");
            draw_appearance_hitboxes(&position, &appearance, &equipment, database);
        }
    }
}

fn draw_texture_hitbox(texture: &Texture2D, x: f32, y: f32) {
    draw_rectangle_lines(x, y, texture.width(), texture.height(), 0.5, LIME);
}

fn draw_appearance_hitboxes(
    position: &Position,
    appearance: &Appearence,
    equipment: &Equipment,
    database: &Database,
) {
    let race = database
        .races
        .get(appearance.body.race)
        .expect("character race not found");
    let body = if appearance.body.genre {
        &race.male_sprite
    } else {
        &race.female_sprite
    };
    let base_x = position.value.x + position.offset.x;
    let base_y = position.value.y + position.offset.y;
    let body_texture = database.texture(&body.texture_path);
    draw_texture_hitbox(body_texture, base_x, base_y);

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
        let (x, y) = if matches!(slot, EquipmentSlot::MainHand | EquipmentSlot::OffHand) {
            let is_main_hand = slot == EquipmentSlot::MainHand;
            let is_on_right = is_main_hand != appearance.is_flipped;
            let item_x = if is_on_right {
                base_x + body_texture.width()
            } else {
                base_x - texture.width()
            };
            (item_x + offset_x, base_y + item_equipment.offset.y)
        } else {
            (base_x + offset_x, base_y + item_equipment.offset.y)
        };
        draw_texture_hitbox(texture, x, y);
    }

    if let Some(beard_id) = appearance.beard {
        let beard = &database.beards[beard_id];
        let offset = beard.offset_for(appearance.body.genre);
        let texture = database.texture(beard.texture_path);
        let x = if appearance.is_flipped {
            base_x + body_texture.width() - offset.x - texture.width()
        } else {
            base_x + offset.x
        };
        draw_texture_hitbox(texture, x, base_y + offset.y);
    }

    if let Some(hair_id) = appearance.hair {
        let hair = &database.hairs[hair_id];
        let offset = hair.offset_for(appearance.body.genre);
        let texture = database.texture(hair.texture_path);
        let hair_x = if appearance.is_flipped {
            base_x + body_texture.width() - offset.x - texture.width()
        } else {
            base_x + offset.x
        };
        draw_texture_hitbox(texture, hair_x, base_y + offset.y);
    }
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
    let body_texture = database.texture(&body_sprite.texture_path);

    draw_texture_ex(
        body_texture,
        position.value.x + position.offset.x,
        position.value.y + position.offset.y,
        appearence.skin_color,
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
    let body_width = body_texture.width();
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
        let beard = &database.beards[beard_id];
        let offset = beard.offset_for(appearence.body.genre);
        let texture = database.texture(beard.texture_path);
        let beard_x = if appearence.is_flipped {
            body_left + body_width - offset.x - texture.width()
        } else {
            body_left + offset.x
        };
        draw_texture_ex(
            texture,
            beard_x,
            body_top + offset.y,
            appearence.hair_color,
            DrawTextureParams {
                flip_x: appearence.is_flipped,
                ..Default::default()
            },
        );
    }

    if let Some(hair_id) = appearence.hair {
        let hair = &database.hairs[hair_id];
        let offset = hair.offset_for(appearence.body.genre);
        let texture = database.texture(hair.texture_path);
        let hair_x = if appearence.is_flipped {
            body_left + body_width - offset.x - texture.width()
        } else {
            body_left + offset.x
        };

        draw_texture_ex(
            texture,
            hair_x,
            body_top + offset.y,
            appearence.hair_color,
            DrawTextureParams {
                flip_x: appearence.is_flipped,
                ..Default::default()
            },
        );
    }
}

pub fn draw_tilemap(world: &World, database: &Database, camera: &Camera2D) {
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
                            database.texture(tile.texture_path),
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn static_candidate_collection_is_limited_to_view_cells_and_deduplicates_entities() {
        let mut world = World::new();
        let visible_entity = world.spawn(());
        let outside_entity = world.spawn(());
        let mut cache = EntityRenderCache::default();
        cache
            .static_entities_by_cell
            .insert((0, 0), vec![visible_entity]);
        cache
            .static_entities_by_cell
            .insert((1, 0), vec![visible_entity]);
        cache
            .static_entities_by_cell
            .insert((2, 0), vec![outside_entity]);

        collect_static_candidates(Rect::new(0.0, 0.0, TILE_SIZE, TILE_SIZE), &mut cache);

        assert_eq!(cache.static_candidates, vec![visible_entity]);
    }
}
