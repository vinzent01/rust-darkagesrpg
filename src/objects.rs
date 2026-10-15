use hecs::{Entity, World};
use macroquad::audio::{load_sound_from_bytes, play_sound_once, Sound};
use macroquad::prelude::*;

use crate::chest_inventory;
use crate::database::Database;
use crate::inventory::{self, ItemRef};
use crate::{body::Sprite, movement::Position};
use crate::utils;

pub struct OnOffSprite{
    pub on : Sprite,
    pub off : Sprite,
    pub is_on : bool
}

pub struct Solid;

pub struct Tree {
    pub hits: u8,
    pub fall: Option<TreeFall>,
    pub felled: bool,
}

pub struct TreeFall {
    pub direction: Vec2,
    pub elapsed: f32,
}

pub struct TreeSounds {
    pub chop: Sound,
    pub fall: Sound,
}

const TREE_HITS_TO_FELL: u8 = 4;
pub const TREE_FALL_ROTATION_DURATION: f32 = 0.8;
pub const TREE_FALL_FADE_DURATION: f32 = 0.7;
pub const TREE_FALL_DURATION: f32 = TREE_FALL_ROTATION_DURATION + TREE_FALL_FADE_DURATION;

pub async fn load_tree_sounds() -> Result<TreeSounds, String> {
    Ok(TreeSounds {
        chop: load_mp3_sound("./content/sound-effects/chopdown-axe.mp3").await?,
        fall: load_mp3_sound("./content/sound-effects/falling-tree.mp3").await?,
    })
}

async fn load_mp3_sound(path: &str) -> Result<Sound, String> {
    let mp3_data = macroquad::file::load_file(path)
        .await
        .map_err(|error| format!("failed to read {path}: {error}"))?;
    let wav_data = decode_mp3_to_wav(&mp3_data)
        .map_err(|error| format!("failed to decode {path}: {error}"))?;
    load_sound_from_bytes(&wav_data)
        .await
        .map_err(|error| format!("failed to load decoded sound {path}: {error}"))
}

fn decode_mp3_to_wav(mp3_data: &[u8]) -> Result<Vec<u8>, String> {
    let mut decoder = minimp3::Decoder::new(std::io::Cursor::new(mp3_data));
    let mut pcm = Vec::new();
    let mut sample_rate = None;
    let mut channels = None;

    loop {
        let frame = match decoder.next_frame() {
            Ok(frame) => frame,
            Err(minimp3::Error::Eof) => break,
            Err(error) => return Err(error.to_string()),
        };

        if !matches!(frame.channels, 1 | 2) {
            return Err(format!("unsupported MP3 channel count: {}", frame.channels));
        }
        let frame_sample_rate =
            u32::try_from(frame.sample_rate).map_err(|_| "invalid MP3 sample rate".to_string())?;
        if sample_rate.is_some_and(|rate| rate != frame_sample_rate)
            || channels.is_some_and(|count| count != frame.channels)
        {
            return Err("MP3 audio format changes between frames".to_string());
        }
        sample_rate = Some(frame_sample_rate);
        channels = Some(frame.channels);
        for sample in frame.data {
            pcm.extend_from_slice(&sample.to_le_bytes());
        }
    }

    let sample_rate = sample_rate.ok_or_else(|| "MP3 contains no audio frames".to_string())?;
    let channels = channels.expect("sample rate and channel count are set together");
    let data_length = u32::try_from(pcm.len()).map_err(|_| "MP3 audio is too large".to_string())?;
    let channel_count = u16::try_from(channels).expect("channel count was validated");
    let bytes_per_second = sample_rate
        .checked_mul(u32::from(channel_count) * 2)
        .ok_or_else(|| "MP3 sample rate is too large".to_string())?;
    let block_align = channel_count * 2;

    let riff_length = 36_u32
        .checked_add(data_length)
        .ok_or_else(|| "MP3 audio is too large for WAV format".to_string())?;
    let mut wav = Vec::with_capacity(44 + pcm.len());
    wav.extend_from_slice(b"RIFF");
    wav.extend_from_slice(&riff_length.to_le_bytes());
    wav.extend_from_slice(b"WAVEfmt ");
    wav.extend_from_slice(&16_u32.to_le_bytes());
    wav.extend_from_slice(&1_u16.to_le_bytes());
    wav.extend_from_slice(&channel_count.to_le_bytes());
    wav.extend_from_slice(&sample_rate.to_le_bytes());
    wav.extend_from_slice(&bytes_per_second.to_le_bytes());
    wav.extend_from_slice(&block_align.to_le_bytes());
    wav.extend_from_slice(&16_u16.to_le_bytes());
    wav.extend_from_slice(b"data");
    wav.extend_from_slice(&data_length.to_le_bytes());
    wav.extend_from_slice(&pcm);
    Ok(wav)
}

