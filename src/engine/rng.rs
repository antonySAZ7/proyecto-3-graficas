//! Generador congruencial lineal (LCG) propio, determinista por semilla.
//! Única fuente de números pseudoaleatorios reproducibles del motor (cielo
//! estrellado, distribución de objetos de superficie, etc.), sin depender de
//! ninguna librería externa de aleatoriedad.

pub struct Lcg {
    state: u32,
}

impl Lcg {
    pub fn new(seed: u32) -> Self {
        Self { state: seed }
    }

    /// Siguiente valor en `[0, 1)`.
    pub fn next_f32(&mut self) -> f32 {
        self.state = self.state.wrapping_mul(1664525).wrapping_add(1013904223);
        self.state as f32 / u32::MAX as f32
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn same_seed_is_reproducible() {
        let mut a = Lcg::new(42);
        let mut b = Lcg::new(42);
        for _ in 0..20 {
            assert_eq!(a.next_f32(), b.next_f32());
        }
    }

    #[test]
    fn different_seed_differs() {
        let mut a = Lcg::new(1);
        let mut b = Lcg::new(2);
        assert_ne!(a.next_f32(), b.next_f32());
    }

    #[test]
    fn values_stay_in_unit_range() {
        let mut rng = Lcg::new(7);
        for _ in 0..500 {
            let v = rng.next_f32();
            assert!((0.0..=1.0).contains(&v));
        }
    }
}
