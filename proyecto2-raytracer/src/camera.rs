use crate::ray::Ray;
use crate::vec3::Vec3;

pub struct Camera {
    pub eye: Vec3,
    pub target: Vec3,
    pub up: Vec3,
    pub fov_y: f32,
}

impl Camera {
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
