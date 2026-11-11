//! HUD mínimo de la Fase 2: el manual de controles. Se genera a partir de
//! `game::input::MANUAL`/`BINDINGS`, así que nunca queda desincronizado de
//! las teclas reales (cambiar una tecla es solo editar `input::BINDINGS`).

use crate::engine::font;
use crate::engine::framebuffer::Framebuffer;
use crate::engine::math::Vec4;
use crate::game::config;
use crate::game::input::{self, MANUAL};

pub struct EnemyIndicator {
    /// Dirección en espacio de pantalla desde el centro hacia el enemigo.
    pub x: f32,
    pub y: f32,
    /// `true` si el centro de Galactus ya está dentro del framebuffer.
    pub visible: bool,
}

/// Nombre y barra de salud siempre visibles arriba al centro. Si Galactus
/// sale de la pantalla, una flecha se ancla al borde en su dirección.
pub fn draw_enemy_status(fb: &mut Framebuffer, health: u32, max_health: u32, indicator: EnemyIndicator) {
    let label = format!("GALACTUS {health}/{max_health}");
    let label_w = font::text_width(&label, config::ENEMY_HUD_TEXT_SCALE);
    let label_x = (fb.width as i32 - label_w) / 2;
    let label_y = config::ENEMY_HUD_MARGIN_TOP;
    font::draw_text(fb, label_x, label_y, &label, config::ENEMY_HUD_TEXT_SCALE, config::THREAT_COLOR);

    let bar_x = (fb.width as i32 - config::ENEMY_HUD_BAR_WIDTH) / 2;
    let bar_y = label_y + font::GLYPH_HEIGHT as i32 * config::ENEMY_HUD_TEXT_SCALE + 5;
    font::draw_panel(
        fb,
        bar_x - 2,
        bar_y - 2,
        config::ENEMY_HUD_BAR_WIDTH + 4,
        config::ENEMY_HUD_BAR_HEIGHT + 4,
        Vec4::new(0.08, 0.015, 0.09, 0.88),
    );
    let ratio = if max_health == 0 { 0.0 } else { health as f32 / max_health as f32 };
    let fill = (config::ENEMY_HUD_BAR_WIDTH as f32 * ratio.clamp(0.0, 1.0)).round() as i32;
    if fill > 0 {
        font::draw_panel(fb, bar_x, bar_y, fill, config::ENEMY_HUD_BAR_HEIGHT, config::THREAT_COLOR);
    }

    if indicator.visible {
        return;
    }
    let mut dx = indicator.x;
    let mut dy = indicator.y;
    let length = (dx * dx + dy * dy).sqrt();
    if length < 1e-4 {
        dx = 0.0;
        dy = 1.0;
    } else {
        dx /= length;
        dy /= length;
    }
    let half_w = fb.width as f32 * 0.5 - config::ENEMY_HUD_EDGE_MARGIN;
    let half_h = fb.height as f32 * 0.5 - config::ENEMY_HUD_EDGE_MARGIN;
    let tx = if dx.abs() > 1e-4 { half_w / dx.abs() } else { f32::INFINITY };
    let ty = if dy.abs() > 1e-4 { half_h / dy.abs() } else { f32::INFINITY };
    let distance = tx.min(ty);
    let x = (fb.width as f32 * 0.5 + dx * distance) as i32;
    let y = (fb.height as f32 * 0.5 + dy * distance) as i32;
    let arrow = if dx.abs() > dy.abs() {
        if dx > 0.0 { "→" } else { "←" }
    } else if dy > 0.0 {
        "↓"
    } else {
        "↑"
    };
    let arrow_w = font::text_width(arrow, config::ENEMY_HUD_TEXT_SCALE);
    font::draw_text(fb, x - arrow_w / 2, y - 7, arrow, config::ENEMY_HUD_TEXT_SCALE, config::THREAT_COLOR);
}

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

/// Telemetría de vuelo para el panel inferior izquierdo: antes solo vivía en
/// el título de la ventana, invisible en cualquier captura de pantalla.
pub struct Telemetry<'a> {
    pub speed: f32,
    pub cruise: bool,
    pub stabilize: bool,
    pub orbit: bool,
    pub warp_charge: f32,
    pub target_name: &'a str,
    pub target_distance: f32,
    pub target_bearing_deg: f32,
    /// `true` si el cuerpo más cercano es Galactus: la línea de objetivo se
    /// pinta en el color de amenaza en vez del color de héroe.
    pub target_is_threat: bool,
}

fn flag_text(label: &str, on: bool) -> String {
    format!("{label} {}", if on { "ON" } else { "off" })
}

