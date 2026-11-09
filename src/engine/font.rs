//! Fuente bitmap 5x7 propia (sin archivos externos): A-Z, 0-9, algunos signos
//! y flechas de dirección. Pensada para reutilizarse en el HUD completo (Fase 7).

use crate::engine::framebuffer::Framebuffer;
use crate::engine::math::Vec4;

pub const GLYPH_WIDTH: usize = 5;
pub const GLYPH_HEIGHT: usize = 7;

/// `blend_pixel` no hace prueba de profundidad (eso lo decide quien la llama);
/// como el HUD se dibuja último, basta con que el valor guardado quede fuera
/// del rango real [-1, 1] para no confundirse con geometría de la escena.
const OVERLAY_DEPTH: f32 = -2.0;

/// Cada fila son 5 bits (bit 4 = columna izquierda .. bit 0 = columna derecha).
/// Caracteres no soportados se dibujan en blanco (igual que el espacio).
fn glyph(ch: char) -> [u8; GLYPH_HEIGHT] {
    match ch {
        'A' => [0b01110, 0b10001, 0b10001, 0b11111, 0b10001, 0b10001, 0b10001],
        'B' => [0b11110, 0b10001, 0b10001, 0b11110, 0b10001, 0b10001, 0b11110],
        'C' => [0b01111, 0b10000, 0b10000, 0b10000, 0b10000, 0b10000, 0b01111],
        'D' => [0b11110, 0b10001, 0b10001, 0b10001, 0b10001, 0b10001, 0b11110],
        'E' => [0b11111, 0b10000, 0b10000, 0b11110, 0b10000, 0b10000, 0b11111],
        'F' => [0b11111, 0b10000, 0b10000, 0b11110, 0b10000, 0b10000, 0b10000],
        'G' => [0b01111, 0b10000, 0b10000, 0b10011, 0b10001, 0b10001, 0b01111],
        'H' => [0b10001, 0b10001, 0b10001, 0b11111, 0b10001, 0b10001, 0b10001],
        'I' => [0b01110, 0b00100, 0b00100, 0b00100, 0b00100, 0b00100, 0b01110],
        'J' => [0b00111, 0b00010, 0b00010, 0b00010, 0b00010, 0b10010, 0b01100],
        'K' => [0b10001, 0b10010, 0b10100, 0b11000, 0b10100, 0b10010, 0b10001],
        'L' => [0b10000, 0b10000, 0b10000, 0b10000, 0b10000, 0b10000, 0b11111],
        'M' => [0b10001, 0b11011, 0b10101, 0b10101, 0b10001, 0b10001, 0b10001],
        'N' => [0b10001, 0b11001, 0b10101, 0b10101, 0b10011, 0b10001, 0b10001],
        'O' => [0b01110, 0b10001, 0b10001, 0b10001, 0b10001, 0b10001, 0b01110],
        'P' => [0b11110, 0b10001, 0b10001, 0b11110, 0b10000, 0b10000, 0b10000],
        'Q' => [0b01110, 0b10001, 0b10001, 0b10001, 0b10101, 0b10010, 0b01101],
        'R' => [0b11110, 0b10001, 0b10001, 0b11110, 0b10100, 0b10010, 0b10001],
        'S' => [0b01111, 0b10000, 0b10000, 0b01110, 0b00001, 0b00001, 0b11110],
        'T' => [0b11111, 0b00100, 0b00100, 0b00100, 0b00100, 0b00100, 0b00100],
        'U' => [0b10001, 0b10001, 0b10001, 0b10001, 0b10001, 0b10001, 0b01110],
        'V' => [0b10001, 0b10001, 0b10001, 0b10001, 0b10001, 0b01010, 0b00100],
        'W' => [0b10001, 0b10001, 0b10001, 0b10101, 0b10101, 0b10101, 0b01010],
        'X' => [0b10001, 0b10001, 0b01010, 0b00100, 0b01010, 0b10001, 0b10001],
        'Y' => [0b10001, 0b10001, 0b01010, 0b00100, 0b00100, 0b00100, 0b00100],
        'Z' => [0b11111, 0b00001, 0b00010, 0b00100, 0b01000, 0b10000, 0b11111],

        '0' => [0b01110, 0b10001, 0b10011, 0b10101, 0b11001, 0b10001, 0b01110],
        '1' => [0b00100, 0b01100, 0b00100, 0b00100, 0b00100, 0b00100, 0b01110],
        '2' => [0b01110, 0b10001, 0b00001, 0b00010, 0b00100, 0b01000, 0b11111],
        '3' => [0b11111, 0b00010, 0b00100, 0b00010, 0b00001, 0b10001, 0b01110],
        '4' => [0b00010, 0b00110, 0b01010, 0b10010, 0b11111, 0b00010, 0b00010],
        '5' => [0b11111, 0b10000, 0b11110, 0b00001, 0b00001, 0b10001, 0b01110],
        '6' => [0b00110, 0b01000, 0b10000, 0b11110, 0b10001, 0b10001, 0b01110],
        '7' => [0b11111, 0b00001, 0b00010, 0b00100, 0b01000, 0b01000, 0b01000],
        '8' => [0b01110, 0b10001, 0b10001, 0b01110, 0b10001, 0b10001, 0b01110],
        '9' => [0b01110, 0b10001, 0b10001, 0b01111, 0b00001, 0b00010, 0b01100],

        ':' => [0b00000, 0b00100, 0b00000, 0b00000, 0b00000, 0b00100, 0b00000],
        '/' => [0b00001, 0b00010, 0b00100, 0b00100, 0b01000, 0b10000, 0b00000],
        '.' => [0b00000, 0b00000, 0b00000, 0b00000, 0b00000, 0b01100, 0b01100],
        '-' => [0b00000, 0b00000, 0b00000, 0b11111, 0b00000, 0b00000, 0b00000],

        // Flechas (triángulo relleno apuntando en cada dirección).
        '\u{2191}' => [0b00100, 0b01110, 0b11111, 0b00100, 0b00100, 0b00100, 0b00100], // ↑
        '\u{2193}' => [0b00100, 0b00100, 0b00100, 0b00100, 0b11111, 0b01110, 0b00100], // ↓
        '\u{2190}' => [0b00011, 0b00111, 0b01111, 0b11111, 0b01111, 0b00111, 0b00011], // ←
        '\u{2192}' => [0b11000, 0b11100, 0b11110, 0b11111, 0b11110, 0b11100, 0b11000], // →

        _ => [0; GLYPH_HEIGHT], // espacio y no soportados: en blanco
    }
}

