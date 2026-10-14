use macroquad::prelude::*;
use hecs::World;

use crate::database::Database;
use crate::movement::Position;
use crate::objects::OnOffSprite;
use crate::tilemap::*;
use crate::utils;
use crate::body::{Appearence, Sprite, SpriteOverlay};


// systems
pub fn draw_entities(world: &World, database: &Database) {
    for (pos, sprite) in world.query::<(&Position, &Sprite)>().iter() {
        draw_texture(
            &sprite.texture,
            pos.value.x + pos.offset.x + sprite.offset.x,
            pos.value.y + pos.offset.y + sprite.offset.y,
            sprite.color,
        );
    }

    for (pos, overlay) in world.query::<(&Position, &SpriteOverlay)>().iter() {
        let sprite = &overlay.sprite;
        draw_texture(
            &sprite.texture,
            pos.value.x + pos.offset.x + sprite.offset.x,
            pos.value.y + pos.offset.y + sprite.offset.y,
            sprite.color,
        );
    }

    for (pos, appearence) in world.query::<(&Position, &Appearence)>().iter() {

        let race = database.races.get(appearence.body.race).expect("race not found");
        let genre = &appearence.body.genre;
        let body_sprite = if *genre{
            &race.male_sprite
        } else {
            &race.female_sprite
        };

        // draw body
        draw_texture_ex(
            &body_sprite.texture,
            pos.value.x + pos.offset.x,
            pos.value.y + pos.offset.y,
            race.default_skin_color,
            DrawTextureParams {flip_x : appearence.is_flipped, ..Default::default()}
        );
        
        if let Some(beard_id) = &appearence.beard{
            let beard = &database.hairs[*beard_id];
            draw_texture_ex(
            &beard.texture, 
            pos.value.x, pos.value.y, 
            beard.color,
            DrawTextureParams {flip_x : appearence.is_flipped, ..Default::default()}
        )};

        if let Some(hair_id) = &appearence.hair{
            let hair = &database.hairs[*hair_id];
            let body_width = body_sprite.texture.width();
            let hair_width = hair.texture.width();

            let hair_x = if appearence.is_flipped {
                pos.value.x + body_width - hair.offset.x - hair_width
            } else {
                pos.value.x + hair.offset.x
            };

            draw_texture_ex(
                &hair.texture,
                hair_x,
                pos.value.y + hair.offset.y,
                hair.color,
                DrawTextureParams {
                    flip_x: appearence.is_flipped,
                    ..Default::default()
                },
            );
        };
    }
}

pub fn draw_tilemap(world: &World, database : &Database, camera : &Camera2D) {
    for tilemap in world.query::<&TileMap>().iter() {
        let top_left = camera.screen_to_world(vec2(0.0, 0.0));
        let bottom_right = camera.screen_to_world(vec2(screen_width(), screen_height()));

        let left = top_left.x;
        let top = top_left.y;
        let right = bottom_right.x;
        let bottom = bottom_right.y;

        let tile_size = utils::TILE_SIZE;

        for layer in &tilemap.layers {
            let start_x = (left / tile_size).floor().max(0.0) as usize;
            let end_x = (right / tile_size).ceil().min(layer.width as f32) as usize;

            let start_y = (top / tile_size).floor().max(0.0) as usize;
            let end_y = (bottom / tile_size).ceil().min(layer.height as f32) as usize;

            for y in start_y..end_y {
                for x in start_x..end_x {
                    let index = y * layer.width + x;
                    let tile_id = layer.tiles[index];

                    if tile_id == 0 {
                        continue;
                    }

                    if let Some(tile) = database.tiles.get(&tile_id) {
                        draw_texture(
                            &tile.texture,
                            x as f32 * tile_size,
                            y as f32 * tile_size,
                            WHITE,
                        );
                    }
                }
            }
        }
    }
}


pub fn draw_on_off(world: &World, database: &Database) {
    for (pos, obj) in world.query::<(&Position, &OnOffSprite)>().iter() {
        if obj.is_on{
            draw_texture(&obj.on.texture, pos.value.x + pos.offset.x, pos.value.y + pos.offset.y, obj.on.color);
        }
        else {
            draw_texture(&obj.off.texture, pos.value.x + pos.offset.x, pos.value.y + pos.offset.y, obj.off.color);
        }
    }
}
