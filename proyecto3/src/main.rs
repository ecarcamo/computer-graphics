use crate::fragment::Fragment;
use minifb::{Key, Window, WindowOptions};
use nalgebra_glm::{Mat4, Vec3, Vec4};
use rayon::prelude::*;
use std::time::Duration;
use std::time::Instant;

mod color;
mod fragment;
mod framebuffer;
mod line;
mod obj;
mod screenshot;
mod shaders;
mod triangle;
mod vertex;

use framebuffer::Framebuffer;
use obj::Obj;
use screenshot::save_screenshot;
use shaders::vertex_shader;
use triangle::triangle;
use vertex::Vertex;

#[derive(Clone, Copy)]
pub enum PlanetShader {
    Star,      //Sol
    Rocky,     // Planeta rocoso tipo tierra
    GasGiant,  // Júpiter
    Moon,      // Luna
    Lava,      // Planeta volcánico extra
    IceGiant,  // Planeta de huelo extra
    RingRock,  // "piedritas" de los anillos
    Spaceship, // Nave importada de lab4
}

const STAR: usize = 0;
const LAVA: usize = 1;
const ROCKY: usize = 2;
const MOON: usize = 3;
const GAS: usize = 4;
const ICE: usize = 5;

pub struct Uniforms {
    model_matrix: Mat4,
    view_matrix: Mat4,
    planet_shader: PlanetShader,
    time: f32,
}

struct Camera {
    position: Vec3,
    zoom: f32,
    screen_bias: Vec3, // desplaza el centro de proyección para colocar la nave en pantalla
}

struct ShipState {
    position: Vec3,
    velocity: Vec3,
    yaw: f32,
}

impl ShipState {
    fn forward(&self) -> Vec3 {
        let (sin_yaw, cos_yaw) = self.yaw.sin_cos();
        // si ves que la nave avanza "de lado", aquí es donde se cambia el eje
        Vec3::new(cos_yaw, sin_yaw, 0.0)
    }
}

fn create_model_matrix(translation: Vec3, scale: f32, rotation: Vec3) -> Mat4 {
    let (sin_x, cos_x) = rotation.x.sin_cos();
    let (sin_y, cos_y) = rotation.y.sin_cos();
    let (sin_z, cos_z) = rotation.z.sin_cos();

    let rotation_matrix_x = Mat4::new(
        1.0, 0.0, 0.0, 0.0, 0.0, cos_x, -sin_x, 0.0, 0.0, sin_x, cos_x, 0.0, 0.0, 0.0, 0.0, 1.0,
    );

    let rotation_matrix_y = Mat4::new(
        cos_y, 0.0, sin_y, 0.0, 0.0, 1.0, 0.0, 0.0, -sin_y, 0.0, cos_y, 0.0, 0.0, 0.0, 0.0, 1.0,
    );

    let rotation_matrix_z = Mat4::new(
        cos_z, -sin_z, 0.0, 0.0, sin_z, cos_z, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0,
    );

    let rotation_matrix = rotation_matrix_z * rotation_matrix_y * rotation_matrix_x;

    let transform_matrix = Mat4::new(
        scale,
        0.0,
        0.0,
        translation.x,
        0.0,
        scale,
        0.0,
        translation.y,
        0.0,
        0.0,
        scale,
        translation.z,
        0.0,
        0.0,
        0.0,
        1.0,
    );

    transform_matrix * rotation_matrix
}

// === MATRIZ DE VISTA ORIGINAL (ajustada para seguir a la nave) ===
fn build_view_matrix_lookat(
    eye: Vec3,
    center: Vec3,
    zoom: f32,
    screen_center: Vec3,
    screen_bias: Vec3,
) -> Mat4 {
    let forward = (center - eye).normalize();
    let world_up = Vec3::new(0.0, 0.0, 1.0);
    let right = world_up.cross(&forward).normalize();
    let up = forward.cross(&right);

    let rot = Mat4::new(
        right.x, right.y, right.z, 0.0, up.x, up.y, up.z, 0.0, forward.x, forward.y, forward.z,
        0.0, 0.0, 0.0, 0.0, 1.0,
    );

    let translate = Mat4::new(
        1.0, 0.0, 0.0, -eye.x, 0.0, 1.0, 0.0, -eye.y, 0.0, 0.0, 1.0, -eye.z, 0.0, 0.0, 0.0, 1.0,
    );

    let scale = Mat4::new(
        zoom,
        0.0,
        0.0,
        screen_center.x + screen_bias.x,
        0.0,
        zoom,
        0.0,
        screen_center.y + screen_bias.y,
        0.0,
        0.0,
        zoom,
        screen_bias.z,
        0.0,
        0.0,
        0.0,
        1.0,
    );

    scale * rot * translate
}

