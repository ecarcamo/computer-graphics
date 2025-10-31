use nalgebra_glm::{Vec3, dot};
use crate::fragment::Fragment;
use crate::vertex::Vertex;
use crate::color::Color;

pub fn triangle(v1: &Vertex, v2: &Vertex, v3: &Vertex) -> Vec<Fragment> {
    let mut fragments = Vec::new();
    let (a, b, c) = (v1.transformed_position, v2.transformed_position, v3.transformed_position);

    let (min_x, min_y, max_x, max_y) = calculate_bounding_box(&a, &b, &c);

    let light_dir = Vec3::new(0.0, 0.0, -1.0);

    let avg_original_pos = (v1.position + v2.position + v3.position) / 3.0;

    let triangle_area = edge_function(&a, &b, &c);

    for y in min_y..=max_y {
        for x in min_x..=max_x {
            let point = Vec3::new(x as f32 + 0.5, y as f32 + 0.5, 0.0);

            let (w1, w2, w3) = barycentric_coordinates(&point, &a, &b, &c, triangle_area);

            if w1 >= 0.0 && w1 <= 1.0 && 
               w2 >= 0.0 && w2 <= 1.0 &&
               w3 >= 0.0 && w3 <= 1.0 {
                let normal = v1.transformed_normal;
                let normal = normal.normalize();

                let intensity1 = dot(&normal, &light_dir).max(0.0);
                
                let light_dir2 = Vec3::new(0.5, -0.5, -0.5).normalize();
                let intensity2 = dot(&normal, &light_dir2).max(0.0);
                
                let light_dir3 = Vec3::new(-0.3, 0.3, -0.5).normalize();
                let intensity3 = dot(&normal, &light_dir3).max(0.0);

                let ambient = 0.35;
                let diffuse1 = 0.5 * intensity1;
                let diffuse2 = 0.25 * intensity2;
                let diffuse3 = 0.15 * intensity3;
                let light_intensity = (ambient + diffuse1 + diffuse2 + diffuse3).min(1.2);

                let black_metal = Color::new(20, 20, 25);
                let dark_metal = Color::new(40, 40, 45);
                let dark_red = Color::new(120, 20, 20);
                let mid_red = Color::new(180, 40, 40);
                let bright_red = Color::new(220, 60, 60);
                let light_red = Color::new(255, 80, 80);
                
                let y_normalized = (avg_original_pos.y + 1.5) / 3.0;
                
                let base_color = if avg_original_pos.y < -0.8 {
                    black_metal
                } else if avg_original_pos.y < -0.5 {
                    dark_metal
                } else if avg_original_pos.y < -0.2 {
                    dark_red
                } else if avg_original_pos.y > 0.5 && avg_original_pos.z < -0.5 {
                    light_red
                } else if avg_original_pos.z > 0.3 {
                    mid_red
                } else if normal.y > 0.7 {
                    bright_red
                } else if normal.y < -0.6 {
                    dark_metal
                } else {
                    let blend_y = y_normalized.clamp(0.0, 1.0);
                    mid_red.lerp(&bright_red, blend_y)
                };
                let lit_color = base_color * light_intensity;

                let depth = a.z * w1 + b.z * w2 + c.z * w3;

                fragments.push(Fragment::new(x as f32, y as f32, lit_color, depth));
            }
        }
    }

    fragments
}

fn calculate_bounding_box(v1: &Vec3, v2: &Vec3, v3: &Vec3) -> (i32, i32, i32, i32) {
    let min_x = v1.x.min(v2.x).min(v3.x).floor() as i32;
    let min_y = v1.y.min(v2.y).min(v3.y).floor() as i32;
    let max_x = v1.x.max(v2.x).max(v3.x).ceil() as i32;
    let max_y = v1.y.max(v2.y).max(v3.y).ceil() as i32;

    (min_x, min_y, max_x, max_y)
}

fn barycentric_coordinates(p: &Vec3, a: &Vec3, b: &Vec3, c: &Vec3, area: f32) -> (f32, f32, f32) {
    let w1 = edge_function(b, c, p) / area;
    let w2 = edge_function(c, a, p) / area;
    let w3 = edge_function(a, b, p) / area;

    (w1, w2, w3)
}

fn edge_function(a: &Vec3, b: &Vec3, c: &Vec3) -> f32 {
    (c.x - a.x) * (b.y - a.y) - (c.y - a.y) * (b.x - a.x)
}
