use macroquad::prelude::*;
use std::cell::RefCell;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::{fs, vec};

use crate::body::{
    HumanLayer, Race, Sprite, get_default_skin_color,
};
use crate::health::MAX_HEALTH;
use crate::inventory::{EquipmentSlot, Item, ItemEquipment, Recipe, RecipeIngredient};
use crate::tilemap::{TileDefinition, TileId};

pub struct Database {
    pub tiles: HashMap<TileId, TileDefinition>,
    pub hairs: Vec<HumanLayer>,
    pub beards: Vec<HumanLayer>,
    pub races: HashMap<String, Race>,
    pub textures: HashMap<String, Texture2D>,
    pub items: HashMap<String, Item>,
    pub creatures: HashMap<String, CreatureDefinition>,
    texture_alpha: RefCell<HashMap<miniquad::TextureId, TextureAlpha>>,
}

pub struct CreatureDefinition {
    pub id: String,
    pub name: String,
    pub texture_path: String,
    pub maximum_health: i32,
    pub spawn_weight: u32,
}

struct TextureAlpha {
    width: u16,
    height: u16,
    pixels: Vec<u8>,
}

impl TextureAlpha {
    fn is_opaque(&self, x: u32, y: u32) -> bool {
        x < u32::from(self.width)
            && y < u32::from(self.height)
            && self.pixels[(y * u32::from(self.width) + x) as usize] > 0
    }
}

impl Database {
    pub fn new() -> Self {
        Self {
            tiles: HashMap::new(),
            beards: vec![],
            hairs: vec![],
            races: HashMap::new(),
            textures: HashMap::new(),
            items: HashMap::new(),
            creatures: HashMap::new(),
            texture_alpha: RefCell::new(HashMap::new()),
        }
    }

    pub fn texture_pixel_is_opaque(&self, texture: &Texture2D, x: u32, y: u32) -> bool {
        let texture_id = texture.raw_miniquad_id();
        if !self.texture_alpha.borrow().contains_key(&texture_id) {
            let image = texture.get_texture_data();
            let alpha = image
                .get_image_data()
                .iter()
                .map(|pixel| pixel[3])
                .collect();
            self.texture_alpha.borrow_mut().insert(
                texture_id,
                TextureAlpha {
                    width: image.width,
                    height: image.height,
                    pixels: alpha,
                },
            );
        }

        let cache = self.texture_alpha.borrow();
        cache[&texture_id].is_opaque(x, y)
    }

    pub async fn load(&mut self) {
        self.register_textures().await;
        self.register_tiles().await;
        self.register_hairs().await;
        self.register_races().await;
        self.register_items();
        self.register_creatures();
    }

    fn register_creatures(&mut self) {
        for (id, name, texture_path, spawn_weight) in [
            ("bear", "Urso", "./content/creatures/bear.png", 2),
            ("deer", "Cervo", "./content/creatures/deer.png", 6),
            ("griffon", "Grifo", "./content/creatures/griffon.png", 1),
            ("rabbit", "Coelho", "./content/creatures/rabbit.png", 8),
            (
                "white_horse",
                "Cavalo branco",
                "./content/creatures/white-horse.png",
                6,
            ),
            ("horse", "Cavalo", "./content/creatures/horse.png", 6),
            ("wolf", "Lobo", "./content/creatures/wolf.png", 8),
        ] {
            self.creatures.insert(
                id.to_string(),
                CreatureDefinition {
                    id: id.to_string(),
                    name: name.to_string(),
                    texture_path: texture_path.to_string(),
                    maximum_health: MAX_HEALTH,
                    spawn_weight,
                },
            );
        }
    }