/// Dibuja `text` (se convierte a mayúsculas) con esquina superior izquierda en
/// (x, y), `scale` píxeles de framebuffer por píxel de glifo, encima de todo
/// lo ya dibujado (ver `OVERLAY_DEPTH`).
pub fn draw_text(fb: &mut Framebuffer, x: i32, y: i32, text: &str, scale: i32, color: Vec4) {
    let mut cursor_x = x;
    for ch in text.chars() {
        let rows = glyph(ch.to_ascii_uppercase());
        for (row, bits) in rows.iter().enumerate() {
            for col in 0..GLYPH_WIDTH {
                if (bits >> (GLYPH_WIDTH - 1 - col)) & 1 == 0 {
                    continue;
                }
                let px0 = cursor_x + col as i32 * scale;
                let py0 = y + row as i32 * scale;
                for sy in 0..scale {
                    for sx in 0..scale {
                        let px = px0 + sx;
                        let py = py0 + sy;
                        if px < 0 || py < 0 || px as usize >= fb.width || py as usize >= fb.height {
                            continue;
                        }
                        fb.blend_pixel(px as usize, py as usize, OVERLAY_DEPTH, color);
                    }
                }
            }
        }
        cursor_x += (GLYPH_WIDTH as i32 + 1) * scale;
    }
}

/// Ancho en píxeles que ocupará `text` dibujado con `draw_text` a esa escala.
pub fn text_width(text: &str, scale: i32) -> i32 {
    text.chars().count() as i32 * (GLYPH_WIDTH as i32 + 1) * scale
}

/// Rectángulo relleno (panel semitransparente del HUD), misma mezcla alfa que el framebuffer.
pub fn draw_panel(fb: &mut Framebuffer, x: i32, y: i32, w: i32, h: i32, color: Vec4) {
    let x0 = x.max(0);
    let y0 = y.max(0);
    let x1 = (x + w).min(fb.width as i32);
    let y1 = (y + h).min(fb.height as i32);
    for py in y0..y1 {
        for px in x0..x1 {
            fb.blend_pixel(px as usize, py as usize, OVERLAY_DEPTH, color);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unsupported_char_is_blank_not_a_panic() {
        let mut fb = Framebuffer::new(16, 16);
        draw_text(&mut fb, 0, 0, "¡?", 1, Vec4::new(1.0, 1.0, 1.0, 1.0));
    }

    #[test]
    fn text_width_scales_with_length_and_scale() {
        assert_eq!(text_width("AB", 2), text_width("A", 2) * 2);
        assert_eq!(text_width("A", 3), text_width("A", 1) * 3);
    }

    #[test]
    fn draw_text_stays_inside_framebuffer_bounds() {
        let mut fb = Framebuffer::new(8, 8);
        // A propósito se sale del borde; no debe entrar en pánico por índice fuera de rango.
        draw_text(&mut fb, 5, 5, "W", 4, Vec4::new(1.0, 1.0, 1.0, 1.0));
    }
}
