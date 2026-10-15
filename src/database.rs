use std::path::{Path, PathBuf};
use std::{fs,};
use std::{collections::HashMap, vec};
use macroquad::prelude::*;

use crate::body::{Race, Sprite, get_default_hair_color, get_default_skin_color};
use crate::inventory::{EquipmentSlot, Item, ItemEquipment};
use crate::tilemap::{TileDefinition, TileId};

pub struct Database {
    pub tiles : HashMap<TileId, TileDefinition>,
    pub hairs : Vec<Sprite>,
    pub beards : Vec<Sprite>,
    pub races : HashMap<String, Race>,
    pub textures : HashMap<String, Texture2D>,
    pub items : HashMap<String, Item>
}

impl Database {

    pub fn new() -> Self{
        Self { 
            tiles: HashMap::new(),
            beards : vec![],
            hairs : vec![],
            races: HashMap::new(),
            textures : HashMap::new(),
            items : HashMap::new()
        }
    }

    pub async fn load(&mut self){
        self.register_textures().await;
        self.register_tiles().await;
        self.register_hairs().await;
        self.register_races().await;
        self.register_items();
    }

    fn register_items(&mut self) {
        for (id, name, weight, texture_path, equipment) in [
            (
                "stick",
                "Graveto",
                0.1,
                "./content/crafting-items/stick.png",
                None,
            ),
            (
                "stone_axe",
                "Machado de pedra",
                1.5,
                "./content/melee weapons/stone-axe.png",
                Some(ItemEquipment {
                    slot: EquipmentSlot::MainHand,
                    male_texture_path: "./content/melee weapons/stone-axe.png".to_string(),
                    female_texture_path: "./content/melee weapons/stone-axe.png".to_string(),
                    offset: vec2(-8.0, 2.0),
                }),
            ),
            (
                "pickaxe",
                "Picareta",
                2.0,
                "./content/melee weapons/pickAxe.png",
                Some(ItemEquipment {
                    slot: EquipmentSlot::MainHand,
                    male_texture_path: "./content/melee weapons/pickAxe.png".to_string(),
                    female_texture_path: "./content/melee weapons/pickAxe.png".to_string(),
                    offset: vec2(-8.0, 2.0),
                }),
            ),
            (
                "sword",
                "Espada",
                1.5,
                "./content/melee weapons/sword.png",
                Some(ItemEquipment {
                    slot: EquipmentSlot::MainHand,
                    male_texture_path: "./content/melee weapons/sword.png".to_string(),
                    female_texture_path: "./content/melee weapons/sword.png".to_string(),
                    offset: vec2(-8.0, 2.0),
                }),
            ),
            (
                "shield",
                "Escudo",
                2.0,
                "./content/shield/shield.png",
                Some(ItemEquipment {
                    slot: EquipmentSlot::OffHand,
                    male_texture_path: "./content/shield/shield.png".to_string(),
                    female_texture_path: "./content/shield/shield.png".to_string(),
                    offset: vec2(20.0, 8.0),
                }),
            ),
            (
                "linen_tunic",
                "Tunica de linho",
                0.5,
                "./content/male clothes/male linen clothes/linen tunic/linen_tunic.png",
                Some(ItemEquipment {
                    slot: EquipmentSlot::Torso,
                    male_texture_path:
                        "./content/male clothes/male linen clothes/linen tunic/linen_tunic.png"
                            .to_string(),
                    female_texture_path:
                        "./content/female clothes/linen cothes/linen-tunic.png".to_string(),
                    offset: vec2(0.0, 0.0),
                }),
            ),
            (
                "linen_pants",
                "Calcas de linho",
                0.4,
                "./content/male clothes/male linen clothes/linen pants/linen_pants.png",
                Some(ItemEquipment {
                    slot: EquipmentSlot::Legs,
                    male_texture_path:
                        "./content/male clothes/male linen clothes/linen pants/linen_pants.png"
                            .to_string(),
                    female_texture_path:
                        "./content/female clothes/linen cothes/linen-pants.png".to_string(),
                    offset: vec2(0.0, 0.0),
                }),
            ),
            (
                "leather_boots",
                "Botas de couro",
                0.3,
                "./content/male clothes/leather armor/leather boots/leather_boots.png",
                Some(ItemEquipment {
                    slot: EquipmentSlot::Feet,
                    male_texture_path:
                        "./content/male clothes/leather armor/leather boots/leather_boots.png"
                            .to_string(),
                    female_texture_path:
                        "./content/female clothes/leather armor/leather-boots.png".to_string(),
                    offset: vec2(0.0, 0.0),
                }),
            ),
        ] {
            self.items.insert(
                id.to_string(),
                Item {
                    id: id.to_string(),
                    name: name.to_string(),
                    weight,
                    texture_path: texture_path.to_string(),
                    equipment,
                },
            );
        }
    }

