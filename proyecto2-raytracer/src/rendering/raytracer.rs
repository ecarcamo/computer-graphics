//! Construye la escena de bloques y ejecuta el trazador de rayos en CPU.

use std::collections::HashSet;
use std::thread;

use super::lighting::{Skybox, Tex, reflect, refract, sample_skybox, sky, specular_phong, to_rgba};
use crate::camera::Camera;
use crate::geometry::{SolidBlock, TexturedBlock};
use crate::math::Vec3;
use crate::ray::Ray;
use crate::scene::Intersectable;

type DynObject<'a> = Box<dyn Intersectable + Send + Sync + 'a>;

/// Identifica qué diorama (Overworld o Nether) se va a renderizar.
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

/// Datos de intersección utilizados durante el recorrido de rayos.
struct Hit<'a> {
    t: f32,
    point: Vec3,
    normal: Vec3,
    object: &'a dyn Intersectable,
}

/// Manejadores de texturas y skyboxes que permanecen válidos durante el render.
#[derive(Copy, Clone)]
pub struct Assets<'a> {
    pub grass_cover: Option<Tex<'a>>,
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
    pub ice: Option<Tex<'a>>,
    pub portal: Option<Tex<'a>>,
    pub skybox_overworld: Option<Skybox<'a>>,
    pub skybox_nether: Option<Skybox<'a>>,
}

/// Geometría ya preparada para renderizar.
pub struct SceneData<'a> {
    pub objects: Vec<DynObject<'a>>,
    pub skybox: Option<Skybox<'a>>,
}

/// Inserta un cubo sólido o texturizado en la lista de objetos.
fn push_block<'a>(objects: &mut Vec<DynObject<'a>>, min: Vec3, max: Vec3, mat: BlockMaterial<'a>) {
    let inner = SolidBlock {
        min,
        max,
        albedo_color: mat.albedo,
        specular_strength: mat.specular,
        shininess: mat.shininess,
        reflectivity: mat.reflectivity,
        transparency: mat.transparency,
        ior: mat.ior,
        emissive: mat.emissive,
    };

    if let Some(t) = mat.tex {
        objects.push(Box::new(TexturedBlock::from_raw(
            inner,
            t.pix,
            t.w,
            t.h,
            mat.specular,
            mat.shininess,
            mat.reflectivity,
            mat.transparency,
            mat.ior,
            mat.emissive,
        )));
    } else {
        objects.push(Box::new(inner));
    }
}

#[derive(Copy, Clone)]
/// Agrupa la textura y los parámetros de material de un bloque específico.
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
    /// Inserta un cubo completo en las coordenadas indicadas.
    fn place(&self, objects: &mut Vec<DynObject<'a>>, x: i32, y: i32, z: i32) {
        let min = Vec3::new(x as f32 - 0.5, y as f32 - 0.5, z as f32 - 0.5);
        let max = Vec3::new(x as f32 + 0.5, y as f32 + 0.5, z as f32 + 0.5);
        push_block(objects, min, max, *self);
    }

    /// Coloca solo la “rebanada” superior (para superponer césped sobre tierra).
    fn place_cover(
        &self,
        objects: &mut Vec<DynObject<'a>>,
        x: i32,
        y: i32,
        z: i32,
        thickness: f32,
    ) {
        let top = y as f32 + 0.5;
        let min = Vec3::new(x as f32 - 0.5, top - thickness, z as f32 - 0.5);
        let max = Vec3::new(x as f32 + 0.5, top, z as f32 + 0.5);
        push_block(objects, min, max, *self);
    }
}

/// Inserta un bloque sólo si no se ha colocado antes con el mismo tag.
fn place_with_tag<'a>(
    objects: &mut Vec<DynObject<'a>>,
    used: &mut HashSet<(i32, i32, i32, u8)>,
    mat: BlockMaterial<'a>,
    x: i32,
    y: i32,
    z: i32,
    tag: u8,
) {
    if used.insert((x, y, z, tag)) {
        mat.place(objects, x, y, z);
    }
}

/// Envoltorio para insertar un bloque sólido una sola vez.
fn place_block<'a>(
    objects: &mut Vec<DynObject<'a>>,
    used: &mut HashSet<(i32, i32, i32, u8)>,
    mat: BlockMaterial<'a>,
    x: i32,
    y: i32,
    z: i32,
) {
    place_with_tag(objects, used, mat, x, y, z, 0);
}

