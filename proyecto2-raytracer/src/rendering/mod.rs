//! Núcleo del trazador y utilidades de iluminación.

pub mod lighting;
pub mod raytracer;

pub use lighting::{Skybox, Tex};
pub use raytracer::{Assets, WorldKind, build_scene, render};
