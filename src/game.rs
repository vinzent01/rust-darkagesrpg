use hecs::World;
use macroquad::prelude::*;

use crate::chest_inventory;
use crate::character_creator;
use crate::crafting;
use crate::database::Database;
use crate::hud;
use crate::input;
use crate::inventory;
use crate::menu::{self, Menu};
use crate::movement;
use crate::objects::{self, GameSounds};
use crate::render;
use crate::spawner;
use crate::tilemap::MAP_CENTER_CELL;
use crate::utils;

mod scene;
pub use scene::spawn_entities;

#[derive(PartialEq, Eq)]
pub enum TurnPhase {
    Ready,
    ActorsMoving,
}

pub fn update(
    world: &mut World,
    database: &Database,
    camera: &mut Camera2D,
    zoom: &mut f32,
    turn_phase: &mut TurnPhase,
    menu: &mut Menu,
    game_sounds: &GameSounds,
    dt: f32,
) {
    menu::update(menu);
    let chest_open = chest_inventory::update(world, database);
    let inventory_open = hud::update_ui(world, database);
    let ground_stack_open = inventory::update_ground_stack_ui(world, database, camera);
    let crafting_open = crafting::update(world, database);
    let character_creator_open = character_creator::update(world, database);
    let ui_open = chest_open
        || inventory_open
        || ground_stack_open
        || crafting_open
        || character_creator_open;

    if !ui_open {
        input::camera_zoom(camera, zoom, dt);
    }

    let attack_mode = input::attack_mode_active();
    objects::update_tree_falls(world, dt);
    let attack_target = (!ui_open && *turn_phase == TurnPhase::Ready)
        .then(|| input::attack_input(world, database, camera))
        .flatten();

    if attack_target.is_some() {
        spawner::update_creature_spawners(world, database, camera);
        movement::random_walking(world);
        *turn_phase = TurnPhase::ActorsMoving;
    } else if !ui_open && !attack_mode {
        let picked_up_item = inventory::pick_up_ground_item(world, database, camera);
        if !picked_up_item {
            let gathered_item = input::gather_input(world, database, camera);
            if !gathered_item {
                objects::interact_chest(world, camera);
            }
        }
    }

    if !ui_open && !attack_mode && *turn_phase == TurnPhase::Ready && input::movement_input(world) {
        spawner::update_creature_spawners(world, database, camera);
        movement::random_walking(world);
        *turn_phase = TurnPhase::ActorsMoving;
    }

    if *turn_phase == TurnPhase::Ready {
        return;
    }

    movement::bezier_movement(world, dt, &game_sounds.footstep);
    movement::advance_attack_actions(world, database, Some(game_sounds));
    movement::flip_system(world);
    movement::camera_follow(world, camera);

    if *turn_phase == TurnPhase::ActorsMoving && !movement::entities_are_moving(world) {
        *turn_phase = TurnPhase::Ready;
    }
}

pub fn draw_world(
    world: &World,
    database: &Database,
    camera: &Camera2D,
    outline_material: &Material,
    show_sprite_hitboxes: bool,
    render_cache: &mut render::EntityRenderCache,
) {
    clear_background(BLACK);
    render::draw_tilemap(world, database, camera);
    render::draw_entities(
        world,
        database,
        camera,
        outline_material,
        show_sprite_hitboxes,
        render_cache,
    );
    inventory::draw_ground_items(world, database);
}

pub fn draw_hud(world: &World, database: &Database, menu: &Menu) {
    utils::draw_ui_text("WASD mover | P personagem | C crafting", 10.0, 24.0, 20.0, WHITE);
    utils::draw_ui_text(&format!("FPS {}", get_fps()), 10.0, 46.0, 20.0, WHITE);
    hud::draw_inventory_ui(&world, database);
    chest_inventory::draw(world, database);
    inventory::draw_ground_stack_ui(world, database);
    menu::draw(menu);
    if input::attack_mode_active() {
        let width = 220.0;
        let x = screen_width() - width - 12.0;
        draw_rectangle(x, 12.0, width, 32.0, Color::from_rgba(100, 35, 25, 230));
        draw_rectangle_lines(x, 12.0, width, 32.0, 1.0, ORANGE);
        utils::draw_ui_text("MODO DE ATAQUE", x + 12.0, 34.0, 18.0, WHITE);
    }
    crafting::draw(world, database);
    character_creator::draw(world, database);
}

pub fn resize_camera(camera: &mut Camera2D, zoom: f32) {
    camera.zoom = vec2(zoom / screen_width(), zoom / screen_height());
}

pub fn create_camera(zoom: f32) -> Camera2D {
    Camera2D {
        target: utils::cell_to_world(MAP_CENTER_CELL.0, MAP_CENTER_CELL.1),
        zoom: vec2(zoom / screen_width(), zoom / screen_height()),
        ..Default::default()
    }
}
