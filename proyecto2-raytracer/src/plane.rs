//! Infinite plane primitive (currently unused) retained for experimentation.

use crate::math::Vec3;
use crate::ray::Ray;
use crate::scene::{Intersectable, MaterialParams};

#[derive(Copy, Clone)]
pub struct Plane {
    pub point: Vec3,
    pub normal: Vec3,
    pub albedo_color: Vec3,

    pub specular_strength: f32,
    pub shininess: f32,
    pub reflectivity: f32,
    pub transparency: f32,
    pub ior: f32,
    pub emissive: Vec3,
}

impl Plane {
    pub fn new(point: Vec3, normal: Vec3, albedo_color: Vec3) -> Self {
        Self {
            point,
            normal: normal.norm(),
            albedo_color,
            specular_strength: 0.0,
            shininess: 16.0,
            reflectivity: 0.0,
            transparency: 0.0,
            ior: 1.0,
            emissive: Vec3::new(0.0, 0.0, 0.0),
        }
    }

    pub fn intersect(&self, ray: &Ray) -> Option<f32> {
        let denom = self.normal.dot(ray.dir);
        if denom.abs() < 1e-6 { return None; }
        let v = self.point.sub(ray.orig);
        let t = v.dot(self.normal) / denom;
        if t > 0.0 { Some(t) } else { None }
    }

    pub fn normal_at(&self, _p: Vec3) -> Vec3 { self.normal }
}

impl Intersectable for Plane {
    fn intersect(&self, ray: &Ray) -> Option<f32> { self.intersect(ray) }
    fn normal_at(&self, _point: Vec3) -> Vec3 { self.normal }
    fn albedo(&self) -> Vec3 { self.albedo_color }
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
