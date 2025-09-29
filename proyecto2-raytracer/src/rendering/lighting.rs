//! Utilidades de iluminación: cielo procedural, reflejos y muestreo de skybox.

use crate::math::Vec3;

pub fn to_rgba(c: Vec3) -> [u8; 4] {
    let g = c.clamp01();
    [
        (g.x * 255.0) as u8,
        (g.y * 255.0) as u8,
        (g.z * 255.0) as u8,
        255,
    ]
}

// Cielo procedural: degrada azules según el ángulo de la mirada
pub fn sky(dir: Vec3) -> Vec3 {
    let t = dir.y.clamp(-1.0, 1.0);
    let daytime = 0.5 * (t + 1.0);
    let horizon = Vec3::new(0.8, 0.85, 0.9);
    let zenith = Vec3::new(0.1, 0.3, 0.7);
    zenith.mul(daytime).add(horizon.mul(1.0 - daytime))
}

// Reflexión especular
pub fn reflect(i: Vec3, n: Vec3) -> Vec3 {
    i.sub(n.mul(2.0 * i.dot(n))).norm()
}

// Refracción por Snell. Devuelve None si hay RTI.
pub fn refract(i: Vec3, n: Vec3, eta: f32) -> Option<Vec3> {
    let cosi = (-i).dot(n).clamp(-1.0, 1.0);
    let sin2_t = eta * eta * (1.0 - cosi * cosi);
    if sin2_t > 1.0 {
        return None;
    }
    let cost = (1.0 - sin2_t).sqrt();
    Some(i.mul(eta).add(n.mul(eta * cosi - cost)).norm())
}

// Especular Phong
pub fn specular_phong(r: Vec3, v: Vec3, k_s: f32, shininess: f32) -> f32 {
    let rv = r.dot(v).max(0.0);
    k_s * rv.powf(shininess.max(1.0))
}

#[derive(Copy, Clone)]
pub struct Tex<'a> {
    pub pix: &'a [u8],
    pub w: u32,
    pub h: u32,
}

#[derive(Copy, Clone)]
pub struct Skybox<'a> {
    pub px: Tex<'a>,
    pub nx: Tex<'a>,
    pub py: Tex<'a>,
    pub ny: Tex<'a>,
    pub pz: Tex<'a>,
    pub nz: Tex<'a>,
    pub tint: Vec3,
}

// Muestrea el cubemap y aplica un tinte opcional.
pub fn sample_skybox(dir: Vec3, sb: &Skybox) -> Vec3 {
    let d = dir.norm();
    let ax = d.x.abs();
    let ay = d.y.abs();
    let az = d.z.abs();
    let (face, u, v) = if ax >= ay && ax >= az {
        if d.x > 0.0 {
            ("px", -d.z / ax, d.y / ax)
        } else {
            ("nx", d.z / ax, d.y / ax)
        }
    } else if ay >= ax && ay >= az {
        if d.y > 0.0 {
            ("py", d.x / ay, -d.z / ay)
        } else {
            ("ny", d.x / ay, d.z / ay)
        }
    } else if d.z > 0.0 {
        ("pz", d.x / az, d.y / az)
    } else {
        ("nz", -d.x / az, d.y / az)
    };
    let uu = (u + 1.0) * 0.5;
    let vv = (v + 1.0) * 0.5;

    let sample = |t: &Tex, u: f32, v: f32| -> Vec3 {
        let uu = u.clamp(0.0, 1.0);
        let vv = v.clamp(0.0, 1.0);
        let px = (uu * (t.w as f32 - 1.0)).round() as u32;
        let py = ((1.0 - vv) * (t.h as f32 - 1.0)).round() as u32;
        let idx = ((py * t.w + px) * 4) as usize;
        if idx + 3 >= t.pix.len() {
            return Vec3::new(0.5, 0.7, 1.0);
        }
        let r = t.pix[idx] as f32 / 255.0;
        let g = t.pix[idx + 1] as f32 / 255.0;
        let b = t.pix[idx + 2] as f32 / 255.0;
        Vec3::new(r, g, b)
    };

    let base = match face {
        "px" => sample(&sb.px, uu, vv),
        "nx" => sample(&sb.nx, uu, vv),
        "py" => sample(&sb.py, uu, vv),
        "ny" => sample(&sb.ny, uu, vv),
        "pz" => sample(&sb.pz, uu, vv),
        _ => sample(&sb.nz, uu, vv),
    };

    base.hadamard(sb.tint)
}
