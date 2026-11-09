pub mod star;
pub mod rocky;
pub mod gas_giant;
pub mod rings;
pub mod moon;

use glam::{Vec3, Vec4};

/// Datos que recibe un shader sobre el punto que va a colorear.
pub struct FragInput {
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

pub trait FragmentShader {
    /// Color RGB lineal más alfa (alfa < 1 mezcla con lo ya dibujado, usado por los anillos).
    fn shade(&self, input: &FragInput, uniforms: &Uniforms) -> Vec4;
}
