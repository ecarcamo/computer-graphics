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
use renderer::{Assets, WorldKind, render};
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

fn load_cubemap(base: &str) -> Option<Vec<(Vec<u8>, u32, u32)>> {
    let face_names = ["px", "nx", "py", "ny", "pz", "nz"];
    let mut faces = Vec::with_capacity(6);
    for n in face_names {
        let p_jpg = format!("{}/{}.jpg", base, n);
        let p_png = format!("{}/{}.png", base, n);
        if let Some(t) = load_texture_rgba(&p_jpg).or_else(|| load_texture_rgba(&p_png)) {
            faces.push(t);
        } else {
            return None;
        }
    }
    Some(faces)
}

fn make_skybox<'a>(imgs: &'a [(Vec<u8>, u32, u32)], tint: Vec3) -> Skybox<'a> {
    Skybox {
        px: Tex {
            pix: &imgs[0].0[..],
            w: imgs[0].1,
            h: imgs[0].2,
        },
        nx: Tex {
            pix: &imgs[1].0[..],
            w: imgs[1].1,
            h: imgs[1].2,
        },
        py: Tex {
            pix: &imgs[2].0[..],
            w: imgs[2].1,
            h: imgs[2].2,
        },
        ny: Tex {
            pix: &imgs[3].0[..],
            w: imgs[3].1,
            h: imgs[3].2,
        },
        pz: Tex {
            pix: &imgs[4].0[..],
            w: imgs[4].1,
            h: imgs[4].2,
        },
        nz: Tex {
            pix: &imgs[5].0[..],
            w: imgs[5].1,
            h: imgs[5].2,
        },
        tint,
    }
}

