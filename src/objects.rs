use hecs::{Entity, World};
use macroquad::prelude::*;

use crate::chest_inventory;
use crate::{body::Sprite, movement::Position};
use crate::utils;

pub struct OnOffSprite{
    pub on : Sprite,
    pub off : Sprite,
    pub is_on : bool
}

pub struct Solid;

pub struct Tree;

pub struct Chest{

}

pub fn interact_chest(world: &mut World, camera: &Camera2D) {
    if !is_mouse_button_pressed(MouseButton::Left) {
        return;
    }

    let mouse = mouse_position();
    let mouse_world = camera.screen_to_world(vec2(mouse.0, mouse.1));

    let tile = vec2(
        (mouse_world.x / utils::TILE_SIZE).floor(),
        (mouse_world.y / utils::TILE_SIZE).floor(),
    );

    let chest_entity = world
        .query::<(Entity, &Chest, &Position)>()
        .iter()
        .find(|(_, _, position)| {
            let pos_tile = vec2(
                (position.value.x / utils::TILE_SIZE).floor(),
                (position.value.y / utils::TILE_SIZE).floor(),
            );
            pos_tile == tile
        })
        .map(|(entity, _, _)| entity);

    let Some(chest_entity) = chest_entity else {
        return;
    };

    if let Ok(mut sprite) = world.get::<&mut OnOffSprite>(chest_entity) {
        sprite.is_on = true;
    }
    chest_inventory::open(world, chest_entity);
}