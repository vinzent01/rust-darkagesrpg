use hecs::World;
use macroquad::prelude::*;

mod body;
mod chest_inventory;
mod character_creator;
mod crafting;
mod database;
mod game;
mod health;
mod hud;
mod input;
mod inventory;
mod menu;
mod movement;
mod objects;
mod player;
mod render;
mod spawner;
mod tilemap;
mod utils;

struct WindowState {
    width: f32,
    height: f32,
}

#[macroquad::main("Bezier Grid Movement")]
async fn main() {
    let mut world = World::new();
    let mut database = database::Database::new();
    let mut zoom = 4.0;
    let mut turn_phase = game::TurnPhase::Ready;
    let mut menu = menu::Menu::new();
    let mut show_sprite_hitboxes = false;
    let mut render_cache = render::EntityRenderCache::default();
    let outline_material =
        render::create_outline_material().expect("failed to create outline material");

    database.load().await;
    let game_sounds = objects::load_game_sounds()
        .await
        .expect("failed to load game sounds");
    let mut camera = game::create_camera(zoom);
    let mut window = WindowState {
        width: screen_width(),
        height: screen_height(),
    };
    game::spawn_entities(&mut world, &database).await;

    loop {
        let dt = get_frame_time();
        if is_key_pressed(KeyCode::H) {
            show_sprite_hitboxes = !show_sprite_hitboxes;
        }
        let screen_size = (screen_width(), screen_height());
        if screen_size != (window.width, window.height) {
            (window.width, window.height) = screen_size;
            game::resize_camera(&mut camera, zoom);
        }
        set_camera(&camera);

        game::update(
            &mut world,
            &database,
            &mut camera,
            &mut zoom,
            &mut turn_phase,
            &mut menu,
            &game_sounds,
            dt,
        );

        game::draw_world(
            &world,
            &database,
            &camera,
            &outline_material,
            show_sprite_hitboxes,
            &mut render_cache,
        );

        set_default_camera();

        game::draw_hud(&world, &database, &menu);

        next_frame().await;
    }
}