fn main() {
    // Ajusta estas dimensiones para cambiar el tamaño de la ventana y el framebuffer.
    let (fb_w, fb_h) = (1280, 720);
    let (mut rl, thread) = raylib::init()
        .size(fb_w, fb_h)
        .title("Raytracer CPU + Raylib (Skyblock)")
        .build();

    let img = Image::gen_image_color(fb_w, fb_h, Color::BLACK);
    let mut tex2d = rl.load_texture_from_image(&thread, &img).expect("texture");

    // --- Texturas de bloques ---
    // grass: cesped.jpg o pasto.jpg (usa la primera que exista)
    let grass =
        load_texture_rgba("assets/cesped.jpg").or_else(|| load_texture_rgba("assets/pasto.jpg"));
    let dirt = load_texture_rgba("assets/tierra.jpg");
    let stone = load_texture_rgba("assets/piedra.jpg");
    let wood = load_texture_rgba("assets/madera.jpg");
    let leaves = load_texture_rgba("assets/hojas.jpg");
    let water = load_texture_rgba("assets/agua.jpg");
    let lava = load_texture_rgba("assets/lava.jpg");
    let obsidian = load_texture_rgba("assets/obsidiana.jpg");
    let glowstone = load_texture_rgba("assets/glow.jpg");
    let diamond = load_texture_rgba("assets/diamante.jpg");
    let iron = load_texture_rgba("assets/hierro.jpg");
    let chest = load_texture_rgba("assets/cofre.jpg");

    // Mantener buffers vivos y crear Tex
    let (mut grass_buf, mut grass_wh) = (None, (0u32, 0u32));
    let (mut dirt_buf, mut dirt_wh) = (None, (0u32, 0u32));
    let (mut stone_buf, mut stone_wh) = (None, (0u32, 0u32));
    let (mut wood_buf, mut wood_wh) = (None, (0u32, 0u32));
    let (mut leaves_buf, mut leaves_wh) = (None, (0u32, 0u32));
    let (mut water_buf, mut water_wh) = (None, (0u32, 0u32));
    let (mut lava_buf, mut lava_wh) = (None, (0u32, 0u32));
    let (mut obsidian_buf, mut obsidian_wh) = (None, (0u32, 0u32));
    let (mut glowstone_buf, mut glowstone_wh) = (None, (0u32, 0u32));
    let (mut diamond_buf, mut diamond_wh) = (None, (0u32, 0u32));
    let (mut iron_buf, mut iron_wh) = (None, (0u32, 0u32));
    let (mut chest_buf, mut chest_wh) = (None, (0u32, 0u32));

    if let Some((b, w, h)) = grass {
        grass_buf = Some(b);
        grass_wh = (w, h);
    }
    if let Some((b, w, h)) = dirt {
        dirt_buf = Some(b);
        dirt_wh = (w, h);
    }
    if let Some((b, w, h)) = stone {
        stone_buf = Some(b);
        stone_wh = (w, h);
    }
    if let Some((b, w, h)) = wood {
        wood_buf = Some(b);
        wood_wh = (w, h);
    }
    if let Some((b, w, h)) = leaves {
        leaves_buf = Some(b);
        leaves_wh = (w, h);
    }
    if let Some((b, w, h)) = water {
        water_buf = Some(b);
        water_wh = (w, h);
    }
    if let Some((b, w, h)) = lava {
        lava_buf = Some(b);
        lava_wh = (w, h);
    }
    if let Some((b, w, h)) = obsidian {
        obsidian_buf = Some(b);
        obsidian_wh = (w, h);
    }
    if let Some((b, w, h)) = glowstone {
        glowstone_buf = Some(b);
        glowstone_wh = (w, h);
    }
    if let Some((b, w, h)) = diamond {
        diamond_buf = Some(b);
        diamond_wh = (w, h);
    }
    if let Some((b, w, h)) = iron {
        iron_buf = Some(b);
        iron_wh = (w, h);
    }
    if let Some((b, w, h)) = chest {
        chest_buf = Some(b);
        chest_wh = (w, h);
    }

    let grass_tex = grass_buf.as_ref().map(|b| Tex {
        pix: &b[..],
        w: grass_wh.0,
        h: grass_wh.1,
    });
    let dirt_tex = dirt_buf.as_ref().map(|b| Tex {
        pix: &b[..],
        w: dirt_wh.0,
        h: dirt_wh.1,
    });
    let stone_tex = stone_buf.as_ref().map(|b| Tex {
        pix: &b[..],
        w: stone_wh.0,
        h: stone_wh.1,
    });
    let wood_tex = wood_buf.as_ref().map(|b| Tex {
        pix: &b[..],
        w: wood_wh.0,
        h: wood_wh.1,
    });
    let leaves_tex = leaves_buf.as_ref().map(|b| Tex {
        pix: &b[..],
        w: leaves_wh.0,
        h: leaves_wh.1,
    });
    let water_tex = water_buf.as_ref().map(|b| Tex {
        pix: &b[..],
        w: water_wh.0,
        h: water_wh.1,
    });
    let lava_tex = lava_buf.as_ref().map(|b| Tex {
        pix: &b[..],
        w: lava_wh.0,
        h: lava_wh.1,
    });
    let obsidian_tex = obsidian_buf.as_ref().map(|b| Tex {
        pix: &b[..],
        w: obsidian_wh.0,
        h: obsidian_wh.1,
    });
    let glowstone_tex = glowstone_buf.as_ref().map(|b| Tex {
        pix: &b[..],
        w: glowstone_wh.0,
        h: glowstone_wh.1,
    });
    let diamond_tex = diamond_buf.as_ref().map(|b| Tex {
        pix: &b[..],
        w: diamond_wh.0,
        h: diamond_wh.1,
    });
    let iron_tex = iron_buf.as_ref().map(|b| Tex {
        pix: &b[..],
        w: iron_wh.0,
        h: iron_wh.1,
    });
    let chest_tex = chest_buf.as_ref().map(|b| Tex {
        pix: &b[..],
        w: chest_wh.0,
        h: chest_wh.1,
    });

    // --- Skyboxes ---
    let overworld_cubemap = load_cubemap("assets/skybox");
    let nether_cubemap = load_cubemap("assets/skybox_nether");

    let skybox_overworld = overworld_cubemap
        .as_ref()
        .map(|faces| make_skybox(faces, Vec3::new(1.0, 1.0, 1.0)));
    let skybox_nether = if let Some(faces) = nether_cubemap.as_ref() {
        Some(make_skybox(faces, Vec3::new(1.0, 1.0, 1.0)))
    } else {
        overworld_cubemap
            .as_ref()
            .map(|faces| make_skybox(faces, Vec3::new(1.3, 0.4, 0.4)))
    };

    // Cámara orbital y luz
    let mut yaw: f32 = 0.6;
    let mut pitch: f32 = 0.25;
    let mut radius: f32 = 4.0; // un poco más lejos para ver la isla
    let mut light_pos = Vec3::new(2.5, 3.0, 2.5);
    let mut world = WorldKind::Overworld;

    let mut frame = vec![0u8; (fb_w * fb_h * 4) as usize];

    while !rl.window_should_close() {
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
        if rl.is_key_pressed(KeyboardKey::KEY_N) {
            world = world.toggle();
        }
        pitch = pitch.clamp(-PI * 0.49, PI * 0.49);

        let eye = Vec3::new(
            radius * yaw.sin() * pitch.cos(),
            radius * pitch.sin(),
            radius * yaw.cos() * pitch.cos(),
        );
        let cam = Camera {
            eye,
            target: Vec3::new(1.0, 0.0, 1.0),
            up: Vec3::new(0.0, 1.0, 0.0),
            fov_y: 60.0,
        };

        let assets = Assets {
            grass: grass_tex,
            dirt: dirt_tex,
            stone: stone_tex,
            wood: wood_tex,
            leaves: leaves_tex,
            water: water_tex,
            lava: lava_tex,
            obsidian: obsidian_tex,
            glowstone: glowstone_tex,
            diamond: diamond_tex,
            iron: iron_tex,
            chest: chest_tex,
            skybox_overworld,
            skybox_nether,
        };

        render(&mut frame, fb_w, fb_h, &cam, light_pos, &assets, 4, world);

        let _ = tex2d.update_texture(&frame);
        let mut d = rl.begin_drawing(&thread);
        d.clear_background(Color::BLACK);
        d.draw_texture(&tex2d, 0, 0, Color::WHITE);
        d.draw_text(
            "Flechas: orbitar | Q/E: zoom | WASD: luz | N: cambiar mundo",
            12,
            12,
            20,
            Color::WHITE,
        );
        let world_text = match world {
            WorldKind::Overworld => "Mundo: Overworld",
            WorldKind::Nether => "Mundo: Nether",
        };
        d.draw_text(world_text, 12, 40, 20, Color::WHITE);
    }
}
