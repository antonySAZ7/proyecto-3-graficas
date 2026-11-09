//! Selección de nivel de detalle por distancia relativa al tamaño del cuerpo:
//! se compara `distancia / radio`, no la distancia absoluta, así un cuerpo
//! grande visto de cerca y uno chico más cerca usan el mismo criterio. Los
//! umbrales son datos que vienen de `game::config`, no números fijos aquí.

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum LodLevel {
    High,
    Medium,
    Low,
}

pub fn select_lod(distance: f32, radius: f32, medium_ratio: f32, low_ratio: f32) -> LodLevel {
    let ratio = distance / radius.max(1e-4);
    if ratio > low_ratio {
        LodLevel::Low
    } else if ratio > medium_ratio {
        LodLevel::Medium
    } else {
        LodLevel::High
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn close_body_uses_high_detail() {
        assert_eq!(select_lod(5.0, 6.0, 8.0, 20.0), LodLevel::High);
    }

    #[test]
    fn mid_distance_uses_medium_detail() {
        assert_eq!(select_lod(60.0, 6.0, 8.0, 20.0), LodLevel::Medium);
    }

    #[test]
    fn far_body_uses_low_detail() {
        assert_eq!(select_lod(200.0, 6.0, 8.0, 20.0), LodLevel::Low);
    }

    #[test]
    fn threshold_scales_with_body_radius() {
        // Mismo múltiplo relativo del radio, cuerpos de tamaño distinto: igual nivel.
        assert_eq!(select_lod(50.0, 5.0, 8.0, 20.0), select_lod(500.0, 50.0, 8.0, 20.0));
    }
}
