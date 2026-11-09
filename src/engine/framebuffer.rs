use crate::engine::math::{Vec3, Vec4};

pub struct Framebuffer {
    pub width: usize,
    pub height: usize,
    pub pixels: Vec<u32>,
    depth: Vec<f32>,
}

impl Framebuffer {
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            width,
            height,
            pixels: vec![0; width * height],
            depth: vec![f32::INFINITY; width * height],
        }
    }

    #[inline]
    fn index(&self, x: usize, y: usize) -> usize {
        y * self.width + x
    }

    pub fn clear(&mut self, color: u32) {
        self.pixels.fill(color);
        self.depth.fill(f32::INFINITY);
    }

    #[inline]
    pub fn depth_at(&self, x: usize, y: usize) -> f32 {
        self.depth[self.index(x, y)]
    }

    fn get_rgb(&self, x: usize, y: usize) -> Vec3 {
        let p = self.pixels[self.index(x, y)];
        Vec3::new(
            ((p >> 16) & 0xFF) as f32 / 255.0,
            ((p >> 8) & 0xFF) as f32 / 255.0,
            (p & 0xFF) as f32 / 255.0,
        )
    }

    /// Escribe un fragmento, mezclando con lo que ya había si `color.w < 1.0` (para los anillos).
    pub fn blend_pixel(&mut self, x: usize, y: usize, depth: f32, color: Vec4) {
        let idx = self.index(x, y);
        let rgb = Vec3::new(color.x, color.y, color.z);
        let final_rgb = if color.w >= 0.999 {
            rgb
        } else {
            let bg = self.get_rgb(x, y);
            bg * (1.0 - color.w) + rgb * color.w
        };
        let r = (final_rgb.x.clamp(0.0, 1.0) * 255.0) as u32;
        let g = (final_rgb.y.clamp(0.0, 1.0) * 255.0) as u32;
        let b = (final_rgb.z.clamp(0.0, 1.0) * 255.0) as u32;
        self.pixels[idx] = (r << 16) | (g << 8) | b;
        self.depth[idx] = depth;
    }
}
