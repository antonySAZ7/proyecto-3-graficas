//! Cuerpos celestes del sistema solar, definidos como datos: agregar un
//! planeta es agregar una entrada aquí, sin tocar lógica (ver CLAUDE.md).

/// Qué shader de superficie usa cada planeta. El sol y la luna no están aquí
/// porque no comparten su tipo con ningún planeta.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ShaderKind {
    Rocky,
    GasGiant,
    Moon,
}

pub struct BodyDef {
    pub name: &'static str,
    pub radius: f32,
    pub orbit_radius: f32,
    /// Velocidad angular de la órbita, rad/s.
    pub orbit_speed: f32,
    /// Ángulo inicial en la órbita, rad.
    pub phase: f32,
    /// Inclinación del plano orbital respecto al del sol, rad.
    pub inclination: f32,
    /// Velocidad de rotación propia, rad/s.
    pub rotation_speed: f32,
    pub shader: ShaderKind,
    pub seed: u32,
}

pub struct MoonDef {
    pub name: &'static str,
    pub radius: f32,
    pub orbit_radius: f32,
    pub orbit_speed: f32,
    pub phase: f32,
    pub rotation_speed: f32,
    pub seed: u32,
}

pub const SUN_NAME: &str = "Sol";
pub const SUN_RADIUS: f32 = 9.0;
pub const SUN_SEED: u32 = 1;

pub const EARTH_INDEX: usize = 0;

pub const PLANETS: &[BodyDef] = &[
    BodyDef {
        name: "Tierra",
        radius: 6.0,
        orbit_radius: 42.0,
        orbit_speed: 0.028,
        phase: -std::f32::consts::FRAC_PI_2,
        inclination: 0.05,
        rotation_speed: 0.5,
        shader: ShaderKind::Rocky,
        seed: 11,
    },
    BodyDef {
        name: "Mundo Forja",
        radius: 4.2,
        orbit_radius: 24.0,
        orbit_speed: 0.05,
        phase: 0.65,
        inclination: 0.09,
        rotation_speed: -0.7,
        shader: ShaderKind::GasGiant,
        seed: 23,
    },
    BodyDef {
        name: "Niflheim",
        radius: 5.0,
        orbit_radius: 58.0,
        orbit_speed: 0.018,
        phase: 2.7,
        inclination: 0.12,
        rotation_speed: 0.35,
        shader: ShaderKind::Rocky,
        seed: 37,
    },
    BodyDef {
        name: "Planeta Fragmentado",
        radius: 5.4,
        orbit_radius: 74.0,
        orbit_speed: 0.013,
        phase: 4.35,
        inclination: 0.2,
        rotation_speed: 0.4,
        shader: ShaderKind::GasGiant,
        seed: 53,
    },
    BodyDef {
        name: "Mundo de Plata",
        radius: 4.6,
        orbit_radius: 90.0,
        orbit_speed: 0.01,
        phase: 1.72,
        inclination: 0.07,
        rotation_speed: -0.28,
        shader: ShaderKind::Moon,
        seed: 61,
    },
];

pub const MOON: MoonDef = MoonDef {
    name: "Luna",
    radius: 1.45,
    orbit_radius: 9.0,
    orbit_speed: 0.12,
    phase: 0.25,
    rotation_speed: 0.2,
    seed: 71,
};