/// Rutina de trazado recursivo con poca profundidad para reflejos/refracciones.
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

    // Cálculo de sombra simple (shadow ray).
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

    // Iluminación local (Phong).
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

    // Componentes emisivas.
    local = local.add(mat.emissive);

    if depth <= 0 {
        return local;
    }

    // Rayos secundarios para refracción/reflexión.
    let mut accum = Vec3::new(0.0, 0.0, 0.0);
    let mut weight = 1.0;

    // Refracción (si aplica).
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

    // Reflexión especular (si aplica).
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
/// Genera todos los bloques del mundo actual usando los assets disponibles.
pub fn build_scene<'a>(assets: &Assets<'a>, world: WorldKind) -> SceneData<'a> {
    let mut objects: Vec<DynObject<'a>> = Vec::new();

    // Materiales disponibles
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
        transparency: 0.15,
        ior: 1.2,
        emissive: Vec3::new(0.0, 0.0, 0.0),
    };
    let water_mat = BlockMaterial {
        tex: assets.water,
        albedo: Vec3::new(0.85, 0.9, 1.0),
        specular: 0.14,
        shininess: 85.0,
        reflectivity: 0.08,
        transparency: 0.92,
        ior: 1.333,
        emissive: Vec3::new(0.0, 0.0, 0.0),
    };
    let lava_mat = BlockMaterial {
        tex: assets.lava,
        albedo: Vec3::new(1.0, 0.9, 0.85),
        specular: 0.2,
        shininess: 35.0,
        reflectivity: 0.08,
        transparency: 0.0,
        ior: 1.0,
        emissive: Vec3::new(2.2, 0.9, 0.25),
    };
    let obsidian_mat = BlockMaterial {
        tex: assets.obsidian,
        albedo: Vec3::new(0.6, 0.65, 0.8),
        specular: 0.18,
        shininess: 70.0,
        reflectivity: 0.08,
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
        specular: 0.85,
        shininess: 110.0,
        reflectivity: 0.15,
        transparency: 0.0,
        ior: 2.4,
        emissive: Vec3::new(0.0, 0.0, 0.0),
    };
    let iron_mat = BlockMaterial {
        tex: assets.iron,
        albedo: Vec3::new(0.95, 0.95, 0.98),
        specular: 0.4,
        shininess: 75.0,
        reflectivity: 0.1,
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
    let grass_cover_mat = BlockMaterial {
        tex: assets.grass_cover,
        albedo: Vec3::new(0.95, 1.0, 0.95),
        specular: 0.08,
        shininess: 20.0,
        reflectivity: 0.01,
        transparency: 0.0,
        ior: 1.0,
        emissive: Vec3::new(0.0, 0.0, 0.0),
    };
    let ice_mat = BlockMaterial {
        tex: assets.ice,
        albedo: Vec3::new(0.8, 0.9, 1.0),
        specular: 0.2,
        shininess: 70.0,
        reflectivity: 0.08,
        transparency: 0.6,
        ior: 1.31,
        emissive: Vec3::new(0.0, 0.0, 0.0),
    };
    let portal_mat = BlockMaterial {
        tex: assets.portal,
        albedo: Vec3::new(1.0, 0.4, 1.2),
        specular: 0.6,
        shininess: 60.0,
        reflectivity: 0.12,
        transparency: 0.55,
        ior: 1.6,
        emissive: Vec3::new(1.5, 0.3, 1.8),
    };

    let mut used: HashSet<(i32, i32, i32, u8)> = HashSet::new();
    let center_x: i32 = 3;
    let center_z: i32 = 2;
    let adjust = |x: i32, z: i32| -> (i32, i32) { (x - center_x, z - center_z) };

    match world {
        WorldKind::Overworld => {
            let max_x = 6;
            let max_z = 5;

            let inner_grass: HashSet<(i32, i32)> = [
                (1, 1),
                (1, 2),
                (1, 3),
                (2, 1),
                (2, 2),
                (3, 1),
                (3, 2),
                (4, 2),
                (4, 3),
                (5, 2),
                (5, 3),
                (5, 4),
            ]
            .into_iter()
            .collect();

            let water_tiles = [(2, 3), (3, 3)];
            let lava_tiles = [(2, 4)];
            let chest_tile = (1, 4);
            let pedestal_tiles = [(2, 1), (3, 1)];
            let portal_base_tiles = [(4, 2), (5, 2), (6, 2)];
            let cascade_surface = (3, 4);

            let mut occupied: HashSet<(i32, i32)> = portal_base_tiles.iter().copied().collect();
            occupied.insert(chest_tile);
            occupied.insert(cascade_surface);
            for &(x, z) in &water_tiles {
                occupied.insert((x, z));
            }
            for &(x, z) in &lava_tiles {
                occupied.insert((x, z));
            }
            for &(x, z) in &pedestal_tiles {
                occupied.insert((x, z));
            }

            let no_cover: HashSet<(i32, i32)> = occupied.clone();

            for x in 0..=max_x {
                for z in 0..=max_z {
                    let boundary = x == 0 || x == max_x || z == 0 || z == max_z;
                    let fill = boundary || inner_grass.contains(&(x, z));
                    if !fill {
                        continue;
                    }
                    if !occupied.contains(&(x, z)) {
                        let (sx, sz) = adjust(x, z);
                        place_block(&mut objects, &mut used, dirt_mat, sx, 0, sz);
                        if !no_cover.contains(&(x, z)) {
                            grass_cover_mat.place_cover(&mut objects, sx, 0, sz, 0.18);
                        }
                    }
                }
            }

            for &(x, z) in &water_tiles {
                let (sx, sz) = adjust(x, z);
                place_with_tag(&mut objects, &mut used, water_mat, sx, 0, sz, 1);
                place_with_tag(&mut objects, &mut used, stone_mat, sx, -1, sz, 1);
            }

            let (sx_ice, sz_ice) = adjust(4, 4);
            place_with_tag(&mut objects, &mut used, ice_mat, sx_ice, 0, sz_ice, 1);
            place_with_tag(&mut objects, &mut used, stone_mat, sx_ice, -1, sz_ice, 1);
            for &(y, z) in &[(0, 4), (-1, 4), (-2, 4), (-3, 4)] {
                let (sx, sz) = adjust(3, z);
                place_with_tag(&mut objects, &mut used, water_mat, sx, y, sz, 1);
            }
            let (sx_cascade, sz_cascade) = adjust(3, 5);
            place_with_tag(
                &mut objects,
                &mut used,
                water_mat,
                sx_cascade,
                -3,
                sz_cascade,
                1,
            );
            place_with_tag(
                &mut objects,
                &mut used,
                stone_mat,
                sx_cascade,
                -4,
                sz_cascade,
                1,
            );

            for &(x, z) in &lava_tiles {
                let (sx, sz) = adjust(x, z);
                place_with_tag(&mut objects, &mut used, lava_mat, sx, 0, sz, 1);
                place_with_tag(&mut objects, &mut used, stone_mat, sx, -1, sz, 1);
            }

            for y in 1..=3 {
                let (sx, sz) = adjust(2, 2);
                place_with_tag(&mut objects, &mut used, wood_mat, sx, y, sz, 1);
            }
            for y in 3..=4 {
                for dx in -1..=1 {
                    for dz in -1..=1 {
                        if y == 3 && dx == 0 && dz == 0 {
                            continue;
                        }
                        let (sx, sz) = adjust(2 + dx, 2 + dz);
                        place_with_tag(&mut objects, &mut used, leaves_mat, sx, y, sz, 1);
                    }
                }
            }

            let (sx_ped1, sz_ped1) = adjust(2, 1);
            place_with_tag(&mut objects, &mut used, stone_mat, sx_ped1, 0, sz_ped1, 1);
            place_with_tag(&mut objects, &mut used, diamond_mat, sx_ped1, 1, sz_ped1, 1);
            let (sx_ped2, sz_ped2) = adjust(3, 1);
            place_with_tag(&mut objects, &mut used, stone_mat, sx_ped2, 0, sz_ped2, 1);
            place_with_tag(&mut objects, &mut used, iron_mat, sx_ped2, 1, sz_ped2, 1);
            let (sx_chest, sz_chest) = adjust(chest_tile.0, chest_tile.1);
            place_with_tag(&mut objects, &mut used, chest_mat, sx_chest, 0, sz_chest, 1);

            let portal_left = 4;
            let portal_right = 6;
            let portal_z = 2;
            let portal_top = 4;
            for y in 0..=portal_top {
                let (sx_l, sz) = adjust(portal_left, portal_z);
                let (sx_r, sz_r) = adjust(portal_right, portal_z);
                place_with_tag(&mut objects, &mut used, obsidian_mat, sx_l, y, sz, 1);
                place_with_tag(&mut objects, &mut used, obsidian_mat, sx_r, y, sz_r, 1);
            }
            for x in portal_left..=portal_right {
                let (sx, sz) = adjust(x, portal_z);
                place_with_tag(&mut objects, &mut used, obsidian_mat, sx, 0, sz, 1);
                place_with_tag(&mut objects, &mut used, obsidian_mat, sx, -1, sz, 1);
                place_with_tag(&mut objects, &mut used, obsidian_mat, sx, portal_top, sz, 1);
            }
            for y in 1..portal_top {
                for x in (portal_left + 1)..portal_right {
                    let (sx, sz) = adjust(x, portal_z);
                    place_with_tag(&mut objects, &mut used, portal_mat, sx, y, sz, 1);
                }
            }
        }
        WorldKind::Nether => {
            let max_x = 6;
            let max_z = 5;

            for x in 0..=max_x {
                for z in 0..=max_z {
                    if x == 0 || x == max_x || z == 0 || z == max_z {
                        let (sx, sz) = adjust(x, z);
                        place_block(&mut objects, &mut used, obsidian_mat, sx, -1, sz);
                        place_with_tag(&mut objects, &mut used, obsidian_mat, sx, 0, sz, 1);
                    }
                }
            }

            for &(x, z) in &[(1, 1), (1, 4), (4, 1), (4, 4)] {
                let (sx, sz) = adjust(x, z);
                place_with_tag(&mut objects, &mut used, obsidian_mat, sx, 0, sz, 1);
            }

            for x in 2..=3 {
                for z in 2..=3 {
                    let (sx, sz) = adjust(x, z);
                    place_with_tag(&mut objects, &mut used, lava_mat, sx, 0, sz, 1);
                }
            }

            for &(x, z) in &[(1, 1), (4, 4)] {
                for y in 1..=3 {
                    let (sx, sz) = adjust(x, z);
                    place_with_tag(&mut objects, &mut used, obsidian_mat, sx, y, sz, 1);
                }
                let (sx, sz) = adjust(x, z);
                place_with_tag(&mut objects, &mut used, glowstone_mat, sx, 4, sz, 1);
            }
            for &(x, z) in &[(1, 4), (4, 1)] {
                let (sx, sz) = adjust(x, z);
                place_with_tag(&mut objects, &mut used, glowstone_mat, sx, 1, sz, 1);
            }

            let (sx_dia, sz_dia) = adjust(2, 4);
            place_with_tag(&mut objects, &mut used, diamond_mat, sx_dia, 1, sz_dia, 1);
            let (sx_iron, sz_iron) = adjust(3, 1);
            place_with_tag(&mut objects, &mut used, iron_mat, sx_iron, 1, sz_iron, 1);

            let portal_left = 4;
            let portal_right = 6;
            let portal_z = 2;
            let portal_top = 4;
            for y in 0..=portal_top {
                let (sx_l, sz) = adjust(portal_left, portal_z);
                let (sx_r, sz_r) = adjust(portal_right, portal_z);
                place_with_tag(&mut objects, &mut used, obsidian_mat, sx_l, y, sz, 1);
                place_with_tag(&mut objects, &mut used, obsidian_mat, sx_r, y, sz_r, 1);
            }
            for x in portal_left..=portal_right {
                let (sx, sz) = adjust(x, portal_z);
                place_with_tag(&mut objects, &mut used, obsidian_mat, sx, 0, sz, 1);
                place_with_tag(&mut objects, &mut used, obsidian_mat, sx, -1, sz, 1);
                place_with_tag(&mut objects, &mut used, obsidian_mat, sx, portal_top, sz, 1);
            }
            for y in 1..portal_top {
                for x in (portal_left + 1)..portal_right {
                    let (sx, sz) = adjust(x, portal_z);
                    place_with_tag(&mut objects, &mut used, portal_mat, sx, y, sz, 1);
                }
            }

            let (sx_lava1, sz_lava1) = adjust(2, 2);
            place_with_tag(&mut objects, &mut used, lava_mat, sx_lava1, -1, sz_lava1, 1);
            let (sx_lava2, sz_lava2) = adjust(3, 3);
            place_with_tag(&mut objects, &mut used, lava_mat, sx_lava2, -1, sz_lava2, 1);
        }
    }

    let skybox = match world {
        WorldKind::Overworld => assets.skybox_overworld,
        WorldKind::Nether => assets.skybox_nether,
    };

    SceneData { objects, skybox }
}

/// Traza la escena resultante y escribe el color final (RGBA) dentro de `frame`.
pub fn render<'a>(
    frame: &mut [u8],
    w: i32,
    h: i32,
    cam: &Camera,
    light_pos: Vec3,
    scene: &SceneData<'a>,
    max_depth: i32,
) {
    let aspect = w as f32 / h as f32;
    let width = w as usize;
    let height = h as usize;

    let threads = thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(1)
        .min(height.max(1));
    let rows_per_chunk = (height + threads - 1) / threads;
    let pixels_per_row = width * 4;
    let objects_ref: &[DynObject<'a>] = &scene.objects;
    let skybox = scene.skybox.as_ref();

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
