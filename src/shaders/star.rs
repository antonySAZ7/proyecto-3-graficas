use crate::engine::math::{Vec3, Vec4};
use crate::engine::noise::Fbm;
use crate::shaders::{FragInput, FragmentShader, Uniforms};

/// Estrella tipo sol. Cinco capas: gradiente base núcleo-borde, turbulencia
/// de plasma animada, manchas solares, limb darkening y brillo pulsante.
pub struct StarShader {
    turbulence: Fbm,
    spots: Fbm,
}

impl StarShader {
    pub fn new(seed: u32) -> Self {
        let turbulence = Fbm::new(seed)
            .set_octaves(5)
            .set_frequency(1.6)
            .set_lacunarity(2.0)
            .set_persistence(0.55);
        let spots = Fbm::new(seed.wrapping_add(101))
            .set_octaves(3)
            .set_frequency(3.2)
            .set_persistence(0.5);
        Self { turbulence, spots }
    }
}

impl FragmentShader for StarShader {
    fn shade(&self, input: &FragInput, uniforms: &Uniforms) -> Vec4 {
        let p = input.local.normalize();
        let t = uniforms.time;

        // Capa 1: gradiente base, núcleo amarillo cálido hacia naranja en los polos.
        let lat = p.y.abs();
        let base = Vec3::new(1.0, 0.86, 0.4).lerp(Vec3::new(1.0, 0.45, 0.08), lat * 0.4);

        // Capa 2: turbulencia de plasma animada (ruido fbm que se mueve con el tiempo).
        let n1 = self.turbulence.sample3(Vec3::new(p.x * 2.0, p.y * 2.0, p.z * 2.0 + t * 0.25));
        let turb = (n1 * 0.5 + 0.5).clamp(0.0, 1.0);
        let plasma = base.lerp(Vec3::new(1.0, 0.97, 0.7), turb * 0.45);

        // Capa 3: manchas solares (parches oscuros por umbral de ruido).
        let n2 = self.spots.sample3(Vec3::new(p.x * 4.0, p.y * 4.0, p.z * 4.0));
        let spot_mask = ((n2 - 0.5).max(0.0) * 6.0).clamp(0.0, 1.0);
        let spotted = plasma.lerp(Vec3::new(0.4, 0.1, 0.03), spot_mask);

        // Capa 4: limb darkening, más brillo al centro del disco y más oscuro hacia el borde.
        let ndotv = input.normal.dot(input.view_dir).clamp(0.0, 1.0);
        let limb = 0.55 + 0.45 * ndotv;
        let darkened = spotted * limb;

        // Capa 5: pulso de brillo lento, como una corona respirando.
        let pulse = 1.0 + 0.08 * (t * 1.7 + p.x * 3.0).sin();
        let final_color = darkened * pulse;

        Vec4::new(final_color.x, final_color.y, final_color.z, 1.0)
    }
}
