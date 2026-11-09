//! Estrellas de fondo: puntos fijos en una esfera lejana que giran con la
//! cámara pero no se trasladan con ella (están "en el infinito").

use crate::engine::framebuffer::Framebuffer;
use crate::engine::math::{Mat4, Vec3, Vec4};
use crate::engine::rng::Lcg;
use crate::game::config;

struct Star {
    direction: Vec3,
    brightness: f32,
}

pub struct Skybox {
    stars: Vec<Star>,
}

impl Skybox {
    pub fn new(seed: u32, count: usize) -> Self {
        let mut rng = Lcg::new(seed);
        let mut stars = Vec::with_capacity(count);
        for _ in 0..count {
            let y = rng.next_f32() * 2.0 - 1.0;
            let angle = rng.next_f32() * std::f32::consts::TAU;
            let radial = (1.0 - y * y).max(0.0).sqrt();
            let direction = Vec3::new(angle.cos() * radial, y, angle.sin() * radial);
            let brightness = 0.35 + rng.next_f32() * 0.65;
            stars.push(Star { direction, brightness });
        }
        Self { stars }
    }

    /// `view_proj_rot`: proyección combinada con la rotación de cámara (sin traslación).
    pub fn draw(&self, fb: &mut Framebuffer, view_proj_rot: Mat4) {
        let width = fb.width as f32;
        let height = fb.height as f32;
        for star in &self.stars {
            let world = star.direction * config::STARFIELD_RADIUS;
            let clip = view_proj_rot * world.extend(1.0);
            if clip.w <= 0.0001 {
                continue;
            }
            let inv_w = 1.0 / clip.w;
            let x = (((clip.x * inv_w) * 0.5 + 0.5) * width) as i32;
            let y = ((1.0 - ((clip.y * inv_w) * 0.5 + 0.5)) * height) as i32;
            if x < 0 || y < 0 || x >= width as i32 || y >= height as i32 {
                continue;
            }
            // Profundidad al borde del far plane: cualquier geometría real lo tapa.
            let color = Vec4::new(star.brightness, star.brightness, star.brightness, 1.0);
            fb.blend_pixel(x as usize, y as usize, 1.0, color);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn same_seed_is_reproducible() {
        let a = Skybox::new(42, 50);
        let b = Skybox::new(42, 50);
        for (sa, sb) in a.stars.iter().zip(b.stars.iter()) {
            assert_eq!(sa.direction, sb.direction);
        }
    }

    #[test]
    fn star_directions_are_normalized() {
        let sky = Skybox::new(7, 100);
        for star in &sky.stars {
            assert!((star.direction.length() - 1.0).abs() < 1e-4);
        }
    }
}
