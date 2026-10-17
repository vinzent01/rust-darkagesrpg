use std::{collections::HashMap, vec};

use hecs::World;
use macroquad::prelude::*;

pub type TileId = u16;

pub const MAP_WIDTH: usize = 100;
pub const MAP_HEIGHT: usize = 100;
pub const MAP_CENTER_CELL: (i32, i32) = ((MAP_WIDTH / 2) as i32, (MAP_HEIGHT / 2) as i32);

pub struct TileDefinition {
    pub name: String,
    pub texture_path: &'static str,
    pub solid: bool,
}

pub struct TileRegistry {
    pub tiles: HashMap<TileId, TileDefinition>,
}

pub struct TileLayer {
    pub name: String,
    pub tiles: Vec<TileId>,
    pub width: usize,
    pub height: usize,
    pub depth: i32, // ordem de renderização
}

pub struct TileMap {
    pub layers: Vec<TileLayer>,
}

impl TileMap {
    pub fn new() -> Self{

        Self {
            layers : vec![]
        }
    }
}

impl TileLayer {
    pub fn get(&self, x: usize, y: usize) -> TileId {
        self.tiles[y * self.width + x]
    }

    pub fn set(&mut self, x: usize, y: usize, id: TileId) {
        self.tiles[y * self.width + x] = id;
    }
}


pub fn create_map() -> TileMap {
    let width = MAP_WIDTH;
    let height = MAP_HEIGHT;

    let mut ground_tiles = vec![1; width * height]; // tudo grass
    let mut ground2_tiles = vec![2; width * height];

    // exemplo: uma faixa de grass
    for x in 20..40 {
        for y in 20..40 {
            ground_tiles[y * width + x] = 1;
        }
    }

    for x in 20..40 {
        for y in 20..40 {
            ground2_tiles[y * width + x] = 2;
        }
    }

    let ground_layer = TileLayer {
        name: "Ground".into(),
        width,
        height,
        depth: 0,
        tiles: ground_tiles,
    };

    let ground2_layer = TileLayer {
        name: "Ground 2".into(),
        width,
        height,
        depth: 0,
        tiles: ground2_tiles,
    };

    TileMap {
        layers: vec![ground_layer, ground2_layer],
    }
}