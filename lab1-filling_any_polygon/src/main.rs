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

    

    let polygon2 = vec![
        Vector2::new(321.0, 335.0), Vector2::new(288.0, 286.0),
        Vector2::new(339.0, 251.0), Vector2::new(374.0,302.0)
    ];
    fb.set_current_color(Color::BLUE);
    fill_polygon(&mut fb, &polygon2);
    fb.set_current_color(Color::WHITE);
    for i in 0..polygon2.len() {
        let a = polygon2[i];
        let b = polygon2[(i + 1) % polygon2.len()];
        line(&mut fb, a, b);
    }



    fb.render_to_file("out.bmp");
    println!("Renderizado guardado en out.bmp");
}


   
