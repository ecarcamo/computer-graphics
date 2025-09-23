mod aabb;
mod camera;
mod object;
mod plane;
mod ray;
mod renderer;
mod shading;
mod textured_aabb;
mod textured_plane;
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
    // --- Ventana ---
    let (fb_w, fb_h) = (800, 600);
    let (mut rl, thread) = raylib::init()
        .size(fb_w, fb_h)
        .title("Raytracer CPU + Raylib (Phong, Reflexión/Refracción, Skybox)")
        .build();

    let img = Image::gen_image_color(fb_w, fb_h, Color::BLACK);
    let mut tex2d = rl.load_texture_from_image(&thread, &img).expect("texture");

    // --- Carga de texturas de materiales (bytes persistentes) ---
    let agua      = load_texture_rgba("assets/agua.jpg");
    let lava      = load_texture_rgba("assets/lava.jpg");
    let obsidiana = load_texture_rgba("assets/obsidiana.jpg");
    let hierro    = load_texture_rgba("assets/hierro.jpg");
    let diamante  = load_texture_rgba("assets/diamante.jpg");
    let ground    = load_texture_rgba("assets/piedra.jpg")
                    .or_else(|| load_texture_rgba("assets/tierra.jpg"));

    // Guardar buffers y dims por separado para generar Tex (slices) estables
    let (mut agua_buf, mut agua_wh)         = (None, (0u32,0u32));
    let (mut lava_buf, mut lava_wh)         = (None, (0,0));
    let (mut obs_buf,  mut obs_wh)          = (None, (0,0));
    let (mut hierro_buf, mut hierro_wh)     = (None, (0,0));
    let (mut diam_buf, mut diam_wh)         = (None, (0,0));
    let (mut ground_buf, mut ground_wh)     = (None, (0,0));

    if let Some((b,w,h)) = agua      { agua_buf   = Some(b); agua_wh   = (w,h); }
    if let Some((b,w,h)) = lava      { lava_buf   = Some(b); lava_wh   = (w,h); }
    if let Some((b,w,h)) = obsidiana { obs_buf    = Some(b); obs_wh    = (w,h); }
    if let Some((b,w,h)) = hierro    { hierro_buf = Some(b); hierro_wh = (w,h); }
    if let Some((b,w,h)) = diamante  { diam_buf   = Some(b); diam_wh   = (w,h); }
    if let Some((b,w,h)) = ground    { ground_buf = Some(b); ground_wh = (w,h); }

    // Crear Tex (copyable) a partir de los buffers
    let agua_tex   = agua_buf.as_ref().map(|b| Tex{ pix: &b[..], w: agua_wh.0,   h: agua_wh.1   });
    let lava_tex   = lava_buf.as_ref().map(|b| Tex{ pix: &b[..], w: lava_wh.0,   h: lava_wh.1   });
    let obs_tex    = obs_buf.as_ref().map(|b|  Tex{ pix: &b[..], w: obs_wh.0,    h: obs_wh.1    });
    let hierro_tex = hierro_buf.as_ref().map(|b|Tex{ pix: &b[..], w: hierro_wh.0,h: hierro_wh.1 });
    let diam_tex   = diam_buf.as_ref().map(|b|  Tex{ pix: &b[..], w: diam_wh.0,  h: diam_wh.1   });
    let ground_tex = ground_buf.as_ref().map(|b|Tex{ pix: &b[..], w: ground_wh.0,h: ground_wh.1 });

    // --- Skybox opcional: intenta cargar 6 caras ---
    let mut sky_imgs: Option<Vec<(Vec<u8>, u32, u32)>> = {
        let names = ["px","nx","py","ny","pz","nz"];
        let mut acc: Vec<(Vec<u8>,u32,u32)> = Vec::new();
        let mut ok = true;
        for n in names {
            let p = format!("assets/skybox/{}.png", n);
            if let Some(t) = load_texture_rgba(&p) {
                acc.push(t);
            } else { ok = false; break; }
        }
        if ok { Some(acc) } else { None }
    };

    // Parámetros de cámara y luz
    let mut yaw: f32 = 0.6;
    let mut pitch: f32 = 0.25;
    let mut radius: f32 = 3.0;
    let mut light_pos = Vec3::new(2.2, 2.5, 2.0);

    let mut frame = vec![0u8; (fb_w * fb_h * 4) as usize];

    while !rl.window_should_close() {
        // Controles
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
            target: Vec3::new(0.0, 0.0, 0.0),
            up: Vec3::new(0.0, 1.0, 0.0),
            fov_y: 60.0,
        };

        // Construir vista de skybox (si existe)
        let skybox_view = sky_imgs.as_ref().map(|v| Skybox {
            px: Tex { pix: &v[0].0[..], w: v[0].1, h: v[0].2 },
            nx: Tex { pix: &v[1].0[..], w: v[1].1, h: v[1].2 },
            py: Tex { pix: &v[2].0[..], w: v[2].1, h: v[2].2 },
            ny: Tex { pix: &v[3].0[..], w: v[3].1, h: v[3].2 },
            pz: Tex { pix: &v[4].0[..], w: v[4].1, h: v[4].2 },
            nz: Tex { pix: &v[5].0[..], w: v[5].1, h: v[5].2 },
        });

        let assets = Assets {
            agua: agua_tex,
            lava: lava_tex,
            obsidiana: obs_tex,
            hierro: hierro_tex,
            diamante: diam_tex,
            ground: ground_tex,
            skybox: skybox_view,
        };

        // Render
        render(&mut frame, fb_w, fb_h, &cam, light_pos, assets, 3);

        // Subir a GPU y dibujar
        tex2d.update_texture(&frame);
        let mut d = rl.begin_drawing(&thread);
        d.clear_background(Color::BLACK);
        d.draw_texture(&tex2d, 0, 0, Color::WHITE);
        d.draw_text("Flechas: orbitar | Q/E: zoom | WASD: luz", 12, 12, 20, Color::WHITE);
    }
}
