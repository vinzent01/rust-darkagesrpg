use hecs::World;
use macroquad::math::{Rect, vec2};

use crate::inventory::{
    Equipment, Inventory, ItemRef, INVENTORY_SLOT_COUNT, MAX_INVENTORY_WEIGHT_KG,
};
use crate::movement::{AttackAction, Position, BezierMovement, MOVEMENT_SPEED};
use crate::body::{Appearence, Body, get_default_hair_color, get_default_skin_color};
use crate::health::{Health, MAX_HEALTH};
use crate::input::Input;
use crate::tilemap::MAP_CENTER_CELL;
use crate::utils::cell_to_world;

pub struct Player {}

pub fn default_appearance() -> Appearence {
    Appearence {
        is_flipped: false,
        body: Body {
            genre: true,
            race: "human",
        },
        beard: None,
        hair: Some(0),
        skin_color: get_default_skin_color(),
        hair_color: get_default_hair_color(),
    }
}

pub fn spawn_player(world: &mut World, appearance: Appearence) {
    let mut items = vec![None; INVENTORY_SLOT_COUNT];
    
    world.spawn((
        Player{},
        Health::new(MAX_HEALTH),
        Position {
            value : cell_to_world(MAP_CENTER_CELL.0, MAP_CENTER_CELL.1),
            offset : vec2(0.0, -32.0)
        },
        BezierMovement {
            p0 : cell_to_world(MAP_CENTER_CELL.0, MAP_CENTER_CELL.1),
            p1 : cell_to_world(MAP_CENTER_CELL.0, MAP_CENTER_CELL.1),
            p2 : cell_to_world(MAP_CENTER_CELL.0, MAP_CENTER_CELL.1),
            t : 0.0,
            has_started : false,
            speed : MOVEMENT_SPEED,
        },
        AttackAction {
            origin: vec2(0.0, 0.0),
            target: vec2(0.0, 0.0),
            target_entity: None,
            path: Vec::new(),
            returning: false,
            striking: false,
            active: false,
        },
        appearance,
        Input,
        Equipment::default(),
        Inventory {
            items,
            weight: 4.0,
            max_weight : MAX_INVENTORY_WEIGHT_KG,
            rect : Rect{
                x : 0.0,
                y : 0.0,
                w : 100.0,
                h : 100.0
            }
        }
    ));
}