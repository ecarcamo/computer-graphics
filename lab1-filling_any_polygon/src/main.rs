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

    
    let polygon3 = vec![
        Vector2::new(377.0, 249.0), Vector2::new(411.0, 197.0),
        Vector2::new(436.0, 249.0)
    ];
    fb.set_current_color(Color::RED);
    fill_polygon(&mut fb, &polygon3);
    fb.set_current_color(Color::WHITE);
    for i in 0..polygon3.len() {
        let a = polygon3[i];
        let b = polygon3[(i + 1) % polygon3.len()];
        line(&mut fb, a, b);
    }

    fb.render_to_file("out.bmp");
    println!("Renderizado guardado en out.bmp");
}


   
