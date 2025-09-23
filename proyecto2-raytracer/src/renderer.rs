use crate::aabb::Aabb;
use crate::camera::Camera;
use crate::object::Intersectable;
use crate::ray::Ray;
use crate::shading::{Skybox, Tex, reflect, refract, sample_skybox, sky, specular_phong, to_rgba};
use crate::textured_aabb::TexturedAabb;
use crate::vec3::Vec3;

struct Hit<'a> {
    t: f32,
    point: Vec3,
    normal: Vec3,
    object: &'a dyn Intersectable,
}

pub struct Assets<'a> {
    pub grass: Option<Tex<'a>>,
    pub dirt: Option<Tex<'a>>,
    pub stone: Option<Tex<'a>>,
    pub wood: Option<Tex<'a>>,
    pub leaves: Option<Tex<'a>>,
    pub lava: Option<Tex<'a>>,
    pub skybox: Option<Skybox<'a>>,
}

// ----------------- helper: añade un bloque -----------------
fn add_block<'a>(
    objects: &mut Vec<Box<dyn Intersectable + 'a>>,
    gx: i32,
    gy: i32,
    gz: i32,
    tex: Option<Tex<'a>>,
    albedo: Vec3,
    spec: f32,
    shininess: f32,
    refl: f32,
    transp: f32,
    ior: f32,
    emissive: Vec3,
) {
    let min = Vec3::new(gx as f32 - 0.5, gy as f32 - 0.5, gz as f32 - 0.5);
    let max = Vec3::new(gx as f32 + 0.5, gy as f32 + 0.5, gz as f32 + 0.5);
    let inner = Aabb {
        min,
        max,
        albedo_color: albedo,
        specular_strength: spec,
        shininess,
        reflectivity: refl,
        transparency: transp,
        ior,
        emissive,
    };

    if let Some(t) = tex {
        objects.push(Box::new(TexturedAabb::from_raw(
            inner, t.pix, t.w, t.h, spec, shininess, refl, transp, ior, emissive,
        )));
    } else {
        objects.push(Box::new(inner));
    }
}

// ----------------- trazado recursivo -----------------
fn trace<'a>(
    ray: &Ray,
    objects: &'a [Box<dyn Intersectable + 'a>],
    light_pos: Vec3,
    depth: i32,
    skybox: Option<&Skybox<'a>>,
) -> Vec3 {
    // Hit más cercano
    let mut closest: Option<Hit> = None;
    for o in objects.iter() {
        if let Some(t) = o.intersect(ray) {
            if t > 0.0 && (closest.is_none() || t < closest.as_ref().unwrap().t) {
                let p = ray.orig.add(ray.dir.mul(t));
                let n = o.normal_at(p);
                closest = Some(Hit {
                    t,
                    point: p,
                    normal: n,
                    object: o.as_ref(),
                });
            }
        }
    }

    // Fondo
    if closest.is_none() {
        return if let Some(sb) = skybox {
            sample_skybox(ray.dir, sb)
        } else {
            sky(ray.dir)
        };
    }

    let hit = closest.unwrap();
    let mat = hit.object.material_at(hit.point);

    // Sombras
    let bias = 1e-3;
    let shadow_origin = hit.point.add(hit.normal.mul(bias));
    let ldir = light_pos.sub(hit.point).norm();
    let light_distance = light_pos.sub(hit.point).len();
    let sray = Ray {
        orig: shadow_origin,
        dir: ldir,
    };
    let mut in_shadow = false;
    for o in objects.iter() {
        if let Some(t) = o.intersect(&sray) {
            if t < light_distance {
                in_shadow = true;
                break;
            }
        }
    }

    // Phong local
    let ambient = 0.1;
    let n = hit.normal.norm();
    let l = ldir;
    let v = (-ray.dir).norm();
    let ndotl = n.dot(l).max(0.0);
    let mut local = mat
        .albedo
        .mul(ambient + if in_shadow { 0.0 } else { ndotl });

    if !in_shadow && ndotl > 0.0 && mat.specular_strength > 0.0 {
        let r = reflect(-l, n);
        let spec = specular_phong(r, v, mat.specular_strength, mat.shininess);
        local = local.add(Vec3::new(spec, spec, spec));
    }

    // Emisión
    local = local.add(mat.emissive);

    if depth <= 0 {
        return local;
    }

    // Rayos secundarios
    let mut accum = Vec3::new(0.0, 0.0, 0.0);
    let mut weight = 1.0;

    // Refracción (si aplica)
    if mat.transparency > 0.0 {
        let mut n_out = n;
        let mut eta = 1.0 / mat.ior;
        if ray.dir.dot(n) > 0.0 {
            n_out = -n;
            eta = mat.ior;
        }
        if let Some(tdir) = refract(ray.dir, n_out, eta) {
            let ro = hit.point.add(tdir.mul(bias));
            let rr = Ray {
                orig: ro,
                dir: tdir,
            };
            let refr_col = trace(&rr, objects, light_pos, depth - 1, skybox);
            accum = accum.add(refr_col.mul(mat.transparency));
            weight -= mat.transparency;
        }
    }

    // Reflexión (si aplica)
    if mat.reflectivity > 0.0 && weight > 0.0 {
        let rdir = reflect(ray.dir, n).norm();
        let ro = hit.point.add(n.mul(bias));
        let rr = Ray {
            orig: ro,
            dir: rdir,
        };
        let refl_col = trace(&rr, objects, light_pos, depth - 1, skybox);
        accum = accum.add(refl_col.mul(mat.reflectivity));
        weight -= mat.reflectivity;
    }

    local.mul(weight.max(0.0)).add(accum)
}

