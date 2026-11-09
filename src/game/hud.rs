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

/// Contadores de rendimiento de un frame, para el panel de la Fase 4.
pub struct PerfStats {
    pub fps: f32,
    pub triangles_drawn: u32,
    pub triangles_culled: u32,
    pub objects_visible: u32,
    pub objects_culled: u32,
    pub frustum_culling_enabled: bool,
    pub max_lod_forced: bool,
}

/// Panel anclado a la esquina superior izquierda con FPS, triángulos
/// procesados/descartados y objetos descartados por frustum culling; `F2`/`F3`
/// cambian los dos últimos campos para poder comparar en vivo.
pub fn draw_performance_panel(fb: &mut Framebuffer, stats: &PerfStats) {
    let lines = [
        format!("FPS {:.0}", stats.fps),
        format!("TRIANGULOS {} / {}", stats.triangles_drawn, stats.triangles_drawn + stats.triangles_culled),
        format!("OBJETOS VISIBLES {} / {}", stats.objects_visible, stats.objects_visible + stats.objects_culled),
        format!("CULLING {}", if stats.frustum_culling_enabled { "ON" } else { "OFF" }),
        format!("LOD MAX {}", if stats.max_lod_forced { "ON" } else { "off" }),
    ];

    let line_height = font::GLYPH_HEIGHT as i32 * config::PERF_TEXT_SCALE + config::PERF_LINE_SPACING;
    let widest = lines.iter().map(|l| font::text_width(l, config::PERF_TEXT_SCALE)).max().unwrap_or(0);

    let panel_w = widest + config::PERF_PADDING * 2;
    let panel_h = line_height * lines.len() as i32 + config::PERF_PADDING * 2;
    let panel_x = config::PERF_MARGIN;
    let panel_y = config::PERF_MARGIN;

    font::draw_panel(fb, panel_x, panel_y, panel_w, panel_h, config::PERF_PANEL_COLOR);

    let text_x = panel_x + config::PERF_PADDING;
    let mut text_y = panel_y + config::PERF_PADDING;
    for line in &lines {
        font::draw_text(fb, text_x, text_y, line, config::PERF_TEXT_SCALE, config::PERF_TEXT_COLOR);
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

    #[test]
    fn draw_performance_panel_does_not_panic_on_small_framebuffer() {
        let mut fb = Framebuffer::new(64, 64);
        let stats = PerfStats {
            fps: 60.0,
            triangles_drawn: 100,
            triangles_culled: 20,
            objects_visible: 3,
            objects_culled: 2,
            frustum_culling_enabled: true,
            max_lod_forced: false,
        };
        draw_performance_panel(&mut fb, &stats);
    }
}
