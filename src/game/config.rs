//! Todos los números ajustables del juego viven aquí.

use crate::engine::math::Vec3;

pub const WIDTH: usize = 900;
pub const HEIGHT: usize = 620;

pub const FOV_DEGREES: f32 = 45.0;
pub const NEAR_PLANE: f32 = 0.1;
pub const FAR_PLANE: f32 = 100.0;

pub const SPHERE_STACKS: u32 = 48;
pub const SPHERE_SLICES: u32 = 64;

pub const RING_INNER_RADIUS: f32 = 1.7;
pub const RING_OUTER_RADIUS: f32 = 2.7;
pub const RING_SEGMENTS: u32 = 128;

pub const GAS_GIANT_RADIUS: f32 = 1.2;

pub const BACKGROUND_COLOR: u32 = 0x05070d;

// --- Nave: giro y aceleración (Fase 2) ---
/// Velocidad angular máxima por eje (rad/s) a entrada completa.
pub const SHIP_PITCH_RATE: f32 = 1.0;
pub const SHIP_YAW_RATE: f32 = 0.95;
pub const SHIP_ROLL_RATE: f32 = 1.3;
/// Qué tan rápido la velocidad angular alcanza su objetivo (mayor = más ágil).
pub const SHIP_TURN_RESPONSE: f32 = 5.2;
/// Aceleración lineal normal y con impulso (unidades/s^2).
pub const SHIP_ACCEL: f32 = 9.0;
pub const SHIP_BOOST_ACCEL: f32 = 18.0;
/// Fricción exponencial que frena la nave cuando no se acelera.
pub const SHIP_DRAG: f32 = 0.5;
/// Cuánto ajusta la velocidad de crucero por segundo mientras se mantiene W/S.
pub const SHIP_CRUISE_ADJUST_RATE: f32 = 6.0;
pub const SHIP_CRUISE_RESPONSE: f32 = 3.0;
pub const SHIP_MAX_SPEED: f32 = 22.0;

// --- Límite de velocidad por cercanía (punto de referencia temporal; en Fase 3 será el planeta más cercano) ---
pub const NAV_TARGET_POINT: Vec3 = Vec3::new(0.0, 0.0, 0.0);
/// Debajo de esta distancia la velocidad máxima se reduce linealmente hasta `NAV_MIN_SPEED_NEAR`.
pub const NAV_SLOWDOWN_RADIUS: f32 = 14.0;
pub const NAV_MIN_SPEED_NEAR: f32 = 5.0;

// --- Cámara perseguidora ---
pub const CAMERA_BACK_DISTANCE: f32 = 8.2;
pub const CAMERA_HEIGHT: f32 = 2.4;
pub const CAMERA_LOOK_AHEAD: f32 = 14.0;
/// Suavizado de posición/orientación de la cámara (mayor = alcanza antes al objetivo).
pub const CAMERA_SMOOTHING: f32 = 4.5;
pub const CAMERA_UP_SMOOTHING: f32 = 7.0;

// --- Skybox de estrellas ---
pub const STARFIELD_SEED: u32 = 20260;
pub const STARFIELD_COUNT: usize = 420;
pub const STARFIELD_RADIUS: f32 = 90.0;
