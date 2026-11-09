use crate::engine::math::{Vec3, Vec4};
use crate::shaders::{FragInput, FragmentShader, Uniforms};

/// Shader provisional de la nave: metal azulado con luz Lambertiana y
/// reborde fresnel. Se reemplaza por el definitivo en la fase de shaders.
pub struct ShipShader {
    base_color: Vec3,
}

impl ShipShader {
    pub fn new() -> Self {
        Self { base_color: Vec3::new(0.55, 0.62, 0.72) }
    }
}

impl FragmentShader for ShipShader {
    fn shade(&self, input: &FragInput, uniforms: &Uniforms) -> Vec4 {
        let ndotl = input.normal.dot(uniforms.light_dir).max(0.0);
        let lit = self.base_color * (0.18 + 0.82 * ndotl);

        let fres = (1.0 - input.normal.dot(input.view_dir).clamp(0.0, 1.0)).powf(2.5);
        let final_color = lit + Vec3::new(0.5, 0.7, 1.0) * fres * 0.6;

        Vec4::new(final_color.x, final_color.y, final_color.z, 1.0)
    }
}
