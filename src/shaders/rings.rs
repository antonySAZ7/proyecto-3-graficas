use crate::engine::math::{Vec3, Vec4};
use crate::engine::noise::Perlin;
use crate::shaders::{FragInput, FragmentShader, Uniforms};

/// Anillos del planeta, geometría aparte. Cuatro capas: gradiente radial
/// base, bandas de densidad/grano (que también controlan el alfa), luz
/// Lambertiana sobre el plano del anillo, y la sombra del planeta.
pub struct RingShader {
    grain: Perlin,
    inner: f32,
    outer: f32,
    planet_center: Vec3,
    planet_radius: f32,
}

impl RingShader {
    pub fn new(seed: u32, planet_center: Vec3, planet_radius: f32, inner: f32, outer: f32) -> Self {
        Self { grain: Perlin::new(seed), inner, outer, planet_center, planet_radius }
    }

    fn in_planet_shadow(&self, point: Vec3, light_dir: Vec3) -> bool {
        let to_point = point - self.planet_center;
        let along_light = to_point.dot(light_dir);
        if along_light > 0.0 {
            return false; // lado iluminado del planeta
        }
        let perp = to_point - light_dir * along_light;
        perp.length() < self.planet_radius
    }
}

impl FragmentShader for RingShader {
    fn shade(&self, input: &FragInput, uniforms: &Uniforms) -> Vec4 {
        let r = Vec3::new(input.local.x, 0.0, input.local.z).length();
        let t = ((r - self.inner) / (self.outer - self.inner)).clamp(0.0, 1.0);

        // Capa 1: gradiente radial, tono cálido cerca del planeta y gris pálido hacia afuera.
        let base = Vec3::new(0.78, 0.68, 0.52).lerp(Vec3::new(0.58, 0.57, 0.56), t);

        // Capa 2: bandas de densidad y grano fino, afectan color y transparencia.
        let bands = (r * 36.0).sin() * 0.5 + 0.5;
        let grain_n = self.grain.sample3(Vec3::new(input.local.x * 25.0, 0.0, input.local.z * 25.0));
        let density = (bands * 0.65 + (grain_n * 0.5 + 0.5) * 0.35).clamp(0.0, 1.0);
        let banded = base * (0.45 + 0.55 * density);

        // Capa 3: iluminación Lambertiana sobre el plano del anillo.
        let ndotl = input.normal.dot(uniforms.light_dir).abs();
        let lit = banded * (0.35 + 0.65 * ndotl);

        // Capa 4: sombra que el planeta proyecta sobre los anillos.
        let shadowed = if self.in_planet_shadow(input.world_pos, uniforms.light_dir) {
            lit * 0.2
        } else {
            lit
        };

        let alpha = 0.3 + 0.6 * density;
        Vec4::new(shadowed.x, shadowed.y, shadowed.z, alpha)
    }
}
