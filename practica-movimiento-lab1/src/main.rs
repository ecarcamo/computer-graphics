mod framebuffer;
mod line;
mod polygon;

use std::{thread, time::Duration};

use raylib::prelude::*;
use framebuffer::Framebuffer;
use line::line;
use polygon::fill_polygon;

fn main() {
    let window_width = 800;
    let window_height = 600;

    let (mut window, raylib_thread) = raylib::init()
        .size(window_width, window_height)
        .title("Polygons with Holes")
        .log_level(TraceLogLevel::LOG_WARNING)
        .build();

    let mut framebuffer: Framebuffer = Framebuffer::new(window_width as i32, window_height as i32);
    framebuffer.set_background_color(Color::new(50, 50, 100, 255));
    framebuffer.clear();

    let mut translate_x = 0.0;
    let mut translate_y = 0.0;
    let mut velocity_x = 1.0;
    let mut velocity_y = 1.0;

    while !window.window_should_close() {
        if window.is_key_pressed(KeyboardKey::KEY_S) {  
            framebuffer.render_to_file("out.bmp");
            println!("Renderizado guardado en out.bmp");
        }

        if translate_x <= -165.0 || translate_x >= (window_width as f32 - 761.0) {
            velocity_x = -velocity_x;
        }

        if translate_y <= -36.0 || translate_y >= (window_height as f32 - 410.0) {
            velocity_y = -velocity_y;
        }

        translate_x += velocity_x;
        translate_y += velocity_y;

        framebuffer.clear();

        render(&mut framebuffer, translate_x, translate_y);

        framebuffer.swap_buffers(&mut window, &raylib_thread);

        thread::sleep(Duration::from_millis(16));
    }

}


   
fn render (
    framebuffer: &mut Framebuffer,
    translate_x: f32,
    translate_y: f32
){
    framebuffer.set_current_color(Color::GREEN);

    let polygon1 = vec![
        Vector2::new(165.0 + translate_x, 380.0 + translate_y), 
        Vector2::new(185.0 + translate_x, 360.0 + translate_y),
        Vector2::new(180.0 + translate_x, 330.0 + translate_y), 
        Vector2::new(207.0 + translate_x, 345.0 + translate_y),
        Vector2::new(233.0 + translate_x, 330.0 + translate_y), 
        Vector2::new(230.0 + translate_x, 360.0 + translate_y),
        Vector2::new(250.0 + translate_x, 380.0 + translate_y), 
        Vector2::new(220.0 + translate_x, 385.0 + translate_y),
        Vector2::new(205.0 + translate_x, 410.0 + translate_y), 
        Vector2::new(193.0 + translate_x, 383.0 + translate_y),
    ];
    framebuffer.set_current_color(Color::YELLOW);
    fill_polygon(framebuffer, &polygon1);
    framebuffer.set_current_color(Color::WHITE);
    for i in 0..polygon1.len() {
        let a = polygon1[i];
        let b = polygon1[(i + 1) % polygon1.len()];
        line(framebuffer, a, b);
    }

    let polygon2 = vec![
        Vector2::new(321.0 + translate_x, 335.0 + translate_y), 
        Vector2::new(288.0 + translate_x, 286.0 + translate_y),
        Vector2::new(339.0 + translate_x, 251.0 + translate_y), 
        Vector2::new(374.0 + translate_x, 302.0 + translate_y)
    ];
    framebuffer.set_current_color(Color::BLUE);
    fill_polygon(framebuffer, &polygon2);
    framebuffer.set_current_color(Color::WHITE);
    for i in 0..polygon2.len() {
        let a = polygon2[i];
        let b = polygon2[(i + 1) % polygon2.len()];
        line(framebuffer, a, b);
    }

    let polygon3 = vec![
        Vector2::new(377.0 + translate_x, 249.0 + translate_y), 
        Vector2::new(411.0 + translate_x, 197.0 + translate_y),
        Vector2::new(436.0 + translate_x, 249.0 + translate_y)
    ];
    framebuffer.set_current_color(Color::RED);
    fill_polygon(framebuffer, &polygon3);
    framebuffer.set_current_color(Color::WHITE);
    for i in 0..polygon3.len() {
        let a = polygon3[i];
        let b = polygon3[(i + 1) % polygon3.len()];
        line(framebuffer, a, b);
    }

    let polygon4 = vec![
        Vector2::new(413.0 + translate_x, 177.0 + translate_y), 
        Vector2::new(448.0 + translate_x, 159.0 + translate_y),
        Vector2::new(502.0 + translate_x, 88.0 + translate_y),  
        Vector2::new(553.0 + translate_x, 53.0 + translate_y),
        Vector2::new(535.0 + translate_x, 36.0 + translate_y),  
        Vector2::new(676.0 + translate_x, 37.0 + translate_y),
        Vector2::new(660.0 + translate_x, 52.0 + translate_y),  
        Vector2::new(750.0 + translate_x, 145.0 + translate_y),
        Vector2::new(761.0 + translate_x, 179.0 + translate_y), 
        Vector2::new(672.0 + translate_x, 192.0 + translate_y),
        Vector2::new(659.0 + translate_x, 214.0 + translate_y), 
        Vector2::new(615.0 + translate_x, 214.0 + translate_y),
        Vector2::new(632.0 + translate_x, 230.0 + translate_y), 
        Vector2::new(580.0 + translate_x, 230.0 + translate_y),
        Vector2::new(597.0 + translate_x, 215.0 + translate_y), 
        Vector2::new(552.0 + translate_x, 214.0 + translate_y),
        Vector2::new(517.0 + translate_x, 144.0 + translate_y), 
        Vector2::new(466.0 + translate_x, 180.0 + translate_y),
    ];
    framebuffer.set_current_color(Color::GREEN);
    fill_polygon(framebuffer, &polygon4);
    framebuffer.set_current_color(Color::WHITE);
    for i in 0..polygon4.len() {
        let a = polygon4[i];
        let b = polygon4[(i + 1) % polygon4.len()];
        line(framebuffer, a, b);
    }

    let polygon5_hole = vec![
        Vector2::new(682.0 + translate_x, 175.0 + translate_y), 
        Vector2::new(708.0 + translate_x, 120.0 + translate_y),
        Vector2::new(735.0 + translate_x, 148.0 + translate_y), 
        Vector2::new(739.0 + translate_x, 170.0 + translate_y),
    ];
    framebuffer.set_current_color(Color::new(50, 50, 100, 255));
    fill_polygon(framebuffer, &polygon5_hole);
}