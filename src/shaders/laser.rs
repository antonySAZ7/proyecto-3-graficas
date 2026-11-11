use crate::engine::math::{Vec3, Vec4};
use crate::shaders::{FragInput, FragmentShader, Uniforms};

/// Proyectil de la nave. Fase 9 lo vuelve fuego rojo/naranja con núcleo
/// amarillo y parpadeo; la bandera permite recuperar el rayo cian anterior.
pub struct LaserShader {
    /// Radio físico de la malla del rayo, para normalizar el gradiente radial.
    radius: f32,
    combat_legible: bool,
}

impl LaserShader {
    pub fn new(radius: f32, combat_legible: bool) -> Self {
        Self { radius, combat_legible }
    }
}

impl FragmentShader for LaserShader {
    fn shade(&self, input: &FragInput, uniforms: &Uniforms) -> Vec4 {
        // El cilindro tiene su eje en Y local; el radio real está en el plano XZ local.
        let radial = Vec3::new(input.local.x, 0.0, input.local.z).length();
        let core = (1.0 - (radial / self.radius).clamp(0.0, 1.0)).powf(1.5);
        let color = if self.combat_legible {
            let flicker = 0.78 + 0.22 * (uniforms.time * 24.0 + input.local.y * 11.0).sin();
            let shell = Vec3::new(0.85, 0.025, 0.005).lerp(Vec3::new(1.0, 0.28, 0.015), flicker);
            shell.lerp(Vec3::new(1.0, 0.9, 0.32), core.powf(0.7))
        } else {
            Vec3::new(0.4, 0.85, 1.0).lerp(Vec3::new(1.0, 1.0, 1.0), core)
        };
        Vec4::new(color.x, color.y, color.z, 1.0)
    }
}
