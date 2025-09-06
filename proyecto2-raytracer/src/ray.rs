use crate::vec3::Vec3;

#[derive(Copy, Clone)]
pub struct Ray { 
    pub orig: Vec3, 
    pub dir: Vec3 
}