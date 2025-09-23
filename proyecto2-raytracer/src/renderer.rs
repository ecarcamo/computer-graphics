use crate::aabb::Aabb;
use crate::camera::Camera;
use crate::object::Intersectable;
use crate::plane::Plane;
use crate::ray::Ray;
use crate::shading::{reflect, refract, sample_skybox, sky, specular_phong, to_rgba, Skybox, Tex};
use crate::textured_aabb::TexturedAabb;
use crate::textured_plane::TexturedPlane;
use crate::vec3::Vec3;

struct Hit<'a> {
    t: f32,
    point: Vec3,
    normal: Vec3,
    object: &'a dyn Intersectable,
}

pub struct Assets<'a> {
    pub agua: Option<Tex<'a>>,
    pub lava: Option<Tex<'a>>,
    pub obsidiana: Option<Tex<'a>>,
    pub hierro: Option<Tex<'a>>,
    pub diamante: Option<Tex<'a>>,
    pub ground: Option<Tex<'a>>,
    pub skybox: Option<Skybox<'a>>,
}

// Rayo-trazado con recursión para reflejos/refracciones
fn trace<'a>(
    ray: &Ray,
    objects: &'a [&dyn Intersectable],
    light_pos: Vec3,
    depth: i32,
    skybox: Option<&Skybox<'a>>,
) -> Vec3 {
    // Buscar hit más cercano
    let mut closest: Option<Hit> = None;
    for o in objects {
        if let Some(t) = o.intersect(ray) {
            if t > 0.0 && (closest.is_none() || t < closest.as_ref().unwrap().t) {
                let p = ray.orig.add(ray.dir.mul(t));
                let n = o.normal_at(p);
                closest = Some(Hit { t, point: p, normal: n, object: *o });
            }
        }
    }

    if closest.is_none() {
        return if let Some(sb) = skybox { sample_skybox(ray.dir, sb) } else { sky(ray.dir) };
    }

    let hit = closest.unwrap();
    let mat = hit.object.material_at(hit.point);

    // Sombra
    let shadow_bias = 1e-3;
    let shadow_origin = hit.point.add(hit.normal.mul(shadow_bias));
    let ldir = light_pos.sub(hit.point).norm();
    let light_distance = light_pos.sub(hit.point).len();
    let sray = Ray { orig: shadow_origin, dir: ldir };
    let mut in_shadow = false;
    for o in objects {
        if let Some(t) = o.intersect(&sray) {
            if t < light_distance { in_shadow = true; break; }
        }
    }

    // Iluminación local (Phong)
    let ambient = 0.1;
    let n = hit.normal.norm();
    let l = ldir;
    let v = (-ray.dir).norm();
    let ndotl = n.dot(l).max(0.0);
    let mut local = mat.albedo.mul(ambient + ndotl.max(0.0) * if in_shadow { 0.0 } else { 1.0 });

    if !in_shadow && ndotl > 0.0 && mat.specular_strength > 0.0 {
        let r = reflect(-l, n);
        let spec = specular_phong(r, v, mat.specular_strength, mat.shininess);
        local = local.add(Vec3::new(spec, spec, spec));
    }

    // Emisión (lava/glow)
    local = local.add(mat.emissive);

    // Rayos secundarios
    if depth <= 0 { return local; }

    let mut accum = Vec3::new(0.0,0.0,0.0);
    let mut weight = 1.0;

    // Refracción
    if mat.transparency > 0.0 {
        let mut n_out = n;
        let mut eta = 1.0 / mat.ior;
        if ray.dir.dot(n) > 0.0 {
            // Saliendo del material
            n_out = -n;
            eta = mat.ior;
        }

        if let Some(tdir) = refract(ray.dir, n_out, eta) {
            let ro = hit.point.add(tdir.mul(shadow_bias));
            let rr = Ray { orig: ro, dir: tdir };
            let refr_col = trace(&rr, objects, light_pos, depth - 1, skybox);
            accum = accum.add(refr_col.mul(mat.transparency));
            weight -= mat.transparency;
        }
        // Si hubo RTI, esa energía irá a reflexión abajo
    }

    // Reflexión
    if mat.reflectivity > 0.0 && weight > 0.0 {
        let rdir = reflect(ray.dir, n).norm();
        let ro = hit.point.add(n.mul(shadow_bias));
        let rr = Ray { orig: ro, dir: rdir };
        let refl_col = trace(&rr, objects, light_pos, depth - 1, skybox);
        accum = accum.add(refl_col.mul(mat.reflectivity));
        weight -= mat.reflectivity;
    }

    // Mezcla final
    local.mul(weight.max(0.0)).add(accum)
}

