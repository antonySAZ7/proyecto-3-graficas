use crate::engine::framebuffer::{self, Framebuffer};
use crate::engine::math::{Mat4, Vec3};
use crate::engine::mesh::Mesh;
use crate::shaders::{FragInput, FragmentShader, Uniforms};

#[inline]
fn edge(a: (f32, f32), b: (f32, f32), p: (f32, f32)) -> f32 {
    (b.0 - a.0) * (p.1 - a.1) - (b.1 - a.1) * (p.0 - a.0)
}

/// Triángulo ya transformado a espacio de pantalla, listo para rasterizarse.
/// Resulta de la etapa de vértices (`RenderQueue::push_mesh`); guardarlo así
/// permite repartir la etapa de píxeles en bandas sin repetir el trabajo de
/// transformar y descartar por cada una.
struct PreparedTriangle<'a> {
    screen: [(f32, f32); 3],
    inv_w: [f32; 3],
    ndc_z: [f32; 3],
    world_pos: [Vec3; 3],
    local_pos: [Vec3; 3],
    world_normal: [Vec3; 3],
    cam_pos: Vec3,
    area: f32,
    min_x: i32,
    max_x: i32,
    min_y: i32,
    max_y: i32,
    shader: &'a dyn FragmentShader,
}

/// Contadores de la última llamada a `rasterize_parallel`, para el HUD de
/// rendimiento (ver `game::hud`) y para comparar visualmente con `F2`/`F3`.
#[derive(Clone, Copy, Default)]
pub struct RenderStats {
    /// Triángulos que llegaron a la etapa de píxeles (pasaron backface/near-plane).
    pub triangles_drawn: u32,
    /// Triángulos descartados antes de rasterizar (backface, detrás del near plane o degenerados).
    pub triangles_culled: u32,
}

/// Cola de triángulos del frame actual. El `Vec` interno se reutiliza entre
/// frames (`clear` solo vacía su contenido, no libera memoria), así que no
/// hay asignaciones por frame en el camino caliente.
pub struct RenderQueue<'a> {
    triangles: Vec<PreparedTriangle<'a>>,
    pub stats: RenderStats,
}

impl<'a> RenderQueue<'a> {
    pub fn new() -> Self {
        Self { triangles: Vec::new(), stats: RenderStats::default() }
    }

    pub fn clear(&mut self) {
        self.triangles.clear();
        self.stats = RenderStats::default();
    }

    /// Etapa de vértices: transforma `mesh` a espacio de pantalla y descarta
    /// por normal (backface, no depende del orden de vértices) y por near
    /// plane. Lo que sobrevive queda listo para `rasterize_parallel`.
    pub fn push_mesh(
        &mut self,
        mesh: &Mesh,
        model: Mat4,
        view_proj: Mat4,
        cam_pos: Vec3,
        shader: &'a dyn FragmentShader,
        viewport: (usize, usize),
    ) {
        let (width, height) = viewport;
        let normal_mat = model.inverse().transpose();

        for tri in &mesh.indices {
            let verts = [&mesh.vertices[tri[0] as usize], &mesh.vertices[tri[1] as usize], &mesh.vertices[tri[2] as usize]];

            let local_pos = [verts[0].position, verts[1].position, verts[2].position];
            let world_pos = [
                model.transform_point3(local_pos[0]),
                model.transform_point3(local_pos[1]),
                model.transform_point3(local_pos[2]),
            ];
            let world_normal = [
                normal_mat.transform_vector3(verts[0].normal).normalize(),
                normal_mat.transform_vector3(verts[1].normal).normalize(),
                normal_mat.transform_vector3(verts[2].normal).normalize(),
            ];

            let centroid = (world_pos[0] + world_pos[1] + world_pos[2]) / 3.0;
            let avg_normal = (world_normal[0] + world_normal[1] + world_normal[2]).normalize();
            if avg_normal.dot(cam_pos - centroid) <= 0.0 {
                self.stats.triangles_culled += 1;
                continue;
            }

            let clip = [
                view_proj * world_pos[0].extend(1.0),
                view_proj * world_pos[1].extend(1.0),
                view_proj * world_pos[2].extend(1.0),
            ];
            if clip.iter().any(|c| c.w <= 0.0001) {
                self.stats.triangles_culled += 1;
                continue; // cruza o está detrás del near plane
            }

            let inv_w = [1.0 / clip[0].w, 1.0 / clip[1].w, 1.0 / clip[2].w];
            let ndc = [
                Vec3::new(clip[0].x * inv_w[0], clip[0].y * inv_w[0], clip[0].z * inv_w[0]),
                Vec3::new(clip[1].x * inv_w[1], clip[1].y * inv_w[1], clip[1].z * inv_w[1]),
                Vec3::new(clip[2].x * inv_w[2], clip[2].y * inv_w[2], clip[2].z * inv_w[2]),
            ];

            let screen = [
                ((ndc[0].x * 0.5 + 0.5) * width as f32, (1.0 - (ndc[0].y * 0.5 + 0.5)) * height as f32),
                ((ndc[1].x * 0.5 + 0.5) * width as f32, (1.0 - (ndc[1].y * 0.5 + 0.5)) * height as f32),
                ((ndc[2].x * 0.5 + 0.5) * width as f32, (1.0 - (ndc[2].y * 0.5 + 0.5)) * height as f32),
            ];

            let area = edge(screen[0], screen[1], screen[2]);
            if area.abs() < 1e-6 {
                self.stats.triangles_culled += 1;
                continue; // triángulo degenerado (p.ej. polos de la esfera)
            }

            let min_x = screen.iter().map(|s| s.0).fold(f32::MAX, f32::min).floor().max(0.0) as i32;
            let max_x = screen.iter().map(|s| s.0).fold(f32::MIN, f32::max).ceil().min(width as f32 - 1.0) as i32;
            let min_y = screen.iter().map(|s| s.1).fold(f32::MAX, f32::min).floor().max(0.0) as i32;
            let max_y = screen.iter().map(|s| s.1).fold(f32::MIN, f32::max).ceil().min(height as f32 - 1.0) as i32;
            if min_x > max_x || min_y > max_y {
                self.stats.triangles_culled += 1;
                continue;
            }

            self.stats.triangles_drawn += 1;
            self.triangles.push(PreparedTriangle {
                screen,
                inv_w,
                ndc_z: [ndc[0].z, ndc[1].z, ndc[2].z],
                world_pos,
                local_pos,
                world_normal,
                cam_pos,
                area,
                min_x,
                max_x,
                min_y,
                max_y,
                shader,
            });
        }
    }

