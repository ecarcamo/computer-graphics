use crate::vec3::Vec3;
use crate::ray::Ray;
use crate::aabb::Aabb;
use crate::object::Intersectable; // Añadir esta línea

pub fn sky(dir: Vec3) -> Vec3 {
    let t = 0.5*(dir.y + 1.0);
    Vec3::new(0.2,0.6,0.35).mul(1.0-t).add(Vec3::new(0.9,0.9,0.2).mul(t))
}

pub fn lambert(normal: Vec3, pos: Vec3, light_pos: Vec3, albedo: Vec3, in_shadow: bool) -> Vec3 {
    // Si está en sombra, devuelve solo iluminación ambiente
    if in_shadow {
        // Factor de luz ambiente (ajusta según necesites)
        let ambient = 0.1;
        return albedo.mul(ambient);
    }
    
    // Cálculo normal de iluminación difusa
    let l = light_pos.sub(pos).norm();
    let ndotl = normal.norm().dot(l).max(0.0);
    albedo.mul(ndotl)
}

pub fn is_shadowed(pos: Vec3, light_pos: Vec3, normal: Vec3, cube: &Aabb) -> bool {
    // Vector dirección desde el punto hacia la luz
    let light_dir = light_pos.sub(pos).norm();
    
    // Desplazar ligeramente el origen para evitar auto-intersección
    let shadow_bias = 1e-3;
    let shadow_origin = pos.add(normal.mul(shadow_bias));
    
    // Crear un rayo de sombra hacia la luz
    let shadow_ray = Ray { orig: shadow_origin, dir: light_dir };
    
    // Calcular la distancia al punto de luz
    let light_distance = light_pos.sub(pos).len();
    
    // Si hay una intersección entre el punto y la luz, entonces está en sombra
    if let Some(t) = cube.intersect(&shadow_ray) {
        return t < light_distance;
    }
    
    false
}

pub fn to_rgba(c: Vec3) -> [u8;4] {
    let g = c.clamp01();
    [(g.x*255.0) as u8, (g.y*255.0) as u8, (g.z*255.0) as u8, 255]
}
