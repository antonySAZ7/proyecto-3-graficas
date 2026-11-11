use std::sync::atomic::{AtomicU32, Ordering};

use crate::engine::math::{Vec3, Vec4};
use crate::engine::noise::Fbm;
use crate::shaders::{spherical_uv, FragInput, FragmentShader, Uniforms};

/// Galactus, la amenaza colosal. Cinco capas: coraza oscura, red de grietas
/// de energía cósmica, pulso magenta animado dentro de ellas, iluminación
/// tenue (se ilumina sobre todo con su propia energía) y reborde violeta.
/// Paleta a propósito opuesta a la de la nave/HUD (cian/blanco de los
/// héroes contra púrpura/magenta de la amenaza).
pub struct GalactusShader {
    hide: Fbm,
    crack: Fbm,
    damage_flash_bits: AtomicU32,
}

impl GalactusShader {
    pub fn new(seed: u32) -> Self {
        let hide = Fbm::new(seed).set_octaves(4).set_frequency(2.2).set_persistence(0.5);
        let crack = Fbm::new(seed.wrapping_add(211)).set_octaves(3).set_frequency(6.0).set_persistence(0.55);
        Self { hide, crack, damage_flash_bits: AtomicU32::new(0.0f32.to_bits()) }
    }

    pub fn set_damage_flash(&self, amount: f32) {
        self.damage_flash_bits.store(amount.clamp(0.0, 1.0).to_bits(), Ordering::Relaxed);
    }
}

impl FragmentShader for GalactusShader {
    fn shade(&self, input: &FragInput, uniforms: &Uniforms) -> Vec4 {
        let p = input.local.normalize();
        let t = uniforms.time;
        let (u, v) = spherical_uv(p);

        // Capa 1: coraza oscura con variación sutil de tono.
        let r = self.hide.sample3(p * 2.5) * 0.5 + 0.5;
        let base = Vec3::new(0.05, 0.02, 0.08).lerp(Vec3::new(0.14, 0.04, 0.18), r);

        // Capa 2: red de grietas de energía cósmica (crestas deformadas por ruido).
        let ridge = (u * 60.0 + (v * 22.0).sin() * 3.0).sin().abs();
        let n = self.crack.sample3(p * 7.0) * 0.5 + 0.5;
        let crack_mask = ((ridge + n * 0.3 - 1.08).max(0.0) * 7.0).clamp(0.0, 1.0);

        // Capa 3: pulso de energía magenta animado dentro de las grietas.
        let pulse = 0.5 + 0.5 * (t * 2.2 + u * 30.0).sin();
        let energy = Vec3::new(0.55, 0.05, 0.75).lerp(Vec3::new(0.95, 0.15, 0.85), pulse);
        let cracked = base.lerp(energy, crack_mask);

        // Capa 4: iluminación Lambertiana tenue más la luz propia de las grietas.
        let ndotl = input.normal.dot(uniforms.light_dir).max(0.0);
        let lit = cracked * (0.12 + 0.5 * ndotl) + energy * crack_mask * 0.6;

        // Capa 5: reborde violeta (fresnel), silueta amenazante contra el espacio.
        let fres = (1.0 - input.normal.dot(input.view_dir).clamp(0.0, 1.0)).powf(2.2);
        let mut final_color = lit + Vec3::new(0.65, 0.1, 0.85) * fres * 0.7;

        // Fase 9: el pulso alterna rojo/blanco para que cada impacto sea
        // legible incluso si el destello central queda detrás de la silueta.
        let damage_flash = f32::from_bits(self.damage_flash_bits.load(Ordering::Relaxed));
        if damage_flash > 0.0 {
            let blink = 0.5 + 0.5 * (t * 55.0).sin();
            let damage_color = Vec3::new(1.0, 0.12, 0.06).lerp(Vec3::new(1.0, 1.0, 1.0), blink);
            final_color = final_color.lerp(damage_color, damage_flash * 0.82);
        }

        Vec4::new(final_color.x, final_color.y, final_color.z, 1.0)
    }
}
