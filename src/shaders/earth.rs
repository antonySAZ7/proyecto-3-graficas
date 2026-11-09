use crate::engine::math::{Vec3, Vec4};
use crate::engine::noise::Fbm;
use crate::shaders::{FragInput, FragmentShader, Uniforms};

const TERRAIN_SCALE: f32 = 2.2;

/// Perturba la normal geométrica con el gradiente (diferencias finitas) del
/// mismo ruido de terreno: mapeo de normales procedural calculado aquí, en
/// el renderer por software, sin textura ni UV de malla.
fn bump_normal(p: Vec3, normal: Vec3, terrain: &Fbm, strength: f32) -> Vec3 {
    let eps = 0.05;
    let up = if normal.x.abs() < 0.9 { Vec3::X } else { Vec3::Y };
    let tangent = up.cross(normal).normalize();
    let bitangent = normal.cross(tangent);
    let h0 = terrain.sample3(p * TERRAIN_SCALE);
    let hu = terrain.sample3((p + tangent * eps) * TERRAIN_SCALE);
    let hv = terrain.sample3((p + bitangent * eps) * TERRAIN_SCALE);
    let du = (hu - h0) / eps;
    let dv = (hv - h0) / eps;
    (normal - tangent * du * strength - bitangent * dv * strength).normalize()
}

/// Tierra. Cinco capas: océano/continente por altura de ruido, casquetes
/// polares, mapeo de normales procedural, nubes animadas y fresnel
/// atmosférico.
pub struct EarthShader {
    terrain: Fbm,
    clouds: Fbm,
}

impl EarthShader {
    pub fn new(seed: u32) -> Self {
        let terrain = Fbm::new(seed).set_octaves(5).set_frequency(1.0).set_persistence(0.5);
        let clouds = Fbm::new(seed.wrapping_add(173)).set_octaves(4).set_frequency(1.0).set_persistence(0.55);
        Self { terrain, clouds }
    }
}

impl FragmentShader for EarthShader {
    fn shade(&self, input: &FragInput, uniforms: &Uniforms) -> Vec4 {
        let p = input.local.normalize();
        let t = uniforms.time;

        // Capa 1: altura de terreno decide océano u continente, y su matiz.
        let h = self.terrain.sample3(p * TERRAIN_SCALE);
        let land = h > 0.04;
        let base = if land {
            let height = ((h - 0.04) / 0.6).clamp(0.0, 1.0);
            Vec3::new(0.14, 0.42, 0.22).lerp(Vec3::new(0.62, 0.55, 0.38), height)
        } else {
            let depth = ((h + 0.5) / 0.54).clamp(0.0, 1.0);
            Vec3::new(0.02, 0.08, 0.28).lerp(Vec3::new(0.05, 0.42, 0.62), depth)
        };

        // Capa 2: casquetes polares de hielo.
        let polar = ((p.y.abs() - 0.82).max(0.0) * 5.5).clamp(0.0, 1.0);
        let based = base.lerp(Vec3::new(0.9, 0.95, 1.0), polar);

        // Capa 3: normal perturbada (relieve) + iluminación Lambertiana y
        // brillo especular del mar.
        let bumped = bump_normal(p, input.normal, &self.terrain, if land { 0.5 } else { 0.15 });
        let ndotl = bumped.dot(uniforms.light_dir).max(0.0);
        let half = (uniforms.light_dir + input.view_dir).normalize();
        let spec = if land { 0.0 } else { bumped.dot(half).max(0.0).powf(28.0) * 0.6 };
        let lit = based * (0.18 + 0.82 * ndotl) + Vec3::splat(spec);

        // Capa 4: nubes animadas (el ruido se desplaza con el tiempo).
        let c = self.clouds.sample3(Vec3::new(p.x * 3.2 + t * 0.04, p.y * 3.2, p.z * 3.2));
        let cloud_mask = ((c - 0.15).max(0.0) * 2.2).clamp(0.0, 1.0);
        let clouded = lit.lerp(Vec3::new(0.92, 0.95, 0.98), cloud_mask * 0.8);

        // Capa 5: fresnel atmosférico azul en el borde del disco.
        let fres = (1.0 - input.normal.dot(input.view_dir).clamp(0.0, 1.0)).powf(2.5);
        let final_color = clouded + Vec3::new(0.25, 0.45, 0.9) * fres * 0.55;

        Vec4::new(final_color.x, final_color.y, final_color.z, 1.0)
    }
}
