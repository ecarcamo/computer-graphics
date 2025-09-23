mod aabb;
mod camera;
mod object;
mod ray;
mod renderer;
mod shading;
mod textured_aabb;
mod vec3;

use camera::Camera;
use raylib::prelude::*;
use renderer::{render, Assets};
use shading::{Skybox, Tex};
use std::f32::consts::PI;
use vec3::Vec3;

// Cargar imagen como RGBA8 bytes + dims
fn load_texture_rgba(path: &str) -> Option<(Vec<u8>, u32, u32)> {
    if let Ok(img) = image::open(path) {
        let rgba = img.to_rgba8();
        let (w, h) = rgba.dimensions();
        Some((rgba.into_raw(), w, h))
    } else {
        None
    }
}

fn main() {
    let (fb_w, fb_h) = (800, 600);
    let (mut rl, thread) = raylib::init()
        .size(fb_w, fb_h)
        .title("Raytracer CPU + Raylib (Skyblock)")
        .build();

    let img = Image::gen_image_color(fb_w, fb_h, Color::BLACK);
    let mut tex2d = rl.load_texture_from_image(&thread, &img).expect("texture");

    // --- Texturas para la isla ---
    let grass   = load_texture_rgba("assets/cesped.jpg")
                    .or_else(|| load_texture_rgba("assets/pasto.jpg"));
    let dirt    = load_texture_rgba("assets/tierra.jpg");

    // Mantener buffers vivos y crear Tex
    let (mut grass_buf, mut grass_wh) = (None, (0u32,0u32));
    let (mut dirt_buf,  mut dirt_wh)  = (None, (0u32,0u32));

    if let Some((b,w,h)) = grass { grass_buf = Some(b); grass_wh = (w,h); }
    if let Some((b,w,h)) = dirt  { dirt_buf  = Some(b); dirt_wh  = (w,h); }

    let grass_tex = grass_buf.as_ref().map(|b| Tex{ pix: &b[..], w: grass_wh.0, h: grass_wh.1 });
    let dirt_tex  = dirt_buf.as_ref().map(|b|  Tex{ pix: &b[..], w: dirt_wh.0,  h: dirt_wh.1  });

    // --- Skybox opcional (6 caras) ---
    let mut sky_imgs: Option<Vec<(Vec<u8>, u32, u32)>> = {
        let names = ["px","nx","py","ny","pz","nz"];
        let mut acc: Vec<(Vec<u8>,u32,u32)> = Vec::new();
        let mut ok = true;
        for n in names {
            let p = format!("assets/skybox/{}.jpg", n);
            if let Some(t) = load_texture_rgba(&p) { acc.push(t); } else { ok = false; break; }
        }
        if ok { Some(acc) } else { None }
    };

    // Cámara orbital y luz
    let mut yaw: f32 = 0.6;
    let mut pitch: f32 = 0.25;
    let mut radius: f32 = 4.0;              // un poco más lejos para ver la isla
    let mut light_pos = Vec3::new(2.5, 3.0, 2.5);

    let mut frame = vec![0u8; (fb_w * fb_h * 4) as usize];

    while !rl.window_should_close() {
        let dt = rl.get_frame_time();
        let speed = 1.6;
        if rl.is_key_down(KeyboardKey::KEY_LEFT)  { yaw   -= speed * dt; }
        if rl.is_key_down(KeyboardKey::KEY_RIGHT) { yaw   += speed * dt; }
        if rl.is_key_down(KeyboardKey::KEY_UP)    { pitch -= speed * dt; }
        if rl.is_key_down(KeyboardKey::KEY_DOWN)  { pitch += speed * dt; }
        if rl.is_key_down(KeyboardKey::KEY_Q)     { radius = (radius - 1.5 * dt).max(1.2); }
        if rl.is_key_down(KeyboardKey::KEY_E)     { radius += 1.5 * dt; }
        if rl.is_key_down(KeyboardKey::KEY_A)     { light_pos.x -= 2.0 * dt; }
        if rl.is_key_down(KeyboardKey::KEY_D)     { light_pos.x += 2.0 * dt; }
        if rl.is_key_down(KeyboardKey::KEY_W)     { light_pos.y += 2.0 * dt; }
        if rl.is_key_down(KeyboardKey::KEY_S)     { light_pos.y -= 2.0 * dt; }
        pitch = pitch.clamp(-PI * 0.49, PI * 0.49);

        let eye = Vec3::new(
            radius * yaw.sin() * pitch.cos(),
            radius * pitch.sin(),
            radius * yaw.cos() * pitch.cos(),
        );
        let cam = Camera {
            eye,
            target: Vec3::new(1.0, 0.0, 1.0), // centra la órbita sobre la isla (aprox en (1,0,1))
            up: Vec3::new(0.0, 1.0, 0.0),
            fov_y: 60.0,
        };

        // Vista del skybox (si existe)
        let skybox_view = sky_imgs.as_ref().map(|v| Skybox {
            px: Tex { pix: &v[0].0[..], w: v[0].1, h: v[0].2 },
            nx: Tex { pix: &v[1].0[..], w: v[1].1, h: v[1].2 },
            py: Tex { pix: &v[2].0[..], w: v[2].1, h: v[2].2 },
            ny: Tex { pix: &v[3].0[..], w: v[3].1, h: v[3].2 },
            pz: Tex { pix: &v[4].0[..], w: v[4].1, h: v[4].2 },
            nz: Tex { pix: &v[5].0[..], w: v[5].1, h: v[5].2 },
        });

        let assets = Assets { grass: grass_tex, dirt: dirt_tex, skybox: skybox_view };

        render(&mut frame, fb_w, fb_h, &cam, light_pos, assets, 3);

        tex2d.update_texture(&frame);
        let mut d = rl.begin_drawing(&thread);
        d.clear_background(Color::BLACK);
        d.draw_texture(&tex2d, 0, 0, Color::WHITE);
        d.draw_text("Flechas: orbitar | Q/E: zoom | WASD: luz", 12, 12, 20, Color::WHITE);
    }
}
