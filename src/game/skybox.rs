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

    /// `view_proj_rot`: proyección combinada con la rotación de cámara (sin
    /// traslación). `warp_intensity` en 0 dibuja estrellas como puntos; por
    /// encima de 0 las estira en estelas de luz que radian desde el centro
    /// de la pantalla, hacia el color de `config::WARP_STREAK_COLOR` (el
    /// efecto barato de la Fase 7 para sentir el impulso de warp, `H`).
    pub fn draw(&self, fb: &mut Framebuffer, view_proj_rot: Mat4, warp_intensity: f32) {
        let width = fb.width as f32;
        let height = fb.height as f32;
        let center = (width * 0.5, height * 0.5);
        for star in &self.stars {
            let world = star.direction * config::STARFIELD_RADIUS;
            let clip = view_proj_rot * world.extend(1.0);
            if clip.w <= 0.0001 {
                continue;
            }
            let inv_w = 1.0 / clip.w;
            let x = ((clip.x * inv_w) * 0.5 + 0.5) * width;
            let y = (1.0 - ((clip.y * inv_w) * 0.5 + 0.5)) * height;
            if x < 0.0 || y < 0.0 || x >= width || y >= height {
                continue;
            }

            // Profundidad al borde del far plane: cualquier geometría real lo tapa.
            if warp_intensity <= 0.001 {
                let color = Vec4::new(star.brightness, star.brightness, star.brightness, 1.0);
                fb.blend_pixel(x as usize, y as usize, 1.0, color);
                continue;
            }

            let dir_x = x - center.0;
            let dir_y = y - center.1;
            let len = dir_x.hypot(dir_y);
            let (ndx, ndy) = if len > 1.0 { (dir_x / len, dir_y / len) } else { (1.0, 0.0) };
            let streak_len = warp_intensity * config::WARP_STREAK_LENGTH_SCALE * (0.4 + star.brightness * 0.6);
            let streak_color = Vec3::splat(star.brightness).lerp(config::WARP_STREAK_COLOR, warp_intensity.min(1.0));

            let steps = 8;
            for i in 0..=steps {
                let t = i as f32 / steps as f32;
                let px = x + ndx * streak_len * t;
                let py = y + ndy * streak_len * t;
                if px < 0.0 || py < 0.0 || px >= width || py >= height {
                    continue;
                }
                let alpha = (1.0 - t) * star.brightness;
                fb.blend_pixel(px as usize, py as usize, 1.0, Vec4::new(streak_color.x, streak_color.y, streak_color.z, alpha));
            }
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
