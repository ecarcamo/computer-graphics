//! Rendering pipeline and lighting helpers.

pub mod lighting;
pub mod raytracer;

pub use lighting::{Skybox, Tex};
pub use raytracer::{Assets, WorldKind, build_scene, render};