//! Builds the block-based scenes and performs the CPU raytracing loop.

use std::collections::HashSet;
use std::thread;

use super::lighting::{Skybox, Tex, reflect, refract, sample_skybox, sky, specular_phong, to_rgba};
use crate::camera::Camera;
use crate::geometry::{SolidBlock, TexturedBlock};
use crate::math::Vec3;
use crate::ray::Ray;
use crate::scene::Intersectable;

type DynObject<'a> = Box<dyn Intersectable + Send + Sync + 'a>;

/// Identifies which themed diorama should be rendered.
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

/// Intersection bookkeeping used during ray traversal.
struct Hit<'a> {
    t: f32,
    point: Vec3,
    normal: Vec3,
    object: &'a dyn Intersectable,
}

/// Texture handles and optional skyboxes that remain valid while rendering a frame.
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

/// Fully prepared world geometry ready to be rendered.
pub struct SceneData<'a> {
    pub objects: Vec<DynObject<'a>>,
    pub skybox: Option<Skybox<'a>>,
}

/// Allocates either a solid or textured cube and pushes it into the scene list.
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
/// Bundles texture data and material coefficients for a single block type.
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
    /// Inserts a full cube for this material at the requested grid coordinates.
    fn place(&self, objects: &mut Vec<DynObject<'a>>, x: i32, y: i32, z: i32) {
        let min = Vec3::new(x as f32 - 0.5, y as f32 - 0.5, z as f32 - 0.5);
        let max = Vec3::new(x as f32 + 0.5, y as f32 + 0.5, z as f32 + 0.5);
        push_block(objects, min, max, *self);
    }

    /// Places only the top slice of a cube (used to layer grass over dirt).
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

/// Inserts a block if it has not been placed before with the same tag.
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

/// Wrapper that inserts a solid block only once at the requested coordinates.
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

