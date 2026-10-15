use hecs::World;
use macroquad::prelude::*;

use crate::database::Database;
use crate::chest_inventory;
use crate::hud;
use crate::input;
use crate::inventory;
use crate::menu::{self, Menu};
use crate::movement;
use crate::objects;
use crate::render;
use crate::spawner;
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
    dt: f32,
) {
    menu::update(menu);
    let chest_open = chest_inventory::update(world, database);
    let inventory_open = hud::update_ui(world, database);
    let ground_stack_open = inventory::update_ground_stack_ui(world, database, camera);
    let ui_open = chest_open || inventory_open || ground_stack_open;

    if !ui_open {
        input::camera_zoom(camera, zoom, dt);
    }

    let picked_up_item = !ui_open && inventory::pick_up_ground_item(world, database, camera);
    if !ui_open && !picked_up_item {
        objects::interact_chest(world, camera);
    }

    if !ui_open && *turn_phase == TurnPhase::Ready && input::movement_input(world) {
        spawner::update_creature_spawners(world, database, camera);
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

pub fn draw_world(world: &World, database: &Database, camera: &Camera2D) {
    clear_background(BLACK);
    render::draw_tilemap(world, database, camera);
    utils::draw_grid();
    render::draw_entities(world, database, camera);
    inventory::draw_ground_items(world, database);
}

pub fn draw_hud(world: &World, database: &Database, menu: &Menu) {
    hud::draw_inventory_ui(&world, database);
    chest_inventory::draw(world, database);
    inventory::draw_ground_stack_ui(world, database);
    menu::draw(menu);
}

pub fn resize_camera(camera: &mut Camera2D, zoom: f32) {
    camera.zoom = vec2(zoom / screen_width(), zoom / screen_height());
}

pub fn create_camera(zoom: f32) -> Camera2D {
    Camera2D {
        target: vec2(0.0, 0.0),
        zoom: vec2(zoom / screen_width(), zoom / screen_height()),
        ..Default::default()
    }
}
