use raylib::prelude::*;
use std::f32::consts::PI;

// -------------------- Vec3 util --------------------
#[derive(Copy, Clone, Debug, Default)]
struct Vec3 { x: f32, y: f32, z: f32 }
impl Vec3 {
    fn new(x: f32, y: f32, z: f32) -> Self { Self { x, y, z } }
    fn add(self, o: Vec3) -> Self { Self::new(self.x + o.x, self.y + o.y, self.z + o.z) }
    fn sub(self, o: Vec3) -> Self { Self::new(self.x - o.x, self.y - o.y, self.z - o.z) }
    fn mul(self, s: f32) -> Self { Self::new(self.x * s, self.y * s, self.z * s) }
    fn hadamard(self, o: Vec3) -> Self { Self::new(self.x*o.x, self.y*o.y, self.z*o.z) }
    fn dot(self, o: Vec3) -> f32 { self.x*o.x + self.y*o.y + self.z*o.z }
    fn cross(self, o: Vec3) -> Vec3 {
        Vec3::new(self.y*o.z - self.z*o.y, self.z*o.x - self.x*o.z, self.x*o.y - self.y*o.x)
    }
    fn len(self) -> f32 { self.dot(self).sqrt() }
    fn norm(self) -> Self { let l = self.len(); if l==0.0 { self } else { self.mul(1.0/l) } }
    fn clamp01(self) -> Self {
        Self::new(self.x.clamp(0.0,1.0), self.y.clamp(0.0,1.0), self.z.clamp(0.0,1.0))
    }
}

#[derive(Copy, Clone)]
struct Ray { orig: Vec3, dir: Vec3 }

// -------------------- AABB (cubo) --------------------
#[derive(Copy, Clone)]
struct Aabb { min: Vec3, max: Vec3 }
impl Aabb {
    fn unit() -> Self { Self { min: Vec3::new(-0.5,-0.5,-0.5), max: Vec3::new(0.5,0.5,0.5) } }
    fn intersect(&self, ray: &Ray) -> Option<f32> {
        let inv = |d: f32| if d != 0.0 { 1.0/d } else { f32::INFINITY };
        let (ix,iy,iz) = (inv(ray.dir.x), inv(ray.dir.y), inv(ray.dir.z));
        let (mut tmin, mut tmax) = (((self.min.x-ray.orig.x)*ix, (self.max.x-ray.orig.x)*ix));
        if tmin>tmax { std::mem::swap(&mut tmin, &mut tmax); }
        let (mut tymin, mut tymax) = (((self.min.y-ray.orig.y)*iy, (self.max.y-ray.orig.y)*iy));
        if tymin>tymax { std::mem::swap(&mut tymin, &mut tymax); }
        if tmin>tymax || tymin>tmax { return None; }
        if tymin>tmin { tmin = tymin; } if tymax<tmax { tmax = tymax; }
        let (mut tzmin, mut tzmax) = (((self.min.z-ray.orig.z)*iz, (self.max.z-ray.orig.z)*iz));
        if tzmin>tzmax { std::mem::swap(&mut tzmin, &mut tzmax); }
        if tmin>tzmax || tzmin>tmax { return None; }
        if tzmin>tmin { tmin = tzmin; } if tzmax<tmax { tmax = tzmax; }
        if tmax < 0.0 { return None; }
        Some(if tmin>=0.0 { tmin } else { tmax })
    }
    fn normal(&self, p: Vec3) -> Vec3 {
        let eps = 1e-3;
        if (p.x - self.min.x).abs()<eps { return Vec3::new(-1.0,0.0,0.0); }
        if (self.max.x - p.x).abs()<eps { return Vec3::new( 1.0,0.0,0.0); }
        if (p.y - self.min.y).abs()<eps { return Vec3::new(0.0,-1.0,0.0); }
        if (self.max.y - p.y).abs()<eps { return Vec3::new(0.0, 1.0,0.0); }
        if (p.z - self.min.z).abs()<eps { return Vec3::new(0.0,0.0,-1.0); }
        if (self.max.z - p.z).abs()<eps { return Vec3::new(0.0,0.0, 1.0); }
        Vec3::new(0.0,0.0,1.0) // fallback
    }
}

// -------------------- Cámara look-at --------------------
struct Camera { eye: Vec3, target: Vec3, up: Vec3, fov_y: f32 }
impl Camera {
    fn make_ray(&self, u: f32, v: f32, aspect: f32) -> Ray {
        let fov = self.fov_y.to_radians();
        let scale = (fov*0.5).tan();
        let forward = self.target.sub(self.eye).norm();
        let right = forward.cross(self.up).norm();
        let up = right.cross(forward).norm();
        let x = (2.0*u - 1.0) * aspect * scale;
        let y = (1.0 - 2.0*v) * scale;
        let dir = right.mul(x).add(up.mul(y)).add(forward).norm();
        Ray { orig: self.eye, dir }
    }
}

