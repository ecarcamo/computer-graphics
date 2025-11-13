use nalgebra_glm::{Vec3, Mat4};
use minifb::{Key, Window, WindowOptions};
use std::time::Duration;
use std::f32::consts::PI;
use std::time::Instant;
use rayon::prelude::*;
use crate::fragment::Fragment;

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

const STAR: usize  = 0;
const LAVA: usize  = 1;
const ROCKY: usize = 2;
const MOON: usize  = 3;
const GAS: usize   = 4;
const ICE: usize   = 5;


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
    // ============================
    // 1) Vertex Shader (paralelo)
    // ============================
    let transformed_vertices: Vec<Vertex> = vertex_array
        .par_iter()                              // <-- paralelo
        .map(|vertex| vertex_shader(vertex, uniforms))
        .collect();

    // ================================
    // 2) Primitive Assembly (igual)
    // ================================
    let mut triangles: Vec<[Vertex; 3]> = Vec::new();
    for i in (0..transformed_vertices.len()).step_by(3) {
        if i + 2 < transformed_vertices.len() {
            triangles.push([
                transformed_vertices[i].clone(),
                transformed_vertices[i + 1].clone(),
                transformed_vertices[i + 2].clone(),
            ]);
        }
    }

    // ========================================
    // 3) Rasterización de triángulos (paralelo)
    // ========================================
    let fragments: Vec<Fragment> = triangles
        .par_iter()   // <-- cada triángulo en paralelo
        .flat_map(|tri| triangle(&tri[0], &tri[1], &tri[2]))
        .collect();

    // ========================================
    // 4) Escribir fragmentos en el framebuffer
    //    (secuencial; aquí ya es solo escribir
    //    en memoria compartida)
    // ========================================
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
    let window_width = 900;
    let window_height = 600;
    let framebuffer_width = 900;
    let framebuffer_height = 600;
    let frame_delay = Duration::from_millis(16);

    // Escalas base de los planetas
    let sun_scale   = 50.0;
    let rocky_scale = 35.0;
    let gas_scale   = 45.0;
    let moon_scale  = 15.0;

    // Offset global del sistema (para mover con WASD)
    let mut center_offset = Vec3::new(0.0, 0.0, 0.0);

    // Controles por planeta
    let mut selected_planet: usize = STAR;     // 0 por defecto (sol)
    let mut extra_rot   = [0.0f32; 6];         // rotación extra en Y
    let mut extra_scale = [1.0f32; 6];         // escala multiplicativa

    // Tiempo acumulado
    let mut base_time = 0.0f32;
    let mut last_instant = Instant::now();

    // Toggle de pausa (P)
    let mut paused = false;
    let mut last_p_state = false;

    // Framebuffer y ventana
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

    // Cargamos la esfera
    let obj = Obj::load("assets/models/sphere.obj").expect("Failed to load sphere");
    let vertex_arrays = obj.get_vertex_array();

    // Centro base de la “cámara”
    let center = Vec3::new(450.0, 300.0, 0.0);

    let mut screenshot_taken = false;

    while window.is_open() {
        // ===================== TIEMPO + PAUSA (P) ======================
        let now = Instant::now();
        let dt = now.duration_since(last_instant).as_secs_f32();
        last_instant = now;

        // edge detection de la tecla P (toggle)
        let p_down = window.is_key_down(Key::P);
        if p_down && !last_p_state {
            paused = !paused;
            println!("PAUSA = {}", paused);
        }
        last_p_state = p_down;

        if !paused {
            base_time += dt;
        }
        let time_sec = base_time;

        // Salir con ESC
        if window.is_key_down(Key::Escape) {
            break;
        }

        // ===================== CONTROLES DE ENTRADA ======================
        handle_input(
            &window,
            &mut center_offset,
            &mut selected_planet,
            &mut extra_rot,
            &mut extra_scale,
        );

        let center_pos = center + center_offset;

        framebuffer.clear();

        // ====================================================
        // ===============    SOL (CENTRADO)    ================
        // ====================================================
        let star_model = create_model_matrix(
            center_pos,
            sun_scale * extra_scale[STAR],
            Vec3::new(0.0, time_sec * 0.2 + extra_rot[STAR], 0.0),
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

        let lava_x = center_pos.x + lava_orbit_radius * lava_angle.cos();
        let lava_y = center_pos.y + lava_orbit_radius * lava_angle.sin();

        let lava_model = create_model_matrix(
            Vec3::new(lava_x, lava_y, 0.0),
            25.0 * extra_scale[LAVA],
            Vec3::new(0.0, time_sec * 0.9 + extra_rot[LAVA], 0.0),
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

        let rocky_x = center_pos.x + rocky_orbit_radius * rocky_angle.cos();
        let rocky_y = center_pos.y + rocky_orbit_radius * rocky_angle.sin();

        let rocky_model = create_model_matrix(
            Vec3::new(rocky_x, rocky_y, 0.0),
            rocky_scale * extra_scale[ROCKY],
            Vec3::new(0.0, time_sec * 0.6 + extra_rot[ROCKY], 0.0),
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
            moon_scale * extra_scale[MOON],
            Vec3::new(0.0, time_sec * 0.8 + extra_rot[MOON], 0.0),
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

        let gas_x = center_pos.x + gas_orbit * gas_angle.cos();
        let gas_y = center_pos.y + gas_orbit * gas_angle.sin();

        let gas_model = create_model_matrix(
            Vec3::new(gas_x, gas_y, 0.0),
            gas_scale * extra_scale[GAS],
            Vec3::new(0.0, time_sec * 0.3 + extra_rot[GAS], 0.0),
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
        let ring_radius = gas_scale * 2.1 * extra_scale[GAS];
        let ring_scale = 5.0 * extra_scale[GAS];
        let ring_count = 12;

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

        let ice_x = center_pos.x + ice_orbit_radius * ice_angle.cos();
        let ice_y = center_pos.y + ice_orbit_radius * ice_angle.sin();

        let ice_model = create_model_matrix(
            Vec3::new(ice_x, ice_y, 0.0),
            40.0 * extra_scale[ICE],
            Vec3::new(0.0, time_sec * 0.35 + extra_rot[ICE], 0.0),
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

        // Captura automática solo una vez
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

        // ===================== SCREENSHOT MANUAL (O) =====================
        if window.is_key_down(Key::O) {
            let filename = format!("screenshot_{}.png", base_time as u32);
            save_screenshot(
                &framebuffer.buffer,
                framebuffer_width,
                framebuffer_height,
                &filename,
            );
            println!("Screenshot guardado: {}", filename);
            std::thread::sleep(Duration::from_millis(200)); // evita repetir muchas capturas
        }

    }
}


fn handle_input(
    window: &Window,
    center_offset: &mut Vec3,
    selected_planet: &mut usize,
    extra_rot: &mut [f32; 6],
    extra_scale: &mut [f32; 6],
) {
    // ===== mover todo el sistema con WASD =====
    let move_step = 8.0;
    if window.is_key_down(Key::D) {
        center_offset.x += move_step;
    }
    if window.is_key_down(Key::A) {
        center_offset.x -= move_step;
    }
    if window.is_key_down(Key::W) {
        center_offset.y -= move_step;
    }
    if window.is_key_down(Key::S) {
        center_offset.y += move_step;
    }

    // ===== seleccionar planeta con números 1-6 =====
    if window.is_key_down(Key::Key1) {
        *selected_planet = STAR;
    }
    if window.is_key_down(Key::Key2) {
        *selected_planet = LAVA;
    }
    if window.is_key_down(Key::Key3) {
        *selected_planet = ROCKY;
    }
    if window.is_key_down(Key::Key4) {
        *selected_planet = MOON;
    }
    if window.is_key_down(Key::Key5) {
        *selected_planet = GAS;
    }
    if window.is_key_down(Key::Key6) {
        *selected_planet = ICE;
    }

    // ===== rotar planeta seleccionado con Z / X =====
    let rot_step = 0.05;
    if window.is_key_down(Key::Z) {
        extra_rot[*selected_planet] -= rot_step;
    }
    if window.is_key_down(Key::X) {
        extra_rot[*selected_planet] += rot_step;
    }

    // ===== escalar planeta seleccionado con C / V =====
    let scale_step = 0.02;
    if window.is_key_down(Key::C) {
        extra_scale[*selected_planet] *= 1.0 + scale_step;
    }
    if window.is_key_down(Key::V) {
        extra_scale[*selected_planet] *= 1.0 - scale_step;
    }

    // Clamp para que no exploten
    for s in extra_scale.iter_mut() {
        if *s < 0.2 {
            *s = 0.2;
        }
        if *s > 3.0 {
            *s = 3.0;
        }
    }
}
