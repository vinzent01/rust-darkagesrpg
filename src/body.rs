use macroquad::{color::Color, math::Vec2};

#[derive(Clone)]
pub struct Sprite{
    pub color: Color,
    pub texture_path: String,
    pub offset : Vec2
}

pub struct SpriteOverlay {
    pub sprite: Sprite,
}

#[derive(Clone)]
pub struct Body {
    pub race : raceId,
    pub genre : bool
}

#[derive(Clone)]
pub struct Appearence {
    pub is_flipped : bool,
    pub body: Body,
    pub beard: Option<usize>,
    pub hair: Option<usize>,
    pub skin_color: Color,
    pub hair_color: Color,
}

pub struct HumanLayer {
    pub name: &'static str,
    pub texture_path: &'static str,
    pub male_offset: Vec2,
    pub female_offset: Vec2,
}

impl HumanLayer {
    pub fn offset_for(&self, is_male: bool) -> Vec2 {
        if is_male {
            self.male_offset
        } else {
            self.female_offset
        }
    }
}

pub struct Race{
    pub name : String,
    pub description : String,
    pub female_sprite : Sprite,
    pub male_sprite : Sprite,
}

pub type bodyId = &'static str;
pub type raceId = &'static str;


pub fn get_default_skin_color() -> Color{
    return Color::from_rgba(219, 185, 140, 255);
}

pub fn get_all_skin_colors() -> Vec<Color>{
    vec![
        get_default_skin_color(),
        Color::from_rgba(128, 100, 75, 255),
        Color::from_rgba(71,51,39,255),
    ]
}

pub fn get_default_hair_color() -> Color{
    Color::from_rgba(69, 47, 33, 255)
}

pub fn get_all_hair_colors() -> Vec<Color>  {
    vec![
        Color::from_rgba(138, 47, 17, 255),
        Color::from_rgba(224, 224, 224, 255),
        Color::from_rgba(194, 156, 60, 255),
        get_default_hair_color(),
    ]
}
    
