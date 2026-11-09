//! Estado y actualización de la nave: entradas -> rotación -> velocidad -> posición.
//! Todo depende de `dt`, nunca del número de frames.

use crate::engine::math::{Quat, Vec3};
use crate::game::config;
use crate::game::input::InputState;
use crate::game::physics;

pub struct Ship {
    pub position: Vec3,
    pub orientation: Quat,
    pub velocity: Vec3,
    /// Velocidad angular suavizada por eje local (pitch, yaw, roll), rad/s.
    angular_velocity: Vec3,
    pub speed: f32,
    pub cruise_enabled: bool,
    /// Velocidad objetivo mientras el crucero está activo.
    cruise_speed: f32,
    pub stabilize_assist: bool,
}

impl Ship {
    pub fn new(position: Vec3) -> Self {
        Self {
            position,
            orientation: Quat::IDENTITY,
            velocity: Vec3::ZERO,
            angular_velocity: Vec3::ZERO,
            speed: 0.0,
            cruise_enabled: false,
            cruise_speed: 0.0,
            stabilize_assist: true,
        }
    }

    /// El modelo apunta hacia -Z local (ver assets/models/fantasticar.obj).
    pub fn forward(&self) -> Vec3 {
        self.orientation.rotate_vec3(Vec3::new(0.0, 0.0, -1.0))
    }

    pub fn up(&self) -> Vec3 {
        self.orientation.rotate_vec3(Vec3::Y)
    }

    pub fn right(&self) -> Vec3 {
        self.orientation.rotate_vec3(Vec3::X)
    }

    fn max_speed_here(&self) -> f32 {
        let distance = self.position.distance(config::NAV_TARGET_POINT);
        if distance >= config::NAV_SLOWDOWN_RADIUS {
            config::SHIP_MAX_SPEED
        } else {
            let t = (distance / config::NAV_SLOWDOWN_RADIUS).clamp(0.0, 1.0);
            config::NAV_MIN_SPEED_NEAR + (config::SHIP_MAX_SPEED - config::NAV_MIN_SPEED_NEAR) * t
        }
    }

    pub fn update(&mut self, input: &InputState, dt: f32) {
        if input.toggle_stabilize {
            self.stabilize_assist = !self.stabilize_assist;
        }

        self.update_rotation(input, dt);
        self.update_translation(input, dt);
    }

    fn update_rotation(&mut self, input: &InputState, dt: f32) {
        let mut pitch_target = input.pitch * config::SHIP_PITCH_RATE;
        let mut roll_target = input.roll * config::SHIP_ROLL_RATE;
        let yaw_target = input.yaw * config::SHIP_YAW_RATE;

        // Estabilización asistida: sin entrada de cabeceo/alabeo, la nave se
        // nivela sola para no marear al piloto (mejora sobre el JS original).
        if self.stabilize_assist {
            if input.pitch.abs() < 0.01 {
                pitch_target = 0.0;
            }
            if input.roll.abs() < 0.01 {
                roll_target = 0.0;
            }
        }

        let response = 1.0 - (-dt * config::SHIP_TURN_RESPONSE).exp();
        self.angular_velocity.x += (pitch_target - self.angular_velocity.x) * response;
        self.angular_velocity.y += (yaw_target - self.angular_velocity.y) * response;
        self.angular_velocity.z += (roll_target - self.angular_velocity.z) * response;

        // Cuaternión incremental en espacio local: evita gimbal lock y permite
        // looping completo (a diferencia del Euler clamp del JS original).
        let delta = Quat::from_axis_angle(Vec3::X, self.angular_velocity.x * dt)
            * Quat::from_axis_angle(Vec3::Y, self.angular_velocity.y * dt)
            * Quat::from_axis_angle(Vec3::Z, self.angular_velocity.z * dt);
        self.orientation = (self.orientation * delta).normalize();
    }

    fn update_translation(&mut self, input: &InputState, dt: f32) {
        if input.toggle_cruise {
            self.cruise_enabled = !self.cruise_enabled;
            self.cruise_speed = self.speed;
        }

        let accel = if input.boost { config::SHIP_BOOST_ACCEL } else { config::SHIP_ACCEL };
        let forward = self.forward();

        if self.cruise_enabled {
            // W/S ajustan el objetivo de crucero en vez de acelerar directamente.
            self.cruise_speed = (self.cruise_speed + input.throttle * config::SHIP_CRUISE_ADJUST_RATE * dt)
                .clamp(0.0, config::SHIP_MAX_SPEED);
            let target_velocity = forward * self.cruise_speed;
            self.velocity =
                physics::smooth_towards_vec3(self.velocity, target_velocity, config::SHIP_CRUISE_RESPONSE, dt);
        } else {
            self.velocity = self.velocity + forward * (input.throttle * accel * dt);
            self.velocity = self.velocity * physics::drag_factor(config::SHIP_DRAG, dt);
        }

        self.velocity = physics::clamp_length(self.velocity, self.max_speed_here());
        self.position = self.position + self.velocity * dt;
        self.speed = self.velocity.length();
    }

    /// Dirección normalizada, en espacio del mundo, hacia el punto de interés
    /// actual (indicador de dirección al objetivo; en Fase 3 será el planeta
    /// más cercano en vez de un punto fijo).
    pub fn direction_to_target(&self) -> Vec3 {
        (config::NAV_TARGET_POINT - self.position).normalize()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::input::InputState;

    #[test]
    fn throttle_accelerates_forward() {
        let mut ship = Ship::new(Vec3::ZERO);
        let input = InputState { throttle: 1.0, ..Default::default() };
        for _ in 0..30 {
            ship.update(&input, 1.0 / 60.0);
        }
        assert!(ship.speed > 0.0);
        assert!(ship.position.z < 0.0); // forward local es -Z con orientación identidad
    }

    #[test]
    fn drag_without_throttle_slows_ship_down() {
        let mut ship = Ship::new(Vec3::ZERO);
        ship.velocity = Vec3::new(0.0, 0.0, -10.0);
        let input = InputState::default();
        for _ in 0..120 {
            ship.update(&input, 1.0 / 60.0);
        }
        assert!(ship.speed < 10.0);
    }

    #[test]
    fn stabilize_assist_returns_roll_to_level() {
        let mut ship = Ship::new(Vec3::ZERO);
        ship.update(&InputState { roll: 1.0, ..Default::default() }, 0.2);
        assert!(ship.angular_velocity.z.abs() > 0.0);
        for _ in 0..120 {
            ship.update(&InputState::default(), 1.0 / 60.0);
        }
        assert!(ship.angular_velocity.z.abs() < 0.01);
    }

    #[test]
    fn turning_is_frame_rate_independent() {
        let mut a = Ship::new(Vec3::ZERO);
        let mut b = Ship::new(Vec3::ZERO);
        let input = InputState { yaw: 1.0, throttle: 1.0, ..Default::default() };
        for _ in 0..60 {
            a.update(&input, 1.0 / 60.0);
        }
        for _ in 0..30 {
            b.update(&input, 1.0 / 30.0);
        }
        // Integración explícita: con trayectoria curva habrá algo de error entre
        // pasos grandes y chicos, pero debe ser pequeño frente a la distancia recorrida.
        assert!(a.position.distance(b.position) < 0.1);
    }
}
