mod aabb;
mod camera;
mod object;
mod plane; // Nuevo módulo
mod ray;
mod renderer;
mod shading;
mod textured_aabb;
mod vec3; // Nuevo módulo para el trait // nuevo

use camera::Camera;
use raylib::prelude::*;
use renderer::render;
use std::f32::consts::PI;
use vec3::Vec3;

// Añadir: cargar textura usando crate image
fn try_load_texture(path: &str) -> Option<(Vec<u8>, u32, u32)> {
    if let Ok(img) = image::open(path) {
        let rgba = img.to_rgba8();
        let (w, h) = rgba.dimensions();
        Some((rgba.into_raw(), w, h))
    } else {
        None
    }
}

fn main() {
    // Tamaño del framebuffer (y ventana)
    let (fb_w, fb_h) = (800, 600);
    // Cargar textura (si existe)
    let tex = try_load_texture("assets/diamante.jpg");
    // tex ahora es Option<(Vec<u8>, w, h)>

    // Inicializar raylib
    let (mut rl, thread) = raylib::init()
        .size(fb_w, fb_h)
        .title("Raytracer + raylib (cubo difuso)")
        .build();

    // Texture2D inicial (vacía)
    let img = Image::gen_image_color(fb_w, fb_h, Color::BLACK);
    let mut tex2d = rl.load_texture_from_image(&thread, &img).expect("texture");

    // Parámetros de cámara orbital
    let mut yaw: f32 = 0.6;
    let mut pitch: f32 = 0.25;
    let mut radius: f32 = 3.0;
    let mut light_pos = Vec3::new(2.2, 2.5, 2.0);

    // Framebuffer RGBA8
    let mut frame = vec![0u8; (fb_w * fb_h * 4) as usize];

    while !rl.window_should_close() {
        // Controles
        let dt = rl.get_frame_time();
        let speed = 1.6;
        if rl.is_key_down(KeyboardKey::KEY_LEFT) {
            yaw -= speed * dt;
        }
        if rl.is_key_down(KeyboardKey::KEY_RIGHT) {
            yaw += speed * dt;
        }
        if rl.is_key_down(KeyboardKey::KEY_UP) {
            pitch -= speed * dt;
        }
        if rl.is_key_down(KeyboardKey::KEY_DOWN) {
            pitch += speed * dt;
        }
        if rl.is_key_down(KeyboardKey::KEY_Q) {
            radius = (radius - 1.5 * dt).max(1.2);
        }
        if rl.is_key_down(KeyboardKey::KEY_E) {
            radius += 1.5 * dt;
        }
        if rl.is_key_down(KeyboardKey::KEY_A) {
            light_pos.x -= 2.0 * dt;
        }
        if rl.is_key_down(KeyboardKey::KEY_D) {
            light_pos.x += 2.0 * dt;
        }
        if rl.is_key_down(KeyboardKey::KEY_W) {
            light_pos.y += 2.0 * dt;
        }
        if rl.is_key_down(KeyboardKey::KEY_S) {
            light_pos.y -= 2.0 * dt;
        }

        pitch = pitch.clamp(-PI * 0.49, PI * 0.49);

        // Cámara look-at (orbital alrededor del origen)
        let eye = Vec3::new(
            radius * yaw.sin() * pitch.cos(),
            radius * pitch.sin(),
            radius * yaw.cos() * pitch.cos(),
        );
        let cam = Camera {
            eye,
            target: Vec3::new(0.0, 0.0, 0.0),
            up: Vec3::new(0.0, 1.0, 0.0),
            fov_y: 60.0,
        };

        // Preparar parámetro de textura para render
        let textured_param = tex.as_ref().map(|(buf, w, h)| (buf.as_slice(), *w, *h));

        // Render CPU -> framebuffer
        render(&mut frame, fb_w, fb_h, &cam, light_pos, textured_param);

        // Subir framebuffer a la textura y dibujar
        tex2d.update_texture(&frame);
        let mut d = rl.begin_drawing(&thread);
        d.clear_background(Color::BLACK);
        d.draw_texture(&tex2d, 0, 0, Color::WHITE);
        d.draw_text(
            "Flechas: orbitar | Q/E: zoom | WASD: luz",
            12,
            12,
            20,
            Color::WHITE,
        );
    }
}