fn render(framebuffer: &mut Framebuffer, uniforms: &Uniforms, vertex_array: &[Vertex]) {
    // 1) Vertex Shader (paralelo)
    let transformed_vertices: Vec<Vertex> = vertex_array
        .par_iter() // <-- paralelo
        .map(|vertex| vertex_shader(vertex, uniforms))
        .collect();

    // 2) Primitive Assembly (igual)
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

    // 3) Rasterización de triángulos (paralelo)
    let fragments: Vec<Fragment> = triangles
        .par_iter() // <-- cada triángulo en paralelo
        .flat_map(|tri| triangle(&tri[0], &tri[1], &tri[2]))
        .collect();

    // 4) Escribir fragmentos en el framebuffer (secuencial; aquí ya solo escribimos en memoria compartida)
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
    let screen_center = Vec3::new(window_width as f32 * 0.5, window_height as f32 * 0.5, 0.0);
    let frame_delay = Duration::from_millis(16);

    // Escalas base de los planetas
    let sun_scale = 50.0;
    let rocky_scale = 35.0;
    let gas_scale = 45.0;
    let moon_scale = 15.0;

    // Nave y cámara
    let ship_scale = 86.0;
    let mut camera_distance = 230.0;
    let camera_height = 140.0;
    let camera_pitch = 0.8;
    // Desplaza el “centro” de la pantalla hacia arriba para que la nave quede en el tercio inferior
    let screen_bias = Vec3::new(0.0, 140.0, 0.0);
    let mut ship = ShipState {
        position: Vec3::new(0.0, -220.0, 0.0),
        velocity: Vec3::new(0.0, 0.0, 0.0),
        yaw: std::f32::consts::FRAC_PI_2,
    };
    let mut camera = Camera {
        position: ship.position + Vec3::new(0.0, 0.0, camera_height),
        zoom: 1.0,
        screen_bias,
    };

    // Factores fijos para los planetas (sin teclas que alteren)
    let extra_rot = [0.0f32; 6];
    let extra_scale = [1.0f32; 6];

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

    // Geometrías
    let obj = Obj::load("assets/models/sphere.obj").expect("Failed to load sphere");
    let vertex_arrays = obj.get_vertex_array();
    let ship_obj = Obj::load("assets/models/ship.obj").expect("Failed to load ship");
    let ship_vertices = ship_obj.get_vertex_array();

    let stars = create_starfield(620, 1400.0);

    let mut screenshot_taken = false;

    while window.is_open() {
        // Tiempo y pausa (P)
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

        if !paused {
            update_ship_controls(&window, &mut ship, dt);
        }
        handle_camera_distance(&window, &mut camera_distance, dt);

        // === CÁMARA SIGUIENDO A LA NAVE (LOOK-AT) ===
        let forward = ship.forward().normalize();
        let eye = ship.position - forward * camera_distance + Vec3::new(0.0, 0.0, camera_height);
        let mut center = ship.position + forward * 200.0;
        center.z -= camera_pitch; // inclina la mirada hacia abajo
        camera.position = eye;

        let view_matrix =
            build_view_matrix_lookat(eye, center, camera.zoom, screen_center, camera.screen_bias);

        framebuffer.clear();

        draw_starfield(&mut framebuffer, &stars, &view_matrix);

        let center_pos = Vec3::new(0.0, 0.0, 0.0);
        draw_orbit_ring(&mut framebuffer, &view_matrix, 90.0);
        draw_orbit_ring(&mut framebuffer, &view_matrix, 140.0);
        draw_orbit_ring(&mut framebuffer, &view_matrix, 220.0);

        // Sol centrado
        let star_model = create_model_matrix(
            center_pos,
            sun_scale * extra_scale[STAR],
            Vec3::new(0.0, time_sec * 0.2 + extra_rot[STAR], 0.0),
        );

        let star_uniforms = Uniforms {
            model_matrix: star_model,
            view_matrix: view_matrix.clone(),
            planet_shader: PlanetShader::Star,
            time: time_sec,
        };

        render(&mut framebuffer, &star_uniforms, &vertex_arrays);

        // Planeta Lava (órbita interior)
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
            view_matrix: view_matrix.clone(),
            planet_shader: PlanetShader::Lava,
            time: time_sec,
        };

        render(&mut framebuffer, &lava_uniforms, &vertex_arrays);

        // Planeta rocoso orbitando
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
            view_matrix: view_matrix.clone(),
            planet_shader: PlanetShader::Rocky,
            time: time_sec,
        };

        render(&mut framebuffer, &rocky_uniforms, &vertex_arrays);

        // Luna
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
            view_matrix: view_matrix.clone(),
            planet_shader: PlanetShader::Moon,
            time: time_sec,
        };

        render(&mut framebuffer, &moon_uniforms, &vertex_arrays);

        // Gigante gaseoso orbitando (Júpiter)
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
            view_matrix: view_matrix.clone(),
            planet_shader: PlanetShader::GasGiant,
            time: time_sec,
        };

        render(&mut framebuffer, &gas_uniforms, &vertex_arrays);

        // Nave
        let ship_rotation = Vec3::new(0.0, 0.0, ship.yaw + std::f32::consts::FRAC_PI_2);
        let ship_model = create_model_matrix(ship.position, ship_scale, ship_rotation);
        let ship_uniforms = Uniforms {
            model_matrix: ship_model,
            view_matrix: view_matrix.clone(),
            planet_shader: PlanetShader::Spaceship,
            time: time_sec,
        };
        render(&mut framebuffer, &ship_uniforms, &ship_vertices);

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

        // Screenshot manual (O)
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