/// Panel anclado a la esquina inferior izquierda.
pub fn draw_telemetry_panel(fb: &mut Framebuffer, telemetry: &Telemetry) {
    let lines = [
        format!("VEL {:.1} U/S", telemetry.speed),
        flag_text("CRUCERO", telemetry.cruise),
        flag_text("ESTABILIZAR", telemetry.stabilize),
        flag_text("ORBITA", telemetry.orbit),
        flag_text("WARP", telemetry.warp_charge > 0.05),
        format!("OBJETIVO {} {:.0}U {:.0}G", telemetry.target_name, telemetry.target_distance, telemetry.target_bearing_deg),
    ];

    let line_height = font::GLYPH_HEIGHT as i32 * config::TELEMETRY_TEXT_SCALE + config::TELEMETRY_LINE_SPACING;
    let widest = lines.iter().map(|l| font::text_width(l, config::TELEMETRY_TEXT_SCALE)).max().unwrap_or(0);

    let panel_w = widest + config::TELEMETRY_PADDING * 2;
    let panel_h = line_height * lines.len() as i32 + config::TELEMETRY_PADDING * 2;
    let panel_x = config::TELEMETRY_MARGIN;
    let panel_y = fb.height as i32 - panel_h - config::TELEMETRY_MARGIN;

    font::draw_panel(fb, panel_x, panel_y, panel_w, panel_h, config::TELEMETRY_PANEL_COLOR);

    let text_x = panel_x + config::TELEMETRY_PADDING;
    let mut text_y = panel_y + config::TELEMETRY_PADDING;
    let target_color = if telemetry.target_is_threat { config::THREAT_COLOR } else { config::HERO_COLOR };
    for (i, line) in lines.iter().enumerate() {
        let color = if i == lines.len() - 1 { target_color } else { config::HERO_COLOR };
        font::draw_text(fb, text_x, text_y, line, config::TELEMETRY_TEXT_SCALE, color);
        text_y += line_height;
    }
}

/// Mensajes transitorios (impactos, colisiones, warp) con desvanecido lineal,
/// apilados sobre el centro inferior de la pantalla.
pub struct Messages {
    entries: Vec<(String, f32, Vec4)>,
}

impl Messages {
    pub fn new() -> Self {
        Self { entries: Vec::new() }
    }

    pub fn push(&mut self, text: impl Into<String>, color: Vec4) {
        self.entries.push((text.into(), config::MESSAGE_DURATION, color));
    }

    pub fn update(&mut self, dt: f32) {
        for entry in &mut self.entries {
            entry.1 -= dt;
        }
        self.entries.retain(|entry| entry.1 > 0.0);
    }

    pub fn draw(&self, fb: &mut Framebuffer) {
        let line_height = font::GLYPH_HEIGHT as i32 * config::MESSAGE_TEXT_SCALE + config::MESSAGE_LINE_SPACING;
        let mut text_y = fb.height as i32 - config::MESSAGE_MARGIN_BOTTOM - line_height * self.entries.len() as i32;
        for (text, remaining, color) in &self.entries {
            let alpha = (remaining / config::MESSAGE_DURATION).clamp(0.0, 1.0);
            let width = font::text_width(text, config::MESSAGE_TEXT_SCALE);
            let x = (fb.width as i32 - width) / 2;
            let faded = Vec4::new(color.x, color.y, color.z, color.w * alpha);
            font::draw_text(fb, x, text_y, text, config::MESSAGE_TEXT_SCALE, faded);
            text_y += line_height;
        }
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
    fn enemy_status_and_offscreen_arrow_do_not_panic() {
        let mut fb = Framebuffer::new(320, 180);
        draw_enemy_status(
            &mut fb,
            21,
            30,
            EnemyIndicator { x: 1.0, y: -0.3, visible: false },
        );
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

    #[test]
    fn telemetry_panel_does_not_panic_on_small_framebuffer() {
        let mut fb = Framebuffer::new(64, 64);
        let telemetry = Telemetry {
            speed: 5.0,
            cruise: false,
            stabilize: true,
            orbit: false,
            warp_charge: 0.0,
            target_name: "Tierra",
            target_distance: 10.0,
            target_bearing_deg: 0.0,
            target_is_threat: false,
        };
        draw_telemetry_panel(&mut fb, &telemetry);
    }

    #[test]
    fn messages_fade_out_and_are_removed_after_their_duration() {
        let mut messages = Messages::new();
        messages.push("PRUEBA", Vec4::new(1.0, 1.0, 1.0, 1.0));
        assert_eq!(messages.entries.len(), 1);
        messages.update(config::MESSAGE_DURATION + 0.1);
        assert_eq!(messages.entries.len(), 0);
    }

    #[test]
    fn messages_draw_does_not_panic_on_small_framebuffer() {
        let mut fb = Framebuffer::new(64, 64);
        let mut messages = Messages::new();
        messages.push("HOLA", config::HERO_COLOR);
        messages.draw(&mut fb);
    }
}