    fn collect_files(&self, path: &Path, files: &mut Vec<PathBuf>) {
        let entries = fs::read_dir(path).unwrap();

        for entry in entries {
            let entry = entry.unwrap();
            let path = entry.path();

            if path.is_dir() {
                self.collect_files(&path, files);
            } else {
                files.push(path);
            }
        }
    }

    pub async fn register_textures(&mut self) {
        let mut files = Vec::new();

        self.collect_files(Path::new("./content"), &mut files);

        for path in files {
            let ext = path
                .extension()
                .and_then(|e| e.to_str())
                .unwrap_or("")
                .to_lowercase();

            if matches!(ext.as_str(), "png" | "jpg" | "jpeg" | "bmp" | "tga") {

                let path = path.to_string_lossy().replace("\\", "/");

                let texture = load_texture(&path).await.unwrap();
                texture.set_filter(FilterMode::Nearest);

                self.textures.insert(String::from(path), texture);
            }
        }
    }

    pub async fn register_races(&mut self){
        let female_texture = load_texture("./content/humans/body/female-body.png").await.unwrap();
        female_texture.set_filter(FilterMode::Nearest);

        let male_texture = load_texture("./content/humans/body/male-body.png").await.unwrap();
        male_texture.set_filter(FilterMode::Nearest);

        self.races.insert(String::from("human"), Race{
            description : String::from("humans"),
            name : String::from("human"),
            female_sprite : Sprite { 
                color: get_default_skin_color(), 
                texture: female_texture,
                offset: vec2(0.0, 0.0)
            },
            male_sprite : Sprite {
                color: get_default_skin_color(), 
                texture: male_texture,
                offset: vec2(0.0, 0.0)
            },
            default_skin_color : get_default_skin_color(),
            default_hair_color : get_default_hair_color()
        });
    }

    pub async fn register_hairs(&mut self){
        let texture = load_texture("./content/humans/hair/medium-hair.png").await.unwrap();
        texture.set_filter(FilterMode::Nearest);

        self.hairs.push(
            Sprite { 
                color: get_default_hair_color(), 
                texture,
                offset : vec2(8.0, -32.0)
            }
        );
    }

    pub async fn register_tiles(&mut self){
        let grass_texture = load_texture("./content/vegetation/Grass.png").await.unwrap();
        grass_texture.set_filter(FilterMode::Nearest);

        self.tiles.insert(
            1,
            TileDefinition {
                name: "Grass".into(),
                texture: grass_texture,
                solid: false,
            },
        );

        let grass_object_texture = load_texture("./content/vegetation/grass-object.png").await.unwrap();
        grass_object_texture.set_filter(FilterMode::Nearest);

        self.tiles.insert(
            2,
            TileDefinition {
                name: "Grass object".into(),
                texture: grass_object_texture,
                solid: false,
            },
        );

        let dirt_texture = load_texture("./content/vegetation/Dirt.png").await.unwrap();
        dirt_texture.set_filter(FilterMode::Nearest);

        self.tiles.insert(
            3,
            TileDefinition {
                name: "Dirt".into(),
                texture: dirt_texture,
                solid: true,
            },
        );

    }
}