impl Tree {
    pub fn take_axe_hit(&mut self, fall_direction: Vec2) -> Option<bool> {
        if self.fall.is_some() || self.felled {
            return None;
        }

        self.hits = self.hits.saturating_add(1);
        if self.hits < TREE_HITS_TO_FELL {
            return Some(false);
        }

        self.fall = Some(TreeFall {
            direction: fall_direction.normalize_or_zero(),
            elapsed: 0.0,
        });
        Some(true)
    }
}

pub fn update_tree_falls(world: &mut World, dt: f32) {
    for (_, tree) in world.query_mut::<(Entity, &mut Tree)>() {
        let Some(fall) = &mut tree.fall else {
            continue;
        };

        fall.elapsed = (fall.elapsed + dt).min(TREE_FALL_DURATION);
        if fall.elapsed >= TREE_FALL_DURATION {
            tree.felled = true;
        }
    }
}

pub fn finish_tree_felling(
    world: &mut World,
    database: &Database,
    sounds: Option<&TreeSounds>,
    entity: Entity,
    player_position: Vec2,
) -> Option<bool> {
    let Ok(tree_position) = world.get::<&Position>(entity).map(|position| position.value) else {
        return None;
    };
    let direction = tree_position - player_position;
    let fall_direction = if direction.x.abs() >= direction.y.abs() {
        vec2(direction.x.signum(), 0.0)
    } else {
        vec2(0.0, direction.y.signum())
    };

    let felled = world
        .get::<&mut Tree>(entity)
        .ok()
        .and_then(|mut tree| tree.take_axe_hit(fall_direction));
    let Some(felled) = felled else {
        return None;
    };

    if let Some(sounds) = sounds {
        play_sound_once(&sounds.chop);
    }
    if felled {
        if let Some(sounds) = sounds {
            play_sound_once(&sounds.fall);
        }
        let item = ItemRef {
            id: "wood".to_string(),
            quantity: 1,
        };
        if !inventory::add_item_to_player(world, database, item.clone()) {
            if let Some(position) = inventory::find_drop_position(world, tree_position) {
                world.spawn((inventory::GroundItem { item, position },));
            }
        }
    }
    Some(felled)
}

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

#[cfg(test)]
mod tests {
    use super::{Tree, TreeFall, decode_mp3_to_wav};
    use hecs::World;
    use macroquad::prelude::vec2;

    #[test]
    fn tree_falls_on_fourth_axe_hit_and_only_once() {
        let mut tree = Tree {
            hits: 0,
            fall: None,
            felled: false,
        };

        assert_eq!(tree.take_axe_hit(vec2(-1.0, 0.0)), Some(false));
        assert_eq!(tree.take_axe_hit(vec2(-1.0, 0.0)), Some(false));
        assert_eq!(tree.take_axe_hit(vec2(-1.0, 0.0)), Some(false));
        assert_eq!(tree.take_axe_hit(vec2(-1.0, 0.0)), Some(true));
        assert_eq!(tree.hits, 4);
        assert_eq!(
            tree.fall.as_ref().map(|fall| fall.direction),
            Some(vec2(-1.0, 0.0))
        );
        assert_eq!(tree.take_axe_hit(vec2(1.0, 0.0)), None);
    }

    #[test]
    fn tree_is_marked_felled_after_the_fall_animation() {
        let mut world = World::new();
        let tree = world.spawn((Tree {
            hits: 4,
            fall: Some(TreeFall {
                direction: vec2(0.0, 1.0),
                elapsed: 0.0,
            }),
            felled: false,
        },));

        super::update_tree_falls(&mut world, super::TREE_FALL_ROTATION_DURATION);
        assert!(!world.get::<&Tree>(tree).unwrap().felled);

        super::update_tree_falls(&mut world, super::TREE_FALL_FADE_DURATION);

        assert!(world.get::<&Tree>(tree).unwrap().felled);
    }

    #[test]
    fn tree_sound_assets_decode_to_wav() {
        for path in [
            "./content/sound-effects/chopdown-axe.mp3",
            "./content/sound-effects/falling-tree.mp3",
        ] {
            let mp3 = std::fs::read(path).unwrap();
            let wav = decode_mp3_to_wav(&mp3).unwrap();
            assert_eq!(&wav[..4], b"RIFF");
            assert_eq!(&wav[8..12], b"WAVE");
        }
    }
}