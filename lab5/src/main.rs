use nalgebra_glm::{Vec3, Mat4};
use minifb::{Key, Window, WindowOptions};
use std::time::Duration;
use std::f32::consts::PI;
use std::time::Instant;

mod framebuffer;
mod triangle;
mod line;
mod vertex;
mod obj;
mod color;
mod fragment;
mod shaders;
mod screenshot;

use screenshot::save_screenshot;
use framebuffer::Framebuffer;
use vertex::Vertex;
use obj::Obj;
use triangle::triangle;
use shaders::vertex_shader;

#[derive(Clone, Copy)]
pub enum PlanetShader {
    Star,       //Sol
    Rocky,      // Planeta rocoso tipo tierra
    GasGiant,   // Júpiter
    Moon,       // Luna
    Lava,       // PLantea volcánico extra
    IceGiant,   // Planeta de huelo extra
    RingRock,   // "piedritas" de los anillos
}


pub struct Uniforms {
    model_matrix: Mat4,
    planet_shader: PlanetShader,
    time: f32,
}


fn create_model_matrix(translation: Vec3, scale: f32, rotation: Vec3) -> Mat4 {
    let (sin_x, cos_x) = rotation.x.sin_cos();
    let (sin_y, cos_y) = rotation.y.sin_cos();
    let (sin_z, cos_z) = rotation.z.sin_cos();

    let rotation_matrix_x = Mat4::new(
        1.0,  0.0,    0.0,   0.0,
        0.0,  cos_x, -sin_x, 0.0,
        0.0,  sin_x,  cos_x, 0.0,
        0.0,  0.0,    0.0,   1.0,
    );

    let rotation_matrix_y = Mat4::new(
        cos_y,  0.0,  sin_y, 0.0,
        0.0,    1.0,  0.0,   0.0,
        -sin_y, 0.0,  cos_y, 0.0,
        0.0,    0.0,  0.0,   1.0,
    );

    let rotation_matrix_z = Mat4::new(
        cos_z, -sin_z, 0.0, 0.0,
        sin_z,  cos_z, 0.0, 0.0,
        0.0,    0.0,  1.0, 0.0,
        0.0,    0.0,  0.0, 1.0,
    );

    let rotation_matrix = rotation_matrix_z * rotation_matrix_y * rotation_matrix_x;

    let transform_matrix = Mat4::new(
        scale, 0.0,   0.0,   translation.x,
        0.0,   scale, 0.0,   translation.y,
        0.0,   0.0,   scale, translation.z,
        0.0,   0.0,   0.0,   1.0,
    );

    transform_matrix * rotation_matrix
}

fn render(framebuffer: &mut Framebuffer, uniforms: &Uniforms, vertex_array: &[Vertex]) {
    // Vertex Shader Stage
    let mut transformed_vertices = Vec::with_capacity(vertex_array.len());
    for vertex in vertex_array {
        let transformed = vertex_shader(vertex, uniforms);
        transformed_vertices.push(transformed);
    }

    // Primitive Assembly Stage
    let mut triangles = Vec::new();
    for i in (0..transformed_vertices.len()).step_by(3) {
        if i + 2 < transformed_vertices.len() {
            triangles.push([
                transformed_vertices[i].clone(),
                transformed_vertices[i + 1].clone(),
                transformed_vertices[i + 2].clone(),
            ]);
        }
    }

    // Rasterization Stage
    let mut fragments = Vec::new();
    for tri in &triangles {
        fragments.extend(triangle(&tri[0], &tri[1], &tri[2]));
    }

    // Fragment Processing Stage
    for fragment in fragments {
        let x = fragment.position.x as usize;
        let y = fragment.position.y as usize;
        if x < framebuffer.width && y < framebuffer.height {
            let color = fragment.color.to_hex();
            framebuffer.set_current_color(color);
            framebuffer.point(x, y, fragment.depth);
        }
    }
}

