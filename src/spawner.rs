use hecs::World;
use macroquad::{color, prelude::*};

use crate::body::Sprite;
use crate::database::Database;
use crate::movement::{BezierMovement, Position, RandomWalker, MOVEMENT_SPEED};
use crate::player::Player;
use crate::tilemap::{MAP_HEIGHT, MAP_WIDTH};
use crate::utils::{cell_to_world, TILE_SIZE};

const CREATURE_SPAWN_INTERVAL_TURNS: u32 = 10;
const CREATURE_TEXTURES: [&str; 7] = [
    "./content/creatures/bear.png",
    "./content/creatures/deer.png",
    "./content/creatures/griffon.png",
    "./content/creatures/rabbit.png",
    "./content/creatures/white-horse.png",
    "./content/creatures/horse.png",
    "./content/creatures/wolf.png",
];
const CREATURE_SPAWN_WEIGHTS: [u32; 7] = [2, 6, 1, 8, 6, 6, 8];
const CAMERA_SPAWN_MARGIN: f32 = TILE_SIZE * 4.0;

pub struct CreatureSpawner {
    turns_until_spawn: u32,
}

pub fn spawn_creature_spawner(world: &mut World) {
    world.spawn((CreatureSpawner {
        turns_until_spawn: CREATURE_SPAWN_INTERVAL_TURNS,
    },));
}

pub fn spawn_random_creatures(world: &mut World, database: &Database, camera: &Camera2D) {
    let mut available_cells = available_creature_spawn_cells(world, camera);
    for _ in 0..CREATURE_TEXTURES.len() {
        if available_cells.is_empty() {
            break;
        }

        let cell_index = macroquad::rand::gen_range(0, available_cells.len() as i32) as usize;
        let cell = available_cells.swap_remove(cell_index);
        spawn_creature_at(world, database, random_creature_texture(), cell);
    }
}

pub fn update_creature_spawners(world: &mut World, database: &Database, camera: &Camera2D) {
    let spawn_count = {
        let mut spawn_count = 0;
        for spawner in world.query_mut::<&mut CreatureSpawner>() {
            spawner.turns_until_spawn -= 1;
            if spawner.turns_until_spawn == 0 {
                spawner.turns_until_spawn = CREATURE_SPAWN_INTERVAL_TURNS;
                spawn_count += 1;
            }
        }
        spawn_count
    };

    for _ in 0..spawn_count {
        let mut available_cells = available_creature_spawn_cells(world, camera);
        if available_cells.is_empty() {
            continue;
        }

        let cell_index = macroquad::rand::gen_range(0, available_cells.len() as i32) as usize;
        let cell = available_cells.swap_remove(cell_index);
        spawn_creature_at(world, database, random_creature_texture(), cell);
    }
}

fn random_creature_texture() -> &'static str {
    let total_weight: u32 = CREATURE_SPAWN_WEIGHTS.iter().sum();
    let mut choice = macroquad::rand::gen_range(0, total_weight as i32) as u32;

    for (texture_path, weight) in CREATURE_TEXTURES.iter().zip(CREATURE_SPAWN_WEIGHTS) {
        if choice < weight {
            return texture_path;
        }
        choice -= weight;
    }

    unreachable!("creature spawn weights must have a non-zero total")
}

fn available_creature_spawn_cells(world: &World, camera: &Camera2D) -> Vec<Vec2> {
    const SPAWN_RADIUS: i32 = 10;
    let view_corners = [
        camera.screen_to_world(vec2(0.0, 0.0)),
        camera.screen_to_world(vec2(screen_width(), 0.0)),
        camera.screen_to_world(vec2(0.0, screen_height())),
        camera.screen_to_world(vec2(screen_width(), screen_height())),
    ];
    let view_min = view_corners.iter().copied().reduce(Vec2::min).unwrap();
    let view_max = view_corners.iter().copied().reduce(Vec2::max).unwrap();
    let player_position = world
        .query::<(&Player, &Position, &BezierMovement)>()
        .iter()
        .next()
        .map(|(_, _, movement)| {
            if movement.has_started {
                movement.p2
            } else {
                movement.p0
            }
        })
        .expect("player must be spawned before creatures");
    let player_cell = (player_position / TILE_SIZE).round();
    let mut occupied_cells: Vec<Vec2> = world
        .query::<&Position>()
        .iter()
        .map(|position| (position.value / TILE_SIZE).round())
        .collect();
    if let Some(player_destination) = world
        .query::<(&Player, &BezierMovement)>()
        .iter()
        .next()
        .and_then(|(_, movement)| movement.has_started.then_some(movement.p2))
    {
        occupied_cells.push((player_destination / TILE_SIZE).round());
    }

    (-SPAWN_RADIUS..=SPAWN_RADIUS)
        .flat_map(|x| (-SPAWN_RADIUS..=SPAWN_RADIUS).map(move |y| (x, y)))
        .filter(|(x, y)| x * x + y * y <= SPAWN_RADIUS * SPAWN_RADIUS)
        .map(|(x, y)| player_cell + vec2(x as f32, y as f32))
        .filter(|cell| {
            cell.x >= 0.0
                && cell.y >= 0.0
                && cell.x < MAP_WIDTH as f32
                && cell.y < MAP_HEIGHT as f32
                && !occupied_cells.contains(cell)
                && {
                    let world_position = *cell * TILE_SIZE;
                    world_position.x < view_min.x - CAMERA_SPAWN_MARGIN
                        || world_position.x > view_max.x + CAMERA_SPAWN_MARGIN
                        || world_position.y < view_min.y - CAMERA_SPAWN_MARGIN
                        || world_position.y > view_max.y + CAMERA_SPAWN_MARGIN
                }
        })
        .collect()
}

fn spawn_creature_at(world: &mut World, database: &Database, texture_path: &str, cell: Vec2) {
    let position = cell_to_world(cell.x as i32, cell.y as i32);
    let texture = database
        .textures
        .get(texture_path)
        .expect("creature texture not found")
        .clone();

    world.spawn((
        Position {
            value: position,
            offset: vec2(0.0, -32.0),
        },
        BezierMovement {
            p0: position,
            p1: position,
            p2: position,
            t: 0.0,
            has_started: false,
            speed: MOVEMENT_SPEED,
        },
        RandomWalker {},
        Sprite {
            color: color::WHITE,
            texture,
            offset: vec2(0.0, 0.0),
        },
    ));
}
