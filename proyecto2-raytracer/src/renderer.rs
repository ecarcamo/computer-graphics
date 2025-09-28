use crate::aabb::Aabb;
use crate::camera::Camera;
use crate::object::Intersectable;
use crate::ray::Ray;
use crate::shading::{Skybox, Tex, reflect, refract, sample_skybox, sky, specular_phong, to_rgba};
use crate::textured_aabb::TexturedAabb;
use crate::vec3::Vec3;
use std::thread;

type DynObject<'a> = Box<dyn Intersectable + Send + Sync + 'a>;

#[derive(Copy, Clone, Eq, PartialEq)]
pub enum WorldKind {
    Overworld,
    Nether,
}

impl WorldKind {
    pub fn toggle(self) -> Self {
        match self {
            WorldKind::Overworld => WorldKind::Nether,
            WorldKind::Nether => WorldKind::Overworld,
        }
    }
}

struct Hit<'a> {
    t: f32,
    point: Vec3,
    normal: Vec3,
    object: &'a dyn Intersectable,
}

#[derive(Copy, Clone)]
pub struct Assets<'a> {
    pub grass: Option<Tex<'a>>,
    pub dirt: Option<Tex<'a>>,
    pub stone: Option<Tex<'a>>,
    pub wood: Option<Tex<'a>>,
    pub leaves: Option<Tex<'a>>,
    pub water: Option<Tex<'a>>,
    pub lava: Option<Tex<'a>>,
    pub obsidian: Option<Tex<'a>>,
    pub glowstone: Option<Tex<'a>>,
    pub diamond: Option<Tex<'a>>,
    pub iron: Option<Tex<'a>>,
    pub chest: Option<Tex<'a>>,
    pub skybox_overworld: Option<Skybox<'a>>,
    pub skybox_nether: Option<Skybox<'a>>,
}

// ----------------- helper: añade un bloque -----------------
fn add_block<'a>(
    objects: &mut Vec<DynObject<'a>>,
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

#[derive(Copy, Clone)]
struct BlockMaterial<'a> {
    tex: Option<Tex<'a>>,
    albedo: Vec3,
    specular: f32,
    shininess: f32,
    reflectivity: f32,
    transparency: f32,
    ior: f32,
    emissive: Vec3,
}

impl<'a> BlockMaterial<'a> {
    fn place(&self, objects: &mut Vec<DynObject<'a>>, x: i32, y: i32, z: i32) {
        add_block(
            objects,
            x,
            y,
            z,
            self.tex,
            self.albedo,
            self.specular,
            self.shininess,
            self.reflectivity,
            self.transparency,
            self.ior,
            self.emissive,
        );
    }
}