fn main() {

    let window_width = 800;
    let window_height = 600;
    let framebuffer_width = 800;
    let framebuffer_height = 600;
    let frame_delay = Duration::from_millis(16);

    let mut framebuffer = Framebuffer::new(framebuffer_width, framebuffer_height);
    let mut window = Window::new(
        "Rust Graphics - Lab 5 - Solar System",
        window_width,
        window_height,
        WindowOptions::default(),
    )
    .unwrap();

    window.set_position(500, 300);
    window.update();

    framebuffer.set_background_color(0x030314);

    // Para compatibilidad con tu manejador de input
    let mut translation = Vec3::new(0.0, 0.0, 0.0);
    let mut rotation = Vec3::new(0.0, 0.0, 0.0);
    let mut scale = 100.0f32;

    // Cargamos la esfera
    let obj = Obj::load("assets/models/sphere.obj").expect("Failed to load sphere");
    let vertex_arrays = obj.get_vertex_array();
    let start_time = Instant::now();

    // Centro de la “cámara”
    let center = Vec3::new(400.0, 300.0, 0.0);

    // Escalas reducidas (zoom out)
    let sun_scale   = 50.0;
    let rocky_scale = 35.0;
    let gas_scale   = 45.0;
    let moon_scale  = 15.0;

    let mut screenshot_taken = false;

    while window.is_open() {
        let elapsed = start_time.elapsed();
        let time_sec = elapsed.as_secs_f32();

        if window.is_key_down(Key::Escape) {
            break;
        }

        handle_input(&window, &mut translation, &mut rotation, &mut scale);

        framebuffer.clear();

        // ====================================================
        // ===============    SOL (CENTRADO)    ================
        // ====================================================
        let star_model = create_model_matrix(
            center,
            sun_scale,
            Vec3::new(0.0, time_sec * 0.2, 0.0),
        );

        let star_uniforms = Uniforms {
            model_matrix: star_model,
            planet_shader: PlanetShader::Star,
            time: time_sec,
        };

        render(&mut framebuffer, &star_uniforms, &vertex_arrays);

        // ====================================================
        // ======== PLANETA EXTRA 1: LAVA (ÓRBITA INTERIOR) ===
        // ====================================================
        let lava_orbit_radius = 90.0;
        let lava_angle = time_sec * 0.7;

        let lava_x = center.x + lava_orbit_radius * lava_angle.cos();
        let lava_y = center.y + lava_orbit_radius * lava_angle.sin();

        let lava_model = create_model_matrix(
            Vec3::new(lava_x, lava_y, 0.0),
            25.0,
            Vec3::new(0.0, time_sec * 0.9, 0.0),
        );

        let lava_uniforms = Uniforms {
            model_matrix: lava_model,
            planet_shader: PlanetShader::Lava,
            time: time_sec,
        };

        render(&mut framebuffer, &lava_uniforms, &vertex_arrays);

        // ====================================================
        // =========== PLANETA ROCOSO ORBITANDO ===============
        // ====================================================
        let rocky_orbit_radius = 140.0;
        let rocky_angle = time_sec * 0.4;

        let rocky_x = center.x + rocky_orbit_radius * rocky_angle.cos();
        let rocky_y = center.y + rocky_orbit_radius * rocky_angle.sin();

        let rocky_model = create_model_matrix(
            Vec3::new(rocky_x, rocky_y, 0.0),
            rocky_scale,
            Vec3::new(0.0, time_sec * 0.6, 0.0),
        );

        let rocky_uniforms = Uniforms {
            model_matrix: rocky_model,
            planet_shader: PlanetShader::Rocky,
            time: time_sec,
        };

        render(&mut framebuffer, &rocky_uniforms, &vertex_arrays);

        // ====================================================
        // ====================   LUNA   =======================
        // ====================================================
        let moon_orbit = rocky_scale * 2.0;
        let moon_angle = time_sec * 1.2;

        let moon_x = rocky_x + moon_orbit * moon_angle.cos();
        let moon_y = rocky_y + moon_orbit * moon_angle.sin();

        let moon_model = create_model_matrix(
            Vec3::new(moon_x, moon_y, 0.0),
            moon_scale,
            Vec3::new(0.0, time_sec * 0.8, 0.0),
        );

        let moon_uniforms = Uniforms {
            model_matrix: moon_model,
            planet_shader: PlanetShader::Moon,
            time: time_sec,
        };

        render(&mut framebuffer, &moon_uniforms, &vertex_arrays);

        // ====================================================
        // ========= GIGANTE GASEOSO ORBITANDO (JÚPITER) ======
        // ====================================================
        let gas_orbit = 220.0;
        let gas_angle = time_sec * 0.25;

        let gas_x = center.x + gas_orbit * gas_angle.cos();
        let gas_y = center.y + gas_orbit * gas_angle.sin();

        let gas_model = create_model_matrix(
            Vec3::new(gas_x, gas_y, 0.0),
            gas_scale,
            Vec3::new(0.0, time_sec * 0.3, 0.0),
        );

        let gas_uniforms = Uniforms {
            model_matrix: gas_model,
            planet_shader: PlanetShader::GasGiant,
            time: time_sec,
        };

        render(&mut framebuffer, &gas_uniforms, &vertex_arrays);

        // ====================================================
        // ============ SISTEMA DE ANILLOS (ROCAS) ============
        // ====================================================
        let ring_radius = gas_scale * 2.1;
        let ring_scale = 6.0;
        let ring_count = 32;

        for i in 0..ring_count {
            let angle = (i as f32 / ring_count as f32) * 2.0 * PI + time_sec * 0.15;

            let ring_x = gas_x + ring_radius * angle.cos();
            let ring_y = gas_y + ring_radius * angle.sin();

            let ring_model = create_model_matrix(
                Vec3::new(ring_x, ring_y, 0.0),
                ring_scale,
                Vec3::new(0.0, 0.0, 0.0),
            );

            let ring_uniforms = Uniforms {
                model_matrix: ring_model,
                planet_shader: PlanetShader::RingRock,
                time: time_sec,
            };

            render(&mut framebuffer, &ring_uniforms, &vertex_arrays);
        }

        // ====================================================
        // ====== PLANETA EXTRA 2: ICE GIANT (ÓRBITA EXTERIOR)
        // ====================================================
        let ice_orbit_radius = 300.0;
        let ice_angle = time_sec * 0.18;

        let ice_x = center.x + ice_orbit_radius * ice_angle.cos();
        let ice_y = center.y + ice_orbit_radius * ice_angle.sin();

        let ice_model = create_model_matrix(
            Vec3::new(ice_x, ice_y, 0.0),
            40.0,
            Vec3::new(0.0, time_sec * 0.35, 0.0),
        );

        let ice_uniforms = Uniforms {
            model_matrix: ice_model,
            planet_shader: PlanetShader::IceGiant,
            time: time_sec,
        };

        render(&mut framebuffer, &ice_uniforms, &vertex_arrays);

        // ACTUALIZAR VENTANA
        window
            .update_with_buffer(&framebuffer.buffer, framebuffer_width, framebuffer_height)
            .unwrap();

        if !screenshot_taken {
            save_screenshot(
                &framebuffer.buffer,
                framebuffer_width,
                framebuffer_height,
                "planetas.png",
            );
            screenshot_taken = true;
            println!("Captura guardada como planetas.png");
        }
        std::thread::sleep(frame_delay);
    }

}



fn handle_input(window: &Window, translation: &mut Vec3, rotation: &mut Vec3, scale: &mut f32) {
    if window.is_key_down(Key::Right) {
        translation.x += 10.0;
    }
    if window.is_key_down(Key::Left) {
        translation.x -= 10.0;
    }
    if window.is_key_down(Key::Up) {
        translation.y -= 10.0;
    }
    if window.is_key_down(Key::Down) {
        translation.y += 10.0;
    }
    if window.is_key_down(Key::S) {
        *scale += 2.0;
    }
    if window.is_key_down(Key::A) {
        *scale -= 2.0;
    }
    if window.is_key_down(Key::Q) {
        rotation.x -= PI / 10.0;
    }
    if window.is_key_down(Key::W) {
        rotation.x += PI / 10.0;
    }
    if window.is_key_down(Key::E) {
        rotation.y -= PI / 10.0;
    }
    if window.is_key_down(Key::R) {
        rotation.y += PI / 10.0;
    }
    if window.is_key_down(Key::T) {
        rotation.z -= PI / 10.0;
    }
    if window.is_key_down(Key::Y) {
        rotation.z += PI / 10.0;
    }
}