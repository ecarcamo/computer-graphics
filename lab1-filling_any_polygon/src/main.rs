mod framebuffer;
mod line;
mod polygon;

use raylib::prelude::*;
use framebuffer::Framebuffer;
use line::line;
use polygon::fill_polygon;

fn main() {
    let mut fb = Framebuffer::new(800, 600);
    fb.set_background_color(Color::new(50, 50, 100, 255));
    fb.clear();

    let polygon1 = vec![
        Vector2::new(165.0, 380.0), Vector2::new(185.0, 360.0),
        Vector2::new(180.0, 330.0), Vector2::new(207.0, 345.0),
        Vector2::new(233.0, 330.0), Vector2::new(230.0, 360.0),
        Vector2::new(250.0, 380.0), Vector2::new(220.0, 385.0),
        Vector2::new(205.0, 410.0), Vector2::new(193.0, 383.0),
    ];
    fb.set_current_color(Color::YELLOW);
    fill_polygon(&mut fb, &polygon1);
    fb.set_current_color(Color::WHITE);
    for i in 0..polygon1.len() {
        let a = polygon1[i];
        let b = polygon1[(i + 1) % polygon1.len()];
        line(&mut fb, a, b);
    }


    fb.render_to_file("out.bmp");
    println!("Renderizado guardado en out.bmp");
}


   
