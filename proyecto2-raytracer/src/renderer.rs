use crate::aabb::Aabb;
use crate::camera::Camera;
use crate::object::Intersectable;
use crate::plane::Plane;
use crate::ray::Ray;
use crate::shading::{is_shadowed, lambert, sky, to_rgba};
use crate::textured_aabb::TexturedAabb; // nuevo
use crate::vec3::Vec3;
use std::rc::Rc;

// Estructura para almacenar información de intersección
struct Hit<'a> {
    t: f32,
    point: Vec3,
    normal: Vec3,
    object: &'a dyn Intersectable,
}

// Cambiar firma: pasar opción de textura como (pixels, w, h)
pub fn render(
    frame: &mut [u8],
    w: i32,
    h: i32,
    cam: &Camera,
    light_pos: Vec3,
    textured_opt: Option<(&[u8], u32, u32)>,
) {
    // Crear los objetos de la escena
    let cube = Aabb::unit();
    let floor = Plane::new(
        Vec3::new(0.0, -0.5, 0.0), // Punto en el plano (justo debajo del cubo)
        Vec3::new(0.0, 1.0, 0.0),  // Normal hacia arriba
        Vec3::new(0.8, 0.8, 0.8),  // Color gris claro
    );

    // Si se proporcionó una textura, construir un TexturedAabb para el segundo cubo
    // Mantener la referencia viva durante el render() creando variable local opcional
    let mut textured_holder: Option<TexturedAabb> = None;
    if let Some((pix, tw, th)) = textured_opt {
        let inner = Aabb {
            min: Vec3::new(0.7, -0.5, -0.5), // desplazar el cubo texturizado a la derecha
            max: Vec3::new(1.7, 0.5, 0.5),
            albedo_color: Vec3::new(1.0, 1.0, 1.0),
        };
        textured_holder = Some(TexturedAabb::from_raw(inner, pix, tw, th));
    }

    // Lista de objetos (referencias)
    let mut objects: Vec<&dyn Intersectable> = vec![&cube, &floor];
    if let Some(ref t) = textured_holder {
        objects.push(t as &dyn Intersectable);
    }

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
                let shadow_ray = Ray {
                    orig: shadow_origin,
                    dir: light_dir,
                };

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

                // Usar albedo_at (soporta texturas)
                let obj_albedo = hit.object.albedo_at(hit.point);

                // Calcular la iluminación
                lambert(hit.normal, hit.point, light_pos, obj_albedo, in_shadow)
            } else {
                sky(ray.dir)
            };

            let i = ((y * w + x) * 4) as usize;
            let px = to_rgba(color);
            frame[i..i + 4].copy_from_slice(&px);
        }
    }
}