    /// Rasteriza todos los triángulos encolados. El framebuffer se reparte
    /// recursivamente a la mitad (`rayon::join`) hasta llegar a franjas de
    /// `band_rows` filas; como cada mitad es un sub-slice disjunto de
    /// `pixels`/`depth`, no hace falta ningún bloqueo ni asignación: todo el
    /// reparto ocurre con `split_at_mut` sobre los buffers ya existentes.
    pub fn rasterize_parallel(&self, fb: &mut Framebuffer, uniforms: &Uniforms, band_rows: usize) {
        let (width, pixels, depth) = fb.buffers_mut();
        rasterize_band(pixels, depth, width, 0, band_rows.max(1), &self.triangles, uniforms);
    }
}

/// Divide `[pixels, depth]` a la mitad (en filas) y procesa cada mitad en
/// paralelo con `rayon::join`, hasta que una franja mide `band_rows` filas o
/// menos, momento en el que se rasteriza directamente con `rasterize_rows`.
fn rasterize_band(
    pixels: &mut [u32],
    depth: &mut [f32],
    width: usize,
    y0: usize,
    band_rows: usize,
    triangles: &[PreparedTriangle],
    uniforms: &Uniforms,
) {
    let total_rows = pixels.len() / width;
    if total_rows <= band_rows {
        rasterize_rows(pixels, depth, width, y0, triangles, uniforms);
        return;
    }

    let mid_rows = total_rows / 2;
    let split = mid_rows * width;
    let (pixels_top, pixels_bottom) = pixels.split_at_mut(split);
    let (depth_top, depth_bottom) = depth.split_at_mut(split);
    rayon::join(
        || rasterize_band(pixels_top, depth_top, width, y0, band_rows, triangles, uniforms),
        || rasterize_band(pixels_bottom, depth_bottom, width, y0 + mid_rows, band_rows, triangles, uniforms),
    );
}

