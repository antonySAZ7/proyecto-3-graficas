use crate::engine::math::{Vec3, Vec4};
use crate::engine::noise::Fbm;
use crate::shaders::{spherical_uv, FragInput, FragmentShader, Uniforms};

/// Mundo Forja. Cinco capas: roca volcánica oscura, red de grietas,
/// resplandor de lava animado, iluminación Lambertiana y reborde cálido.
pub struct LavaShader {
    rock: Fbm,
    fissure: Fbm,
}

impl LavaShader {
    pub fn new(seed: u32) -> Self {
        let rock = Fbm::new(seed).set_octaves(4).set_frequency(2.0).set_persistence(0.5);
        let fissure = Fbm::new(seed.wrapping_add(151)).set_octaves(3).set_frequency(5.5).set_persistence(0.55);
        Self { rock, fissure }
    }
}

impl FragmentShader for LavaShader {
    fn shade(&self, input: &FragInput, uniforms: &Uniforms) -> Vec4 {
        let p = input.local.normalize();
        let t = uniforms.time;
        let (u, v) = spherical_uv(p);

        // Capa 1: roca volcánica oscura, tono variable por ruido.
        let r = self.rock.sample3(p * 3.0) * 0.5 + 0.5;
        let base = Vec3::new(0.08, 0.05, 0.07).lerp(Vec3::new(0.32, 0.12, 0.16), r);

        // Capa 2: red de grietas (crestas sinusoidales deformadas por ruido).
        let ridge = (u * 50.0 + (v * 18.0).sin() * 2.3).sin().abs();
        let n = self.fissure.sample3(p * 6.0) * 0.5 + 0.5;
        let fissure_mask = ((ridge + n * 0.35 - 1.05).max(0.0) * 6.0).clamp(0.0, 1.0);

        // Capa 3: brillo de lava animado dentro de las grietas.
        let glow = Vec3::new(1.0, 0.25, 0.02).lerp(Vec3::new(1.0, 0.85, 0.25), 0.5 + 0.5 * (t * 2.6 + u * 24.0).sin());
        let molten = base.lerp(glow, fissure_mask);

        // Capa 4: iluminación Lambertiana (la lava aporta algo de luz propia).
        let ndotl = input.normal.dot(uniforms.light_dir).max(0.0);
        let lit = molten * (0.2 + 0.8 * ndotl) + glow * fissure_mask * 0.5;

        // Capa 5: reborde cálido (fresnel) por el calor que irradia el planeta.
        let fres = (1.0 - input.normal.dot(input.view_dir).clamp(0.0, 1.0)).powf(2.5);
        let final_color = lit + Vec3::new(0.6, 0.2, 0.05) * fres * 0.4;

        Vec4::new(final_color.x, final_color.y, final_color.z, 1.0)
    }
}
