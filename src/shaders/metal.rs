use crate::engine::math::{Vec3, Vec4};
use crate::engine::noise::Fbm;
use crate::shaders::{spherical_uv, FragInput, FragmentShader, Uniforms};

/// Mundo de Plata. Cuatro capas: placas metálicas, red de circuitos
/// luminosos animados, iluminación Lambertiana con especular metálico
/// marcado y reborde frío.
pub struct MetalShader {
    noise: Fbm,
}

impl MetalShader {
    pub fn new(seed: u32) -> Self {
        let noise = Fbm::new(seed).set_octaves(3).set_frequency(2.5).set_persistence(0.5);
        Self { noise }
    }
}

impl FragmentShader for MetalShader {
    fn shade(&self, input: &FragInput, uniforms: &Uniforms) -> Vec4 {
        let p = input.local.normalize();
        let t = uniforms.time;
        let (u, v) = spherical_uv(p);

        // Capa 1: placas metálicas, tono variable por ruido.
        let n = self.noise.sample3(p * 4.0) * 0.5 + 0.5;
        let base = Vec3::new(0.32, 0.36, 0.42).lerp(Vec3::new(0.5, 0.54, 0.6), n);

        // Capa 2: red de circuitos (rejilla por senos en longitud y latitud) con pulso animado.
        let grid_u = ((u * 90.0).sin().abs() - 0.93).max(0.0) * 14.0;
        let grid_v = ((v * 70.0).sin().abs() - 0.93).max(0.0) * 14.0;
        let circuit = grid_u.max(grid_v).clamp(0.0, 1.0) * (0.6 + 0.4 * (t * 1.6 + v * 20.0).sin());
        let wired = base.lerp(Vec3::new(0.25, 0.9, 1.0), circuit);

        // Capa 3: iluminación Lambertiana con especular metálico marcado.
        let ndotl = input.normal.dot(uniforms.light_dir).max(0.0);
        let half = (uniforms.light_dir + input.view_dir).normalize();
        let spec = input.normal.dot(half).max(0.0).powf(45.0) * 0.7;
        let lit = wired * (0.2 + 0.8 * ndotl) + Vec3::splat(spec);

        // Capa 4: reborde frío (fresnel).
        let fres = (1.0 - input.normal.dot(input.view_dir).clamp(0.0, 1.0)).powf(2.5);
        let final_color = lit + Vec3::new(0.3, 0.5, 0.6) * fres * 0.4;

        Vec4::new(final_color.x, final_color.y, final_color.z, 1.0)
    }
}
