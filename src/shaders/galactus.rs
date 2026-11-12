use std::sync::{
    atomic::{AtomicU32, Ordering},
    Arc,
};

use crate::engine::math::{Vec3, Vec4};
use crate::engine::noise::Fbm;
use crate::shaders::{spherical_uv, FragInput, FragmentShader, Uniforms};

/// Galactus, la amenaza colosal. Cinco capas: coraza oscura, red de grietas
/// de energía cósmica, pulso magenta animado dentro de ellas, iluminación
/// tenue (se ilumina sobre todo con su propia energía) y reborde violeta.
/// Paleta a propósito opuesta a la de la nave/HUD (cian/blanco de los
/// héroes contra púrpura/magenta de la amenaza).
#[derive(Clone)]
pub struct DamageFlash {
    bits: Arc<AtomicU32>,
}

impl DamageFlash {
    fn new() -> Self {
        Self { bits: Arc::new(AtomicU32::new(0.0f32.to_bits())) }
    }

    fn get(&self) -> f32 {
        f32::from_bits(self.bits.load(Ordering::Relaxed))
    }

    fn set(&self, amount: f32) {
        self.bits.store(amount.clamp(0.0, 1.0).to_bits(), Ordering::Relaxed);
    }
}

pub struct GalactusShader {
    hide: Fbm,
    crack: Fbm,
    damage_flash: DamageFlash,
}

impl GalactusShader {
    pub fn new(seed: u32) -> Self {
        let hide = Fbm::new(seed).set_octaves(4).set_frequency(2.2).set_persistence(0.5);
        let crack = Fbm::new(seed.wrapping_add(211)).set_octaves(3).set_frequency(6.0).set_persistence(0.55);
        Self { hide, crack, damage_flash: DamageFlash::new() }
    }

    pub fn set_damage_flash(&self, amount: f32) {
        self.damage_flash.set(amount);
    }

    pub fn damage_flash(&self) -> DamageFlash {
        self.damage_flash.clone()
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
        let damage_flash = self.damage_flash.get();
        if damage_flash > 0.0 {
            let blink = 0.5 + 0.5 * (t * 55.0).sin();
            let damage_color = Vec3::new(1.0, 0.12, 0.06).lerp(Vec3::new(1.0, 1.0, 1.0), blink);
            final_color = final_color.lerp(damage_color, damage_flash * 0.82);
        }

        Vec4::new(final_color.x, final_color.y, final_color.z, 1.0)
    }
}

/// Rostro rosado y suave, separado del casco para que la silueta se lea como
/// personaje y no como otro planeta.
pub struct GalactusFaceShader {
    damage_flash: DamageFlash,
}

impl GalactusFaceShader {
    pub fn new(damage_flash: DamageFlash) -> Self {
        Self { damage_flash }
    }
}

impl FragmentShader for GalactusFaceShader {
    fn shade(&self, input: &FragInput, uniforms: &Uniforms) -> Vec4 {
        let local = input.local.normalize();
        let cheek = (1.0 - local.x.abs()).clamp(0.0, 1.0);
        let base = Vec3::new(0.46, 0.16, 0.22).lerp(Vec3::new(0.93, 0.48, 0.52), cheek * 0.72);
        let ndotl = input.normal.dot(uniforms.light_dir).max(0.0);
        let fresnel = (1.0 - input.normal.dot(input.view_dir).max(0.0)).powf(2.0);
        let mut color = base * (0.28 + ndotl * 0.72) + Vec3::new(0.35, 0.08, 0.2) * fresnel * 0.25;
        let flash = self.damage_flash.get();
        if flash > 0.0 {
            color = color.lerp(Vec3::new(1.0, 0.82, 0.78), flash * 0.85);
        }
        Vec4::new(color.x, color.y, color.z, 1.0)
    }
}

/// Ojos cian emisivos con pulso propio; siguen visibles en la zona oscura
/// porque no dependen de la iluminación del sol.
pub struct GalactusEyeShader {
    damage_flash: DamageFlash,
}

impl GalactusEyeShader {
    pub fn new(damage_flash: DamageFlash) -> Self {
        Self { damage_flash }
    }
}

impl FragmentShader for GalactusEyeShader {
    fn shade(&self, _input: &FragInput, uniforms: &Uniforms) -> Vec4 {
        let pulse = 0.72 + 0.28 * (uniforms.time * 4.5).sin().abs();
        let flash = self.damage_flash.get();
        let color = Vec3::new(0.05, 0.75, 0.9)
            .lerp(Vec3::new(0.65, 1.0, 1.0), pulse)
            .lerp(Vec3::new(1.0, 1.0, 1.0), flash);
        Vec4::new(color.x, color.y, color.z, 1.0)
    }
}
