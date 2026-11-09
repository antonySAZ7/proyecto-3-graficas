use crate::engine::math::{Vec3, Vec4};
use crate::game::surface::SurfaceKind;
use crate::shaders::{FragInput, FragmentShader, Uniforms};

/// Shader simple y barato para los props de superficie (Fase 5): color plano
/// por tipo con luz Lambertiana y un reborde fresnel. Los shaders "de
/// verdad" (animados, con mapeo de normales) llegan en la Fase 6.
pub struct SurfaceShader {
    base_color: Vec3,
    accent_color: Vec3,
}

impl SurfaceShader {
    pub fn for_kind(kind: SurfaceKind) -> Self {
        match kind {
            SurfaceKind::Tree => Self { base_color: Vec3::new(0.12, 0.5, 0.58), accent_color: Vec3::new(0.4, 0.95, 1.0) },
            SurfaceKind::Rock => Self { base_color: Vec3::new(0.32, 0.3, 0.3), accent_color: Vec3::new(0.5, 0.48, 0.46) },
            SurfaceKind::Tower => Self { base_color: Vec3::new(0.4, 0.44, 0.5), accent_color: Vec3::new(0.65, 0.78, 0.9) },
            SurfaceKind::Beacon => Self { base_color: Vec3::new(0.5, 0.42, 0.18), accent_color: Vec3::new(1.0, 0.75, 0.3) },
        }
    }
}

impl FragmentShader for SurfaceShader {
    fn shade(&self, input: &FragInput, uniforms: &Uniforms) -> Vec4 {
        let ndotl = input.normal.dot(uniforms.light_dir).max(0.0);
        let lit = self.base_color * (0.2 + 0.8 * ndotl);

        let fres = (1.0 - input.normal.dot(input.view_dir).clamp(0.0, 1.0)).powf(2.5);
        let final_color = lit + self.accent_color * fres * 0.5;

        Vec4::new(final_color.x, final_color.y, final_color.z, 1.0)
    }
}
