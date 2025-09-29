//! Primitiva de rayo utilizada en todo el motor de render.

use crate::math::Vec3;

#[derive(Copy, Clone)]
/// Rayo con origen y dirección expresados en espacio mundial.
pub struct Ray {
    pub orig: Vec3,
    pub dir: Vec3,
}
