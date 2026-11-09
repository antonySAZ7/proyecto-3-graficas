//! Helpers genéricos de integración, independientes del número de frames.
//! Reutilizados por `ship.rs` y, en fases posteriores, por la gravedad/colisión.

use crate::engine::math::Vec3;

/// Avanza `current` hacia `target` a una tasa exponencial: suave, estable a
/// cualquier dt y sin overshoot, a diferencia de `current += (target-current)*dt*k`.
pub fn smooth_towards_vec3(current: Vec3, target: Vec3, rate: f32, dt: f32) -> Vec3 {
    current + (target - current) * (1.0 - (-rate * dt).exp())
}

/// Fricción exponencial: multiplicar por esto cada frame frena sin depender de los FPS.
pub fn drag_factor(drag: f32, dt: f32) -> f32 {
    (-drag * dt).exp()
}

pub fn clamp_length(v: Vec3, max_length: f32) -> Vec3 {
    let len = v.length();
    if len > max_length && len > 1e-6 {
        v * (max_length / len)
    } else {
        v
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn smooth_towards_vec3_reaches_target_eventually() {
        let mut v = Vec3::ZERO;
        let target = Vec3::new(10.0, 0.0, 0.0);
        for _ in 0..500 {
            v = smooth_towards_vec3(v, target, 5.0, 1.0 / 60.0);
        }
        assert!((v.x - 10.0).abs() < 0.01);
    }

    #[test]
    fn smooth_towards_vec3_is_frame_rate_independent() {
        // Dar dos pasos pequeños debe ser casi igual que un paso grande equivalente.
        let target = Vec3::new(10.0, 0.0, 0.0);
        let a = smooth_towards_vec3(smooth_towards_vec3(Vec3::ZERO, target, 5.0, 0.01), target, 5.0, 0.01);
        let b = smooth_towards_vec3(Vec3::ZERO, target, 5.0, 0.02);
        assert!((a.x - b.x).abs() < 1e-4);
    }

    #[test]
    fn clamp_length_caps_without_changing_direction() {
        let v = Vec3::new(3.0, 0.0, 4.0); // longitud 5
        let clamped = clamp_length(v, 2.0);
        assert!((clamped.length() - 2.0).abs() < 1e-5);
        assert!((clamped.x / clamped.z - v.x / v.z).abs() < 1e-5);
    }

    #[test]
    fn clamp_length_leaves_short_vectors_untouched() {
        let v = Vec3::new(1.0, 0.0, 0.0);
        assert_eq!(clamp_length(v, 5.0), v);
    }
}