// ----------------- trazado recursivo -----------------
fn trace<'a>(
    ray: &Ray,
    objects: &'a [DynObject<'a>],
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
    assets: &Assets<'a>,
    max_depth: i32,
    world: WorldKind,
) {
    let aspect = w as f32 / h as f32;
    let width = w as usize;
    let height = h as usize;

    let mut objects: Vec<DynObject<'a>> = Vec::new();

    // Materiales disponibles
    let grass_mat = BlockMaterial {
        tex: assets.grass,
        albedo: Vec3::new(0.95, 1.0, 0.95),
        specular: 0.08,
        shininess: 20.0,
        reflectivity: 0.02,
        transparency: 0.0,
        ior: 1.0,
        emissive: Vec3::new(0.0, 0.0, 0.0),
    };
    let dirt_mat = BlockMaterial {
        tex: assets.dirt,
        albedo: Vec3::new(0.85, 0.76, 0.6),
        specular: 0.02,
        shininess: 10.0,
        reflectivity: 0.0,
        transparency: 0.0,
        ior: 1.0,
        emissive: Vec3::new(0.0, 0.0, 0.0),
    };
    let stone_mat = BlockMaterial {
        tex: assets.stone,
        albedo: Vec3::new(0.95, 0.95, 0.95),
        specular: 0.18,
        shininess: 40.0,
        reflectivity: 0.05,
        transparency: 0.0,
        ior: 1.0,
        emissive: Vec3::new(0.0, 0.0, 0.0),
    };
    let wood_mat = BlockMaterial {
        tex: assets.wood,
        albedo: Vec3::new(1.0, 0.98, 0.92),
        specular: 0.04,
        shininess: 18.0,
        reflectivity: 0.01,
        transparency: 0.0,
        ior: 1.0,
        emissive: Vec3::new(0.0, 0.0, 0.0),
    };
    let leaves_mat = BlockMaterial {
        tex: assets.leaves,
        albedo: Vec3::new(0.7, 1.0, 0.75),
        specular: 0.05,
        shininess: 12.0,
        reflectivity: 0.03,
        transparency: 0.35,
        ior: 1.2,
        emissive: Vec3::new(0.0, 0.0, 0.0),
    };
    let water_mat = BlockMaterial {
        tex: assets.water,
        albedo: Vec3::new(0.85, 0.9, 1.0),
        specular: 0.12,
        shininess: 80.0,
        reflectivity: 0.06,
        transparency: 0.9,
        ior: 1.333,
        emissive: Vec3::new(0.0, 0.0, 0.0),
    };
    let lava_mat = BlockMaterial {
        tex: assets.lava,
        albedo: Vec3::new(1.0, 0.9, 0.85),
        specular: 0.1,
        shininess: 22.0,
        reflectivity: 0.0,
        transparency: 0.0,
        ior: 1.0,
        emissive: Vec3::new(2.0, 0.8, 0.2),
    };
    let obsidian_mat = BlockMaterial {
        tex: assets.obsidian,
        albedo: Vec3::new(0.6, 0.65, 0.8),
        specular: 0.25,
        shininess: 90.0,
        reflectivity: 0.25,
        transparency: 0.0,
        ior: 1.46,
        emissive: Vec3::new(0.0, 0.0, 0.0),
    };
    let glowstone_mat = BlockMaterial {
        tex: assets.glowstone,
        albedo: Vec3::new(1.0, 0.95, 0.8),
        specular: 0.22,
        shininess: 28.0,
        reflectivity: 0.02,
        transparency: 0.0,
        ior: 1.0,
        emissive: Vec3::new(3.5, 3.2, 2.6),
    };
    let diamond_mat = BlockMaterial {
        tex: assets.diamond,
        albedo: Vec3::new(1.0, 1.0, 1.0),
        specular: 0.9,
        shininess: 120.0,
        reflectivity: 0.45,
        transparency: 0.1,
        ior: 2.4,
        emissive: Vec3::new(0.0, 0.0, 0.0),
    };
    let iron_mat = BlockMaterial {
        tex: assets.iron,
        albedo: Vec3::new(0.95, 0.95, 0.98),
        specular: 0.4,
        shininess: 70.0,
        reflectivity: 0.2,
        transparency: 0.0,
        ior: 1.0,
        emissive: Vec3::new(0.0, 0.0, 0.0),
    };
    let chest_mat = BlockMaterial {
        tex: assets.chest,
        albedo: Vec3::new(1.0, 0.95, 0.85),
        specular: 0.06,
        shininess: 18.0,
        reflectivity: 0.01,
        transparency: 0.0,
        ior: 1.0,
        emissive: Vec3::new(0.0, 0.0, 0.0),
    };

    match world {
        WorldKind::Overworld => {
            // Capa superior
            for &(x, z) in &[(0, 0), (1, 0), (2, 0), (0, 1), (0, 2), (1, 2), (2, 2)] {
                grass_mat.place(&mut objects, x, 0, z);
            }

            // Agua central (refracción) y lava adyacente
            water_mat.place(&mut objects, 1, 0, 1);
            lava_mat.place(&mut objects, 2, 0, 1);

            // Plataforma inferior
            for x in 0..3 {
                for z in 0..3 {
                    dirt_mat.place(&mut objects, x, -1, z);
                }
            }
            for x in 1..3 {
                for z in 1..3 {
                    dirt_mat.place(&mut objects, x - 1, -2, z - 1);
                }
            }

            // Bloque de piedra frontal
            stone_mat.place(&mut objects, 0, -1, 3);

            // Tronco
            for y in 1..=3 {
                wood_mat.place(&mut objects, 0, y, 1);
            }
            // Copa de hojas
            let cx = 0;
            let cz = 1;
            for y in 3..=4 {
                for dx in -1..=1 {
                    for dz in -1..=1 {
                        if y == 3 && dx == 0 && dz == 0 {
                            continue;
                        }
                        leaves_mat.place(&mut objects, cx + dx, y, cz + dz);
                    }
                }
            }

            // Minerales y utilería
            diamond_mat.place(&mut objects, 2, 1, 0);
            iron_mat.place(&mut objects, 1, 1, 0);
            chest_mat.place(&mut objects, 2, 1, 2);
        }
        WorldKind::Nether => {
            // Plataforma base de obsidiana
            for x in 0..4 {
                for z in 0..4 {
                    obsidian_mat.place(&mut objects, x, -1, z);
                    if x == 0 || x == 3 || z == 0 || z == 3 {
                        obsidian_mat.place(&mut objects, x, 0, z);
                    }
                }
            }

            // Piscina de lava en el centro
            for x in 1..3 {
                for z in 1..3 {
                    lava_mat.place(&mut objects, x, 0, z);
                }
            }

            // Pilares de obsidiana con glowstone
            for &(x, z) in &[(0, 0), (3, 3)] {
                for y in 0..3 {
                    obsidian_mat.place(&mut objects, x, y, z);
                }
                glowstone_mat.place(&mut objects, x, 3, z);
            }

            // Decoración adicional
            obsidian_mat.place(&mut objects, 3, 0, 1);
            obsidian_mat.place(&mut objects, 3, 0, 2);

            glowstone_mat.place(&mut objects, 1, 1, 3);
            glowstone_mat.place(&mut objects, 2, 1, 0);

            // Fragmentos de diamante incrustados
            diamond_mat.place(&mut objects, 0, 0, 3);
            diamond_mat.place(&mut objects, 3, 1, 2);
        }
    }

    let skybox = match world {
        WorldKind::Overworld => assets.skybox_overworld.as_ref(),
        WorldKind::Nether => assets.skybox_nether.as_ref(),
    };

    let threads = thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(1)
        .min(height.max(1));
    let rows_per_chunk = (height + threads - 1) / threads;
    let pixels_per_row = width * 4;
    let objects_ref: &[DynObject<'a>] = &objects;

    thread::scope(|scope| {
        let mut start_row = 0usize;
        let mut remaining: &mut [u8] = frame;
        for _ in 0..threads {
            if start_row >= height {
                break;
            }
            let rows_left = height - start_row;
            let rows_here = rows_per_chunk.min(rows_left);
            let bytes_here = rows_here * pixels_per_row;
            let (chunk, rest) = remaining.split_at_mut(bytes_here);
            let chunk_start = start_row;
            remaining = rest;
            let cam_ref = cam;
            scope.spawn(move || {
                for (row_offset, row) in chunk.chunks_mut(pixels_per_row).enumerate() {
                    let y = (chunk_start + row_offset) as i32;
                    let v = (y as f32 + 0.5) / h as f32;
                    for x in 0..width {
                        let u = (x as f32 + 0.5) / w as f32;
                        let ray = cam_ref.make_ray(u, v, aspect);
                        let color = trace(&ray, objects_ref, light_pos, max_depth, skybox);
                        let idx = x * 4;
                        row[idx..idx + 4].copy_from_slice(&to_rgba(color));
                    }
                }
            });
            start_row += rows_here;
        }
    });
}