/// Rasteriza todos los triángulos que caen dentro de esta franja de filas
/// `[y0, y0 + pixels.len() / width)`, escribiendo directamente en su slice.
fn rasterize_rows(
    pixels: &mut [u32],
    depth: &mut [f32],
    width: usize,
    y0: usize,
    triangles: &[PreparedTriangle],
    uniforms: &Uniforms,
) {
    let y0_i = y0 as i32;
    let y1_i = y0_i + (pixels.len() / width) as i32;

    for tri in triangles {
        let y_lo = tri.min_y.max(y0_i);
        let y_hi = tri.max_y.min(y1_i - 1);
        if y_lo > y_hi {
            continue;
        }

        for y in y_lo..=y_hi {
            let row = (y - y0_i) as usize;
            for x in tri.min_x..=tri.max_x {
                let p = (x as f32 + 0.5, y as f32 + 0.5);
                let w0 = edge(tri.screen[1], tri.screen[2], p);
                let w1 = edge(tri.screen[2], tri.screen[0], p);
                let w2 = edge(tri.screen[0], tri.screen[1], p);

                let inside = if tri.area > 0.0 {
                    w0 >= 0.0 && w1 >= 0.0 && w2 >= 0.0
                } else {
                    w0 <= 0.0 && w1 <= 0.0 && w2 <= 0.0
                };
                if !inside {
                    continue;
                }

                let b0 = w0 / tri.area;
                let b1 = w1 / tri.area;
                let b2 = w2 / tri.area;

                let frag_depth = b0 * tri.ndc_z[0] + b1 * tri.ndc_z[1] + b2 * tri.ndc_z[2];
                let x = x as usize;
                let idx = row * width + x;
                if frag_depth >= depth[idx] {
                    continue;
                }

                let iw = b0 * tri.inv_w[0] + b1 * tri.inv_w[1] + b2 * tri.inv_w[2];
                let persp = |a0: Vec3, a1: Vec3, a2: Vec3| -> Vec3 {
                    (a0 * (b0 * tri.inv_w[0]) + a1 * (b1 * tri.inv_w[1]) + a2 * (b2 * tri.inv_w[2])) / iw
                };

                let world = persp(tri.world_pos[0], tri.world_pos[1], tri.world_pos[2]);
                let local = persp(tri.local_pos[0], tri.local_pos[1], tri.local_pos[2]);
                let normal = persp(tri.world_normal[0], tri.world_normal[1], tri.world_normal[2]).normalize();
                let view_dir = (tri.cam_pos - world).normalize();

                let input = FragInput { world_pos: world, local, normal, view_dir };
                let color = tri.shader.shade(&input, uniforms);
                framebuffer::blend_pixel_into(pixels, depth, idx, frag_depth, color);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::framebuffer::Framebuffer;
    use crate::engine::math::Vec4;
    use crate::engine::mesh::{Mesh, Vertex};

    struct SolidShader;
    impl FragmentShader for SolidShader {
        fn shade(&self, _input: &FragInput, _uniforms: &Uniforms) -> Vec4 {
            Vec4::new(1.0, 1.0, 1.0, 1.0)
        }
    }

    fn front_facing_triangle(normal_z: f32) -> Mesh {
        Mesh {
            vertices: vec![
                Vertex { position: Vec3::new(-1.0, -1.0, 0.0), normal: Vec3::new(0.0, 0.0, normal_z) },
                Vertex { position: Vec3::new(1.0, -1.0, 0.0), normal: Vec3::new(0.0, 0.0, normal_z) },
                Vertex { position: Vec3::new(0.0, 1.0, 0.0), normal: Vec3::new(0.0, 0.0, normal_z) },
            ],
            indices: vec![[0, 1, 2]],
        }
    }

    fn test_view_proj(width: usize, height: usize) -> Mat4 {
        let proj = Mat4::perspective_rh(60f32.to_radians(), width as f32 / height as f32, 0.1, 100.0);
        let view = Mat4::look_at_rh(Vec3::new(0.0, 0.0, 5.0), Vec3::ZERO, Vec3::Y);
        proj * view
    }

    #[test]
    fn front_facing_triangle_is_queued_and_backface_is_culled() {
        let mesh = front_facing_triangle(1.0);
        let view_proj = test_view_proj(64, 64);
        let shader = SolidShader;

        let mut front = RenderQueue::new();
        front.push_mesh(&mesh, Mat4::IDENTITY, view_proj, Vec3::new(0.0, 0.0, 5.0), &shader, (64, 64));
        assert_eq!(front.stats.triangles_drawn, 1);
        assert_eq!(front.stats.triangles_culled, 0);

        let back = front_facing_triangle(-1.0);
        let mut culled = RenderQueue::new();
        culled.push_mesh(&back, Mat4::IDENTITY, view_proj, Vec3::new(0.0, 0.0, 5.0), &shader, (64, 64));
        assert_eq!(culled.stats.triangles_drawn, 0);
        assert_eq!(culled.stats.triangles_culled, 1);
    }

    #[test]
    fn band_count_does_not_change_the_rendered_image() {
        let mesh = front_facing_triangle(1.0);
        let view_proj = test_view_proj(64, 64);
        let shader = SolidShader;
        let uniforms = Uniforms { time: 0.0, light_dir: Vec3::new(0.0, 0.0, 1.0) };

        let mut queue = RenderQueue::new();
        queue.push_mesh(&mesh, Mat4::IDENTITY, view_proj, Vec3::new(0.0, 0.0, 5.0), &shader, (64, 64));

        let mut single_band = Framebuffer::new(64, 64);
        queue.rasterize_parallel(&mut single_band, &uniforms, 64);

        let mut many_bands = Framebuffer::new(64, 64);
        queue.rasterize_parallel(&mut many_bands, &uniforms, 5);

        assert_eq!(single_band.pixels, many_bands.pixels);
        // El triángulo cubre el centro del framebuffer: algo se dibujó encima del fondo.
        assert_ne!(single_band.pixels[32 * 64 + 32], 0);
    }
}
