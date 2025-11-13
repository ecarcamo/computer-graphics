use nalgebra_glm::{Vec3, Vec4, Mat3};
use crate::vertex::Vertex;
use crate::Uniforms;
use crate::color::Color;
use crate::PlanetShader;

pub fn vertex_shader(vertex: &Vertex, uniforms: &Uniforms) -> Vertex {

    let position = Vec4::new(
        vertex.position.x,
        vertex.position.y,
        vertex.position.z,
        1.0
    );
    let transformed = uniforms.model_matrix * position;

    let w = if transformed.w.abs() < 1e-5 { 1.0 } else { transformed.w };
    let transformed_position = Vec3::new(
        transformed.x / w,
        transformed.y / w,
        transformed.z / w
    );

    let model_mat3 = Mat3::new(
        uniforms.model_matrix[0], uniforms.model_matrix[1], uniforms.model_matrix[2],
        uniforms.model_matrix[4], uniforms.model_matrix[5], uniforms.model_matrix[6],
        uniforms.model_matrix[8], uniforms.model_matrix[9], uniforms.model_matrix[10]
    );
    let normal_matrix = model_mat3.transpose().try_inverse().unwrap_or(Mat3::identity());
    let transformed_normal = normal_matrix * vertex.normal;

    let p = vertex.position; // posición original en la esfera (−1..1)
    let t = uniforms.time;

    let color = match uniforms.planet_shader {
        PlanetShader::Star => star_color(p, t),
        PlanetShader::Rocky => rocky_color(p, t),
        PlanetShader::GasGiant => gas_giant_color(p, t),
    };


    Vertex {
        position: vertex.position,
        normal: vertex.normal,
        tex_coords: vertex.tex_coords,
        color,
        transformed_position,
        transformed_normal,
    }
}

/// "Sol" con varias capas de color
fn star_color(p: Vec3, time: f32) -> Color {
    let r = (p.x * p.x + p.y * p.y + p.z * p.z).sqrt();

    let base = if r < 0.4 {
        Color::new(255, 255, 230) // casi blanco en el centro
    } else if r < 0.8 {
        Color::new(255, 220, 120) // amarillito
    } else {
        Color::new(255, 120, 40)  // borde naranja/rojizo
    };

    let swirl = ((p.x * 8.0 + time * 2.0).sin()
               + (p.y * 10.0 - time * 1.5).cos()
               + (p.z * 12.0 + time).sin()) * 0.5;

    if swirl > 0.4 {
        Color::new(255, 255, 255)
    } else if swirl < -0.4 {
        Color::new(200, 70, 20)
    } else {
        base
    }
}

fn rocky_color(p: Vec3, time: f32) -> Color {
    let mut color = Color::new(120, 70, 40); // marrón roca

    let latitude = p.y; // -1 a 1
    if latitude.abs() > 0.6 {
        color = Color::new(200, 180, 160);
    }

    let bands = (p.x * 4.0 + time * 0.3).sin() + (p.z * 3.0).cos();
    if bands > 1.0 {
        color = Color::new(170, 100, 60);
    }

    let crater_center = Vec3::new(0.4, 0.1, 0.0);
    let dx = p.x - crater_center.x;
    let dy = p.y - crater_center.y;
    let dz = p.z - crater_center.z;
    let d = (dx * dx + dy * dy + dz * dz).sqrt();

    if d < 0.35 {
        color = Color::new(60, 40, 30);
    }

    color
}

fn gas_giant_color(p: Vec3, time: f32) -> Color {
    let latitude = p.y; // -1..1

    let mut color = Color::new(200, 160, 120);

    let bands = (latitude * 10.0 + time * 0.5).sin(); // [-1,1]
    if bands > 0.0 {
        color = Color::new(230, 210, 180); // banda clara
    } else {
        color = Color::new(160, 120, 90);  // banda oscura
    }

    let swirl = ((p.x * 6.0 + time * 0.3).sin() * (p.z * 6.0 - time * 0.4).cos()).abs();
    if swirl > 0.6 {
        color = Color::new(240, 230, 210); // nubecitas claras
    }

    let storm_center = Vec3::new(0.3, -0.2, 0.0);
    let dx = p.x - storm_center.x;
    let dy = p.y - storm_center.y;
    let dz = p.z - storm_center.z;
    let d = (dx * dx + (dy * 0.5) * (dy * 0.5) + dz * dz).sqrt(); // un poco “aplastada” en Y

    if d < 0.4 {
        color = Color::new(200, 80, 50);
    }

    color
}
