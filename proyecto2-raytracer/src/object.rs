use crate::vec3::Vec3;
use crate::ray::Ray;

// Trait para cualquier objeto que pueda ser intersectado por un rayo
pub trait Intersectable {
    fn intersect(&self, ray: &Ray) -> Option<f32>;
    fn normal_at(&self, point: Vec3) -> Vec3;
    fn albedo(&self) -> Vec3;
}