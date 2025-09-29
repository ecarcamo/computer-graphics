//! Basic ray primitive used throughout the renderer.

use crate::math::Vec3;

#[derive(Copy, Clone)]
/// Ray with origin and direction, both expressed in world space.
pub struct Ray {
    pub orig: Vec3,
    pub dir: Vec3,
}
