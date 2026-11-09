//! Objetos de superficie (Fase 5): 4 modelos propios (árbol cristalino,
//! roca, torre, baliza) distribuidos sobre toda la esfera de un planeta con
//! una semilla fija, orientados a su normal local y rotando junto con él.

use crate::engine::math::{Mat4, Quat, Vec3};
use crate::engine::mesh::{self, Mesh};
use crate::engine::noise::Fbm;
use crate::engine::rng::Lcg;
use crate::game::config;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum SurfaceKind {
    Tree,
    Rock,
    Tower,
    Beacon,
}

pub const KINDS: [SurfaceKind; 4] = [SurfaceKind::Tree, SurfaceKind::Rock, SurfaceKind::Tower, SurfaceKind::Beacon];

impl SurfaceKind {
    pub fn index(self) -> usize {
        match self {
            SurfaceKind::Tree => 0,
            SurfaceKind::Rock => 1,
            SurfaceKind::Tower => 2,
            SurfaceKind::Beacon => 3,
        }
    }
}

pub struct SurfaceModels {
    pub tree: Mesh,
    pub rock: Mesh,
    pub tower: Mesh,
    pub beacon: Mesh,
}

impl SurfaceModels {
    pub fn get(&self, kind: SurfaceKind) -> &Mesh {
        match kind {
            SurfaceKind::Tree => &self.tree,
            SurfaceKind::Rock => &self.rock,
            SurfaceKind::Tower => &self.tower,
            SurfaceKind::Beacon => &self.beacon,
        }
    }
}

/// Construye las 4 mallas una sola vez al inicio (no por frame ni por
/// instancia): piezas simples (cilindro, cono, caja, icosaedro) compuestas
/// con `mesh::transform` y `mesh::merge`.
pub fn build_models(seed: u32) -> SurfaceModels {
    let trunk = mesh::transform(&mesh::cylinder(6, 0.1, 0.8), Mat4::from_translation(Vec3::new(0.0, 0.4, 0.0)));
    let crown = mesh::transform(&mesh::cone(6, 0.5, 1.2), Mat4::from_translation(Vec3::new(0.0, 1.1, 0.0)));
    let tree = mesh::merge(&[trunk, crown]);

    let mut rock = mesh::icosphere(1);
    let rock_noise = Fbm::new(seed).set_octaves(3).set_frequency(2.5).set_persistence(0.5);
    mesh::displace(&mut rock, |dir| 0.55 + rock_noise.sample3(dir * 3.0) * 0.12);

    let tower_base = mesh::transform(&mesh::cuboid(Vec3::new(0.3, 0.55, 0.3)), Mat4::from_translation(Vec3::new(0.0, 0.55, 0.0)));
    let tower_top = mesh::transform(&mesh::cone(6, 0.4, 0.7), Mat4::from_translation(Vec3::new(0.0, 1.45, 0.0)));
    let tower = mesh::merge(&[tower_base, tower_top]);

    let beacon_stem = mesh::transform(&mesh::cylinder(6, 0.08, 0.9), Mat4::from_translation(Vec3::new(0.0, 0.45, 0.0)));
    let beacon_core = mesh::transform(
        &mesh::icosphere(1),
        Mat4::from_translation(Vec3::new(0.0, 1.0, 0.0)) * Mat4::from_scale(Vec3::splat(0.3)),
    );
    let beacon = mesh::merge(&[beacon_stem, beacon_core]);

    SurfaceModels { tree, rock, tower, beacon }
}

pub struct SurfaceInstance {
    pub kind: SurfaceKind,
    /// Normal en espacio local del planeta (antes de su traslación/rotación propia).
    pub normal: Vec3,
    pub spin: f32,
    pub scale: f32,
}

/// Distribuye `count` instancias sobre toda la esfera unitaria con una
/// semilla fija: misma semilla, mismo resultado siempre.
pub fn scatter(seed: u32, count: usize) -> Vec<SurfaceInstance> {
    let mut rng = Lcg::new(seed);
    (0..count)
        .map(|i| {
            let y = rng.next_f32() * 2.0 - 1.0;
            let angle = rng.next_f32() * std::f32::consts::TAU;
            let radial = (1.0 - y * y).max(0.0).sqrt();
            let normal = Vec3::new(angle.cos() * radial, y, angle.sin() * radial);
            SurfaceInstance {
                kind: KINDS[i % KINDS.len()],
                normal,
                spin: rng.next_f32() * std::f32::consts::TAU,
                scale: config::SURFACE_SCALE_MIN + rng.next_f32() * (config::SURFACE_SCALE_MAX - config::SURFACE_SCALE_MIN),
            }
        })
        .collect()
}

/// Matriz de mundo de una instancia: pegada a la superficie del cuerpo
/// (radio + margen) y orientada a su normal; `planet_transform` es solo la
/// traslación+rotación propia del cuerpo (sin su escala), así rota con él.
pub fn instance_model(instance: &SurfaceInstance, planet_transform: Mat4, body_radius: f32) -> Mat4 {
    let local_position = instance.normal * (body_radius + config::SURFACE_OFFSET);
    let aligned = Mat4::align_y_to_normal(instance.normal, local_position);
    let spin = Quat::from_axis_angle(Vec3::Y, instance.spin).to_mat4();
    planet_transform * aligned * spin * Mat4::from_scale(Vec3::splat(instance.scale))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scatter_is_reproducible_with_same_seed() {
        let a = scatter(2026, 30);
        let b = scatter(2026, 30);
        for (ia, ib) in a.iter().zip(b.iter()) {
            assert_eq!(ia.normal, ib.normal);
            assert_eq!(ia.spin, ib.spin);
            assert_eq!(ia.scale, ib.scale);
        }
    }

    #[test]
    fn different_seed_gives_a_different_map() {
        let a = scatter(2026, 30);
        let b = scatter(7, 30);
        assert!(a.iter().zip(b.iter()).any(|(ia, ib)| ia.normal.distance(ib.normal) > 1e-3));
    }

    #[test]
    fn scatter_covers_the_whole_sphere_with_unit_normals() {
        let instances = scatter(2026, 200);
        assert!(instances.iter().any(|i| i.normal.y > 0.5));
        assert!(instances.iter().any(|i| i.normal.y < -0.5));
        for i in &instances {
            assert!((i.normal.length() - 1.0).abs() < 1e-4);
        }
    }

    #[test]
    fn uses_all_four_kinds() {
        let instances = scatter(7, 8);
        for kind in KINDS {
            assert!(instances.iter().any(|i| i.kind == kind));
        }
    }

    #[test]
    fn instance_sits_at_body_radius_plus_offset_from_the_planet_center() {
        let instances = scatter(1, 5);
        let planet_center = Vec3::new(10.0, -3.0, 4.0);
        let planet_transform = Mat4::from_translation(planet_center);
        let radius = 6.0;
        for instance in &instances {
            let model = instance_model(instance, planet_transform, radius);
            let world_pos = model.transform_point3(Vec3::ZERO);
            let distance = world_pos.distance(planet_center);
            assert!((distance - (radius + config::SURFACE_OFFSET)).abs() < 1e-3);
        }
    }
}