// Renderiza la escena entera
pub fn render<'a>(
    frame: &mut [u8],
    w: i32,
    h: i32,
    cam: &Camera,
    light_pos: Vec3,
    assets: Assets<'a>,
    max_depth: i32,
) {
    // --- Construcción de escena ---
    let mut objects: Vec<&dyn Intersectable> = Vec::new();

    // Suelo
    let ground_plane;
    let ground_textured;
    if let Some(tex) = &assets.ground {
        ground_textured = TexturedPlane::new(
            Vec3::new(0.0, -0.5, 0.0),
            Vec3::new(0.0, 1.0, 0.0),
            1.5,
            tex.pix, tex.w, tex.h,
        );
        objects.push(&ground_textured);
    } else {
        ground_plane = Plane {
            point: Vec3::new(0.0, -0.5, 0.0),
            normal: Vec3::new(0.0, 1.0, 0.0),
            albedo_color: Vec3::new(0.8, 0.8, 0.8),
            specular_strength: 0.0, shininess: 16.0,
            reflectivity: 0.0, transparency: 0.0, ior: 1.0,
            emissive: Vec3::new(0.0,0.0,0.0),
        };
        objects.push(&ground_plane);
    }

    // OBSIDIANA
    let obs_inner = Aabb {
        min: Vec3::new(-1.6, -0.5, -0.5),
        max: Vec3::new(-0.6,  0.5,  0.5),
        albedo_color: Vec3::new(0.2,0.2,0.25),
        specular_strength: 0.4, shininess: 64.0,
        reflectivity: 0.60, transparency: 0.0, ior: 1.0,
        emissive: Vec3::new(0.0,0.0,0.0),
    };
    let obs_holder;
    if let Some(tex) = &assets.obsidiana {
        obs_holder = Some(TexturedAabb::from_raw(
            obs_inner, tex.pix, tex.w, tex.h,
            0.4, 64.0, 0.60, 0.0, 1.0, Vec3::new(0.0,0.0,0.0)
        ));
        objects.push(obs_holder.as_ref().unwrap());
    } else {
        obs_holder = None;
        objects.push(&obs_inner);
    }

    // AGUA
    let agua_inner = Aabb {
        min: Vec3::new(-0.5, -0.5, -0.5),
        max: Vec3::new( 0.5,  0.5,  0.5),
        albedo_color: Vec3::new(0.9,0.9,1.0),
        specular_strength: 0.05, shininess: 32.0,
        reflectivity: 0.05, transparency: 0.90, ior: 1.33,
        emissive: Vec3::new(0.0,0.0,0.0),
    };
    let agua_holder;
    if let Some(tex) = &assets.agua {
        agua_holder = Some(TexturedAabb::from_raw(
            agua_inner, tex.pix, tex.w, tex.h,
            0.05, 32.0, 0.05, 0.90, 1.33, Vec3::new(0.0,0.0,0.0)
        ));
        objects.push(agua_holder.as_ref().unwrap());
    } else {
        agua_holder = None;
        objects.push(&agua_inner);
    }

    // LAVA
    let lava_inner = Aabb {
        min: Vec3::new(0.7, -0.5, -0.5),
        max: Vec3::new(1.7,  0.5,  0.5),
        albedo_color: Vec3::new(1.0,0.5,0.2),
        specular_strength: 0.1, shininess: 16.0,
        reflectivity: 0.0, transparency: 0.0, ior: 1.0,
        emissive: Vec3::new(1.0,0.4,0.1),
    };
    let lava_holder;
    if let Some(tex) = &assets.lava {
        lava_holder = Some(TexturedAabb::from_raw(
            lava_inner, tex.pix, tex.w, tex.h,
            0.1, 16.0, 0.0, 0.0, 1.0, Vec3::new(1.0,0.4,0.1)
        ));
        objects.push(lava_holder.as_ref().unwrap());
    } else {
        lava_holder = None;
        objects.push(&lava_inner);
    }

    // HIERRO
    let hierro_inner = Aabb {
        min: Vec3::new(-0.2, -0.5, 0.7),
        max: Vec3::new( 0.8,  0.5, 1.7),
        albedo_color: Vec3::new(1.0,1.0,1.0),
        specular_strength: 0.60, shininess: 64.0,
        reflectivity: 0.30, transparency: 0.0, ior: 1.0,
        emissive: Vec3::new(0.0,0.0,0.0),
    };
    let hierro_holder;
    if let Some(tex) = &assets.hierro {
        hierro_holder = Some(TexturedAabb::from_raw(
            hierro_inner, tex.pix, tex.w, tex.h,
            0.60, 64.0, 0.30, 0.0, 1.0, Vec3::new(0.0,0.0,0.0)
        ));
        objects.push(hierro_holder.as_ref().unwrap());
    } else {
        hierro_holder = None;
        objects.push(&hierro_inner);
    }

    // DIAMANTE
    let diam_inner = Aabb {
        min: Vec3::new(-1.4, -0.5, 0.7),
        max: Vec3::new(-0.4,  0.5, 1.7),
        albedo_color: Vec3::new(1.0,1.0,1.0),
        specular_strength: 0.30, shininess: 32.0,
        reflectivity: 0.15, transparency: 0.0, ior: 1.0,
        emissive: Vec3::new(0.0,0.0,0.0),
    };
    let diam_holder;
    if let Some(tex) = &assets.diamante {
        diam_holder = Some(TexturedAabb::from_raw(
            diam_inner, tex.pix, tex.w, tex.h,
            0.30, 32.0, 0.15, 0.0, 1.0, Vec3::new(0.0,0.0,0.0)
        ));
        objects.push(diam_holder.as_ref().unwrap());
    } else {
        diam_holder = None;
        objects.push(&diam_inner);
    }

    // --- Trazado por píxel ---
    let aspect = w as f32 / h as f32;

    for y in 0..h {
        for x in 0..w {
            let u = (x as f32 + 0.5) / w as f32;
            let v = (y as f32 + 0.5) / h as f32;
            let ray = cam.make_ray(u, v, aspect);
            let color = trace(&ray, &objects, light_pos, max_depth, assets.skybox.as_ref());
            let i = ((y * w + x) * 4) as usize;
            frame[i..i + 4].copy_from_slice(&to_rgba(color));
        }
    }
}
