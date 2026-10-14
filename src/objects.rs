use hecs::World;
use macroquad::prelude::*;

use crate::{body::Sprite, movement::Position};
use crate::utils;

pub struct OnOffSprite{
    pub on : Sprite,
    pub off : Sprite,
    pub is_on : bool
}

pub struct Solid;

pub struct Chest{

}

pub fn interact_chest(world: &mut World, camera: &mut Camera2D) {
    if !is_mouse_button_pressed(MouseButton::Left) {
        return;
    }

    let mouse = mouse_position();
    let mouse_world = camera.screen_to_world(vec2(mouse.0, mouse.1));

    let tile = vec2(
        (mouse_world.x / utils::TILE_SIZE).floor(),
        (mouse_world.y / utils::TILE_SIZE).floor(),
    );

    for (chest, position, on_offs) in
        world.query_mut::<(&Chest, &Position, &mut OnOffSprite)>()
    {
        let pos_tile = vec2(
            (position.value.x / utils::TILE_SIZE).floor(),
            (position.value.y / utils::TILE_SIZE).floor(),
        );

        if pos_tile == tile {
            on_offs.is_on = !on_offs.is_on;
        }
    }
}