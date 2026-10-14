use macroquad::math::Rect;

use crate::database::Database;

pub struct ItemRef {
    pub id : String,
    pub quantity : i32
}

pub struct Inventory{
    pub items : Vec<ItemRef>,
    pub weight : f32,
    pub max_weight : f32,
    pub rect : Rect
}

pub struct Item{
    pub id : String,
    pub name : String,
    pub weight : f32,
}

pub fn get_item<'a>(item_ref: &ItemRef, database: &'a Database) -> Option<&'a Item> {
    database.items.get(&item_ref.id)
}