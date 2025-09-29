use crate::framebuffer::Framebuffer;
use raylib::prelude::*;

/// Rellena un polígono simple (no necesariamente convexo) cerrado con scanline fill
pub fn fill_polygon(fb: &mut Framebuffer, vertices: &[Vector2]) {
    let mut y_min = vertices[0].y as i32;
    let mut y_max = y_min;
    for v in vertices.iter() {
        let y = v.y as i32;
        y_min = y_min.min(y);
        y_max = y_max.max(y);
    }
    for y in y_min..=y_max {
        let mut xints: Vec<i32> = Vec::new();
        let n = vertices.len();
        for i in 0..n {
            let v1 = vertices[i];
            let v2 = vertices[(i + 1) % n];
            let y1 = v1.y as i32;
            let y2 = v2.y as i32;
            if (y1 <= y && y2 > y) || (y2 <= y && y1 > y) {
                let t = (y - y1) as f32 / ((y2 - y1) as f32);
                let x = v1.x + t * (v2.x - v1.x);
                xints.push(x as i32);
            }
        }
        xints.sort_unstable();
        for pair in xints.chunks(2) {
            if let [x_start, x_end] = pair {
                for x in *x_start..=*x_end {
                    fb.point(Vector2::new(x as f32, y as f32));
                }
            }
        }
    }
}
