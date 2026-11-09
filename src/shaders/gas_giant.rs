use crate::engine::math::{Vec3, Vec4};
use crate::engine::noise::Fbm;
use crate::shaders::{FragInput, FragmentShader, Uniforms};

/// Gigante gaseoso. Cuatro capas: bandas de latitud animadas (nubes), un
/// vórtice de tormenta, iluminación Lambertiana y bruma atmosférica gruesa.
pub struct GasGiantShader {
    warp: Fbm,
    storm: Fbm,
    storm_center: Vec3,
}

impl GasGiantShader {
    pub fn new(seed: u32) -> Self {
        let warp = Fbm::new(seed)
            .set_octaves(4)
            .set_frequency(1.2)
            .set_persistence(0.5);
        let storm = Fbm::new(seed.wrapping_add(307))
            .set_octaves(4)
            .set_frequency(4.0)
            .set_persistence(0.55);
        Self { warp, storm, storm_center: Vec3::new(0.3, -0.15, 0.94).normalize() }
    }
}

impl FragmentShader for GasGiantShader {
    fn shade(&self, input: &FragInput, uniforms: &Uniforms) -> Vec4 {
        let p = input.local.normalize();
        let t = uniforms.time;

        // Capa 1: bandas de latitud deformadas por ruido y animadas con el tiempo (nubes fluyendo).
        let warp_n = self.warp.sample3(Vec3::new(p.x * 1.5, p.y * 0.5, p.z * 1.5 + t * 0.06));
        let band = (p.y * 11.0 + warp_n * 2.2).sin() * 0.5 + 0.5;
        let palette_a = Vec3::new(0.87, 0.68, 0.45);
        let palette_b = Vec3::new(0.62, 0.42, 0.3);
        let base = palette_a.lerp(palette_b, band);

        // Capa 2: vórtice de tormenta localizado (tipo "gran mancha").
        let d = p.distance(self.storm_center);
        let storm_n = self.storm.sample3(Vec3::new(p.x * 5.0, p.y * 5.0, p.z * 5.0 + t * 0.15));
        let storm_mask = (1.0 - (d / 0.4)).clamp(0.0, 1.0) * (0.6 + 0.4 * storm_n);
        let with_storm = base.lerp(Vec3::new(0.78, 0.3, 0.22), storm_mask);

        // Capa 3: iluminación difusa Lambertiana más ambiente.
        let ndotl = input.normal.dot(uniforms.light_dir).max(0.0);
        let lit = with_storm * (0.25 + 0.75 * ndotl);

        // Capa 4: bruma atmosférica gruesa en el borde (fresnel).
        let fres = (1.0 - input.normal.dot(input.view_dir).clamp(0.0, 1.0)).powf(2.2);
        let final_color = lit + Vec3::new(0.9, 0.72, 0.5) * fres * 0.4;

        Vec4::new(final_color.x, final_color.y, final_color.z, 1.0)
    }
}
