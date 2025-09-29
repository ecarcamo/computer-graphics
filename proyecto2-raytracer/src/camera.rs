//! Simple pinhole camera used to generate primary rays.
use crate::math::Vec3;
use crate::ray::Ray;

pub struct Camera {
    pub eye: Vec3,
    pub target: Vec3,
    pub up: Vec3,
    pub fov_y: f32,
}

impl Camera {
    /// Generates a ray going through the pixel defined by `(u, v)` in NDC.
    pub fn make_ray(&self, u: f32, v: f32, aspect: f32) -> Ray {
        let fov = self.fov_y.to_radians();
        let scale = (fov * 0.5).tan();
        let forward = self.target.sub(self.eye).norm();
        let right = forward.cross(self.up).norm();
        let up = right.cross(forward).norm();
        let x = (2.0 * u - 1.0) * aspect * scale;
        let y = (1.0 - 2.0 * v) * scale;
        let dir = right.mul(x).add(up.mul(y)).add(forward).norm();
        Ray {
            orig: self.eye,
            dir,
        }
    }
}
