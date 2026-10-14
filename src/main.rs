use hecs::World;
use macroquad::{color, prelude::*};

mod input;
mod movement;
mod utils;
mod render;
mod tilemap;
mod database;
mod player;
mod body;
mod objects;
mod hud;
mod inventory;

use crate::hud::{UIInventory};
use crate::movement::{BezierMovement, Position, RandomWalker};
use crate::objects::{Chest, OnOffSprite, Solid};
use crate::body::{Sprite, SpriteOverlay};
use crate::player::Player;
use crate::tilemap::{create_map, MAP_HEIGHT, MAP_WIDTH};
use crate::database::Database;
use crate::utils::{cell_to_world, TILE_SIZE};

const CREATURE_SPAWN_INTERVAL_TURNS: u32 = 5;
const CREATURE_TEXTURES: [&str; 6] = [
    "./content/creatures/bear.png",
    "./content/creatures/deer.png",
    "./content/creatures/griffon.png",
    "./content/creatures/rabbit.png",
    "./content/creatures/white-horse.png",
    "./content/creatures/wolf.png",
];

struct WindowState {
    width: f32,
    height: f32,
}

struct CreatureSpawner {
    turns_until_spawn: u32,
}

async fn spawn_entities(world : &mut World, database: &Database){
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

    world.spawn((
        Position {
            value: cell_to_world(3, 2),
            offset: vec2(0.0, -32.0),
        },
        BezierMovement {
            p0: cell_to_world(3, 2),
            p1: cell_to_world(3, 2),
            p2: cell_to_world(3, 2),
            t: 0.0,
            has_started: false,
            speed: 1.5,
        },
        RandomWalker {},
        Sprite {
            color: color::WHITE,
            texture: horse_texture,
            offset: vec2(0.0, 0.0),
        },
        SpriteOverlay {
            sprite: Sprite {
                color: color::WHITE,
                texture: saddle_texture,
                offset: vec2(29.0, 47.0),
            },
        },
    ));

    let chest_open_texture = load_texture("./content/chests/chest-open.png").await.unwrap();
    chest_open_texture.set_filter(FilterMode::Nearest);

    let chest_closed_texture = load_texture("./content/chests/chest-closed.png").await.unwrap();
    chest_closed_texture.set_filter(FilterMode::Nearest);

    world.spawn((
        Position{
            value : cell_to_world(0, 0),
            offset : vec2(0.0, -10.0)
        },
        OnOffSprite {
            on : Sprite { 
                color: color::WHITE, 
                texture : chest_open_texture,
                offset: vec2(0.0, 0.0)
            },
            off : Sprite { 
                color: color::WHITE, 
                texture : chest_closed_texture,
                offset: vec2(0.0, 0.0)
            },
            is_on : false
        },
        Chest{},
        Solid{},
    ));


    world.spawn((CreatureSpawner {
        turns_until_spawn: CREATURE_SPAWN_INTERVAL_TURNS,
    },));

    world.spawn((
        create_map(),
    ));
}

fn spawn_random_creatures(world: &mut World, database: &Database) {
    let mut available_cells = available_creature_spawn_cells(world);
    for texture_path in CREATURE_TEXTURES {
        if available_cells.is_empty() {
            break;
        }

        let cell_index = macroquad::rand::gen_range(0, available_cells.len() as i32) as usize;
        let cell = available_cells.swap_remove(cell_index);
        spawn_creature_at(world, database, texture_path, cell);
    }
}

fn available_creature_spawn_cells(world: &World) -> Vec<Vec2> {
    const SPAWN_RADIUS: i32 = 10;
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
        })
        .collect()
}

fn spawn_creature_at(
    world: &mut World,
    database: &Database,
    texture_path: &str,
    cell: Vec2,
) {
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
            speed: 1.5,
        },
        RandomWalker {},
        Sprite {
            color: color::WHITE,
            texture,
            offset: vec2(0.0, 0.0),
        },
    ));
}

fn update_creature_spawners(world: &mut World, database: &Database) {
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
        let mut available_cells = available_creature_spawn_cells(world);
        if available_cells.is_empty() {
            continue;
        }

        let cell_index = macroquad::rand::gen_range(0, available_cells.len() as i32) as usize;
        let cell = available_cells.swap_remove(cell_index);
        let texture_index = macroquad::rand::gen_range(0, CREATURE_TEXTURES.len() as i32) as usize;
        spawn_creature_at(world, database, CREATURE_TEXTURES[texture_index], cell);
    }
}

#[derive(PartialEq, Eq)]
enum TurnPhase {
    Ready,
    ActorsMoving,
}

fn update(
    world: &mut World,
    database: &Database,
    camera: &mut Camera2D,
    zoom: &mut f32,
    turn_phase: &mut TurnPhase,
    dt: f32,
) {
    // update
    hud::update_ui(world);
    input::camera_zoom(camera, zoom, dt);
    objects::interact_chest(world, camera);

    if *turn_phase == TurnPhase::Ready && input::movement_input(world) {
        update_creature_spawners(world, database);
        movement::random_walking(world);
        *turn_phase = TurnPhase::ActorsMoving;
    }

    if *turn_phase == TurnPhase::Ready {
        return;
    }

    movement::bezier_movement(world, dt);
    movement::flip_system(world);
    movement::camera_follow(world, camera);

    if *turn_phase == TurnPhase::ActorsMoving && !movement::entities_are_moving(world) {
        *turn_phase = TurnPhase::Ready;
    }
}

fn draw(
    world : &mut World,
    database : & Database,
    camera : &Camera2D,
){
    clear_background(BLACK);

    render::draw_tilemap(world, database, camera);
    utils::draw_grid();
    
    // entities
    render::draw_on_off(world, database);
    render::draw_entities(world, database);
}


fn draw_hud(
    world : &mut World,
    database : & Database
){
    hud::draw_inventory_ui(world, database);
}

fn on_window_resized(camera : &mut Camera2D, zoom : &mut f32){
    camera.zoom = vec2(
        *zoom / screen_width(),
        *zoom / screen_height(),
    );
}

#[macroquad::main("Bezier Grid Movement")]
async fn main() {
    let mut world = World::new();
    let mut database = Database::new();
    let mut zoom = 4.0;
    let mut turn_phase = TurnPhase::Ready;

    database.load().await;
    
    let mut camera = Camera2D {
        target: vec2(0.0, 0.0),
        zoom: vec2(
            zoom / screen_width(),
            zoom / screen_height(),
        ),
        ..Default::default()
    };

    let mut window = WindowState {
        width: screen_width(),
        height: screen_height(),
    };

    // start systems
    spawn_entities(&mut world, &database).await;

    loop {
        let dt = get_frame_time();

        let new_width = screen_width();
        let new_height = screen_height();

        if new_width != window.width || new_height != window.height {
            window.width = new_width;
            window.height = new_height;

            on_window_resized(&mut camera, &mut zoom);
        }

        update(
            &mut world,
            &database,
            &mut camera,
            &mut zoom,
            &mut turn_phase,
            dt,
        );

        // draw
        set_camera(&camera);

        draw(
            &mut world,
            &database,
            &camera
        );

        set_default_camera();

        draw_hud(&mut world, &database);

        draw_text(
            "WASD ou Setas para mover",
            10.0,
            20.0,
            24.0,
            WHITE,
        );

        
        draw_text(
            &format!("FPS {}", get_fps()),
            10.0,
            30.0,
            24.0,
            WHITE,
        );

        next_frame().await;
    }
}