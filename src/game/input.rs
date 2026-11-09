//! Única fuente de verdad entre teclas y acciones del juego. La lógica de
//! vuelo (`ship.rs`) y el bucle principal solo conocen `Action`, nunca
//! `minifb::Key` directamente: remapear una tecla es cambiar `BINDINGS`.

use minifb::{Key, KeyRepeat, Window};

use crate::game::config;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Action {
    PitchUp,
    PitchDown,
    YawLeft,
    YawRight,
    RollLeft,
    RollRight,
    Accelerate,
    Brake,
    Boost,
    Cruise,
    StabilizeAssist,
    Reset,
    Pause,
    ToggleManual,
    Quit,
    /// Reservadas para fases futuras: aparecen en el manual pero sin lógica todavía.
    OrbitAssist,
    Warp,
    Fire,
}

pub struct Binding {
    pub action: Action,
    pub key: Key,
    /// Texto corto para el manual en pantalla (solo A-Z/0-9, ver `engine::font`).
    pub label: &'static str,
    /// `false` = reservada para una fase futura, sin lógica implementada aún.
    pub implemented: bool,
}

/// Tabla única: acción -> tecla. Cambiar una tecla es editar esta lista.
pub const BINDINGS: &[Binding] = &[
    Binding { action: Action::PitchUp, key: Key::Up, label: "\u{2191}", implemented: true },
    Binding { action: Action::PitchDown, key: Key::Down, label: "\u{2193}", implemented: true },
    Binding { action: Action::YawLeft, key: Key::Left, label: "\u{2190}", implemented: true },
    Binding { action: Action::YawRight, key: Key::Right, label: "\u{2192}", implemented: true },
    Binding { action: Action::RollLeft, key: Key::A, label: "A", implemented: true },
    Binding { action: Action::RollRight, key: Key::D, label: "D", implemented: true },
    Binding { action: Action::Accelerate, key: Key::W, label: "W", implemented: true },
    Binding { action: Action::Brake, key: Key::S, label: "S", implemented: true },
    Binding { action: Action::Boost, key: Key::LeftShift, label: "SHIFT", implemented: true },
    Binding { action: Action::Cruise, key: Key::C, label: "C", implemented: true },
    Binding { action: Action::StabilizeAssist, key: Key::Z, label: "Z", implemented: true },
    Binding { action: Action::OrbitAssist, key: Key::O, label: "O", implemented: false },
    Binding { action: Action::Warp, key: Key::H, label: "H", implemented: false },
    Binding { action: Action::Fire, key: Key::Space, label: "ESPACIO", implemented: false },
    Binding { action: Action::Reset, key: Key::R, label: "R", implemented: true },
    Binding { action: Action::Pause, key: Key::P, label: "P", implemented: true },
    Binding { action: Action::ToggleManual, key: Key::F1, label: "F1", implemented: true },
    Binding { action: Action::Quit, key: Key::Escape, label: "ESC", implemented: true },
];

/// Una línea del manual: una descripción y las acciones cuyas teclas se listan juntas.
pub struct ManualLine {
    pub label: &'static str,
    pub actions: &'static [Action],
}

pub const MANUAL: &[ManualLine] = &[
    ManualLine { label: "CABECEO", actions: &[Action::PitchUp, Action::PitchDown] },
    ManualLine { label: "GIRO", actions: &[Action::YawLeft, Action::YawRight] },
    ManualLine { label: "ALABEO", actions: &[Action::RollLeft, Action::RollRight] },
    ManualLine { label: "ACELERAR/FRENAR", actions: &[Action::Accelerate, Action::Brake] },
    ManualLine { label: "TURBO", actions: &[Action::Boost] },
    ManualLine { label: "CRUCERO", actions: &[Action::Cruise] },
    ManualLine { label: "ESTABILIZAR", actions: &[Action::StabilizeAssist] },
    ManualLine { label: "ORBITA", actions: &[Action::OrbitAssist] },
    ManualLine { label: "WARP", actions: &[Action::Warp] },
    ManualLine { label: "DISPARO", actions: &[Action::Fire] },
    ManualLine { label: "REINICIAR", actions: &[Action::Reset] },
    ManualLine { label: "PAUSA", actions: &[Action::Pause] },
    ManualLine { label: "MANUAL", actions: &[Action::ToggleManual] },
    ManualLine { label: "SALIR", actions: &[Action::Quit] },
];

