use hecs::World;
use macroquad::{color, prelude::*};

use crate::body::{Sprite, SpriteOverlay};
use crate::character_creator;
use crate::database::Database;
use crate::inventory::{INVENTORY_SLOT_COUNT, Inventory};
use crate::movement::Position;
use crate::objects::{Chest, Gatherable, OnOffSprite, Solid, Tree};
use crate::spawner;
use crate::tilemap::{MAP_CENTER_CELL, MAP_HEIGHT, MAP_WIDTH, create_map};
use crate::utils::{TILE_SIZE, cell_to_world};

const TREE_DENSITY_PERCENT: i32 = 7;
const BUSH_DENSITY_PERCENT: i32 = 22;
const STONE_DENSITY_PERCENT: i32 = 20;
const CLEARING_RADIUS: i32 = 6;
const TREE_TOP_TEXTURE: &str = "./content/vegetation/pine-tree.png";
const TREE_BASE_TEXTURE: &str = "./content/vegetation/cutted-tree.png";
const BUSH_TEXTURES: [&str; 2] = [
    "./content/vegetation/tree-bunch.png",
    "./content/vegetation/tree-bunch2.png",
];
const STONE_TEXTURE: &str = "./content/vegetation/stone.png";

pub async fn spawn_entities(world: &mut World, database: &Database) {
    let horse_texture = database
        .textures
        .get("./content/creatures/horse.png")
        .expect("horse texture not found");

    database
        .textures
        .get("./content/creatures/saddle.png")
        .expect("saddle texture not found");

    let horse_sprite_offset = vec2(
        (TILE_SIZE - &horse_texture.width()) * 0.5,
        TILE_SIZE - &horse_texture.height(),
    );

    let horse = spawner::spawn_creature(
        world,
        database,
        "horse",
        cell_to_world(3, 2),
        Vec2::ZERO,
        horse_sprite_offset,
    )
    .expect("initial horse failed to spawn");
    world
        .insert_one(
            horse,
            SpriteOverlay {
                sprite: Sprite {
                    color: color::WHITE,
                    texture_path: "./content/creatures/saddle.png".into(),
                    offset: vec2(29.0, 47.0) + horse_sprite_offset,
                },
            },
        )
        .expect("initial horse disappeared before saddle was added");

    world.spawn((
        Position {
            value: cell_to_world(0, 0),
            offset: vec2(0.0, -10.0),
        },
        OnOffSprite {
            on: Sprite {
                color: color::WHITE,
                texture_path: "./content/chests/chest-open.png".into(),
                offset: vec2(0.0, 0.0),
            },
            off: Sprite {
                color: color::WHITE,
                texture_path: "./content/chests/chest-closed.png".into(),
                offset: vec2(0.0, 0.0),
            },
            is_on: false,
        },
        Chest {},
        Solid {},
        Inventory {
            items: vec![None; INVENTORY_SLOT_COUNT],
            weight: 0.0,
            max_weight: f32::INFINITY,
            rect: Rect::new(0.0, 0.0, 100.0, 100.0),
        },
    ));

    spawn_nature(world, database);
    spawner::spawn_creature_spawner(world);
    world.spawn((create_map(),));
    character_creator::open_for_creation(world);
}

fn spawn_nature(world: &mut World, database: &Database) {
    let tree_top = database.texture(TREE_TOP_TEXTURE);
    let tree_base = database.texture(TREE_BASE_TEXTURE);

    for y in 0..MAP_HEIGHT {
        for x in 0..MAP_WIDTH {
            let x = x as i32;
            let y = y as i32;
            let center_x = MAP_CENTER_CELL.0;
            let center_y = MAP_CENTER_CELL.1;
            let near_center = (x - center_x).pow(2) + (y - center_y).pow(2)
                <= CLEARING_RADIUS * CLEARING_RADIUS;
            let near_origin = x * x + y * y <= CLEARING_RADIUS * CLEARING_RADIUS;
            if near_center || near_origin {
                continue;
            }

            let position = cell_to_world(x, y);
            let roll = macroquad::rand::gen_range(0, 100);
            if roll < TREE_DENSITY_PERCENT {
                let base_offset = vec2(
                    (TILE_SIZE - tree_base.width()) * 0.5,
                    (tree_base.height() / 2.0) - 10.0,
                );
                let top_offset = vec2(
                    base_offset.x + (tree_base.width() - tree_top.width()) * 0.5,
                    base_offset.y - tree_top.height() + 32.0,
                );

                world.spawn((
                    Position {
                        value: position,
                        offset: vec2(0.0, 0.0),
                    },
                    Sprite {
                        color: color::WHITE,
                        texture_path: TREE_BASE_TEXTURE.into(),
                        offset: base_offset,
                    },
                    SpriteOverlay {
                        sprite: Sprite {
                            color: color::WHITE,
                            texture_path: TREE_TOP_TEXTURE.into(),
                            offset: top_offset,
                        },
                    },
                    Solid {},
                    Tree {
                        hits: 0,
                        fall: None,
                        felled: false,
                    },
                ));
            } else if roll < TREE_DENSITY_PERCENT + BUSH_DENSITY_PERCENT {
                let texture_path =
                    BUSH_TEXTURES[macroquad::rand::gen_range(0, BUSH_TEXTURES.len())];
                let texture = database.texture(texture_path);
                spawn_gatherable(
                    world,
                    position,
                    texture_path,
                    "stick",
                    vec2(
                        (TILE_SIZE - texture.width()) * 0.5,
                        TILE_SIZE - texture.height(),
                    ),
                );
            } else if roll < TREE_DENSITY_PERCENT + BUSH_DENSITY_PERCENT + STONE_DENSITY_PERCENT {
                spawn_gatherable(
                    world,
                    position,
                    STONE_TEXTURE,
                    "stone",
                    vec2(
                        (TILE_SIZE - database.texture(STONE_TEXTURE).width()) * 0.5,
                        TILE_SIZE - database.texture(STONE_TEXTURE).height(),
                    ),
                );
            }
        }
    }
}

fn spawn_gatherable(
    world: &mut World,
    position: Vec2,
    texture_path: &str,
    item_id: &str,
    sprite_offset: Vec2,
) {
    world.spawn((
        Position {
            value: position,
            offset: Vec2::ZERO,
        },
        Sprite {
            color: color::WHITE,
            texture_path: texture_path.into(),
            offset: sprite_offset,
        },
        Gatherable {
            item: crate::inventory::ItemRef {
                id: item_id.to_string(),
                quantity: 1,
            },
        },
    ));
}
