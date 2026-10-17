use hecs::World;
use macroquad::{color, prelude::*};

use crate::body::Sprite;
use crate::database::Database;
use crate::health::Health;
use crate::movement::{AttackAction, BezierMovement, Position, RandomWalker, MOVEMENT_SPEED};
use crate::player::Player;
use crate::tilemap::{MAP_HEIGHT, MAP_WIDTH};
use crate::utils::{cell_to_world, TILE_SIZE};

const CREATURE_SPAWN_INTERVAL_TURNS: u32 = 10;
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
    for _ in 0..database.creatures.len() {
        if available_cells.is_empty() {
            break;
        }

        let cell_index = macroquad::rand::gen_range(0, available_cells.len() as i32) as usize;
        let cell = available_cells.swap_remove(cell_index);
        let creature_id = random_creature_id(database);
        let position = cell_to_world(cell.x as i32, cell.y as i32);
        spawn_creature(world, database, creature_id, position, vec2(0.0, -32.0), Vec2::ZERO)
            .expect("registered creature failed to spawn");
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
        let creature_id = random_creature_id(database);
        let position = cell_to_world(cell.x as i32, cell.y as i32);
        spawn_creature(world, database, creature_id, position, vec2(0.0, -32.0), Vec2::ZERO)
            .expect("registered creature failed to spawn");
    }
}

fn random_creature_id(database: &Database) -> &str {
    let total_weight: u32 = database.creatures.values().map(|creature| creature.spawn_weight).sum();
    let mut choice = macroquad::rand::gen_range(0, total_weight as i32) as u32;

    for creature in database.creatures.values() {
        if choice < creature.spawn_weight {
            return &creature.id;
        }
        choice -= creature.spawn_weight;
    }

    unreachable!("creature spawn weights must have a non-zero total")
}

pub fn spawn_creature(
    world: &mut World,
    database: &Database,
    creature_id: &str,
    position: Vec2,
    position_offset: Vec2,
    sprite_offset: Vec2,
) -> Result<hecs::Entity, String> {
    let creature = database
        .creatures
        .get(creature_id)
        .ok_or_else(|| format!("unknown creature id: {creature_id}"))?;
    if !database.textures.contains_key(&creature.texture_path) {
        return Err(format!(
            "texture not loaded for creature: {creature_id}"
        ));
    }

    Ok(world.spawn((
        Health::new(creature.maximum_health),
        Position {
            value: position,
            offset: position_offset,
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
            texture_path: creature.texture_path.clone(),
            offset: sprite_offset,
        },
    )))
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
    for (attack, _) in world.query::<(&AttackAction, &Player)>().iter() {
        occupied_cells.extend(
            attack
                .path
                .iter()
                .map(|position| (*position / TILE_SIZE).round()),
        );
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
