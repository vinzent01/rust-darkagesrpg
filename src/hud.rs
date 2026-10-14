use hecs::{Entity, World};
use macroquad::{color, prelude::*};

use crate::database::Database;
use crate::utils::max_f;

pub struct UIInventory {
    pub rect : Rect,
    pub bg_color : Color,
    pub is_handling : bool,
    pub drag_offset: Vec2,
}

pub struct UIButton{
    pub is_clicked : bool,
    pub id : &'static str
}


fn draw_round_rect(x: f32, y: f32, w: f32, h: f32, r: f32, color: Color) {
    // Centro
    draw_rectangle(x + r, y, w - 2.0 * r, h, color);

    // Laterais
    draw_rectangle(x, y + r, r, h - 2.0 * r, color);
    draw_rectangle(x + w - r, y + r, r, h - 2.0 * r, color);

    // Cantos
    draw_circle(x + r, y + r, r, color);
    draw_circle(x + w - r, y + r, r, color);
    draw_circle(x + r, y + h - r, r, color);
    draw_circle(x + w - r, y + h - r, r, color);
}

pub fn update_ui(world : &mut World){
    handle_spawn_ui(world);
    update_inventory_ui(world);
}   

pub fn handle_spawn_ui(world: &mut World) {

    if is_key_pressed(KeyCode::I) {
        let inventory = {
            let mut query = world.query::<(Entity, &UIInventory)>();
            query.iter().next().map(|(entity, _)| entity)
        };

        if let Some(entity) = inventory {
            world.despawn(entity).unwrap();
        } else {
            let screen_width = screen_width();
            let screen_height = screen_height();
            
            world.spawn((
                UIInventory {
                    bg_color: Color::from_rgba(32, 32, 32, 255),
                    rect: Rect::new(screen_width/2.0, screen_height/2.0, 100.0, 100.0),
                    is_handling: false,
                    drag_offset: vec2(0.0, 0.0),
                },
            ));
        }
    }
}

pub fn draw_inventory_ui(world : &mut World, database : &Database){
    let inventory_slot = database.textures.get("./content/hud/item_slot.png").unwrap();
    let scale = 2.0;

    let slot_width = inventory_slot.width() * scale;
    let slot_height = inventory_slot.height() * scale;

    for inv in world.query::<&mut UIInventory>().iter(){
        let slots = 5;        
        let padding =20.0;


        let total_size_x =  (slot_width * slots as f32) + padding * 2.0; 
        let total_size_y =  (slot_height * slots as f32) + padding * 2.0;

        inv.rect.w = max_f(inv.rect.w, total_size_x);
        inv.rect.h = max_f(inv.rect.h, total_size_y);


        let inventory_text = "Inventory";
        let label_font_size = 24;
        let text_size = measure_text(inventory_text, None, label_font_size, 1.0);
        let middle_pos = inv.rect.x + (inv.rect.w / 2.0) - (text_size.width / 2.0);
        let top_pos = inv.rect.y + 10.0;

        draw_round_rect(
            inv.rect.x, 
            inv.rect.y, 
            inv.rect.w,
            inv.rect.h, 
            12.0, 
            inv.bg_color
        );

        draw_text(inventory_text, middle_pos, top_pos, label_font_size as f32, color::WHITE);

        let slot_start_x = inv.rect.x + padding;
        let slot_start_y = inv.rect.y + padding;

    
        for x in 0..5{
            for y in 0..5{
                draw_texture_ex(
                    inventory_slot, 
                    slot_start_x + x as f32 * slot_width, 
                    slot_start_y + y as f32 * slot_height,
                    color::WHITE,
                    DrawTextureParams{
                        dest_size:  Some(
                            vec2(slot_width, slot_height)
                        ),
                        ..Default::default()
                    }
                );

            }
        }

    }
}

fn update_inventory_ui(world : &mut World){

    let mouse_pos = mouse_position();
    let mouse_pos = vec2(mouse_pos.0, mouse_pos.1);
    
    for inv in world.query::<&mut UIInventory>().iter(){
        
        // Começa o drag apenas no clique inicial
        if !inv.is_handling
            && inv.rect.contains(mouse_pos)
            && is_mouse_button_pressed(MouseButton::Left)
        {
            inv.is_handling = true;
            inv.drag_offset = mouse_pos - vec2(inv.rect.x, inv.rect.y);
        }

        // Termina o drag ao soltar o botão
        if inv.is_handling && is_mouse_button_released(MouseButton::Left) {
            inv.is_handling = false;
        }

        // Move enquanto estiver arrastando
        if inv.is_handling {
            inv.rect.x = mouse_pos.x - inv.drag_offset.x;
            inv.rect.y = mouse_pos.y - inv.drag_offset.y;
        }
    }

}