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
        PlanetShader::Star     => star_color(p, t),
        PlanetShader::Rocky    => rocky_color(p, t),
        PlanetShader::GasGiant => gas_giant_color(p, t),
        PlanetShader::Moon     => moon_color(p, t),
        PlanetShader::Lava     => lava_planet_color(p, t),
        PlanetShader::IceGiant => ice_giant_color(p, t),
        PlanetShader::RingRock => ring_rock_color(p, t),
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
    // distancia al centro
    let r = (p.x * p.x + p.y * p.y + p.z * p.z).sqrt();

    // Capa 1: gradiente radial (centro blanco-amarillo, borde naranja)
    let base = if r < 0.4 {
        Color::new(255, 250, 220) // centro muy brillante
    } else if r < 0.8 {
        Color::new(255, 220, 120) // medio amarillo
    } else {
        Color::new(255, 150, 40)  // borde más naranja
    };

    // Capa 2: “granulación” solar (manchitas claras/osc)
    let granulation = (p.x * 20.0 + time * 4.0).sin()
                    * (p.y * 24.0 - time * 3.0).cos()
                    * (p.z * 18.0 + time * 2.0).sin();
    let mut color = base;

    if granulation > 0.3 {
        // células más calientes
        color = Color::new(255, 255, 240);
    } else if granulation < -0.3 {
        // zonas ligeramente más frías
        color = Color::new(220, 130, 40);
    }

    // Capa 3: ligera “oscurecimiento de borde” (limb darkening)
    let edge_factor = (1.0 - r).clamp(0.0, 1.0); // 1 en centro, 0 en borde
    let darken = 0.6 + 0.4 * edge_factor;       // 0.6 en borde, 1 en centro

    Color::new(
        (color.r as f32 * darken) as u8,
        (color.g as f32 * darken) as u8,
        (color.b as f32 * darken) as u8,
    )
}

fn rocky_color(p: Vec3, time: f32) -> Color {
    let latitude = p.y;          // -1..1
    let longitude = p.z;         // aproximación

    // Capa 1: océanos azules
    let mut color = Color::new(20, 60, 150);

    // Capa 2: continentes (ruido senoidal)
    let continents = (p.x * 4.0 + time * 0.2).sin()
                   + (p.z * 3.0 - time * 0.1).cos();
    if continents > 0.5 {
        // tierra verde
        color = Color::new(30, 120, 40);
    }
    if continents > 1.2 {
        // montañas marrones
        color = Color::new(120, 90, 50);
    }

    // Capa 3: polos helados
    if latitude > 0.7 || latitude < -0.7 {
        color = Color::new(230, 240, 255); // blanco/azulado
    }

    // Capa 4: “nubes” suaves
    let clouds = (p.x * 7.0 + p.y * 5.0 + time * 0.4).sin();
    if clouds > 0.8 {
        // mezclar con blanco
        color = Color::new(
            ((color.r as f32 * 0.5) + 127.0) as u8,
            ((color.g as f32 * 0.5) + 127.0) as u8,
            ((color.b as f32 * 0.5) + 127.0) as u8,
        );
    }

    color
}

fn gas_giant_color(p: Vec3, time: f32) -> Color {
    let latitude = p.y; // -1..1

    // Capa 1: base crema
    let mut color = Color::new(210, 180, 140);

    // Capa 2: bandas horizontales (ancho variable)
    let band_pattern = (latitude * 10.0 + time * 0.3).sin()
                     + (latitude * 4.0).sin() * 0.5;
    if band_pattern > 0.5 {
        color = Color::new(235, 220, 200); // banda clara
    } else if band_pattern < -0.5 {
        color = Color::new(170, 120, 90);  // banda oscura
    }

    // Capa 3: turbulencias finas
    let swirl = ((p.x * 8.0 + time * 0.3).sin()
               * (p.z * 6.0 - time * 0.2).cos()).abs();
    if swirl > 0.7 {
        color = Color::new(240, 230, 215); // nubecitas muy claras
    }

    // Capa 4: Gran Mancha Roja
    let storm_center = Vec3::new(0.4, -0.2, 0.1);
    let dx = p.x - storm_center.x;
    let dy = (p.y - storm_center.y) * 0.6; // aplastada verticalmente
    let dz = p.z - storm_center.z;
    let d = (dx * dx + dy * dy + dz * dz).sqrt();

    if d < 0.35 {
        color = Color::new(200, 80, 50);
    }

    color
}



