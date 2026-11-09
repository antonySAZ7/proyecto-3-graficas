//! Cámara perseguidora en tercera persona: sigue a la nave con suavizado
//! para que cabeceo/guiñada/alabeo se sientan sin saltos ni mareo.

use crate::engine::math::{Mat4, Vec3};
use crate::game::config;
use crate::game::physics;
use crate::game::ship::Ship;

pub struct ChaseCamera {
    pub position: Vec3,
    pub look_at: Vec3,
    pub up: Vec3,
}

impl ChaseCamera {
    /// Arranca ya ubicada detrás de la nave, sin el salto del primer suavizado.
    pub fn new(ship: &Ship) -> Self {
        let mut cam = Self { position: ship.position, look_at: ship.position, up: Vec3::Y };
        cam.snap_to(ship);
        cam
    }

    fn desired(&self, ship: &Ship) -> (Vec3, Vec3) {
        let position = ship.position - ship.forward() * config::CAMERA_BACK_DISTANCE
            + ship.up() * config::CAMERA_HEIGHT;
        let look_at = ship.position + ship.forward() * config::CAMERA_LOOK_AHEAD;
        (position, look_at)
    }

    fn snap_to(&mut self, ship: &Ship) {
        let (position, look_at) = self.desired(ship);
        self.position = position;
        self.look_at = look_at;
        self.up = ship.up();
    }

    pub fn update(&mut self, ship: &Ship, dt: f32) {
        let (position, look_at) = self.desired(ship);
        self.position = physics::smooth_towards_vec3(self.position, position, config::CAMERA_SMOOTHING, dt);
        self.look_at = physics::smooth_towards_vec3(self.look_at, look_at, config::CAMERA_SMOOTHING, dt);
        self.up = physics::smooth_towards_vec3(self.up, ship.up(), config::CAMERA_UP_SMOOTHING, dt).normalize();
    }

    pub fn view(&self) -> Mat4 {
        Mat4::look_at_rh(self.position, self.look_at, self.up)
    }

    /// Solo la rotación de la vista (sin traslación): para el skybox de estrellas,
    /// que debe girar con la cámara pero no desplazarse con ella.
    pub fn view_rotation_only(&self) -> Mat4 {
        let mut m = self.view();
        m.m[12] = 0.0;
        m.m[13] = 0.0;
        m.m[14] = 0.0;
        m
    }
}
