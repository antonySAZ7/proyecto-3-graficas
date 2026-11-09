//! Estado y actualización de la nave: entradas -> rotación -> velocidad -> posición.
//! Todo depende de `dt`, nunca del número de frames.

use crate::engine::math::{Quat, Vec3};
use crate::game::config;
use crate::game::input::InputState;
use crate::game::physics;
use crate::game::world::World;

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
    /// Modo `O`: circunda el cuerpo más cercano en vez de vuelo libre.
    pub orbit_assist: bool,
    /// Último cuerpo contra el que colisionó, si lo hubo este frame.
    pub last_collision: Option<&'static str>,
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
            orbit_assist: false,
            last_collision: None,
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

    fn max_speed_here(&self, world: &World) -> f32 {
        let nearest = world.nearest_body(self.position);
        let slowdown_radius = nearest.radius + config::NAV_SLOWDOWN_MARGIN;
        let distance = nearest.surface_distance.max(0.0);
        if distance >= slowdown_radius {
            config::SHIP_MAX_SPEED
        } else {
            let t = (distance / slowdown_radius).clamp(0.0, 1.0);
            config::NAV_MIN_SPEED_NEAR + (config::SHIP_MAX_SPEED - config::NAV_MIN_SPEED_NEAR) * t
        }
    }

    pub fn update(&mut self, input: &InputState, world: &World, dt: f32) {
        if input.toggle_stabilize {
            self.stabilize_assist = !self.stabilize_assist;
        }
        if input.toggle_orbit_assist {
            self.orbit_assist = !self.orbit_assist;
        }

        self.update_rotation(input, dt);
        self.update_translation(input, world, dt);
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

    fn update_translation(&mut self, input: &InputState, world: &World, dt: f32) {
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

        // Cada cuerpo atrae un poco la trayectoria (inversa del cuadrado, acotada).
        self.velocity = self.velocity + world.gravity_accel(self.position) * dt;

        if self.orbit_assist {
            self.apply_orbit_assist(world, dt);
        }

        self.velocity = physics::clamp_length(self.velocity, self.max_speed_here(world));
        self.position = self.position + self.velocity * dt;

        let (position, velocity, collided) = world.resolve_collision(self.position, self.velocity);
        self.position = position;
        self.velocity = velocity;
        self.last_collision = collided;

        self.speed = self.velocity.length();
    }

    /// Modo `O`: en vez de vuelo libre, circunda el cuerpo más cercano a una
    /// distancia preferida, con velocidad tangencial suavizada.
    fn apply_orbit_assist(&mut self, world: &World, dt: f32) {
        let nearest = world.nearest_body(self.position);
        let radial = (self.position - nearest.position).normalize();
        let mut tangent = Vec3::Y.cross(radial).normalize();
        if tangent.length_squared() < 0.01 {
            tangent = Vec3::X.cross(radial).normalize();
        }

        let desired_speed = (config::ORBIT_ASSIST_MIN_SPEED
            + nearest.surface_distance.max(0.0) * config::ORBIT_ASSIST_SPEED_PER_DISTANCE)
            .clamp(config::ORBIT_ASSIST_MIN_SPEED, config::ORBIT_ASSIST_MAX_SPEED);
        let desired = tangent * desired_speed;
        self.velocity = physics::smooth_towards_vec3(self.velocity, desired, config::ORBIT_ASSIST_RESPONSE, dt);

        let preferred_distance = nearest.radius + config::ORBIT_ASSIST_PREFERRED_MARGIN;
        let current_distance = self.position.distance(nearest.position);
        let correction = ((preferred_distance - current_distance) * config::ORBIT_ASSIST_CORRECTION_GAIN)
            .clamp(-config::ORBIT_ASSIST_MAX_CORRECTION, config::ORBIT_ASSIST_MAX_CORRECTION);
        self.velocity = self.velocity + radial * (correction * dt);
    }

    /// Dirección normalizada, en espacio del mundo, hacia el cuerpo más
    /// cercano (indicador de dirección al objetivo).
    pub fn direction_to_target(&self, world: &World) -> Vec3 {
        let nearest = world.nearest_body(self.position);
        (nearest.position - self.position).normalize()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::input::InputState;

    /// Lejos de cualquier cuerpo: gravedad y límite de velocidad por cercanía
    /// quedan fuera de juego, así las pruebas de vuelo libre siguen siendo puras.
    fn far_from_everything() -> Vec3 {
        Vec3::new(0.0, 0.0, 400.0)
    }

    #[test]
    fn throttle_accelerates_forward() {
        let world = World::new();
        let start = far_from_everything();
        let mut ship = Ship::new(start);
        let input = InputState { throttle: 1.0, ..Default::default() };
        for _ in 0..30 {
            ship.update(&input, &world, 1.0 / 60.0);
        }
        assert!(ship.speed > 0.0);
        assert!(ship.position.z < start.z); // forward local es -Z con orientación identidad
    }

    #[test]
    fn drag_without_throttle_slows_ship_down() {
        let world = World::new();
        let mut ship = Ship::new(far_from_everything());
        ship.velocity = Vec3::new(0.0, 0.0, -10.0);
        let input = InputState::default();
        for _ in 0..120 {
            ship.update(&input, &world, 1.0 / 60.0);
        }
        assert!(ship.speed < 10.0);
    }

    #[test]
    fn stabilize_assist_returns_roll_to_level() {
        let world = World::new();
        let mut ship = Ship::new(far_from_everything());
        ship.update(&InputState { roll: 1.0, ..Default::default() }, &world, 0.2);
        assert!(ship.angular_velocity.z.abs() > 0.0);
        for _ in 0..120 {
            ship.update(&InputState::default(), &world, 1.0 / 60.0);
        }
        assert!(ship.angular_velocity.z.abs() < 0.01);
    }

    #[test]
    fn turning_is_frame_rate_independent() {
        let world = World::new();
        let start = far_from_everything();
        let mut a = Ship::new(start);
        let mut b = Ship::new(start);
        let input = InputState { yaw: 1.0, throttle: 1.0, ..Default::default() };
        for _ in 0..60 {
            a.update(&input, &world, 1.0 / 60.0);
        }
        for _ in 0..30 {
            b.update(&input, &world, 1.0 / 30.0);
        }
        // Integración explícita: con trayectoria curva habrá algo de error entre
        // pasos grandes y chicos, pero debe ser pequeño frente a la distancia recorrida.
        assert!(a.position.distance(b.position) < 0.1);
    }

    #[test]
    fn orbit_assist_keeps_a_stable_distance_from_the_nearest_body() {
        let world = World::new();
        let earth = world.planets[crate::game::bodies::EARTH_INDEX].position;
        let earth_radius = crate::game::bodies::PLANETS[crate::game::bodies::EARTH_INDEX].radius;
        let mut ship = Ship::new(earth + Vec3::new(0.0, 0.0, earth_radius + 6.0));
        ship.orbit_assist = true;
        for _ in 0..600 {
            ship.update(&InputState::default(), &world, 1.0 / 60.0);
        }
        let distance = ship.position.distance(earth);
        assert!(distance > earth_radius, "no debe atravesar el planeta");
        assert!(distance < earth_radius + 12.0, "debe mantenerse cerca de la distancia preferida");
    }

    #[test]
    fn collision_stops_the_ship_from_entering_a_planet() {
        let world = World::new();
        let earth = world.planets[crate::game::bodies::EARTH_INDEX].position;
        let earth_radius = crate::game::bodies::PLANETS[crate::game::bodies::EARTH_INDEX].radius;
        let mut ship = Ship::new(earth + Vec3::new(0.0, 0.0, earth_radius + 1.0));
        ship.velocity = Vec3::new(0.0, 0.0, -30.0);
        for _ in 0..30 {
            ship.update(&InputState::default(), &world, 1.0 / 60.0);
        }
        assert!(ship.position.distance(earth) >= earth_radius);
    }
}
