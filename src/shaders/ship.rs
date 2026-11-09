use crate::engine::math::{Vec3, Vec4};
use crate::shaders::{FragInput, FragmentShader, Uniforms};

/// Nave Fantasticar: aleación azul con paneles blancos alternados,
/// especular marcado (casco pulido) y reborde cian, para distinguirla de
/// lejos sobre el fondo oscuro del espacio.
pub struct ShipShader;

impl ShipShader {
    pub fn new() -> Self {
        Self
    }
}

impl FragmentShader for ShipShader {
    fn shade(&self, input: &FragInput, uniforms: &Uniforms) -> Vec4 {
        // Capa 1: paneles alternados según una franja en el espacio local del modelo.
        let stripe = (input.local.y * 2.1 + input.local.x * 0.7).rem_euclid(1.0);
        let panel = if stripe > 0.72 { Vec3::new(0.92, 0.95, 1.0) } else { Vec3::new(0.09, 0.42, 0.82) };

        // Capa 2: iluminación Lambertiana con especular marcado.
        let ndotl = input.normal.dot(uniforms.light_dir).max(0.0);
        let half = (uniforms.light_dir + input.view_dir).normalize();
        let spec = input.normal.dot(half).max(0.0).powf(70.0) * 0.8;
        let lit = panel * (0.3 + 0.7 * ndotl) + Vec3::splat(spec);

        // Capa 3: reborde cian (fresnel), silueta reconocible contra el espacio.
        let fres = (1.0 - input.normal.dot(input.view_dir).clamp(0.0, 1.0)).powf(3.0);
        let final_color = lit + Vec3::new(0.3, 0.85, 1.0) * fres * 0.55;

        Vec4::new(final_color.x, final_color.y, final_color.z, 1.0)
    }
}
