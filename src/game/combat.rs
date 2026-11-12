//! Disparo de la nave: proyectiles tipo rayo contra Galactus, la amenaza
//! colosal (ver `bodies::GALACTUS` y `world::World::register_galactus_hit`).

use crate::engine::math::Vec3;
use crate::game::bodies;
use crate::game::config;
use crate::game::ship::Ship;
use crate::game::world::World;

pub struct Laser {
    pub position: Vec3,
    pub direction: Vec3,
    pub age: f32,
}

/// Efecto breve que persiste después de retirar el proyectil que impactó.
pub struct ImpactFx {
    pub position: Vec3,
    pub age: f32,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum CombatEvent {
    Fired,
    Hit,
    Retreated,
}

/// Cola de proyectiles del jugador. `scratch` se reutiliza entre frames
/// igual que `RenderQueue::clear`: ninguna asignación nueva en el camino
/// caliente, salvo al disparar (crece el `Vec` principal si hace falta).
pub struct Combat {
    pub lasers: Vec<Laser>,
    pub impacts: Vec<ImpactFx>,
    scratch: Vec<Laser>,
    cooldown: f32,
}

impl Combat {
    pub fn new() -> Self {
        Self { lasers: Vec::new(), impacts: Vec::new(), scratch: Vec::new(), cooldown: 0.0 }
    }

    /// Avanza los proyectiles existentes, dispara uno nuevo si corresponde y
    /// resuelve impactos contra Galactus. Devuelve los eventos del frame para
    /// que quien llama (el bucle principal) decida qué mensaje mostrar.
    pub fn update(&mut self, firing: bool, ship: &Ship, world: &mut World, dt: f32) -> Vec<CombatEvent> {
        let mut events = Vec::new();
        if config::COMBATE_LEGIBLE {
            for impact in &mut self.impacts {
                impact.age += dt;
            }
            self.impacts.retain(|impact| impact.age < config::IMPACT_FLASH_DURATION);
        }
        self.cooldown = (self.cooldown - dt).max(0.0);
        if firing && self.cooldown <= 0.0 {
            self.cooldown = config::LASER_COOLDOWN;
            self.lasers.push(Laser {
                position: ship.position + ship.forward() * config::LASER_SPAWN_OFFSET,
                direction: ship.forward(),
                age: 0.0,
            });
            events.push(CombatEvent::Fired);
        }

        let visual_radius = if config::GALACTUS_DETALLADO {
            bodies::GALACTUS.radius * config::GALACTUS_DETAILED_BOUND_SCALE
        } else {
            bodies::GALACTUS.radius
        };
        let hit_radius = visual_radius + config::LASER_HIT_RADIUS_MARGIN;
        self.scratch.clear();
        for mut laser in self.lasers.drain(..) {
            laser.position = laser.position + laser.direction * (config::LASER_SPEED * dt);
            laser.age += dt;
            if laser.position.distance(world.galactus_position) < hit_radius {
                if config::COMBATE_LEGIBLE {
                    self.impacts.push(ImpactFx { position: laser.position, age: 0.0 });
                }
                events.push(CombatEvent::Hit);
                if world.register_galactus_hit() {
                    events.push(CombatEvent::Retreated);
                }
                continue;
            }
            if laser.age < config::LASER_LIFETIME {
                self.scratch.push(laser);
            }
        }
        std::mem::swap(&mut self.lasers, &mut self.scratch);
        events
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn firing_spawns_a_laser_and_respects_cooldown() {
        let mut world = World::new();
        let ship = Ship::new(Vec3::new(0.0, 0.0, 400.0));
        let mut combat = Combat::new();

        let events = combat.update(true, &ship, &mut world, 1.0 / 60.0);
        assert_eq!(combat.lasers.len(), 1);
        assert!(events.contains(&CombatEvent::Fired));

        let events2 = combat.update(true, &ship, &mut world, 1.0 / 60.0);
        assert_eq!(combat.lasers.len(), 1, "el enfriamiento debe impedir un segundo disparo tan pronto");
        assert!(!events2.contains(&CombatEvent::Fired));
    }

    #[test]
    fn laser_hitting_galactus_registers_a_hit() {
        let mut world = World::new();
        let direction = Vec3::new(0.3, 0.1, -0.9).normalize();
        let ship_pos = world.galactus_position - direction * (bodies::GALACTUS.radius + 5.0);
        let ship = Ship::new(ship_pos);
        let mut combat = Combat::new();
        combat.lasers.push(Laser { position: ship_pos, direction, age: 0.0 });

        let before_hits = world.galactus_hits;
        let mut hit = false;
        for _ in 0..50 {
            let events = combat.update(false, &ship, &mut world, 1.0 / 60.0);
            if events.contains(&CombatEvent::Hit) {
                hit = true;
                break;
            }
        }
        assert!(hit, "el rayo apuntado directo a Galactus debe impactar");
        assert_eq!(world.galactus_hits, before_hits + 1);
        assert_eq!(combat.impacts.len(), 1);
    }

    #[test]
    fn laser_expires_after_its_lifetime_without_hitting_anything() {
        let mut world = World::new();
        // Dirección que se aleja de Galactus y de la Tierra: nunca impacta.
        let direction = (world.galactus_position - world.planets[bodies::EARTH_INDEX].position).normalize() * -1.0;
        let ship = Ship::new(Vec3::ZERO);
        let mut combat = Combat::new();
        combat.lasers.push(Laser { position: Vec3::ZERO, direction, age: 0.0 });

        let steps = (config::LASER_LIFETIME / (1.0 / 60.0)).ceil() as u32 + 2;
        for _ in 0..steps {
            combat.update(false, &ship, &mut world, 1.0 / 60.0);
        }
        assert!(combat.lasers.is_empty());
    }
}
