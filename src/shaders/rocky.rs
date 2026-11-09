use crate::engine::math::{Vec3, Vec4};
use crate::engine::noise::Fbm;
use crate::shaders::{FragInput, FragmentShader, Uniforms};

/// Planeta rocoso. Cinco capas: color por elevación, cráteres, casquetes
/// polares de hielo, iluminación Lambertiana y brillo atmosférico delgado.
pub struct RockyShader {
    elevation: Fbm,
    craters: Fbm,
}

impl RockyShader {
    pub fn new(seed: u32) -> Self {
        let elevation = Fbm::new(seed)
            .set_octaves(5)
            .set_frequency(1.8)
            .set_persistence(0.5);
        let craters = Fbm::new(seed.wrapping_add(211))
            .set_octaves(2)
            .set_frequency(9.0)
            .set_persistence(0.5);
        Self { elevation, craters }
    }
}

impl FragmentShader for RockyShader {
    fn shade(&self, input: &FragInput, _uniforms: &Uniforms) -> Vec4 {
        let p = input.local.normalize();

        // Capa 1: color de terreno según elevación (tierras bajas, medias y altas).
        let e = self.elevation.sample3(Vec3::new(p.x * 2.0, p.y * 2.0, p.z * 2.0));
        let h = (e * 0.5 + 0.5).clamp(0.0, 1.0);
        let base = if h < 0.45 {
            Vec3::new(0.22, 0.16, 0.12).lerp(Vec3::new(0.45, 0.3, 0.18), h / 0.45)
        } else {
            Vec3::new(0.45, 0.3, 0.18).lerp(Vec3::new(0.68, 0.58, 0.42), (h - 0.45) / 0.55)
        };

        // Capa 2: cráteres de impacto (ruido de alta frecuencia con umbral).
        let c = self.craters.sample3(Vec3::new(p.x * 8.0, p.y * 8.0, p.z * 8.0));
        let crater_mask = ((c.abs() - 0.8).max(0.0) * 7.0).clamp(0.0, 1.0);
        let cratered = base * (1.0 - crater_mask * 0.55);

        // Capa 3: casquetes de hielo cerca de los polos.
        let polar = ((p.y.abs() - 0.78).max(0.0) * 4.5).clamp(0.0, 1.0);
        let iced = cratered.lerp(Vec3::new(0.85, 0.9, 0.95), polar);

        // Capa 4: iluminación difusa Lambertiana más ambiente.
        let ndotl = input.normal.dot(_uniforms.light_dir).max(0.0);
        let lit = iced * (0.16 + 0.84 * ndotl);

        // Capa 5: brillo atmosférico delgado en el borde (fresnel).
        let fres = (1.0 - input.normal.dot(input.view_dir).clamp(0.0, 1.0)).powf(3.0);
        let final_color = lit + Vec3::new(0.3, 0.4, 0.55) * fres * 0.5;

        Vec4::new(final_color.x, final_color.y, final_color.z, 1.0)
    }
}
