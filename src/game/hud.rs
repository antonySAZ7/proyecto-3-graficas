//! HUD mínimo de la Fase 2: el manual de controles. Se genera a partir de
//! `game::input::MANUAL`/`BINDINGS`, así que nunca queda desincronizado de
//! las teclas reales (cambiar una tecla es solo editar `input::BINDINGS`).

use crate::engine::font;
use crate::engine::framebuffer::Framebuffer;
use crate::game::config;
use crate::game::input::{self, MANUAL};

fn line_text(line: &input::ManualLine) -> String {
    let keys: Vec<&str> = line.actions.iter().map(|&a| input::label_for(a)).collect();
    let pending = line.actions.iter().any(|&a| !input::is_implemented(a));
    let base = format!("{} {}", keys.join("/"), line.label);
    if pending {
        format!("{base} PRONTO")
    } else {
        base
    }
}

/// Panel anclado a la esquina superior derecha, ancho según el framebuffer.
pub fn draw_controls_manual(fb: &mut Framebuffer) {
    let lines: Vec<String> = MANUAL.iter().map(line_text).collect();

    let line_height = font::GLYPH_HEIGHT as i32 * config::MANUAL_TEXT_SCALE + config::MANUAL_LINE_SPACING;
    let widest = lines.iter().map(|l| font::text_width(l, config::MANUAL_TEXT_SCALE)).max().unwrap_or(0);

    let panel_w = widest + config::MANUAL_PADDING * 2;
    let panel_h = line_height * lines.len() as i32 + config::MANUAL_PADDING * 2;
    let panel_x = fb.width as i32 - panel_w - config::MANUAL_MARGIN;
    let panel_y = config::MANUAL_MARGIN;

    font::draw_panel(fb, panel_x, panel_y, panel_w, panel_h, config::MANUAL_PANEL_COLOR);

    let text_x = panel_x + config::MANUAL_PADDING;
    let mut text_y = panel_y + config::MANUAL_PADDING;
    for line in &lines {
        font::draw_text(fb, text_x, text_y, line, config::MANUAL_TEXT_SCALE, config::MANUAL_TEXT_COLOR);
        text_y += line_height;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn draw_controls_manual_does_not_panic_on_small_framebuffer() {
        let mut fb = Framebuffer::new(64, 64);
        draw_controls_manual(&mut fb);
    }
}
