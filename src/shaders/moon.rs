use glam::{Vec3, Vec4};
use noise::{Fbm, MultiFractal, NoiseFn, Perlin};

use crate::shaders::{FragInput, FragmentShader, Uniforms};

/// Luna, geometría aparte del planeta que acompaña. Cuatro capas: variación
/// de albedo (mares vs tierras altas), cráteres, luz Lambertiana y
/// oscurecimiento sutil del borde.
pub struct MoonShader {
    albedo: Fbm<Perlin>,
    craters: Fbm<Perlin>,
}

impl MoonShader {
    pub fn new(seed: u32) -> Self {
        let albedo = Fbm::<Perlin>::new(seed)
            .set_octaves(4)
            .set_frequency(1.5)
            .set_persistence(0.5);
        let craters = Fbm::<Perlin>::new(seed.wrapping_add(409))
            .set_octaves(2)
            .set_frequency(12.0)
            .set_persistence(0.5);
        Self { albedo, craters }
    }
}

impl FragmentShader for MoonShader {
    fn shade(&self, input: &FragInput, uniforms: &Uniforms) -> Vec4 {
        let p = input.local.normalize();

        // Capa 1: variación de albedo, mares oscuros sobre tierras altas claras.
        let a = self.albedo.get([(p.x * 2.5) as f64, (p.y * 2.5) as f64, (p.z * 2.5) as f64]) as f32;
        let maria = ((a - 0.1).max(0.0) * 3.0).clamp(0.0, 1.0);
        let base = Vec3::new(0.75, 0.74, 0.72).lerp(Vec3::new(0.35, 0.34, 0.35), maria);

        // Capa 2: cráteres pequeños y densos, con reborde más claro.
        let c = self.craters.get([(p.x * 10.0) as f64, (p.y * 10.0) as f64, (p.z * 10.0) as f64]) as f32;
        let crater_mask = ((c.abs() - 0.75).max(0.0) * 8.0).clamp(0.0, 1.0);
        let rim_mask = (((c.abs() - 0.68).max(0.0) * 10.0).clamp(0.0, 1.0) - crater_mask).max(0.0);
        let cratered = base * (1.0 - crater_mask * 0.6) + Vec3::splat(rim_mask * 0.15);

        // Capa 3: luz difusa Lambertiana con poco ambiente (sin atmósfera, terminador duro).
        let ndotl = input.normal.dot(uniforms.light_dir).max(0.0);
        let lit = cratered * (0.08 + 0.92 * ndotl);

        // Capa 4: oscurecimiento sutil hacia el borde, da sensación de profundidad.
        let fres = 1.0 - input.normal.dot(input.view_dir).clamp(0.0, 1.0);
        let final_color = lit * (1.0 - fres * 0.25);

        Vec4::new(final_color.x, final_color.y, final_color.z, 1.0)
    }
}
