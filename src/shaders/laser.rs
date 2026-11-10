use crate::engine::math::{Vec3, Vec4};
use crate::shaders::{FragInput, FragmentShader, Uniforms};

/// Rayo de la nave: geometría emisiva simple (un cilindro delgado), núcleo
/// blanco intenso que se funde a cian hacia el borde del haz.
pub struct LaserShader {
    /// Radio físico de la malla del rayo, para normalizar el gradiente radial.
    radius: f32,
}

impl LaserShader {
    pub fn new(radius: f32) -> Self {
        Self { radius }
    }
}

impl FragmentShader for LaserShader {
    fn shade(&self, input: &FragInput, _uniforms: &Uniforms) -> Vec4 {
        // El cilindro tiene su eje en Y local; el radio real está en el plano XZ local.
        let radial = Vec3::new(input.local.x, 0.0, input.local.z).length();
        let core = (1.0 - (radial / self.radius).clamp(0.0, 1.0)).powf(1.5);
        let color = Vec3::new(0.4, 0.85, 1.0).lerp(Vec3::new(1.0, 1.0, 1.0), core);
        Vec4::new(color.x, color.y, color.z, 1.0)
    }
}