// ----------------- escena skyblock -----------------
pub fn render<'a>(
    frame: &mut [u8],
    w: i32,
    h: i32,
    cam: &Camera,
    light_pos: Vec3,
    assets: Assets<'a>,
    max_depth: i32,
) {
    let aspect = w as f32 / h as f32;

    // Objetos (Box -> direcciones estables, sin líos de lifetimes)
    let mut objects: Vec<Box<dyn Intersectable + 'a>> = Vec::new();

    // Texturas
    let grass_tex = assets.grass;
    let dirt_tex = assets.dirt;
    let stone_tex = assets.stone;
    let wood_tex = assets.wood;
    let leaves_tex = assets.leaves;
    let lava_tex = assets.lava;

    // Materiales base
    // (spec, shin, refl, transp, ior, emissive)
    let grass_p = (0.05, 16.0, 0.02, 0.0, 1.0, Vec3::new(0.0, 0.0, 0.0));
    let dirt_p = (0.00, 8.0, 0.00, 0.0, 1.0, Vec3::new(0.0, 0.0, 0.0));
    let stone_p = (0.05, 32.0, 0.02, 0.0, 1.0, Vec3::new(0.0, 0.0, 0.0));
    let wood_p = (0.03, 16.0, 0.02, 0.0, 1.0, Vec3::new(0.0, 0.0, 0.0));
    let leaves_p = (0.02, 8.0, 0.00, 0.25, 1.0, Vec3::new(0.0, 0.0, 0.0)); // leve transparencia
    let lava_p = (0.05, 8.0, 0.00, 0.00, 1.0, Vec3::new(1.0, 0.4, 0.1)); // emisivo

    // -------- Isla ----------
    // Capa superior (y=0): forma en L, pero dejamos (2,0,1) libre para la lava
    let top_positions = [
        (0, 0, 0),
        (1, 0, 0),
        (2, 0, 0),
        (0, 0, 1),
        (1, 0, 1), /*(2,0,1) lava aquí */
        (0, 0, 2),
        (1, 0, 2), // (2,0,2) ausente para L
    ];
    for (x, y, z) in top_positions {
        add_block(
            &mut objects,
            x,
            y,
            z,
            grass_tex,
            Vec3::new(1.0, 1.0, 1.0),
            grass_p.0,
            grass_p.1,
            grass_p.2,
            grass_p.3,
            grass_p.4,
            grass_p.5,
        );
    }

    // Lava arriba a la derecha (2,0,1)
    add_block(
        &mut objects,
        2,
        0,
        1,
        lava_tex,
        Vec3::new(1.0, 1.0, 1.0),
        lava_p.0,
        lava_p.1,
        lava_p.2,
        lava_p.3,
        lava_p.4,
        lava_p.5,
    );

    // Capa media (y=-1): 3x3 de tierra
    for x in 0..3 {
        for z in 0..3 {
            add_block(
                &mut objects,
                x,
                -1,
                z,
                dirt_tex,
                Vec3::new(1.0, 1.0, 1.0),
                dirt_p.0,
                dirt_p.1,
                dirt_p.2,
                dirt_p.3,
                dirt_p.4,
                dirt_p.5,
            );
        }
    }

    // Capa inferior (y=-2): 2x2 centrado
    for x in 1..3 {
        for z in 1..3 {
            add_block(
                &mut objects,
                x - 1,
                -2,
                z - 1,
                dirt_tex,
                Vec3::new(1.0, 1.0, 1.0),
                dirt_p.0,
                dirt_p.1,
                dirt_p.2,
                dirt_p.3,
                dirt_p.4,
                dirt_p.5,
            );
        }
    }

    // Bloque de piedra saliente en el frente para que se vea
    add_block(
        &mut objects,
        0,
        -1,
        3,
        stone_tex,
        Vec3::new(1.0, 1.0, 1.0),
        stone_p.0,
        stone_p.1,
        stone_p.2,
        stone_p.3,
        stone_p.4,
        stone_p.5,
    );

    // -------- Árbol ----------
    // Tronco (columna) en (0,*,1), desde y=1..3
    for y in 1..=3 {
        add_block(
            &mut objects,
            0,
            y,
            1,
            wood_tex,
            Vec3::new(1.0, 1.0, 1.0),
            wood_p.0,
            wood_p.1,
            wood_p.2,
            wood_p.3,
            wood_p.4,
            wood_p.5,
        );
    }

    // Copa de hojas centrada en (0,3,1)
    let cx = 0;
    let cz = 1;
    for y in 3..=4 {
        for dx in -1..=1 {
            for dz in -1..=1 {
                // evita el centro en y=3 (donde está el tronco)
                if y == 3 && dx == 0 && dz == 0 {
                    continue;
                }
                add_block(
                    &mut objects,
                    cx + dx,
                    y,
                    cz + dz,
                    leaves_tex,
                    Vec3::new(1.0, 1.0, 1.0),
                    leaves_p.0,
                    leaves_p.1,
                    leaves_p.2,
                    leaves_p.3,
                    leaves_p.4,
                    leaves_p.5,
                );
            }
        }
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
