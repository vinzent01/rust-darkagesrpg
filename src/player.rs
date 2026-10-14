use hecs::World;
use macroquad::math::{Rect, vec2};

use crate::inventory::Inventory;
use crate::movement::{Position, BezierMovement};
use crate::body::{Body, Appearence};
use crate::input::Input;

pub struct Player {}

pub fn spawn_player(world :&mut World){
    
    world.spawn((
        Player{},
        Position {
            value : vec2(0.0, 0.0),
            offset : vec2(0.0, -32.0)
        },
        BezierMovement {
            p0 : vec2(0.0, 0.0),
            p1 : vec2(0.0, 0.0),
            p2 : vec2(0.0, 0.0),
            t : 0.0,
            has_started : false,
            speed : 3.0,
        },
        Appearence {
            is_flipped : false,
            body : Body{
                genre : true,
                race : "human"
            },
            beard : None,
            hair : Some(0)
        },
        Input,
        Inventory {
            items : vec![],
            weight : 0.0,
            max_weight : 20.0,
            rect : Rect{
                x : 0.0,
                y : 0.0,
                w : 100.0,
                h : 100.0
            }
        }
    ));
}