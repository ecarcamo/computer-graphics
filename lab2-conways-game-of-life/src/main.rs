mod framebuffer;
mod game_of_life;
mod patterns;

use std::{thread, time::Duration};
use raylib::prelude::*;
use framebuffer::Framebuffer;
use game_of_life::GameOfLife;
use rand::{rng, Rng}; 

use patterns::pulsar::create_multiple_pulsars;
use patterns::blinker::create_multiple_blinkers;
use patterns::glider_creator::{Direction, create_multiple_creators_gliders_with_directions};
use patterns::heavy_weight_spaceship::create_multiple_heavy_weight_spaceships;
use patterns::toad::create_multiple_toads;


fn main() {
    let window_width = 800;
    let window_height = 800;
    
    let game_width = 100;
    let game_height = 100;
    let cell_size = 6;

    let (mut window, raylib_thread) = raylib::init()
        .size(window_width, window_height)
        .title("Conway's Game of Life - Esteban Edition")
        .log_level(TraceLogLevel::LOG_WARNING)
        .build();

    let mut framebuffer: Framebuffer = Framebuffer::new(window_width as i32, window_height as i32);
    framebuffer.set_background_color(Color::BLACK);
    
    let mut game = GameOfLife::new(game_width, game_height);
    

    let positions_pulsars = [
         (25, 25),  
         (75, 25),  
         (25, 75),  
         (75, 75),  
         (50, 50),  
     ];

     create_multiple_pulsars(&mut game, &positions_pulsars);


     let positions_blinkers = [ 
         (50, 50),  
     ];

    create_multiple_blinkers(&mut game, &positions_blinkers);


    let glider_configs = [
        (5, 97, Direction::Up),     
    ];
    
    create_multiple_creators_gliders_with_directions(&mut game, &glider_configs);



    let positions_heavy_weight_spaceships = [ 
        (2, 2), 
        (15,2),
        (27,2),
        (40,2),
        (52,2),
        (65,2),
        (77,2),
        (90,2),
     ];

    create_multiple_heavy_weight_spaceships(&mut game, &positions_heavy_weight_spaceships);


    let mut rng = rng();

    let toad_config: Vec<(usize, usize)> = (0..9)
        .map(|_| {
            let x = rng.random_range(10..90);
            let y = rng.random_range(10..90);
            (x, y)
        })
        .collect();

    create_multiple_toads(&mut game, &toad_config);

    //esperar que cargue el primer frame
    thread::sleep(Duration::from_millis(1000)); 


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

        thread::sleep(Duration::from_millis(100)); // Más lento para ver las 3 fases
    }
}