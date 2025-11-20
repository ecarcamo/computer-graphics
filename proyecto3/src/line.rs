use crate::fragment::Fragment;
use crate::vertex::Vertex;

// Línea básica (no usada, pero se conserva para compatibilidad)
pub fn line(a: &Vertex, b: &Vertex) -> Vec<Fragment> {
    let mut fragments = Vec::new();
    let ax = a.transformed_position.x as i32;
    let ay = a.transformed_position.y as i32;
    let bx = b.transformed_position.x as i32;
    let by = b.transformed_position.y as i32;

    let dx = (bx - ax).abs();
    let dy = -(by - ay).abs();
    let sx = if ax < bx { 1 } else { -1 };
    let sy = if ay < by { 1 } else { -1 };
    let mut err = dx + dy;
    let mut x = ax;
    let mut y = ay;

    while x != bx || y != by {
        fragments.push(Fragment::new(
            x as f32,
            y as f32,
            a.color,
            a.transformed_position.z,
        ));
        let e2 = 2 * err;
        if e2 >= dy {
            err += dy;
            x += sx;
        }
        if e2 <= dx {
            err += dx;
            y += sy;
        }
    }
    fragments.push(Fragment::new(
        bx as f32,
        by as f32,
        b.color,
        b.transformed_position.z,
    ));
    fragments
}
