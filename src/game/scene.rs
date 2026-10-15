use hecs::World;
use macroquad::{color, prelude::*};

use crate::body::{Sprite, SpriteOverlay};
use crate::database::Database;
use crate::inventory::{INVENTORY_SLOT_COUNT, Inventory};
use crate::health::{Health, MAX_HEALTH};
use crate::movement::{BezierMovement, MOVEMENT_SPEED, Position, RandomWalker};
use crate::objects::{Chest, OnOffSprite, Solid, Tree};
use crate::player;
use crate::spawner;
use crate::tilemap::{MAP_HEIGHT, MAP_WIDTH, create_map};
use crate::utils::{TILE_SIZE, cell_to_world};

const TREE_DENSITY_PERCENT: i32 = 7;
const CLEARING_RADIUS: i32 = 6;
const TREE_TOP_TEXTURE: &str = "./content/vegetation/pine-tree.png";
const TREE_BASE_TEXTURE: &str = "./content/vegetation/cutted-tree.png";

pub async fn spawn_entities(world: &mut World, database: &Database) {
    player::spawn_player(world);

    let horse_texture = database
        .textures
        .get("./content/creatures/horse.png")
        .expect("horse texture not found")
        .clone();

    let saddle_texture = database
        .textures
        .get("./content/creatures/saddle.png")
        .expect("saddle texture not found")
        .clone();
    let horse_sprite_offset = vec2(
        (TILE_SIZE - horse_texture.width()) * 0.5,
        (TILE_SIZE - horse_texture.height()),
    );

    world.spawn((
        Health::new(MAX_HEALTH),
        Position {
            value: cell_to_world(3, 2),
            offset: Vec2::ZERO,
        },
        BezierMovement {
            p0: cell_to_world(3, 2),
            p1: cell_to_world(3, 2),
            p2: cell_to_world(3, 2),
            t: 0.0,
            has_started: false,
            speed: MOVEMENT_SPEED,
        },
        RandomWalker {},
        Sprite {
            color: color::WHITE,
            texture: horse_texture,
            offset: horse_sprite_offset,
        },
        SpriteOverlay {
            sprite: Sprite {
                color: color::WHITE,
                texture: saddle_texture,
                offset: vec2(29.0, 47.0) + horse_sprite_offset,
            },
        },
    ));

    let chest_open_texture = load_texture("./content/chests/chest-open.png")
        .await
        .unwrap();
    chest_open_texture.set_filter(FilterMode::Nearest);

    let chest_closed_texture = load_texture("./content/chests/chest-closed.png")
        .await
        .unwrap();
    chest_closed_texture.set_filter(FilterMode::Nearest);

    world.spawn((
        Position {
            value: cell_to_world(0, 0),
            offset: vec2(0.0, -10.0),
        },
        OnOffSprite {
            on: Sprite {
                color: color::WHITE,
                texture: chest_open_texture,
                offset: vec2(0.0, 0.0),
            },
            off: Sprite {
                color: color::WHITE,
                texture: chest_closed_texture,
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

    spawn_trees(world, database);
    spawner::spawn_creature_spawner(world);
    world.spawn((create_map(),));
}

fn spawn_trees(world: &mut World, database: &Database) {
    let tree_top = database
        .textures
        .get(TREE_TOP_TEXTURE)
        .expect("tree top texture not found")
        .clone();
    let tree_base = database
        .textures
        .get(TREE_BASE_TEXTURE)
        .expect("tree base texture not found")
        .clone();

    for y in 0..MAP_HEIGHT {
        for x in 0..MAP_WIDTH {
            let x = x as i32;
            let y = y as i32;
            if x * x + y * y <= CLEARING_RADIUS * CLEARING_RADIUS
                || macroquad::rand::gen_range(0, 100) >= TREE_DENSITY_PERCENT
            {
                continue;
            }

            let position = cell_to_world(x, y);
            let base_offset = vec2((TILE_SIZE - tree_base.width()) * 0.5, (tree_base.height() / 2.0) - 10.0);
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
                    texture: tree_base.clone(),
                    offset: base_offset,
                },
                SpriteOverlay {
                    sprite: Sprite {
                        color: color::WHITE,
                        texture: tree_top.clone(),
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
        }
    }
}
