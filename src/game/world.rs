//! Simulación del sistema solar: posiciones orbitales por tiempo, gravedad y
//! colisión esférica contra todos los cuerpos. Los cuerpos en sí son datos
//! (`bodies.rs`); aquí vive solo el movimiento y la física que dependen de `dt`.

use crate::engine::math::Vec3;
use crate::game::bodies::{self, BodyDef};
use crate::game::config;

pub struct BodyInstance {
    pub def: &'static BodyDef,
    pub position: Vec3,
    pub rotation: f32,
}

/// Un cuerpo visto como fuente de gravedad/colisión: mismo trato para el sol,
/// los planetas y la luna sin repetir la lógica tres veces.
pub struct Attractor {
    pub name: &'static str,
    pub position: Vec3,
    pub radius: f32,
}

pub struct Nearest {
    pub name: &'static str,
    pub position: Vec3,
    pub radius: f32,
    /// Distancia a la superficie (negativa si ya se está dentro del cuerpo).
    pub surface_distance: f32,
}

pub struct World {
    pub sun_position: Vec3,
    pub planets: Vec<BodyInstance>,
    pub moon_position: Vec3,
    pub moon_rotation: f32,
}

/// Posición sobre una órbita circular inclinada: primero se ubica en el plano
/// XZ y luego se inclina ese plano rotando alrededor del eje X.
fn orbit_position(center: Vec3, radius: f32, angle: f32, inclination: f32) -> Vec3 {
    let (s, c) = inclination.sin_cos();
    let x = angle.cos() * radius;
    let z = angle.sin() * radius;
    center + Vec3::new(x, z * s, z * c)
}

impl World {
    pub fn new() -> Self {
        let planets = bodies::PLANETS
            .iter()
            .map(|def| BodyInstance { def, position: Vec3::ZERO, rotation: 0.0 })
            .collect();
        let mut world = Self { sun_position: Vec3::ZERO, planets, moon_position: Vec3::ZERO, moon_rotation: 0.0 };
        world.update(0.0);
        world
    }

    pub fn update(&mut self, time: f32) {
        for planet in &mut self.planets {
            let angle = planet.def.phase + time * planet.def.orbit_speed;
            planet.position = orbit_position(self.sun_position, planet.def.orbit_radius, angle, planet.def.inclination);
            planet.rotation = time * planet.def.rotation_speed;
        }

        let earth = &self.planets[bodies::EARTH_INDEX];
        let moon_angle = bodies::MOON.phase + time * bodies::MOON.orbit_speed;
        self.moon_position = orbit_position(earth.position, bodies::MOON.orbit_radius, moon_angle, 0.0);
        self.moon_rotation = time * bodies::MOON.rotation_speed;
    }

    pub fn attractors(&self) -> impl Iterator<Item = Attractor> + '_ {
        std::iter::once(Attractor { name: bodies::SUN_NAME, position: self.sun_position, radius: bodies::SUN_RADIUS })
            .chain(self.planets.iter().map(|p| Attractor { name: p.def.name, position: p.position, radius: p.def.radius }))
            .chain(std::iter::once(Attractor { name: bodies::MOON.name, position: self.moon_position, radius: bodies::MOON.radius }))
    }

    /// El cuerpo más cercano a `position`, por distancia a su superficie.
    pub fn nearest_body(&self, position: Vec3) -> Nearest {
        self.attractors()
            .map(|a| Nearest { name: a.name, position: a.position, radius: a.radius, surface_distance: position.distance(a.position) - a.radius })
            .min_by(|a, b| a.surface_distance.total_cmp(&b.surface_distance))
            .expect("siempre hay al menos el sol")
    }

    /// Aceleración gravitacional combinada de todos los cuerpos en `position`
    /// (inversa del cuadrado, con un piso de distancia y un techo de magnitud
    /// para que no dispare la nave al pasar muy cerca).
    pub fn gravity_accel(&self, position: Vec3) -> Vec3 {
        let mut accel = Vec3::ZERO;
        for a in self.attractors() {
            let delta = a.position - position;
            let dist_sq = delta.length_squared().max(config::GRAVITY_MIN_DIST_SQ);
            let strength = (config::GRAVITY_CONST * a.radius * a.radius / dist_sq).min(config::GRAVITY_MAX_ACCEL);
            accel = accel + delta.normalize() * strength;
        }
        accel
    }

    /// Colisión esférica contra todos los cuerpos: empuja la nave fuera de la
    /// superficie y anula la componente de velocidad que apuntaba hacia adentro.
    pub fn resolve_collision(&self, position: Vec3, velocity: Vec3) -> (Vec3, Vec3, Option<&'static str>) {
        let mut position = position;
        let mut velocity = velocity;
        let mut collided = None;
        for a in self.attractors() {
            let safe_radius = a.radius + config::SHIP_COLLISION_MARGIN;
            let delta = position - a.position;
            let dist = delta.length();
            if dist < safe_radius {
                let normal = if dist > 1e-5 { delta * (1.0 / dist) } else { Vec3::Y };
                position = a.position + normal * safe_radius;
                let inward = velocity.dot(normal);
                if inward < 0.0 {
                    velocity = velocity - normal * (inward * 1.25);
                }
                collided = Some(a.name);
            }
        }
        (position, velocity, collided)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn planets_orbit_the_sun_over_time() {
        let mut world = World::new();
        let start = world.planets[0].position;
        world.update(50.0);
        assert!(world.planets[0].position.distance(start) > 1.0);
    }

    #[test]
    fn moon_follows_earth() {
        let mut world = World::new();
        world.update(12.0);
        let earth = world.planets[bodies::EARTH_INDEX].position;
        assert!(world.moon_position.distance(earth) < bodies::MOON.orbit_radius + 0.5);
    }

    #[test]
    fn nearest_body_is_closer_than_the_rest() {
        let world = World::new();
        let earth = world.planets[bodies::EARTH_INDEX].position;
        let probe = earth + Vec3::new(0.0, 0.0, 10.0);
        let nearest = world.nearest_body(probe);
        assert_eq!(nearest.name, "Tierra");
    }

    #[test]
    fn collision_pushes_ship_out_of_the_body() {
        let world = World::new();
        let earth = world.planets[bodies::EARTH_INDEX].position;
        let inside = earth + Vec3::new(0.0, 0.0, 1.0);
        let (position, velocity, collided) = world.resolve_collision(inside, Vec3::new(0.0, 0.0, -5.0));
        assert_eq!(collided, Some("Tierra"));
        let earth_radius = bodies::PLANETS[bodies::EARTH_INDEX].radius;
        assert!(position.distance(earth) >= earth_radius + config::SHIP_COLLISION_MARGIN - 1e-3);
        assert!(velocity.dot(position - earth) >= 0.0);
    }

    #[test]
    fn gravity_pulls_towards_the_nearest_body() {
        let world = World::new();
        let earth = world.planets[bodies::EARTH_INDEX].position;
        let earth_radius = bodies::PLANETS[bodies::EARTH_INDEX].radius;
        // Cerca de la Tierra, su gravedad domina sobre la del sol u otros
        // planetas mucho más lejanos.
        let probe = earth + Vec3::new(0.0, 0.0, earth_radius + 2.0);
        let accel = world.gravity_accel(probe);
        assert!(accel.dot(Vec3::new(0.0, 0.0, -1.0)) > 0.0);
    }
}
