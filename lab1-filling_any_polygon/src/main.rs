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

    let polygon4 = vec![
        Vector2::new(413.0, 177.0), Vector2::new(448.0, 159.0),
        Vector2::new(502.0, 88.0),  Vector2::new(553.0, 53.0),
        Vector2::new(535.0, 36.0),  Vector2::new(676.0, 37.0),
        Vector2::new(660.0, 52.0),  Vector2::new(750.0, 145.0),
        Vector2::new(761.0, 179.0), Vector2::new(672.0, 192.0),
        Vector2::new(659.0, 214.0), Vector2::new(615.0, 214.0),
        Vector2::new(632.0, 230.0), Vector2::new(580.0, 230.0),
        Vector2::new(597.0, 215.0), Vector2::new(552.0, 214.0),
        Vector2::new(517.0, 144.0), Vector2::new(466.0, 180.0),
    ];
    fb.set_current_color(Color::GREEN);
    fill_polygon(&mut fb, &polygon4);
    fb.set_current_color(Color::WHITE);
    for i in 0..polygon4.len() {
        let a = polygon4[i];
        let b = polygon4[(i + 1) % polygon4.len()];
        line(&mut fb, a, b);
    }

    let polygon5_hole = vec![
        Vector2::new(682.0, 175.0), Vector2::new(708.0, 120.0),
        Vector2::new(735.0, 148.0), Vector2::new(739.0, 170.0),
    ];
    fb.set_current_color(Color::new(50, 50, 100, 255));
    fill_polygon(&mut fb, &polygon5_hole);


    fb.render_to_file("out.bmp");
    println!("Renderizado guardado en out.bmp");
}


   
