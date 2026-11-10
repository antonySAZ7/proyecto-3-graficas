pub mod star;
pub mod rocky;
pub mod earth;
pub mod galactus;
pub mod lava;
pub mod laser;
pub mod ice;
pub mod metal;
pub mod moon;
pub mod ship;
pub mod surface;

use crate::engine::math::{Vec3, Vec4};

/// Longitud/latitud en [0, 1) a partir de un punto en la esfera unitaria.
/// Los shaders de planetas la usan para patrones de bandas, grietas o
/// circuitos sin depender de UVs de malla (nuestras esferas no las tienen).
pub fn spherical_uv(p: Vec3) -> (f32, f32) {
    use std::f32::consts::{PI, TAU};
    let u = p.z.atan2(p.x) / TAU + 0.5;
    let v = p.y.clamp(-1.0, 1.0).asin() / PI + 0.5;
    (u, v)
}

/// Datos que recibe un shader sobre el punto que va a colorear.
pub struct FragInput {
    /// Posición en espacio de mundo; ningún shader activo la lee hoy (las
    /// sombras entre cuerpos eran de `rings.rs`, retirado), pero es la
    /// entrada natural para ese tipo de efecto entre cuerpos.
    #[allow(dead_code)]
    pub world_pos: Vec3,
    /// Posición en espacio de objeto, para que el ruido no dependa de la escala/posición del cuerpo.
    pub local: Vec3,
    pub normal: Vec3,
    pub view_dir: Vec3,
}

/// Valores compartidos por todos los shaders en un frame.
pub struct Uniforms {
    pub time: f32,
    /// Dirección normalizada desde la superficie hacia la luz.
    pub light_dir: Vec3,
}

/// `Sync`: varias bandas del rasterizador llaman a `shade` en paralelo desde
/// distintos hilos (ver `engine::rasterizer::RenderQueue`).
pub trait FragmentShader: Sync {
    /// Color RGB lineal más alfa (alfa < 1 mezcla con lo ya dibujado, usado por los anillos).
    fn shade(&self, input: &FragInput, uniforms: &Uniforms) -> Vec4;
}
