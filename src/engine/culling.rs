//! Frustum culling por esfera: 6 planos extraídos de la matriz
//! view*proyección (método Gribb-Hartmann) para descartar cuerpos completos
//! antes de encolarlos en el rasterizador. Es matemática pura de cámara, sin
//! conocer los cuerpos del juego.

use crate::engine::math::{Mat4, Vec3};

pub struct Frustum {
    /// (normal, d): un punto `p` está dentro del semiespacio si `normal.dot(p) + d >= 0`.
    planes: [(Vec3, f32); 6],
}

impl Frustum {
    pub fn from_view_proj(vp: Mat4) -> Self {
        let row = |r: usize| (vp.m[r], vp.m[4 + r], vp.m[8 + r], vp.m[12 + r]);
        let (r0x, r0y, r0z, r0w) = row(0);
        let (r1x, r1y, r1z, r1w) = row(1);
        let (r2x, r2y, r2z, r2w) = row(2);
        let (r3x, r3y, r3z, r3w) = row(3);

        let combos = [
            (r3x + r0x, r3y + r0y, r3z + r0z, r3w + r0w), // izquierda
            (r3x - r0x, r3y - r0y, r3z - r0z, r3w - r0w), // derecha
            (r3x + r1x, r3y + r1y, r3z + r1z, r3w + r1w), // abajo
            (r3x - r1x, r3y - r1y, r3z - r1z, r3w - r1w), // arriba
            (r3x + r2x, r3y + r2y, r3z + r2z, r3w + r2w), // cerca
            (r3x - r2x, r3y - r2y, r3z - r2z, r3w - r2w), // lejos
        ];

        let planes = combos.map(|(a, b, c, d)| {
            let normal = Vec3::new(a, b, c);
            let len = normal.length().max(1e-6);
            (normal * (1.0 / len), d / len)
        });

        Self { planes }
    }

    /// `true` si la esfera toca o está dentro del frustum. Puede dar falsos
    /// positivos en las esquinas (intersección de dos planos truncando una
    /// esfera que en realidad ya quedó fuera); aceptable para un descarte
    /// conservador que nunca recorta algo que sí se debería ver.
    pub fn sphere_visible(&self, center: Vec3, radius: f32) -> bool {
        self.planes.iter().all(|(normal, d)| normal.dot(center) + d >= -radius)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_frustum() -> Frustum {
        let proj = Mat4::perspective_rh(60f32.to_radians(), 16.0 / 9.0, 0.1, 100.0);
        let view = Mat4::look_at_rh(Vec3::ZERO, Vec3::new(0.0, 0.0, -1.0), Vec3::Y);
        Frustum::from_view_proj(proj * view)
    }

    #[test]
    fn sphere_directly_ahead_is_visible() {
        let frustum = test_frustum();
        assert!(frustum.sphere_visible(Vec3::new(0.0, 0.0, -10.0), 1.0));
    }

    #[test]
    fn sphere_behind_camera_is_culled() {
        let frustum = test_frustum();
        assert!(!frustum.sphere_visible(Vec3::new(0.0, 0.0, 10.0), 1.0));
    }

    #[test]
    fn sphere_far_to_the_side_is_culled() {
        let frustum = test_frustum();
        assert!(!frustum.sphere_visible(Vec3::new(200.0, 0.0, -10.0), 1.0));
    }

    #[test]
    fn large_sphere_overlapping_a_plane_stays_visible() {
        let frustum = test_frustum();
        // Centro justo fuera del lado derecho, pero el radio alcanza a cruzar el plano.
        assert!(frustum.sphere_visible(Vec3::new(20.0, 0.0, -10.0), 15.0));
    }
}
