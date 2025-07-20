mod framebuffer;
mod game_of_life;
mod patterns;

use std::{thread, time::Duration};
use raylib::prelude::*;
use framebuffer::Framebuffer;
use game_of_life::GameOfLife;

use patterns::pulsar::create_multiple_pulsars;

fn main() {
    let window_width = 800;
    let window_height = 800;
    
    let game_width = 100;
    let game_height = 100;
    let cell_size = 6;

    let (mut window, raylib_thread) = raylib::init()
        .size(window_width, window_height)
        .title("Conway's Game of Life - True Pulsar Edition")
        .log_level(TraceLogLevel::LOG_WARNING)
        .build();

    let mut framebuffer: Framebuffer = Framebuffer::new(window_width as i32, window_height as i32);
    framebuffer.set_background_color(Color::BLACK);
    
    let mut game = GameOfLife::new(game_width, game_height);
    

   let positions = [
        (25, 25),  // Esquina superior izquierda
        (75, 25),  // Esquina superior derecha
        (25, 75),  // Esquina inferior izquierda
        (75, 75),  // Esquina inferior derecha
        (50, 50),  // Centro
    ];

    create_multiple_pulsars(&mut game, &positions);




    thread::sleep(Duration::from_millis(500)); // Más lento para ver las 3 fases


    while !window.window_should_close() {
        if window.is_key_pressed(KeyboardKey::KEY_S) {  
            framebuffer.render_to_file("out.bmp");
            println!("Renderizado guardado en out.bmp");
        }

        framebuffer.clear();
        framebuffer.set_current_color(Color::WHITE);
        
        game.render(&mut framebuffer, cell_size);
        
        game.update();

        framebuffer.swap_buffers(&mut window, &raylib_thread);

        thread::sleep(Duration::from_millis(200)); // Más lento para ver las 3 fases
    }
}