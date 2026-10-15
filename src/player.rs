use hecs::World;
use macroquad::math::{Rect, vec2};

use crate::inventory::{
    Equipment, Inventory, ItemRef, INVENTORY_SLOT_COUNT, MAX_INVENTORY_WEIGHT_KG,
};
use crate::movement::{Position, BezierMovement, MOVEMENT_SPEED};
use crate::body::{Body, Appearence};
use crate::input::Input;

pub struct Player {}

pub fn spawn_player(world :&mut World){
    let mut items = vec![None; INVENTORY_SLOT_COUNT];
    items[0] = Some(ItemRef {
        id: "stick".to_string(),
        quantity: 5,
    });
    items[1] = Some(ItemRef {
        id: "stone_axe".to_string(),
        quantity: 1,
    });
    items[2] = Some(ItemRef {
        id: "pickaxe".to_string(),
        quantity: 1,
    });
    items[3] = Some(ItemRef {
        id: "linen_tunic".to_string(),
        quantity: 1,
    });
    items[4] = Some(ItemRef {
        id: "linen_pants".to_string(),
        quantity: 1,
    });
    items[5] = Some(ItemRef {
        id: "leather_boots".to_string(),
        quantity: 1,
    });
    items[6] = Some(ItemRef {
        id: "sword".to_string(),
        quantity: 1,
    });
    items[7] = Some(ItemRef {
        id: "shield".to_string(),
        quantity: 1,
    });

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
            speed : MOVEMENT_SPEED,
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