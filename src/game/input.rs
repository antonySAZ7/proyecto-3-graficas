//! Lee el teclado de `minifb` y lo traduce a ejes y acciones del juego.
//! Es la única parte que conoce los nombres de las teclas.

use minifb::{Key, KeyRepeat, Window};

#[derive(Clone, Copy, Default)]
pub struct InputState {
    /// -1 (izquierda) .. 1 (derecha)
    pub yaw: f32,
    /// -1 (abajo) .. 1 (arriba)
    pub pitch: f32,
    /// -1 (izquierda) .. 1 (derecha)
    pub roll: f32,
    /// -1 (freno/reversa) .. 1 (acelerar)
    pub throttle: f32,
    pub boost: bool,
    /// Se activó/desactivó la velocidad de crucero en este frame.
    pub toggle_cruise: bool,
    /// Se activó/desactivó la estabilización asistida en este frame.
    pub toggle_stabilize: bool,
}

fn axis(window: &Window, positive: Key, negative: Key) -> f32 {
    let p = if window.is_key_down(positive) { 1.0 } else { 0.0 };
    let n = if window.is_key_down(negative) { 1.0 } else { 0.0 };
    p - n
}

pub fn read(window: &Window) -> InputState {
    InputState {
        yaw: axis(window, Key::D, Key::A),
        pitch: axis(window, Key::Up, Key::Down),
        roll: axis(window, Key::E, Key::Q),
        throttle: axis(window, Key::W, Key::S),
        boost: window.is_key_down(Key::LeftShift) || window.is_key_down(Key::RightShift),
        toggle_cruise: window.is_key_pressed(Key::C, KeyRepeat::No),
        toggle_stabilize: window.is_key_pressed(Key::Z, KeyRepeat::No),
    }
}
