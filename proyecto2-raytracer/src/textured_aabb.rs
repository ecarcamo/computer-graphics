use crate::aabb::Aabb;
use crate::object::Intersectable;
use crate::ray::Ray;
use crate::vec3::Vec3;

pub struct TexturedAabb<'a> {
    pub inner: Aabb,
    pub pixels: &'a [u8], // RGBA8 pixels
    pub w: u32,
    pub h: u32,
}

impl<'a> TexturedAabb<'a> {
    pub fn from_raw(inner: Aabb, pixels: &'a [u8], w: u32, h: u32) -> Self {
        Self {
            inner,
            pixels,
            w,
            h,
        }
    }

    fn sample_rgba(&self, u: f32, v: f32) -> Vec3 {
        // Clamp u,v a [0,1]
        let uu = u.clamp(0.0, 1.0);
        let vv = v.clamp(0.0, 1.0);
        // Convertir a coordenadas de píxel (0..w-1, 0..h-1)
        let px = (uu * (self.w as f32 - 1.0)).round() as u32;
        let py = ((1.0 - vv) * (self.h as f32 - 1.0)).round() as u32; // flip V
        let idx = ((py * self.w + px) * 4) as usize;
        if idx + 3 >= self.pixels.len() {
            return self.inner.albedo_color; // fallback
        }
        let r = self.pixels[idx] as f32 / 255.0;
        let g = self.pixels[idx + 1] as f32 / 255.0;
        let b = self.pixels[idx + 2] as f32 / 255.0;
        Vec3::new(r, g, b)
    }

    // Mapeo UV simple por cara del cubo
    fn uv_from_point(&self, p: Vec3) -> (f32, f32) {
        let n = self.inner.normal_at(p);
        let min = self.inner.min;
        let max = self.inner.max;
        let dx = max.x - min.x;
        let dy = max.y - min.y;
        let dz = max.z - min.z;

        if n.x > 0.5 {
            // +X face: u from z, v from y
            let u = (p.z - min.z) / dz;
            let v = (p.y - min.y) / dy;
            (u, v)
        } else if n.x < -0.5 {
            // -X face
            let u = (max.z - p.z) / dz;
            let v = (p.y - min.y) / dy;
            (u, v)
        } else if n.y > 0.5 {
            // +Y face (top)
            let u = (p.x - min.x) / dx;
            let v = (max.z - p.z) / dz;
            (u, v)
        } else if n.y < -0.5 {
            // -Y face (bottom)
            let u = (p.x - min.x) / dx;
            let v = (p.z - min.z) / dz;
            (u, v)
        } else if n.z > 0.5 {
            // +Z face
            let u = (max.x - p.x) / dx;
            let v = (p.y - min.y) / dy;
            (u, v)
        } else {
            // -Z face
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
}