fn update_ship_controls(window: &Window, ship: &mut ShipState, dt: f32) {
    let accel = if window.is_key_down(Key::LeftShift) {
        320.0
    } else {
        200.0
    };
    let rot_speed = 1.8;

    if window.is_key_down(Key::A) {
        ship.yaw -= rot_speed * dt;
    }
    if window.is_key_down(Key::D) {
        ship.yaw += rot_speed * dt;
    }

    let mut thrust: f32 = 0.0;
    if window.is_key_down(Key::W) {
        thrust += accel;
    }
    if window.is_key_down(Key::S) {
        thrust -= accel * 0.6;
    }

    if thrust.abs() > f32::EPSILON {
        ship.velocity += ship.forward() * thrust * dt;
    }

    ship.velocity *= 0.98_f32;
    let speed = ship.velocity.norm();
    let max_speed = 260.0;
    if speed > max_speed && speed > f32::EPSILON {
        ship.velocity = ship.velocity / speed * max_speed;
    }

    ship.position += ship.velocity * dt;
}

fn handle_camera_distance(window: &Window, camera_distance: &mut f32, dt: f32) {
    let speed = 140.0;
    if window.is_key_down(Key::Up) {
        *camera_distance -= speed * dt;
    }
    if window.is_key_down(Key::Down) {
        *camera_distance += speed * dt;
    }
    *camera_distance = camera_distance.clamp(140.0, 520.0);
}

fn draw_starfield(framebuffer: &mut Framebuffer, stars: &[Vec3], view_matrix: &Mat4) {
    framebuffer.set_current_color(0x111126);
    for (i, star) in stars.iter().enumerate() {
        if let Some((sx, sy, depth)) = project_point(view_matrix, *star) {
            if sx >= 0 && sx < framebuffer.width as i32 && sy >= 0 && sy < framebuffer.height as i32
            {
                let idx = i as u32;
                let twinkle = 0x12 + ((idx.wrapping_mul(31) & 0x0F) as u8);
                let color =
                    (twinkle as u32) << 16 | (twinkle as u32) << 8 | (0x30 + (idx % 32) as u32);
                framebuffer.set_current_color(color);
                framebuffer.point(sx as usize, sy as usize, depth);
            }
        }
    }
}

fn draw_orbit_ring(framebuffer: &mut Framebuffer, view_matrix: &Mat4, radius: f32) {
    let steps = 180;
    framebuffer.set_current_color(0x35354a);
    for i in 0..steps {
        let t = i as f32 / steps as f32 * std::f32::consts::TAU;
        let world = Vec3::new(radius * t.cos(), radius * t.sin(), 0.0);
        if let Some((sx, sy, depth)) = project_point(view_matrix, world) {
            if sx >= 0 && sx < framebuffer.width as i32 && sy >= 0 && sy < framebuffer.height as i32
            {
                framebuffer.point(sx as usize, sy as usize, depth);
            }
        }
    }
}

fn project_point(view_matrix: &Mat4, p: Vec3) -> Option<(i32, i32, f32)> {
    let hp = Vec4::new(p.x, p.y, p.z, 1.0);
    let tp = view_matrix * hp;
    let w = if tp.w.abs() < 1e-5 { 1.0 } else { tp.w };
    let sx = (tp.x / w).round() as i32;
    let sy = (tp.y / w).round() as i32;
    let depth = tp.z / w;
    Some((sx, sy, depth))
}

fn create_starfield(count: usize, spread: f32) -> Vec<Vec3> {
    let mut seed = 123_987u32;
    let mut stars = Vec::with_capacity(count);
    for _ in 0..count {
        seed = seed.wrapping_mul(1664525).wrapping_add(1013904223);
        let rx = (seed as f32 / u32::MAX as f32 - 0.5) * 2.0;
        seed = seed.wrapping_mul(1664525).wrapping_add(1013904223);
        let ry = (seed as f32 / u32::MAX as f32 - 0.5) * 2.0;
        seed = seed.wrapping_mul(1664525).wrapping_add(1013904223);
        let rz = (seed as f32 / u32::MAX as f32) * -200.0;
        stars.push(Vec3::new(rx * spread, ry * spread, rz));
    }
    stars
}