/// Recursive ray-tracing routine with small reflection/refraction depth.
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
/// Generates all blocks for the requested world using the provided assets.
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
        reflectivity: 0.38,
        transparency: 0.0,
        ior: 2.4,
        emissive: Vec3::new(0.0, 0.0, 0.0),
    };
    let iron_mat = BlockMaterial {
        tex: assets.iron,
        albedo: Vec3::new(0.95, 0.95, 0.98),
        specular: 0.45,
        shininess: 85.0,
        reflectivity: 0.25,
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
                        place_block(&mut objects, &mut used, dirt_mat, x, 0, z);
                        if !no_cover.contains(&(x, z)) {
                            grass_cover_mat.place_cover(&mut objects, x, 0, z, 0.18);
                        }
                    }
                }
            }

            for &(x, z) in &water_tiles {
                place_with_tag(&mut objects, &mut used, water_mat, x, 0, z, 1);
                place_with_tag(&mut objects, &mut used, stone_mat, x, -1, z, 1);
            }

            place_with_tag(&mut objects, &mut used, ice_mat, 4, 0, 4, 1);
            place_with_tag(&mut objects, &mut used, stone_mat, 4, -1, 4, 1);
            for &(y, z) in &[(0, 4), (-1, 4), (-2, 4), (-3, 4)] {
                place_with_tag(&mut objects, &mut used, water_mat, 3, y, z, 1);
            }
            place_with_tag(&mut objects, &mut used, water_mat, 3, -3, 5, 1);
            place_with_tag(&mut objects, &mut used, stone_mat, 3, -4, 5, 1);

            for &(x, z) in &lava_tiles {
                place_with_tag(&mut objects, &mut used, lava_mat, x, 0, z, 1);
                place_with_tag(&mut objects, &mut used, stone_mat, x, -1, z, 1);
            }

            for y in 1..=3 {
                place_with_tag(&mut objects, &mut used, wood_mat, 2, y, 2, 1);
            }
            for y in 3..=4 {
                for dx in -1..=1 {
                    for dz in -1..=1 {
                        if y == 3 && dx == 0 && dz == 0 {
                            continue;
                        }
                        place_with_tag(&mut objects, &mut used, leaves_mat, 2 + dx, y, 2 + dz, 1);
                    }
                }
            }

            place_with_tag(&mut objects, &mut used, stone_mat, 2, 0, 1, 1);
            place_with_tag(&mut objects, &mut used, diamond_mat, 2, 1, 1, 1);
            place_with_tag(&mut objects, &mut used, stone_mat, 3, 0, 1, 1);
            place_with_tag(&mut objects, &mut used, iron_mat, 3, 1, 1, 1);
            place_with_tag(
                &mut objects,
                &mut used,
                chest_mat,
                chest_tile.0,
                0,
                chest_tile.1,
                1,
            );

            let portal_left = 4;
            let portal_right = 6;
            let portal_z = 2;
            let portal_top = 4;
            for y in 0..=portal_top {
                place_with_tag(
                    &mut objects,
                    &mut used,
                    obsidian_mat,
                    portal_left,
                    y,
                    portal_z,
                    1,
                );
                place_with_tag(
                    &mut objects,
                    &mut used,
                    obsidian_mat,
                    portal_right,
                    y,
                    portal_z,
                    1,
                );
            }
            for x in portal_left..=portal_right {
                place_with_tag(&mut objects, &mut used, obsidian_mat, x, 0, portal_z, 1);
                place_with_tag(&mut objects, &mut used, obsidian_mat, x, -1, portal_z, 1);
                place_with_tag(
                    &mut objects,
                    &mut used,
                    obsidian_mat,
                    x,
                    portal_top,
                    portal_z,
                    1,
                );
            }
            for y in 1..portal_top {
                for x in (portal_left + 1)..portal_right {
                    place_with_tag(&mut objects, &mut used, portal_mat, x, y, portal_z, 1);
                }
            }
        }
        WorldKind::Nether => {
            let max_x = 6;
            let max_z = 5;

            for x in 0..=max_x {
                for z in 0..=max_z {
                    if x == 0 || x == max_x || z == 0 || z == max_z {
                        place_block(&mut objects, &mut used, obsidian_mat, x, -1, z);
                        place_with_tag(&mut objects, &mut used, obsidian_mat, x, 0, z, 1);
                    }
                }
            }

            for &(x, z) in &[(1, 1), (1, 4), (4, 1), (4, 4)] {
                place_with_tag(&mut objects, &mut used, obsidian_mat, x, 0, z, 1);
            }

            for x in 2..=3 {
                for z in 2..=3 {
                    place_with_tag(&mut objects, &mut used, lava_mat, x, 0, z, 1);
                }
            }

            for &(x, z) in &[(1, 1), (4, 4)] {
                for y in 1..=3 {
                    place_with_tag(&mut objects, &mut used, obsidian_mat, x, y, z, 1);
                }
                place_with_tag(&mut objects, &mut used, glowstone_mat, x, 4, z, 1);
            }
            for &(x, z) in &[(1, 4), (4, 1)] {
                place_with_tag(&mut objects, &mut used, glowstone_mat, x, 1, z, 1);
            }

            place_with_tag(&mut objects, &mut used, diamond_mat, 2, 1, 4, 1);
            place_with_tag(&mut objects, &mut used, iron_mat, 3, 1, 1, 1);

            let portal_left = 4;
            let portal_right = 6;
            let portal_z = 2;
            let portal_top = 4;
            for y in 0..=portal_top {
                place_with_tag(
                    &mut objects,
                    &mut used,
                    obsidian_mat,
                    portal_left,
                    y,
                    portal_z,
                    1,
                );
                place_with_tag(
                    &mut objects,
                    &mut used,
                    obsidian_mat,
                    portal_right,
                    y,
                    portal_z,
                    1,
                );
            }
            for x in portal_left..=portal_right {
                place_with_tag(&mut objects, &mut used, obsidian_mat, x, 0, portal_z, 1);
                place_with_tag(&mut objects, &mut used, obsidian_mat, x, -1, portal_z, 1);
                place_with_tag(
                    &mut objects,
                    &mut used,
                    obsidian_mat,
                    x,
                    portal_top,
                    portal_z,
                    1,
                );
            }
            for y in 1..portal_top {
                for x in (portal_left + 1)..portal_right {
                    place_with_tag(&mut objects, &mut used, portal_mat, x, y, portal_z, 1);
                }
            }

            place_with_tag(&mut objects, &mut used, lava_mat, 2, -1, 2, 1);
            place_with_tag(&mut objects, &mut used, lava_mat, 3, -1, 3, 1);
        }
    }

    let skybox = match world {
        WorldKind::Overworld => assets.skybox_overworld,
        WorldKind::Nether => assets.skybox_nether,
    };

    SceneData { objects, skybox }
}

/// Raytraces the provided scene and writes the shaded RGBA image into `frame`.
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
