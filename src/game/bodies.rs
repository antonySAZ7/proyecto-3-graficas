//! Cuerpos celestes del sistema solar, definidos como datos: agregar un
//! planeta es agregar una entrada aquí, sin tocar lógica (ver CLAUDE.md).

/// Qué shader de superficie usa cada planeta. El sol y la luna no están aquí
/// porque no comparten su tipo con ningún planeta.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ShaderKind {
    Earth,
    Lava,
    Ice,
    Rocky,
    Metal,
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
    /// Si es `true`, su malla no es la esfera UV compartida sino un icosaedro
    /// subdividido y deformado con ruido multioctava (ver `main::build_fractured_lod`).
    pub fractured: bool,
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
        shader: ShaderKind::Earth,
        seed: 11,
        fractured: false,
    },
    BodyDef {
        name: "Mundo Forja",
        radius: 4.2,
        orbit_radius: 24.0,
        orbit_speed: 0.05,
        phase: 0.65,
        inclination: 0.09,
        rotation_speed: -0.7,
        shader: ShaderKind::Lava,
        seed: 23,
        fractured: false,
    },
    BodyDef {
        name: "Niflheim",
        radius: 5.0,
        orbit_radius: 58.0,
        orbit_speed: 0.018,
        phase: 2.7,
        inclination: 0.12,
        rotation_speed: 0.35,
        shader: ShaderKind::Ice,
        seed: 37,
        fractured: false,
    },
    BodyDef {
        name: "Planeta Fragmentado",
        radius: 5.4,
        orbit_radius: 74.0,
        orbit_speed: 0.013,
        phase: 4.35,
        inclination: 0.2,
        rotation_speed: 0.4,
        shader: ShaderKind::Rocky,
        seed: 53,
        fractured: true,
    },
    BodyDef {
        name: "Mundo de Plata",
        radius: 4.6,
        orbit_radius: 90.0,
        orbit_speed: 0.01,
        phase: 1.72,
        inclination: 0.07,
        rotation_speed: -0.28,
        shader: ShaderKind::Metal,
        seed: 61,
        fractured: false,
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

/// Galactus: la amenaza colosal de la Fase 7. No orbita el sol; persigue a
/// la Tierra lentamente y se repliega a un nuevo punto lejano tras recibir
/// suficientes impactos (ver `world::World::update_galactus`).
pub struct GalactusDef {
    pub name: &'static str,
    pub radius: f32,
    pub seed: u32,
    pub rotation_speed: f32,
    /// Velocidad de acercamiento a la Tierra, unidades/s.
    pub approach_speed: f32,
    /// Distancia a la Tierra a la que reaparece tras replegarse.
    pub spawn_distance: f32,
    /// Distancia a la Tierra en la que deja de acercarse (y queda como amenaza latente).
    pub retreat_distance: f32,
    /// Impactos de la nave necesarios para que se repliegue.
    pub hits_to_retreat: u32,
}

pub const GALACTUS: GalactusDef = GalactusDef {
    name: "Galactus",
    radius: 10.0,
    seed: 999,
    rotation_speed: 0.06,
    approach_speed: 1.6,
    spawn_distance: 70.0,
    retreat_distance: 24.0,
    hits_to_retreat: 6,
};
