use hecs::World;
use macroquad::prelude::*;
use crate::objects::Solid;
use crate::utils;
use crate::movement::{BezierMovement, Position, RandomWalker};
use crate::tilemap::{MAP_HEIGHT, MAP_WIDTH};

pub struct Input;

pub fn is_blocked(world: &World, position: Vec2) -> bool {
    let cell = position / utils::TILE_SIZE;
    if cell.x < 0.0
        || cell.y < 0.0
        || cell.x >= MAP_WIDTH as f32
        || cell.y >= MAP_HEIGHT as f32
    {
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
    let direction = if is_key_pressed(KeyCode::A) || is_key_pressed(KeyCode::Left) {
        vec2(-1.0, 0.0)
    } else if is_key_pressed(KeyCode::D) || is_key_pressed(KeyCode::Right) {
        vec2(1.0, 0.0)
    } else if is_key_pressed(KeyCode::W) || is_key_pressed(KeyCode::Up) {
        vec2(0.0, -1.0)
    } else if is_key_pressed(KeyCode::S) || is_key_pressed(KeyCode::Down) {
        vec2(0.0, 1.0)
    } else {
        return false;
    };

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


pub fn camera_zoom(camera : &mut Camera2D, zoom : &mut f32, dt : f32){
    let (_, scroll) = mouse_wheel();
    const ZOOM_FACTOR: f32 = 1.1;

    if scroll != 0.0 {
        *zoom *= ZOOM_FACTOR.powf(scroll*0.01);
        *zoom = zoom.clamp(0.5, 8.0);

        camera.zoom = vec2(
            *zoom / screen_width(),
            *zoom / screen_height(),
        );
    }
}
