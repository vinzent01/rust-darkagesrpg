use hecs::World;
use macroquad::{color, prelude::*};

use crate::body::{Sprite, SpriteOverlay};
use crate::database::Database;
use crate::inventory::{INVENTORY_SLOT_COUNT, Inventory};
use crate::movement::Position;
use crate::objects::{Chest, Gatherable, OnOffSprite, Solid, Tree};
use crate::player;
use crate::spawner;
use crate::tilemap::{MAP_HEIGHT, MAP_WIDTH, create_map};
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
                    texture: saddle_texture,
                    offset: vec2(29.0, 47.0) + horse_sprite_offset,
                },
            },
        )
        .expect("initial horse disappeared before saddle was added");

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

    spawn_nature(world, database);
    spawner::spawn_creature_spawner(world);
    world.spawn((create_map(),));
}

fn spawn_nature(world: &mut World, database: &Database) {
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
    let bushes = BUSH_TEXTURES.map(|path| {
        database
            .textures
            .get(path)
            .unwrap_or_else(|| panic!("bush texture not found: {path}"))
            .clone()
    });
    let stone = database
        .textures
        .get(STONE_TEXTURE)
        .expect("stone texture not found")
        .clone();

    for y in 0..MAP_HEIGHT {
        for x in 0..MAP_WIDTH {
            let x = x as i32;
            let y = y as i32;
            if x * x + y * y <= CLEARING_RADIUS * CLEARING_RADIUS {
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
            } else if roll < TREE_DENSITY_PERCENT + BUSH_DENSITY_PERCENT {
                let texture = &bushes[macroquad::rand::gen_range(0, bushes.len())];
                spawn_gatherable(
                    world,
                    position,
                    texture.clone(),
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
                    stone.clone(),
                    "stone",
                    vec2(
                        (TILE_SIZE - stone.width()) * 0.5,
                        TILE_SIZE - stone.height(),
                    ),
                );
            }
        }
    }
}

fn spawn_gatherable(
    world: &mut World,
    position: Vec2,
    texture: Texture2D,
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
            texture,
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
