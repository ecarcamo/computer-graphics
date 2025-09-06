use crate::vec3::Vec3;
use crate::ray::Ray;
use crate::object::Intersectable;

#[derive(Copy, Clone)]
pub struct Plane {
    pub point: Vec3,  // Un punto en el plano
    pub normal: Vec3, // Vector normal al plano (debe estar normalizado)
    pub albedo_color: Vec3, // Color del plano
}

impl Plane {
    pub fn new(point: Vec3, normal: Vec3, albedo_color: Vec3) -> Self {
        Self {
            point,
            normal: normal.norm(), // Asegurarse de que la normal esté normalizada
            albedo_color,
        }
    }
    
    pub fn intersect(&self, ray: &Ray) -> Option<f32> {
        // Producto punto entre la normal y la dirección del rayo
        let denom = self.normal.dot(ray.dir);
        
        // Si el rayo es paralelo al plano o apunta en la misma dirección que la normal
        if denom.abs() < 1e-6 {
            return None;
        }
        
        // Calcular distancia desde el origen del rayo al plano
        let v = self.point.sub(ray.orig);
        let t = v.dot(self.normal) / denom;
        
        // Solo intersecciones en la dirección positiva del rayo
        if t > 0.0 {
            Some(t)
        } else {
            None
        }
    }
    
    pub fn normal_at(&self, _p: Vec3) -> Vec3 {
        // La normal es constante en cualquier punto del plano
        self.normal
    }
}

impl Intersectable for Plane {
    fn intersect(&self, ray: &Ray) -> Option<f32> {
        // Reutilizar el método existente
        self.intersect(ray)
    }
    
    fn normal_at(&self, _point: Vec3) -> Vec3 {
        self.normal
    }
    
    fn albedo(&self) -> Vec3 {
        self.albedo_color
    }
}