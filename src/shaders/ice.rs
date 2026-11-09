use crate::engine::math::{Vec3, Vec4};
use crate::engine::noise::Fbm;
use crate::shaders::{spherical_uv, FragInput, FragmentShader, Uniforms};

/// Niflheim. Cuatro capas: bandas de hielo iridiscente deformadas por ruido,
/// grietas oscuras finas, iluminación Lambertiana con especular fuerte
/// (superficie pulida) y reborde brillante de aspecto translúcido.
pub struct IceShader {
    warp: Fbm,
    cracks: Fbm,
}

impl IceShader {
    pub fn new(seed: u32) -> Self {
        let warp = Fbm::new(seed).set_octaves(4).set_frequency(1.4).set_persistence(0.5);
        let cracks = Fbm::new(seed.wrapping_add(263)).set_octaves(2).set_frequency(7.0).set_persistence(0.5);
        Self { warp, cracks }
    }
}

impl FragmentShader for IceShader {
    fn shade(&self, input: &FragInput, uniforms: &Uniforms) -> Vec4 {
        let p = input.local.normalize();
        let (_u, v) = spherical_uv(p);

        // Capa 1: bandas de hielo iridiscente, deformadas por ruido.
        let w = self.warp.sample3(p * 2.0);
        let band = (v * 48.0 + w * 3.0).sin() * 0.5 + 0.5;
        let base = Vec3::new(0.1, 0.24, 0.5).lerp(Vec3::new(0.68, 0.92, 1.0), band);

        // Capa 2: grietas oscuras finas.
        let c = self.cracks.sample3(p * 8.0);
        let crack_mask = ((c.abs() - 0.82).max(0.0) * 8.0).clamp(0.0, 1.0);
        let cracked = base * (1.0 - crack_mask * 0.5);

        // Capa 3: iluminación Lambertiana con especular fuerte (hielo pulido).
        let ndotl = input.normal.dot(uniforms.light_dir).max(0.0);
        let half = (uniforms.light_dir + input.view_dir).normalize();
        let spec = input.normal.dot(half).max(0.0).powf(55.0) * 0.9;
        let lit = cracked * (0.2 + 0.8 * ndotl) + Vec3::splat(spec);

        // Capa 4: reborde brillante (fresnel), aspecto translúcido.
        let fres = (1.0 - input.normal.dot(input.view_dir).clamp(0.0, 1.0)).powf(2.0);
        let final_color = lit + Vec3::new(0.7, 0.9, 1.0) * fres * 0.6;

        Vec4::new(final_color.x, final_color.y, final_color.z, 1.0)
    }
}
