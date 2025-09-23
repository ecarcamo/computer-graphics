use crate::aabb::Aabb;
use crate::object::{Intersectable, MaterialParams};
use crate::ray::Ray;
use crate::vec3::Vec3;

pub struct TexturedAabb<'a> {
    pub inner: Aabb,
    pub pixels: &'a [u8], // RGBA8
    pub w: u32,
    pub h: u32,

    // Parámetros de material
    pub specular_strength: f32,
    pub shininess: f32,
    pub reflectivity: f32,
    pub transparency: f32,
    pub ior: f32,
    pub emissive: Vec3,
}

impl<'a> TexturedAabb<'a> {
    pub fn from_raw(
        inner: Aabb,
        pixels: &'a [u8],
        w: u32,
        h: u32,
        specular_strength: f32,
        shininess: f32,
        reflectivity: f32,
        transparency: f32,
        ior: f32,
        emissive: Vec3,
    ) -> Self {
        Self {
            inner,
            pixels,
            w,
            h,
            specular_strength,
            shininess,
            reflectivity,
            transparency,
            ior,
            emissive,
        }
    }

    fn sample_rgba(&self, u: f32, v: f32) -> Vec3 {
        let uu = u.clamp(0.0, 1.0);
        let vv = v.clamp(0.0, 1.0);
        let px = (uu * (self.w as f32 - 1.0)).round() as u32;
        let py = ((1.0 - vv) * (self.h as f32 - 1.0)).round() as u32;
        let idx = ((py * self.w + px) * 4) as usize;
        if idx + 3 >= self.pixels.len() {
            return self.inner.albedo_color;
        }
        let r = self.pixels[idx] as f32 / 255.0;
        let g = self.pixels[idx + 1] as f32 / 255.0;
        let b = self.pixels[idx + 2] as f32 / 255.0;
        Vec3::new(r, g, b)
    }

    fn uv_from_point(&self, p: Vec3) -> (f32, f32) {
        let n = self.inner.normal_at(p);
        let min = self.inner.min;
        let max = self.inner.max;
        let dx = max.x - min.x;
        let dy = max.y - min.y;
        let dz = max.z - min.z;

        if n.x > 0.5 {
            let u = (p.z - min.z) / dz;
            let v = (p.y - min.y) / dy;
            (u, v)
        } else if n.x < -0.5 {
            let u = (max.z - p.z) / dz;
            let v = (p.y - min.y) / dy;
            (u, v)
        } else if n.y > 0.5 {
            let u = (p.x - min.x) / dx;
            let v = (max.z - p.z) / dz;
            (u, v)
        } else if n.y < -0.5 {
            let u = (p.x - min.x) / dx;
            let v = (p.z - min.z) / dz;
            (u, v)
        } else if n.z > 0.5 {
            let u = (max.x - p.x) / dx;
            let v = (p.y - min.y) / dy;
            (u, v)
        } else {
            let u = (p.x - min.x) / dx;
            let v = (p.y - min.y) / dy;
            (u, v)
        }
    }
}

impl<'a> Intersectable for TexturedAabb<'a> {
    fn intersect(&self, ray: &Ray) -> Option<f32> {
        self.inner.intersect(ray)
    }
    fn normal_at(&self, point: Vec3) -> Vec3 {
        self.inner.normal_at(point)
    }
    fn albedo(&self) -> Vec3 {
        self.inner.albedo_color
    }
    fn albedo_at(&self, point: Vec3) -> Vec3 {
        let (u, v) = self.uv_from_point(point);
        self.sample_rgba(u, v)
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