fn binding_for(action: Action) -> &'static Binding {
    BINDINGS.iter().find(|b| b.action == action).expect("acción sin tecla asignada en BINDINGS")
}

pub fn key_for(action: Action) -> Key {
    binding_for(action).key
}

pub fn label_for(action: Action) -> &'static str {
    binding_for(action).label
}

/// `false` si la acción está reservada para una fase futura (sin lógica todavía).
pub fn is_implemented(action: Action) -> bool {
    binding_for(action).implemented
}

fn is_down(window: &Window, action: Action) -> bool {
    window.is_key_down(key_for(action))
}

fn is_pressed(window: &Window, action: Action) -> bool {
    window.is_key_pressed(key_for(action), KeyRepeat::No)
}

#[derive(Clone, Copy, Default)]
pub struct InputState {
    /// -1 (nariz abajo) .. 1 (nariz arriba), ya con la inversión de `config::INVERT_PITCH` aplicada.
    pub pitch: f32,
    /// -1 (derecha) .. 1 (izquierda)
    pub yaw: f32,
    /// -1 (derecha) .. 1 (izquierda)
    pub roll: f32,
    /// -1 (freno/reversa) .. 1 (acelerar)
    pub throttle: f32,
    pub boost: bool,
    pub toggle_cruise: bool,
    pub toggle_stabilize: bool,
    pub toggle_manual: bool,
    pub pause: bool,
    pub reset: bool,
}

pub fn read(window: &Window) -> InputState {
    let up = is_down(window, Action::PitchUp);
    let down = is_down(window, Action::PitchDown);
    // Por defecto (sin invertir) ↑ inclina la nariz hacia abajo.
    let mut pitch = (down as i32 - up as i32) as f32;
    if config::INVERT_PITCH {
        pitch = -pitch;
    }

    InputState {
        pitch,
        yaw: (is_down(window, Action::YawLeft) as i32 - is_down(window, Action::YawRight) as i32) as f32,
        roll: (is_down(window, Action::RollLeft) as i32 - is_down(window, Action::RollRight) as i32) as f32,
        throttle: (is_down(window, Action::Accelerate) as i32 - is_down(window, Action::Brake) as i32) as f32,
        boost: is_down(window, Action::Boost),
        toggle_cruise: is_pressed(window, Action::Cruise),
        toggle_stabilize: is_pressed(window, Action::StabilizeAssist),
        toggle_manual: is_pressed(window, Action::ToggleManual),
        pause: is_pressed(window, Action::Pause),
        reset: is_pressed(window, Action::Reset),
    }
}

pub fn quit_requested(window: &Window) -> bool {
    is_down(window, Action::Quit)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_action_has_exactly_one_binding() {
        use Action::*;
        let all = [
            PitchUp, PitchDown, YawLeft, YawRight, RollLeft, RollRight, Accelerate, Brake, Boost, Cruise,
            StabilizeAssist, Reset, Pause, ToggleManual, Quit, OrbitAssist, Warp, Fire,
        ];
        for action in all {
            let count = BINDINGS.iter().filter(|b| b.action == action).count();
            assert_eq!(count, 1, "{action:?} debe tener exactamente un binding");
        }
    }

    #[test]
    fn manual_only_references_bound_actions() {
        for line in MANUAL {
            for &action in line.actions {
                // No debe entrar en pánico: la acción existe en BINDINGS.
                let _ = key_for(action);
            }
        }
    }
}
