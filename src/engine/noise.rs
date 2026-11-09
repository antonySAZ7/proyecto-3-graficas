//! Ruido Perlin propio (tabla de permutación precalculada a partir de una
//! semilla) más una envoltura fBm (suma de octavas) para shaders procedurales.

use crate::engine::math::Vec3;

pub struct Perlin {
    perm: [u8; 512],
}

impl Perlin {
    pub fn new(seed: u32) -> Self {
        let mut table: [u8; 256] = {
            let mut t = [0u8; 256];
            for (i, v) in t.iter_mut().enumerate() {
                *v = i as u8;
            }
            t
        };

        // Fisher-Yates con un xorshift32 propio, determinista por semilla.
        let mut state = seed ^ 0x9E37_79B9;
        for i in (1..256usize).rev() {
            state ^= state << 13;
            state ^= state >> 17;
            state ^= state << 5;
            let j = (state % (i as u32 + 1)) as usize;
            table.swap(i, j);
        }

        let mut perm = [0u8; 512];
        for i in 0..512 {
            perm[i] = table[i & 255];
        }
        Self { perm }
    }

    #[inline]
    fn fade(t: f32) -> f32 {
        t * t * t * (t * (t * 6.0 - 15.0) + 10.0)
    }

    #[inline]
    fn lerp(t: f32, a: f32, b: f32) -> f32 {
        a + t * (b - a)
    }

    #[inline]
    fn grad(hash: u8, x: f32, y: f32, z: f32) -> f32 {
        let h = hash & 15;
        let u = if h < 8 { x } else { y };
        let v = if h < 4 { y } else if h == 12 || h == 14 { x } else { z };
        (if h & 1 == 0 { u } else { -u }) + (if h & 2 == 0 { v } else { -v })
    }

    /// Ruido Perlin 3D clásico (Ken Perlin, "improved noise"), rango aproximado [-1, 1].
    pub fn sample3(&self, p: Vec3) -> f32 {
        let xi = p.x.floor();
        let yi = p.y.floor();
        let zi = p.z.floor();
        let xf = p.x - xi;
        let yf = p.y - yi;
        let zf = p.z - zi;

        let x = (xi as i32 & 255) as usize;
        let y = (yi as i32 & 255) as usize;
        let z = (zi as i32 & 255) as usize;

        let u = Self::fade(xf);
        let v = Self::fade(yf);
        let w = Self::fade(zf);

        let perm = &self.perm;
        let a = perm[x] as usize + y;
        let aa = perm[a] as usize + z;
        let ab = perm[a + 1] as usize + z;
        let b = perm[x + 1] as usize + y;
        let ba = perm[b] as usize + z;
        let bb = perm[b + 1] as usize + z;

        Self::lerp(
            w,
            Self::lerp(
                v,
                Self::lerp(u, Self::grad(perm[aa], xf, yf, zf), Self::grad(perm[ba], xf - 1.0, yf, zf)),
                Self::lerp(u, Self::grad(perm[ab], xf, yf - 1.0, zf), Self::grad(perm[bb], xf - 1.0, yf - 1.0, zf)),
            ),
            Self::lerp(
                v,
                Self::lerp(u, Self::grad(perm[aa + 1], xf, yf, zf - 1.0), Self::grad(perm[ba + 1], xf - 1.0, yf, zf - 1.0)),
                Self::lerp(u, Self::grad(perm[ab + 1], xf, yf - 1.0, zf - 1.0), Self::grad(perm[bb + 1], xf - 1.0, yf - 1.0, zf - 1.0)),
            ),
        )
    }
}

/// Suma de octavas de ruido Perlin (fractal Brownian motion).
pub struct Fbm {
    perlin: Perlin,
    octaves: u32,
    frequency: f32,
    lacunarity: f32,
    persistence: f32,
}

impl Fbm {
    pub fn new(seed: u32) -> Self {
        Self { perlin: Perlin::new(seed), octaves: 4, frequency: 1.0, lacunarity: 2.0, persistence: 0.5 }
    }

    pub fn set_octaves(mut self, octaves: u32) -> Self {
        self.octaves = octaves.max(1);
        self
    }

    pub fn set_frequency(mut self, frequency: f32) -> Self {
        self.frequency = frequency;
        self
    }

    pub fn set_lacunarity(mut self, lacunarity: f32) -> Self {
        self.lacunarity = lacunarity;
        self
    }

    pub fn set_persistence(mut self, persistence: f32) -> Self {
        self.persistence = persistence;
        self
    }

    /// Suma `octaves` capas de ruido, normalizada por la amplitud total acumulada
    /// (así el resultado se mantiene aproximadamente en [-1, 1] sin importar cuántas octavas haya).
    pub fn sample3(&self, p: Vec3) -> f32 {
        let mut sum = 0.0;
        let mut amp = 1.0;
        let mut freq = self.frequency;
        let mut norm = 0.0;
        for _ in 0..self.octaves {
            sum += self.perlin.sample3(p * freq) * amp;
            norm += amp;
            amp *= self.persistence;
            freq *= self.lacunarity;
        }
        sum / norm.max(1e-6)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn perlin_same_seed_is_deterministic() {
        let a = Perlin::new(42);
        let b = Perlin::new(42);
        let p = Vec3::new(1.37, -2.5, 0.8);
        assert_eq!(a.sample3(p), b.sample3(p));
    }

    #[test]
    fn perlin_different_seed_differs() {
        let a = Perlin::new(1);
        let b = Perlin::new(2);
        let p = Vec3::new(1.37, -2.5, 0.8);
        assert_ne!(a.sample3(p), b.sample3(p));
    }

    #[test]
    fn perlin_integer_lattice_points_are_zero() {
        // Los nodos de la rejilla entera siempre dan 0 en el ruido Perlin clásico.
        let n = Perlin::new(7);
        assert_eq!(n.sample3(Vec3::new(3.0, -2.0, 5.0)), 0.0);
    }

    #[test]
    fn perlin_is_bounded() {
        let n = Perlin::new(9);
        for i in 0..200 {
            let p = Vec3::new(i as f32 * 0.31, i as f32 * 0.17, i as f32 * 0.05);
            let v = n.sample3(p);
            assert!((-1.1..=1.1).contains(&v), "fuera de rango: {v}");
        }
    }

    #[test]
    fn fbm_sample_is_deterministic_and_bounded() {
        let fbm = Fbm::new(5).set_octaves(5).set_frequency(1.6).set_lacunarity(2.0).set_persistence(0.55);
        let fbm2 = Fbm::new(5).set_octaves(5).set_frequency(1.6).set_lacunarity(2.0).set_persistence(0.55);
        for i in 0..50 {
            let p = Vec3::new(i as f32 * 0.21, 0.4, i as f32 * -0.13);
            let v1 = fbm.sample3(p);
            let v2 = fbm2.sample3(p);
            assert_eq!(v1, v2);
            assert!((-1.2..=1.2).contains(&v1), "fuera de rango: {v1}");
        }
    }
}