fn moon_color(p: Vec3, _time: f32) -> Color {
    let mut color = Color::new(200, 200, 200); // gris claro

    let noise = (p.x * 7.0).sin() * (p.z * 5.0).cos();
    if noise > 0.4 {
        color = Color::new(150, 150, 150);
    }

    let crater_center = Vec3::new(-0.3, 0.1, 0.2);
    let dx = p.x - crater_center.x;
    let dy = p.y - crater_center.y;
    let dz = p.z - crater_center.z;
    let d = (dx * dx + dy * dy + dz * dz).sqrt();

    if d < 0.25 {
        color = Color::new(100, 100, 100);
    }

    color
}


// Planeta volcánico (Lava)
fn lava_planet_color(p: Vec3, time: f32) -> Color {
    // Capa 1: roca oscura
    let mut color = Color::new(40, 15, 10);

    // Capa 2: zonas de lava base
    let lava_noise = (p.x * 6.0 + time * 1.0).sin().abs()
                   + (p.z * 8.0 - time * 0.8).sin().abs();
    if lava_noise > 1.3 {
        color = Color::new(200, 80, 20);
    }

    // Capa 3: grietas brillantes (lava muy caliente)
    let cracks = (p.x * 14.0 + time * 2.5).sin()
               * (p.z * 16.0 - time * 2.0).cos();
    if cracks > 0.7 {
        color = Color::new(255, 200, 80);
    }

    // Capa 4: ceniza en los polos
    if p.y > 0.7 || p.y < -0.7 {
        color = Color::new(90, 90, 90);
    }

    color
}

// Planeta de hielo (Ice Giant)
fn ice_giant_color(p: Vec3, time: f32) -> Color {
    let latitude = p.y;

    // Capa 1: azul profundo
    let mut color = Color::new(10, 30, 80);

    // Capa 2: bandas frías azul claro
    let bands = (latitude * 9.0 + time * 0.2).sin()
              + (p.z * 5.0).cos() * 0.5;
    if bands > 0.5 {
        color = Color::new(40, 130, 200);
    } else if bands < -0.5 {
        color = Color::new(20, 70, 150);
    }

    // Capa 3: “auroras” cerca de los polos
    if latitude > 0.6 || latitude < -0.6 {
        let aurora = (p.x * 15.0 + time * 1.2).sin();
        if aurora > 0.4 {
            color = Color::new(80, 220, 200);
        }
    }

    // Capa 4: puntos de hielo brillante
    let ice_spots = (p.x * 18.0 + p.y * 10.0 + time * 0.7).sin();
    if ice_spots > 0.85 {
        color = Color::new(220, 250, 255);
    }

    color
}

// “Piedras” de los anillos del gas giant
fn ring_rock_color(p: Vec3, _time: f32) -> Color {
    // Capa 1: gris base
    let mut color = Color::new(160, 150, 140);

    // Capa 2: variación de tono
    let noise = (p.x * 10.0).sin() * (p.z * 12.0).cos();
    if noise > 0.5 {
        color = Color::new(190, 180, 170);
    } else if noise < -0.5 {
        color = Color::new(120, 110, 100);
    }

    // Capa 3: manchas oscuras
    let spots = (p.x * 20.0 + p.y * 18.0).sin();
    if spots > 0.8 {
        color = Color::new(80, 70, 60);
    }

    color
}
