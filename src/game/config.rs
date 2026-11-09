//! Todos los números ajustables del juego viven aquí.

use crate::engine::math::Vec4;

pub const WIDTH: usize = 900;
pub const HEIGHT: usize = 620;

pub const FOV_DEGREES: f32 = 45.0;
pub const NEAR_PLANE: f32 = 0.1;
pub const FAR_PLANE: f32 = 100.0;

// --- LOD de la esfera (sol/planetas/luna): 3 mallas precalculadas, elegidas
// por distancia relativa al radio del cuerpo (ver engine::lod). ---
pub const SPHERE_STACKS_HIGH: u32 = 48;
pub const SPHERE_SLICES_HIGH: u32 = 64;
pub const SPHERE_STACKS_MEDIUM: u32 = 20;
pub const SPHERE_SLICES_MEDIUM: u32 = 28;
pub const SPHERE_STACKS_LOW: u32 = 8;
pub const SPHERE_SLICES_LOW: u32 = 12;
/// Umbral `distancia / radio` por encima del cual se baja a detalle medio/bajo.
pub const LOD_MEDIUM_RATIO: f32 = 9.0;
pub const LOD_LOW_RATIO: f32 = 22.0;

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
/// Si es `true`, invierte el cabeceo (por defecto, sin invertir: ↑ = nariz abajo).
pub const INVERT_PITCH: bool = false;

// --- Límite de velocidad por cercanía al cuerpo celeste más próximo ---
/// Debajo de `radio_del_cuerpo + este margen` la velocidad máxima se reduce
/// linealmente hasta `NAV_MIN_SPEED_NEAR`.
pub const NAV_SLOWDOWN_MARGIN: f32 = 10.0;
pub const NAV_MIN_SPEED_NEAR: f32 = 5.0;

// --- Sistema solar: gravedad y colisión (Fase 3) ---
/// Constante de gravedad propia (no física real, ajustada a la escala del juego).
pub const GRAVITY_CONST: f32 = 0.75;
/// Piso de distancia al cuadrado: evita que la gravedad se dispare muy cerca de la superficie.
pub const GRAVITY_MIN_DIST_SQ: f32 = 9.0;
/// Magnitud máxima de aceleración gravitacional de un solo cuerpo.
pub const GRAVITY_MAX_ACCEL: f32 = 3.0;
/// Margen sobre el radio del cuerpo para la colisión esférica de la nave.
pub const SHIP_COLLISION_MARGIN: f32 = 0.85;

// --- Modo órbita asistida (`O`) ---
pub const ORBIT_ASSIST_RESPONSE: f32 = 0.85;
pub const ORBIT_ASSIST_MIN_SPEED: f32 = 4.0;
pub const ORBIT_ASSIST_MAX_SPEED: f32 = 9.0;
pub const ORBIT_ASSIST_SPEED_PER_DISTANCE: f32 = 0.12;
pub const ORBIT_ASSIST_PREFERRED_MARGIN: f32 = 4.5;
pub const ORBIT_ASSIST_CORRECTION_GAIN: f32 = 0.9;
pub const ORBIT_ASSIST_MAX_CORRECTION: f32 = 2.0;

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

// --- Manual de controles (HUD mínimo, Fase 2) ---
pub const MANUAL_TEXT_SCALE: i32 = 2;
pub const MANUAL_LINE_SPACING: i32 = 2;
pub const MANUAL_MARGIN: i32 = 10;
pub const MANUAL_PADDING: i32 = 8;
pub const MANUAL_PANEL_COLOR: Vec4 = Vec4::new(0.03, 0.05, 0.09, 0.62);
pub const MANUAL_TEXT_COLOR: Vec4 = Vec4::new(0.75, 0.95, 1.0, 1.0);

// --- Deformación de malla del Planeta Fragmentado (Fase 5) ---
/// Nivel de subdivisión del icosaedro por detalle (alto/medio/bajo).
pub const FRACTURED_ICOSPHERE_SUBDIV_HIGH: u32 = 4;
pub const FRACTURED_ICOSPHERE_SUBDIV_MEDIUM: u32 = 3;
pub const FRACTURED_ICOSPHERE_SUBDIV_LOW: u32 = 1;
pub const FRACTURED_DEFORM_OCTAVES: u32 = 5;
pub const FRACTURED_DEFORM_FREQUENCY: f32 = 1.6;
/// Fracción del radio que puede desplazar el ruido (0.18 = ±18%).
pub const FRACTURED_DEFORM_AMPLITUDE: f32 = 0.18;

// --- Objetos de superficie sobre la Tierra (Fase 5) ---
/// Semilla de las 4 mallas propias (afecta solo el ruido de la roca).
pub const SURFACE_MODEL_SEED: u32 = 303;
/// Semilla de la distribución sobre la esfera: misma semilla, mismo mapa siempre.
pub const SURFACE_SCATTER_SEED: u32 = 2026;
pub const SURFACE_SCATTER_COUNT: usize = 36;
pub const SURFACE_SCALE_MIN: f32 = 0.5;
pub const SURFACE_SCALE_MAX: f32 = 0.85;
/// Margen sobre el radio del planeta para que la base del objeto no quede enterrada.
pub const SURFACE_OFFSET: f32 = 0.02;
/// Distancia a la Tierra por debajo de la cual se dibujan sus objetos de superficie.
pub const SURFACE_DRAW_DISTANCE: f32 = 40.0;

// --- Rendimiento (Fase 4) ---
/// Filas de framebuffer por banda de rasterización en paralelo (`rayon`).
/// Menos filas = más bandas = más paralelismo, con algo más de overhead por banda.
pub const RENDER_BAND_ROWS: usize = 48;

// --- Panel de rendimiento (FPS, triángulos, descarte; `F2`/`F3` los comparan) ---
pub const PERF_TEXT_SCALE: i32 = 2;
pub const PERF_LINE_SPACING: i32 = 2;
pub const PERF_MARGIN: i32 = 10;
pub const PERF_PADDING: i32 = 8;
pub const PERF_PANEL_COLOR: Vec4 = Vec4::new(0.03, 0.05, 0.09, 0.62);
pub const PERF_TEXT_COLOR: Vec4 = Vec4::new(0.8, 1.0, 0.8, 1.0);
