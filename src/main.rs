use hecs::World;
use macroquad::prelude::*;

mod body;
mod chest_inventory;
mod database;
mod game;
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

    database.load().await;
    let mut camera = game::create_camera(zoom);
    let mut window = WindowState {
        width: screen_width(),
        height: screen_height(),
    };
    game::spawn_entities(&mut world, &database).await;

    loop {
        let dt = get_frame_time();
        let screen_size = (screen_width(), screen_height());
        if screen_size != (window.width, window.height) {
            (window.width, window.height) = screen_size;
            game::resize_camera(&mut camera, zoom);
        }

        game::update(
            &mut world,
            &database,
            &mut camera,
            &mut zoom,
            &mut turn_phase,
            &mut menu,
            dt,
        );

        set_camera(&camera);
        game::draw_world(&world, &database, &camera);
        set_default_camera();
        game::draw_hud(&world, &database, &menu);

        draw_text("WASD ou Setas para mover", 10.0, 20.0, 24.0, WHITE);
        draw_text(&format!("FPS {}", get_fps()), 10.0, 30.0, 24.0, WHITE);

        next_frame().await;
    }
}
