use crate::object::Intersectable;
use crate::ray::Ray;
use crate::vec3::Vec3;

#[derive(Copy, Clone)]
pub struct Aabb {
    pub min: Vec3,
    pub max: Vec3,
    pub albedo_color: Vec3,
}

impl Aabb {
    pub fn unit() -> Self {
        Self {
            min: Vec3::new(-0.5, -0.5, -0.5),
            max: Vec3::new(0.5, 0.5, 0.5),
            albedo_color: Vec3::new(1.0, 0.12, 0.12), // rojo
        }
    }

    // Mantener el método de intersección actual como método interno
    fn intersect_impl(&self, ray: &Ray) -> Option<f32> {
        let inv = |d: f32| if d != 0.0 { 1.0 / d } else { f32::INFINITY };
        let (ix, iy, iz) = (inv(ray.dir.x), inv(ray.dir.y), inv(ray.dir.z));
        let (mut tmin, mut tmax) = ((
            (self.min.x - ray.orig.x) * ix,
            (self.max.x - ray.orig.x) * ix,
        ));
        if tmin > tmax {
            std::mem::swap(&mut tmin, &mut tmax);
        }
        let (mut tymin, mut tymax) = ((
            (self.min.y - ray.orig.y) * iy,
            (self.max.y - ray.orig.y) * iy,
        ));
        if tymin > tymax {
            std::mem::swap(&mut tymin, &mut tymax);
        }
        if tmin > tymax || tymin > tmax {
            return None;
        }
        if tymin > tmin {
            tmin = tymin;
        }
        if tymax < tmax {
            tmax = tymax;
        }
        let (mut tzmin, mut tzmax) = ((
            (self.min.z - ray.orig.z) * iz,
            (self.max.z - ray.orig.z) * iz,
        ));
        if tzmin > tzmax {
            std::mem::swap(&mut tzmin, &mut tzmax);
        }
        if tmin > tzmax || tzmin > tmax {
            return None;
        }
        if tzmin > tmin {
            tmin = tzmin;
        }
        if tzmax < tmax {
            tmax = tzmax;
        }
        if tmax < 0.0 {
            return None;
        }
        Some(if tmin >= 0.0 { tmin } else { tmax })
    }

    // Mantener el método de normal actual como método interno
    fn normal_impl(&self, p: Vec3) -> Vec3 {
        let eps = 1e-3;
        if (p.x - self.min.x).abs() < eps {
            return Vec3::new(-1.0, 0.0, 0.0);
        }
        if (self.max.x - p.x).abs() < eps {
            return Vec3::new(1.0, 0.0, 0.0);
        }
        if (p.y - self.min.y).abs() < eps {
            return Vec3::new(0.0, -1.0, 0.0);
        }
        if (self.max.y - p.y).abs() < eps {
            return Vec3::new(0.0, 1.0, 0.0);
        }
        if (p.z - self.min.z).abs() < eps {
            return Vec3::new(0.0, 0.0, -1.0);
        }
        if (self.max.z - p.z).abs() < eps {
            return Vec3::new(0.0, 0.0, 1.0);
        }
        Vec3::new(0.0, 0.0, 1.0) // fallback
    }
}

// Implementar el trait Intersectable para Aabb
impl Intersectable for Aabb {
    fn intersect(&self, ray: &Ray) -> Option<f32> {
        self.intersect_impl(ray)
    }

    fn normal_at(&self, point: Vec3) -> Vec3 {
        self.normal_impl(point)
    }

    fn albedo(&self) -> Vec3 {
        self.albedo_color
    }
}
