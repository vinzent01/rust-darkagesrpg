use hecs::World;
use macroquad::prelude::*;
use crate::body::Appearence;
use crate::utils::{self, TILE_SIZE};
use crate::player::Player;
use crate::tilemap::{MAP_HEIGHT, MAP_WIDTH};
use crate::objects::Solid;

pub struct Position {
    pub value : Vec2,
    pub offset : Vec2
}

pub struct BezierMovement{
    pub p0 : Vec2,
    pub p1 : Vec2,
    pub p2 : Vec2,
    pub t : f32,
    pub has_started :bool,
    pub speed : f32
}

pub struct RandomWalker {
}

pub fn random_walking(world: &World) {
    let blocked_positions: Vec<Vec2> = world
        .query::<(&Position, &Solid)>()
        .iter()
        .map(|(position, _)| position.value)
        .collect();
    let mut occupied_positions: Vec<Vec2> = world
        .query::<(&Position, &RandomWalker)>()
        .iter()
        .map(|(position, _)| (position.value / TILE_SIZE).round() * TILE_SIZE)
        .collect();
    for (position, movement, _) in world
        .query::<(&Position, &BezierMovement, &Player)>()
        .iter()
    {
        occupied_positions.push((position.value / TILE_SIZE).round() * TILE_SIZE);
        if movement.has_started {
            occupied_positions.push(movement.p2);
        }
    }

    for (_, bezier, pos) in world
        .query::<(&mut RandomWalker, &mut BezierMovement, &Position)>()
        .iter()
    {
        let current_cell = pos.value / TILE_SIZE;
        let options: Vec<Vec2> = [
            vec2(-1.0, 0.0),
            vec2(1.0, 0.0),
            vec2(0.0, -1.0),
            vec2(0.0, 1.0),
        ]
        .into_iter()
        .map(|direction| current_cell + direction)
        .filter(|cell| {
            cell.x >= 0.0
                && cell.y >= 0.0
                && cell.x < MAP_WIDTH as f32
                && cell.y < MAP_HEIGHT as f32
                && !blocked_positions.contains(&(*cell * TILE_SIZE))
                && !occupied_positions.contains(&(*cell * TILE_SIZE))
        })
        .collect();

        if options.is_empty() {
            continue;
        }

        let destination = options[macroquad::rand::gen_range(0, options.len() as i32) as usize]
            * TILE_SIZE;
        occupied_positions.push(destination);

        bezier.p0 = pos.value;
        bezier.p1 = (pos.value + destination) * 0.5 + vec2(0.0, -16.0);
        bezier.p2 = destination;
        bezier.t = 0.0;
        bezier.has_started = true;
    }
}

pub fn entities_are_moving(world: &World) -> bool {
    world
        .query::<&BezierMovement>()
        .iter()
        .any(|movement| movement.has_started)
}


pub fn bezier_movement(world: &World, dt : f32) {
    for ( bezier, pos) in world.query::<(&mut BezierMovement, &mut Position)>().iter() {
        if bezier.has_started == false{
            continue;
        }

        bezier.t += dt * bezier.speed;

        let movement = utils::bezier(bezier.p0, bezier.p1, bezier.p2, bezier.t);
        pos.value = movement;


        if bezier.t >= 1.0{
            bezier.has_started = false;
            bezier.t = 0.0;

            pos.value = bezier.p2;
        }
    }
}

pub fn camera_follow(world : &World,camera : &mut Camera2D){

    for (_, position) in world.query::<(&Player, &Position)>().iter(){
        camera.target = position.value;
    }
}

pub fn flip_system(world: &World) {
    for (body, bezier) in world
        .query::<(&mut Appearence, &BezierMovement)>()
        .iter()
    {
        let dir = bezier.p2.x - bezier.p0.x;

        if dir < 0.0 {
            body.is_flipped = true;
        } else if dir > 0.0 {
            body.is_flipped = false;
        }
    }
}