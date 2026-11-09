use glam::{Mat4, Vec3};

use crate::engine::framebuffer::Framebuffer;
use crate::engine::mesh::Mesh;
use crate::shaders::{FragInput, FragmentShader, Uniforms};

#[inline]
fn edge(a: (f32, f32), b: (f32, f32), p: (f32, f32)) -> f32 {
    (b.0 - a.0) * (p.1 - a.1) - (b.1 - a.1) * (p.0 - a.0)
}

/// Transforma y rasteriza `mesh`, llamando a `shader` por cada píxel cubierto.
/// Pipeline completo: objeto -> mundo -> clip -> NDC -> pantalla, interpolación
/// con corrección de perspectiva, backface culling (por normal, no depende del
/// orden de los vértices) y prueba de profundidad con z-buffer.
pub fn draw_mesh(
    fb: &mut Framebuffer,
    mesh: &Mesh,
    model: Mat4,
    view_proj: Mat4,
    cam_pos: Vec3,
    shader: &dyn FragmentShader,
    uniforms: &Uniforms,
) {
    let normal_mat = model.inverse().transpose();
    let width = fb.width;
    let height = fb.height;

    for tri in &mesh.indices {
        let verts = [
            &mesh.vertices[tri[0] as usize],
            &mesh.vertices[tri[1] as usize],
            &mesh.vertices[tri[2] as usize],
        ];

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

        // Cull por normal promedio: no depende del orden de los vértices.
        let centroid = (world_pos[0] + world_pos[1] + world_pos[2]) / 3.0;
        let avg_normal = (world_normal[0] + world_normal[1] + world_normal[2]).normalize();
        if avg_normal.dot(cam_pos - centroid) <= 0.0 {
            continue;
        }

        let clip = [
            view_proj * world_pos[0].extend(1.0),
            view_proj * world_pos[1].extend(1.0),
            view_proj * world_pos[2].extend(1.0),
        ];
        if clip.iter().any(|c| c.w <= 0.0001) {
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
            continue; // triángulo degenerado (p.ej. polos de la esfera)
        }

        let min_x = screen.iter().map(|s| s.0).fold(f32::MAX, f32::min).floor().max(0.0) as i32;
        let max_x = screen.iter().map(|s| s.0).fold(f32::MIN, f32::max).ceil().min(width as f32 - 1.0) as i32;
        let min_y = screen.iter().map(|s| s.1).fold(f32::MAX, f32::min).floor().max(0.0) as i32;
        let max_y = screen.iter().map(|s| s.1).fold(f32::MIN, f32::max).ceil().min(height as f32 - 1.0) as i32;
        if min_x > max_x || min_y > max_y {
            continue;
        }

        for y in min_y..=max_y {
            for x in min_x..=max_x {
                let p = (x as f32 + 0.5, y as f32 + 0.5);
                let w0 = edge(screen[1], screen[2], p);
                let w1 = edge(screen[2], screen[0], p);
                let w2 = edge(screen[0], screen[1], p);

                let inside = if area > 0.0 {
                    w0 >= 0.0 && w1 >= 0.0 && w2 >= 0.0
                } else {
                    w0 <= 0.0 && w1 <= 0.0 && w2 <= 0.0
                };
                if !inside {
                    continue;
                }

                let b0 = w0 / area;
                let b1 = w1 / area;
                let b2 = w2 / area;

                let depth = b0 * ndc[0].z + b1 * ndc[1].z + b2 * ndc[2].z;
                if depth >= fb.depth_at(x as usize, y as usize) {
                    continue;
                }

                let iw = b0 * inv_w[0] + b1 * inv_w[1] + b2 * inv_w[2];
                let persp = |a0: Vec3, a1: Vec3, a2: Vec3| -> Vec3 {
                    (a0 * (b0 * inv_w[0]) + a1 * (b1 * inv_w[1]) + a2 * (b2 * inv_w[2])) / iw
                };

                let world = persp(world_pos[0], world_pos[1], world_pos[2]);
                let local = persp(local_pos[0], local_pos[1], local_pos[2]);
                let normal = persp(world_normal[0], world_normal[1], world_normal[2]).normalize();
                let view_dir = (cam_pos - world).normalize();

                let input = FragInput { world_pos: world, local, normal, view_dir };
                let color = shader.shade(&input, uniforms);
                fb.blend_pixel(x as usize, y as usize, depth, color);
            }
        }
    }
}