// -------------------- Shading --------------------
fn sky(dir: Vec3) -> Vec3 {
    let t = 0.5*(dir.y + 1.0);
    Vec3::new(0.2,0.6,0.35).mul(1.0-t).add(Vec3::new(0.9,0.9,0.2).mul(t))
}
fn lambert(normal: Vec3, pos: Vec3, light_pos: Vec3, albedo: Vec3) -> Vec3 {
    let l = light_pos.sub(pos).norm();
    let ndotl = normal.norm().dot(l).max(0.0);
    albedo.mul(ndotl)
}
fn to_rgba(c: Vec3) -> [u8;4] {
    let g = c.clamp01();
    [(g.x*255.0) as u8, (g.y*255.0) as u8, (g.z*255.0) as u8, 255]
}

// -------------------- Render CPU a framebuffer RGBA8 --------------------
fn render(frame: &mut [u8], w: i32, h: i32, cam: &Camera, light_pos: Vec3) {
    let cube = Aabb::unit();
    let albedo = Vec3::new(1.0, 0.12, 0.12); // rojo
    let aspect = w as f32 / h as f32;

    for y in 0..h {
        for x in 0..w {
            let u = (x as f32 + 0.5) / w as f32;
            let v = (y as f32 + 0.5) / h as f32;
            let ray = cam.make_ray(u, v, aspect);

            let color = if let Some(t) = cube.intersect(&ray) {
                let p = ray.orig.add(ray.dir.mul(t));
                let n = cube.normal(p);
                lambert(n, p, light_pos, albedo)
            } else {
                sky(ray.dir)
            };

            let i = ((y*w + x) * 4) as usize;
            let px = to_rgba(color);
            frame[i..i+4].copy_from_slice(&px);
        }
    }
}

// -------------------- Main (raylib window) --------------------
fn main() {
    // Tamaño del framebuffer (y ventana)
    let (fb_w, fb_h) = (800, 600);

    // Inicializar raylib
    let (mut rl, thread) = raylib::init()
        .size(fb_w, fb_h)
        .title("Raytracer + raylib (cubo difuso)")
        .build();

    // Texture2D inicial (vacía)
    let img = Image::gen_image_color(fb_w, fb_h, Color::BLACK);
    let mut tex = rl.load_texture_from_image(&thread, &img).expect("texture");

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
        if rl.is_key_down(KeyboardKey::KEY_LEFT)  { yaw -= speed*dt; }
        if rl.is_key_down(KeyboardKey::KEY_RIGHT) { yaw += speed*dt; }
        if rl.is_key_down(KeyboardKey::KEY_UP)    { pitch -= speed*dt; }
        if rl.is_key_down(KeyboardKey::KEY_DOWN)  { pitch += speed*dt; }
        if rl.is_key_down(KeyboardKey::KEY_Q)     { radius = (radius-1.5*dt).max(1.2); }
        if rl.is_key_down(KeyboardKey::KEY_E)     { radius += 1.5*dt; }
        if rl.is_key_down(KeyboardKey::KEY_A)     { light_pos.x -= 2.0*dt; }
        if rl.is_key_down(KeyboardKey::KEY_D)     { light_pos.x += 2.0*dt; }
        if rl.is_key_down(KeyboardKey::KEY_W)     { light_pos.y += 2.0*dt; }
        if rl.is_key_down(KeyboardKey::KEY_S)     { light_pos.y -= 2.0*dt; }

        pitch = pitch.clamp(-PI*0.49, PI*0.49);

        // Cámara look-at (orbital alrededor del origen)
        let eye = Vec3::new(
            radius * yaw.sin() * pitch.cos(),
            radius * pitch.sin(),
            radius * yaw.cos() * pitch.cos(),
        );
        let cam = Camera { eye, target: Vec3::new(0.0,0.0,0.0), up: Vec3::new(0.0,1.0,0.0), fov_y: 60.0 };

        // Render CPU -> framebuffer
        render(&mut frame, fb_w, fb_h, &cam, light_pos);

        // Subir framebuffer a la textura y dibujar
        tex.update_texture(&frame);
        let mut d = rl.begin_drawing(&thread);
        d.clear_background(Color::BLACK);
        d.draw_texture(&tex, 0, 0, Color::WHITE);
        d.draw_text("Flechas: orbitar | Q/E: zoom | WASD: luz", 12, 12, 20, Color::WHITE);
    }
}
