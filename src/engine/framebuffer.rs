use crate::engine::math::{Vec3, Vec4};

pub struct Framebuffer {
    pub width: usize,
    pub height: usize,
    pub pixels: Vec<u32>,
    depth: Vec<f32>,
}

/// Mezcla un fragmento directamente sobre buffers crudos. La usan tanto
/// `Framebuffer::blend_pixel` como el rasterizador por bandas
/// (`engine::rasterizer`), que escribe en una sub-rebanada disjunta de estos
/// mismos buffers desde otro hilo (ver `Framebuffer::buffers_mut`).
#[inline]
pub(crate) fn blend_pixel_into(pixels: &mut [u32], depth: &mut [f32], idx: usize, new_depth: f32, color: Vec4) {
    let rgb = Vec3::new(color.x, color.y, color.z);
    let final_rgb = if color.w >= 0.999 {
        rgb
    } else {
        let p = pixels[idx];
        let bg = Vec3::new(((p >> 16) & 0xFF) as f32 / 255.0, ((p >> 8) & 0xFF) as f32 / 255.0, (p & 0xFF) as f32 / 255.0);
        bg * (1.0 - color.w) + rgb * color.w
    };
    let r = (final_rgb.x.clamp(0.0, 1.0) * 255.0) as u32;
    let g = (final_rgb.y.clamp(0.0, 1.0) * 255.0) as u32;
    let b = (final_rgb.z.clamp(0.0, 1.0) * 255.0) as u32;
    pixels[idx] = (r << 16) | (g << 8) | b;
    depth[idx] = new_depth;
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

    /// Escribe un fragmento, mezclando con lo que ya había si `color.w < 1.0` (para los anillos).
    pub fn blend_pixel(&mut self, x: usize, y: usize, depth: f32, color: Vec4) {
        let idx = self.index(x, y);
        blend_pixel_into(&mut self.pixels, &mut self.depth, idx, depth, color);
    }

    /// Acceso directo a los buffers completos (ancho + píxeles + profundidad),
    /// para que `engine::rasterizer` los reparta en bandas recursivamente con
    /// `rayon::join` sin pasar por `Framebuffer` por cada fragmento.
    pub(crate) fn buffers_mut(&mut self) -> (usize, &mut [u32], &mut [f32]) {
        (self.width, &mut self.pixels, &mut self.depth)
    }
}