    fn register_items(&mut self) {
        for (id, name, description, weight, texture_path, equipment, recipe) in [
            (
                "stick",
                "Graveto",
                "Um graveto simples usado para fabricar ferramentas.",
                0.1,
                "./content/crafting-items/stick.png",
                None,
                Some(recipe(&[("wood", 1)], 2)),
            ),
            (
                "wood",
                "Madeira",
                "Madeira obtida de árvores e usada em receitas.",
                1.0,
                "./content/vegetation/wood.png",
                None,
                None,
            ),
            (
                "stone",
                "Pedra",
                "Pedra coletada do chão.",
                1.0,
                "./content/vegetation/stone.png",
                None,
                None,
            ),
            (
                "stone_axe",
                "Machado de pedra",
                "Ferramenta para cortar árvores.",
                1.5,
                "./content/melee weapons/stone-axe.png",
                Some(ItemEquipment {
                    slot: EquipmentSlot::MainHand,
                    male_texture_path: "./content/melee weapons/stone-axe.png".to_string(),
                    female_texture_path: "./content/melee weapons/stone-axe.png".to_string(),
                    offset: vec2(-8.0, 2.0),
                }),
                Some(recipe(&[("stick", 2), ("stone", 2)], 1)),
            ),
            (
                "pickaxe",
                "Picareta",
                "Ferramenta para minerar recursos.",
                2.0,
                "./content/melee weapons/pickAxe.png",
                Some(ItemEquipment {
                    slot: EquipmentSlot::MainHand,
                    male_texture_path: "./content/melee weapons/pickAxe.png".to_string(),
                    female_texture_path: "./content/melee weapons/pickAxe.png".to_string(),
                    offset: vec2(-8.0, 2.0),
                }),
                Some(recipe(&[("stick", 2), ("stone", 3)], 1)),
            ),
            (
                "sword",
                "Espada",
                "Uma arma corpo a corpo.",
                1.5,
                "./content/melee weapons/sword.png",
                Some(ItemEquipment {
                    slot: EquipmentSlot::MainHand,
                    male_texture_path: "./content/melee weapons/sword.png".to_string(),
                    female_texture_path: "./content/melee weapons/sword.png".to_string(),
                    offset: vec2(-8.0, 2.0),
                }),
                None
            ),
            (
                "shield",
                "Escudo",
                "Um escudo leve para defesa.",
                2.0,
                "./content/shield/shield.png",
                Some(ItemEquipment {
                    slot: EquipmentSlot::OffHand,
                    male_texture_path: "./content/shield/shield.png".to_string(),
                    female_texture_path: "./content/shield/shield.png".to_string(),
                    offset: vec2(20.0, 8.0),
                }),
                None
            ),
            (
                "linen_tunic",
                "Tunica de linho",
                "Uma tunica simples feita de linho.",
                0.5,
                "./content/male clothes/male linen clothes/linen tunic/linen_tunic.png",
                Some(ItemEquipment {
                    slot: EquipmentSlot::Torso,
                    male_texture_path:
                        "./content/male clothes/male linen clothes/linen tunic/linen_tunic.png"
                            .to_string(),
                    female_texture_path: "./content/female clothes/linen cothes/linen-tunic.png"
                        .to_string(),
                    offset: vec2(0.0, 0.0),
                }),
                None,
            ),
            (
                "linen_pants",
                "Calcas de linho",
                "Calcas simples feitas de linho.",
                0.4,
                "./content/male clothes/male linen clothes/linen pants/linen_pants.png",
                Some(ItemEquipment {
                    slot: EquipmentSlot::Legs,
                    male_texture_path:
                        "./content/male clothes/male linen clothes/linen pants/linen_pants.png"
                            .to_string(),
                    female_texture_path: "./content/female clothes/linen cothes/linen-pants.png"
                        .to_string(),
                    offset: vec2(0.0, 0.0),
                }),
                None,
            ),
            (
                "leather_boots",
                "Botas de couro",
                "Botas resistentes feitas de couro.",
                0.3,
                "./content/male clothes/leather armor/leather boots/leather_boots.png",
                Some(ItemEquipment {
                    slot: EquipmentSlot::Feet,
                    male_texture_path:
                        "./content/male clothes/leather armor/leather boots/leather_boots.png"
                            .to_string(),
                    female_texture_path: "./content/female clothes/leather armor/leather-boots.png"
                        .to_string(),
                    offset: vec2(0.0, 0.0),
                }),
                None,
            ),
        ] {
            self.items.insert(
                id.to_string(),
                Item {
                    id: id.to_string(),
                    name: name.to_string(),
                    description: description.to_string(),
                    weight,
                    texture_path: texture_path.to_string(),
                    equipment,
                    recipe,
                },
            );
        }

        fn recipe(ingredients: &[(&str, i32)], output_quantity: i32) -> Recipe {
            Recipe {
                ingredients: ingredients
                    .iter()
                    .map(|(item_id, quantity)| RecipeIngredient {
                        item_id: (*item_id).to_string(),
                        quantity: *quantity,
                    })
                    .collect(),
                output_quantity,
            }
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

    pub async fn register_races(&mut self) {
        let female_texture = load_texture("./content/humans/body/female-body.png")
            .await
            .unwrap();
        female_texture.set_filter(FilterMode::Nearest);

        let male_texture = load_texture("./content/humans/body/male-body.png")
            .await
            .unwrap();
        male_texture.set_filter(FilterMode::Nearest);

        self.races.insert(
            String::from("human"),
            Race {
                description: String::from("humans"),
                name: String::from("human"),
                female_sprite: Sprite {
                    color: get_default_skin_color(),
                    texture: female_texture,
                    offset: vec2(0.0, 0.0),
                },
                male_sprite: Sprite {
                    color: get_default_skin_color(),
                    texture: male_texture,
                    offset: vec2(0.0, 0.0),
                },
            },
        );
    }

    pub async fn register_hairs(&mut self) {
        self.hairs = human_layers(
            &self.textures,
            &[
                (
                    "Médio",
                    "./content/humans/hair/medium-hair.png",
                    vec2(9.0, 0.0),
                    vec2(9.0, 0.0),
                ),
                (
                    "Médio curto",
                    "./content/humans/hair/medium-hair2.png",
                    vec2(10.0, 1.0),
                    vec2(10.0, 0.0),
                ),
                (
                    "Samurai",
                    "./content/humans/hair/samurai-hair.png",
                    vec2(9.0, 0.0),
                    vec2(9.0, 1.0),
                ),
                (
                    "Samurai longo",
                    "./content/humans/hair/samurai-hair2.png",
                    vec2(9.0, -1.0),
                    vec2(9.0, -1.0),
                ),
                (
                    "Samurai preso",
                    "./content/humans/hair/samurai-hair3.png",
                    vec2(9.0, 0.0),
                    vec2(9.0, 0.0),
                ),
                (
                    "Curto",
                    "./content/humans/hair/short-hair2.png",
                    vec2(10.0, 1.0),
                    vec2(10.0, 1.0),
                ),
            ],
        );
        self.beards = human_layers(
            &self.textures,
            &[
                (
                    "Rala",
                    "./content/humans/beard/beard1.png",
                    vec2(10.0, 0.0),
                    vec2(10.0, 1.0),
                ),
                (
                    "Curta",
                    "./content/humans/beard/beard2.png",
                    vec2(10.0, 0.0),
                    vec2(10.0, 1.0),
                ),
                (
                    "Cheia",
                    "./content/humans/beard/beard3.png",
                    vec2(10.0, 0.0),
                    vec2(10.0, 1.0),
                ),
                (
                    "Cheia curta",
                    "./content/humans/beard/beard4.png",
                    vec2(10.0, 1.0),
                    vec2(10.0, 1.0),
                ),
                (
                    "Longa",
                    "./content/humans/beard/beard5.png",
                    vec2(10.0, 1.0),
                    vec2(10.0, 2.0),
                ),
            ],
        );
    }

    pub async fn register_tiles(&mut self) {
        let grass_texture = load_texture("./content/vegetation/Grass.png")
            .await
            .unwrap();
        grass_texture.set_filter(FilterMode::Nearest);

        self.tiles.insert(
            1,
            TileDefinition {
                name: "Grass".into(),
                texture: grass_texture,
                solid: false,
            },
        );

        let grass_object_texture = load_texture("./content/vegetation/grass-object.png")
            .await
            .unwrap();
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

fn human_layers(
    textures: &HashMap<String, Texture2D>,
    entries: &[(&'static str, &'static str, Vec2, Vec2)],
) -> Vec<HumanLayer> {
    entries
        .iter()
        .map(|(name, path, male_offset, female_offset)| HumanLayer {
            name,
            texture: textures
                .get(*path)
                .unwrap_or_else(|| panic!("human customization texture not found: {path}"))
                .clone(),
            male_offset: *male_offset,
            female_offset: *female_offset,
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::{Database, TextureAlpha};
    use crate::health::MAX_HEALTH;

    #[test]
    fn texture_alpha_mask_only_hits_nontransparent_pixels() {
        let mask = TextureAlpha {
            width: 2,
            height: 2,
            pixels: vec![0, 1, 128, 255],
        };

        assert!(!mask.is_opaque(0, 0));
        assert!(mask.is_opaque(1, 0));
        assert!(mask.is_opaque(0, 1));
        assert!(mask.is_opaque(1, 1));
        assert!(!mask.is_opaque(2, 0));
        assert!(!mask.is_opaque(0, 2));
    }

    #[test]
    fn registers_all_default_creatures_with_spawn_weights() {
        let mut database = Database::new();
        database.register_creatures();

        let expected = [
            ("bear", "./content/creatures/bear.png", 2),
            ("deer", "./content/creatures/deer.png", 6),
            ("griffon", "./content/creatures/griffon.png", 1),
            ("rabbit", "./content/creatures/rabbit.png", 8),
            ("white_horse", "./content/creatures/white-horse.png", 6),
            ("horse", "./content/creatures/horse.png", 6),
            ("wolf", "./content/creatures/wolf.png", 8),
        ];
        assert_eq!(database.creatures.len(), expected.len());

        for (id, texture_path, spawn_weight) in expected {
            let creature = &database.creatures[id];
            assert_eq!(creature.id, id);
            assert_eq!(creature.texture_path, texture_path);
            assert_eq!(creature.maximum_health, MAX_HEALTH);
            assert_eq!(creature.spawn_weight, spawn_weight);
            assert!(!creature.name.is_empty());
        }
    }

    #[test]
    fn registered_crafting_recipes_reference_valid_items() {
        let mut database = Database::new();
        database.register_items();

        let mut recipe_count = 0;
        for item in database.items.values() {
            assert!(!item.description.is_empty());
            if let Some(recipe) = &item.recipe {
                recipe_count += 1;
                assert!(recipe.output_quantity > 0);
                for ingredient in &recipe.ingredients {
                    assert!(database.items.contains_key(&ingredient.item_id));
                    assert!(ingredient.quantity > 0);
                }
            }
        }
        assert_eq!(recipe_count, 5);
    }
}
