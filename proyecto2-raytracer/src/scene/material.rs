//! Definición de materiales y la interfaz `Intersectable` usada por las primitivas.

use crate::math::Vec3;
use crate::ray::Ray;

#[derive(Copy, Clone)]
/// Parámetros físicos muestreados en cada punto de sombreado.
pub struct MaterialParams {
    pub albedo: Vec3,           // Color base (tras textura)
    pub specular_strength: f32, // [0..1]
    pub shininess: f32,         // >= 1
    pub reflectivity: f32,      // [0..1]
    pub transparency: f32,      // [0..1]
    pub ior: f32,               // índice de refracción (1=aire)
    pub emissive: Vec3,         // luz propia
}

/// Interfaz común para cualquier objeto intersectable por un rayo.
pub trait Intersectable: Send + Sync {
    fn intersect(&self, ray: &Ray) -> Option<f32>;
    fn normal_at(&self, point: Vec3) -> Vec3;

    // Compatibilidad con tu versión previa
    fn albedo(&self) -> Vec3;
    fn albedo_at(&self, _point: Vec3) -> Vec3 {
        self.albedo()
    }

    // NUEVO: material paramétrico por punto
    fn material_at(&self, p: Vec3) -> MaterialParams {
        MaterialParams {
            albedo: self.albedo_at(p),
            specular_strength: 0.0,
            shininess: 16.0,
            reflectivity: 0.0,
            transparency: 0.0,
            ior: 1.0,
            emissive: Vec3::new(0.0, 0.0, 0.0),
        }
    }
}
