use crate::vec3::Vec3;
use crate::aabb::Aabb;
use crate::plane::Plane;
use crate::camera::Camera;
use crate::ray::Ray;
use crate::object::Intersectable;
use crate::shading::{sky, lambert, is_shadowed, to_rgba};
use std::rc::Rc;

// Estructura para almacenar información de intersección
struct Hit<'a> {
    t: f32,
    point: Vec3,
    normal: Vec3,
    object: &'a dyn Intersectable,
}

pub fn render(frame: &mut [u8], w: i32, h: i32, cam: &Camera, light_pos: Vec3) {
    // Crear los objetos de la escena
    let cube = Aabb::unit();
    let floor = Plane::new(
        Vec3::new(0.0, -0.5, 0.0),  // Punto en el plano (justo debajo del cubo)
        Vec3::new(0.0, 1.0, 0.0),   // Normal hacia arriba
        Vec3::new(0.8, 0.8, 0.8)    // Color gris claro
    );
    
    // Lista de objetos
    let objects: Vec<&dyn Intersectable> = vec![&cube, &floor];
    
    let aspect = w as f32 / h as f32;

    for y in 0..h {
        for x in 0..w {
            let u = (x as f32 + 0.5) / w as f32;
            let v = (y as f32 + 0.5) / h as f32;
            let ray = cam.make_ray(u, v, aspect);

            // Buscar la intersección más cercana
            let mut closest_hit: Option<Hit> = None;
            
            for object in &objects {
                if let Some(t) = object.intersect(&ray) {
                    if closest_hit.is_none() || t < closest_hit.as_ref().unwrap().t {
                        let hit_point = ray.orig.add(ray.dir.mul(t));
                        let hit_normal = object.normal_at(hit_point);
                        closest_hit = Some(Hit {
                            t,
                            point: hit_point,
                            normal: hit_normal,
                            object: *object,
                        });
                    }
                }
            }
            
            // Calcular el color basado en la intersección
            let color = if let Some(hit) = closest_hit {
                // Comprobar si el punto está en sombra
                let mut in_shadow = false;
                
                // Crear un rayo de sombra desde el punto hacia la luz
                let shadow_bias = 1e-3;
                let shadow_origin = hit.point.add(hit.normal.mul(shadow_bias));
                let light_dir = light_pos.sub(hit.point).norm();
                let shadow_ray = Ray { orig: shadow_origin, dir: light_dir };
                
                // Calcular la distancia al punto de luz
                let light_distance = light_pos.sub(hit.point).len();
                
                // Comprobar si hay alguna intersección en el camino hacia la luz
                for object in &objects {
                    if let Some(t) = object.intersect(&shadow_ray) {
                        if t < light_distance {
                            in_shadow = true;
                            break;
                        }
                    }
                }
                
                // Calcular la iluminación
                lambert(hit.normal, hit.point, light_pos, hit.object.albedo(), in_shadow)
            } else {
                sky(ray.dir)
            };

            let i = ((y*w + x) * 4) as usize;
            let px = to_rgba(color);
            frame[i..i+4].copy_from_slice(&px);
        }
    }
}