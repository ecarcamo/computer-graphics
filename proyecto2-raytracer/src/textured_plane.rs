//! Optional infinite textured plane primitive (currently not used in the diorama).

use crate::math::Vec3;
use crate::ray::Ray;
use crate::scene::{Intersectable, MaterialParams};

pub struct TexturedPlane<'a> {
    pub point: Vec3,    // un punto sobre el plano
    pub normal: Vec3,   // normal (unitaria)
    pub tile: f32,      // factor de repetición UV
    pub pixels: &'a [u8],
    pub w: u32,
    pub h: u32,

    pub specular_strength: f32,
    pub shininess: f32,
    pub reflectivity: f32,
    pub transparency: f32,
    pub ior: f32,
    pub emissive: Vec3,
    pub tint: Vec3,    // multiplicador de color sobre textura
}

impl<'a> TexturedPlane<'a> {
    pub fn new(
        point: Vec3, normal: Vec3, tile: f32,
        pixels: &'a [u8], w: u32, h: u32,
    ) -> Self {
        Self {
            point, normal: normal.norm(), tile,
            pixels, w, h,
            specular_strength: 0.0,
            shininess: 16.0,
            reflectivity: 0.0,
            transparency: 0.0,
            ior: 1.0,
            emissive: Vec3::new(0.0,0.0,0.0),
            tint: Vec3::new(1.0,1.0,1.0),
        }
    }

    fn sample(&self, u: f32, v: f32) -> Vec3 {
        let uu = u.fract(); // wrap
        let vv = v.fract();
        let uu = if uu < 0.0 { uu + 1.0 } else { uu };
        let vv = if vv < 0.0 { vv + 1.0 } else { vv };
        let px = (uu * (self.w as f32 - 1.0)).round() as u32;
        let py = ((1.0 - vv) * (self.h as f32 - 1.0)).round() as u32;
        let idx = ((py * self.w + px) * 4) as usize;
        if idx + 3 >= self.pixels.len() { return Vec3::new(1.0,1.0,1.0); }
        let r = self.pixels[idx] as f32 / 255.0;
        let g = self.pixels[idx + 1] as f32 / 255.0;
        let b = self.pixels[idx + 2] as f32 / 255.0;
        Vec3::new(r,g,b).hadamard(self.tint)
    }
}

impl<'a> Intersectable for TexturedPlane<'a> {
    fn intersect(&self, ray: &Ray) -> Option<f32> {
        let denom = self.normal.dot(ray.dir);
        if denom.abs() < 1e-6 { return None; }
        let v = self.point.sub(ray.orig);
        let t = v.dot(self.normal) / denom;
        if t > 0.0 { Some(t) } else { None }
    }

    fn normal_at(&self, _point: Vec3) -> Vec3 { self.normal }

    fn albedo(&self) -> Vec3 { Vec3::new(1.0,1.0,1.0) }

    fn albedo_at(&self, p: Vec3) -> Vec3 {
        // Mapeo UV tomando el eje dominante perpendicular.
        // Para normal=(0,1,0) esto cae en u=x, v=z.
        let n = self.normal;
        let (u, v) = if n.y.abs() > n.x.abs() && n.y.abs() > n.z.abs() {
            (p.x * self.tile, p.z * self.tile)
        } else if n.x.abs() > n.z.abs() {
            (p.y * self.tile, p.z * self.tile)
        } else {
            (p.x * self.tile, p.y * self.tile)
        };
        self.sample(u, v)
    }

    fn material_at(&self, p: Vec3) -> MaterialParams {
        MaterialParams {
            albedo: self.albedo_at(p),
            specular_strength: self.specular_strength,
            shininess: self.shininess,
            reflectivity: self.reflectivity,
            transparency: self.transparency,
            ior: self.ior,
            emissive: self.emissive,
        }
    }
}
