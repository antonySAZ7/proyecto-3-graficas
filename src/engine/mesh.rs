use std::collections::HashMap;

use crate::engine::math::{Mat4, Vec3};

/// Un vértice en espacio de objeto. El shader recibe `position` interpolada
/// como `local`, así el ruido procedural no depende de la matriz de modelo.
#[derive(Clone, Copy)]
pub struct Vertex {
    pub position: Vec3,
    pub normal: Vec3,
}

pub struct Mesh {
    pub vertices: Vec<Vertex>,
    pub indices: Vec<[u32; 3]>,
}

/// Esfera UV de radio unitario (el radio real se aplica con la matriz de modelo).
pub fn sphere(stacks: u32, slices: u32) -> Mesh {
    let mut vertices = Vec::with_capacity(((stacks + 1) * (slices + 1)) as usize);
    for i in 0..=stacks {
        let v = i as f32 / stacks as f32;
        let theta = v * std::f32::consts::PI;
        for j in 0..=slices {
            let u = j as f32 / slices as f32;
            let phi = u * std::f32::consts::TAU;
            let dir = Vec3::new(theta.sin() * phi.cos(), theta.cos(), theta.sin() * phi.sin());
            vertices.push(Vertex { position: dir, normal: dir });
        }
    }

    let cols = slices + 1;
    let mut indices = Vec::with_capacity((stacks * slices * 2) as usize);
    for i in 0..stacks {
        for j in 0..slices {
            let a = i * cols + j;
            let b = a + 1;
            let c = a + cols;
            let d = c + 1;
            indices.push([a, c, b]);
            indices.push([b, c, d]);
        }
    }

    Mesh { vertices, indices }
}

/// Anillo plano en el plano XZ (normal hacia +Y), usado para los anillos del planeta.
pub fn ring(inner: f32, outer: f32, segments: u32) -> Mesh {
    let mut vertices = Vec::with_capacity(((segments + 1) * 2) as usize);
    for j in 0..=segments {
        let u = j as f32 / segments as f32;
        let phi = u * std::f32::consts::TAU;
        let (s, c) = phi.sin_cos();
        let inner_pos = Vec3::new(inner * c, 0.0, inner * s);
        let outer_pos = Vec3::new(outer * c, 0.0, outer * s);
        vertices.push(Vertex { position: inner_pos, normal: Vec3::Y });
        vertices.push(Vertex { position: outer_pos, normal: Vec3::Y });
    }

    let mut indices = Vec::with_capacity((segments * 2) as usize);
    for j in 0..segments {
        let i0 = j * 2;
        let i1 = i0 + 1;
        let i2 = i0 + 2;
        let i3 = i0 + 3;
        indices.push([i0, i2, i1]);
        indices.push([i1, i2, i3]);
    }

    Mesh { vertices, indices }
}

/// Cilindro centrado en el origen, eje Y, con tapas. Usado para troncos,
/// torres y postes de los modelos de superficie (Fase 5).
pub fn cylinder(segments: u32, radius: f32, height: f32) -> Mesh {
    let half = height * 0.5;
    let mut vertices = Vec::with_capacity((segments * 8) as usize);
    let mut indices = Vec::with_capacity((segments * 4) as usize);
    for i in 0..segments {
        let a0 = i as f32 / segments as f32 * std::f32::consts::TAU;
        let a1 = (i + 1) as f32 / segments as f32 * std::f32::consts::TAU;
        let n0 = Vec3::new(a0.cos(), 0.0, a0.sin());
        let n1 = Vec3::new(a1.cos(), 0.0, a1.sin());
        let p00 = Vec3::new(n0.x * radius, -half, n0.z * radius);
        let p01 = Vec3::new(n0.x * radius, half, n0.z * radius);
        let p10 = Vec3::new(n1.x * radius, -half, n1.z * radius);
        let p11 = Vec3::new(n1.x * radius, half, n1.z * radius);

        let side = vertices.len() as u32;
        vertices.push(Vertex { position: p00, normal: n0 });
        vertices.push(Vertex { position: p10, normal: n1 });
        vertices.push(Vertex { position: p11, normal: n1 });
        vertices.push(Vertex { position: p01, normal: n0 });
        indices.push([side, side + 1, side + 2]);
        indices.push([side, side + 2, side + 3]);

        let top = vertices.len() as u32;
        vertices.push(Vertex { position: Vec3::new(0.0, half, 0.0), normal: Vec3::Y });
        vertices.push(Vertex { position: p01, normal: Vec3::Y });
        vertices.push(Vertex { position: p11, normal: Vec3::Y });
        indices.push([top, top + 1, top + 2]);

        let bottom_normal = Vec3::new(0.0, -1.0, 0.0);
        let bottom = vertices.len() as u32;
        vertices.push(Vertex { position: Vec3::new(0.0, -half, 0.0), normal: bottom_normal });
        vertices.push(Vertex { position: p10, normal: bottom_normal });
        vertices.push(Vertex { position: p00, normal: bottom_normal });
        indices.push([bottom, bottom + 1, bottom + 2]);
    }
    Mesh { vertices, indices }
}

/// Cono centrado en el origen, eje Y, con tapa inferior. Usado para copas de
/// árbol y techos de torre (Fase 5).
pub fn cone(segments: u32, radius: f32, height: f32) -> Mesh {
    let half = height * 0.5;
    let mut vertices = Vec::with_capacity((segments * 6) as usize);
    let mut indices = Vec::with_capacity((segments * 2) as usize);
    for i in 0..segments {
        let a0 = i as f32 / segments as f32 * std::f32::consts::TAU;
        let a1 = (i + 1) as f32 / segments as f32 * std::f32::consts::TAU;
        let p0 = Vec3::new(a0.cos() * radius, -half, a0.sin() * radius);
        let p1 = Vec3::new(a1.cos() * radius, -half, a1.sin() * radius);
        let n0 = Vec3::new(a0.cos(), radius / height, a0.sin()).normalize();
        let n1 = Vec3::new(a1.cos(), radius / height, a1.sin()).normalize();
        let apex_normal = (n0 + n1).normalize();

        let side = vertices.len() as u32;
        vertices.push(Vertex { position: p0, normal: n0 });
        vertices.push(Vertex { position: p1, normal: n1 });
        vertices.push(Vertex { position: Vec3::new(0.0, half, 0.0), normal: apex_normal });
        indices.push([side, side + 1, side + 2]);

        let bottom_normal = Vec3::new(0.0, -1.0, 0.0);
        let bottom = vertices.len() as u32;
        vertices.push(Vertex { position: Vec3::new(0.0, -half, 0.0), normal: bottom_normal });
        vertices.push(Vertex { position: p1, normal: bottom_normal });
        vertices.push(Vertex { position: p0, normal: bottom_normal });
        indices.push([bottom, bottom + 1, bottom + 2]);
    }
    Mesh { vertices, indices }
}

/// Caja centrada en el origen con `half` como semi-extensión por eje. Usada
/// para la base de la torre (Fase 5).
pub fn cuboid(half: Vec3) -> Mesh {
    let h = half;
    let faces: [(Vec3, [Vec3; 4]); 6] = [
        (Vec3::new(0.0, 0.0, 1.0), [Vec3::new(-h.x, -h.y, h.z), Vec3::new(h.x, -h.y, h.z), Vec3::new(h.x, h.y, h.z), Vec3::new(-h.x, h.y, h.z)]),
        (Vec3::new(0.0, 0.0, -1.0), [Vec3::new(h.x, -h.y, -h.z), Vec3::new(-h.x, -h.y, -h.z), Vec3::new(-h.x, h.y, -h.z), Vec3::new(h.x, h.y, -h.z)]),
        (Vec3::new(1.0, 0.0, 0.0), [Vec3::new(h.x, -h.y, h.z), Vec3::new(h.x, -h.y, -h.z), Vec3::new(h.x, h.y, -h.z), Vec3::new(h.x, h.y, h.z)]),
        (Vec3::new(-1.0, 0.0, 0.0), [Vec3::new(-h.x, -h.y, -h.z), Vec3::new(-h.x, -h.y, h.z), Vec3::new(-h.x, h.y, h.z), Vec3::new(-h.x, h.y, -h.z)]),
        (Vec3::new(0.0, 1.0, 0.0), [Vec3::new(-h.x, h.y, h.z), Vec3::new(h.x, h.y, h.z), Vec3::new(h.x, h.y, -h.z), Vec3::new(-h.x, h.y, -h.z)]),
        (Vec3::new(0.0, -1.0, 0.0), [Vec3::new(-h.x, -h.y, -h.z), Vec3::new(h.x, -h.y, -h.z), Vec3::new(h.x, -h.y, h.z), Vec3::new(-h.x, -h.y, h.z)]),
    ];
    let mut vertices = Vec::with_capacity(24);
    let mut indices = Vec::with_capacity(12);
    for (normal, corners) in faces {
        let base = vertices.len() as u32;
        for c in corners {
            vertices.push(Vertex { position: c, normal });
        }
        indices.push([base, base + 1, base + 2]);
        indices.push([base, base + 2, base + 3]);
    }
    Mesh { vertices, indices }
}

/// Icosaedro subdividido `subdivisions` veces y normalizado a radio unitario:
/// generación/subdivisión procedural de malla (Fase 5), base del Planeta
/// Fragmentado y de la roca de superficie antes de deformarlos con ruido.
pub fn icosphere(subdivisions: u32) -> Mesh {
    let t = (1.0 + 5f32.sqrt()) / 2.0;
    let seed_positions = [
        Vec3::new(-1.0, t, 0.0), Vec3::new(1.0, t, 0.0), Vec3::new(-1.0, -t, 0.0), Vec3::new(1.0, -t, 0.0),
        Vec3::new(0.0, -1.0, t), Vec3::new(0.0, 1.0, t), Vec3::new(0.0, -1.0, -t), Vec3::new(0.0, 1.0, -t),
        Vec3::new(t, 0.0, -1.0), Vec3::new(t, 0.0, 1.0), Vec3::new(-t, 0.0, -1.0), Vec3::new(-t, 0.0, 1.0),
    ];
    let mut positions: Vec<Vec3> = seed_positions.iter().map(|p| p.normalize()).collect();

    let mut faces: Vec<[u32; 3]> = vec![
        [0, 11, 5], [0, 5, 1], [0, 1, 7], [0, 7, 10], [0, 10, 11],
        [1, 5, 9], [5, 11, 4], [11, 10, 2], [10, 7, 6], [7, 1, 8],
        [3, 9, 4], [3, 4, 2], [3, 2, 6], [3, 6, 8], [3, 8, 9],
        [4, 9, 5], [2, 4, 11], [6, 2, 10], [8, 6, 7], [9, 8, 1],
    ];

    for _ in 0..subdivisions {
        let mut midpoint_cache: HashMap<(u32, u32), u32> = HashMap::new();
        let mut next_faces = Vec::with_capacity(faces.len() * 4);
        for f in &faces {
            let [a, b, c] = *f;
            let ab = midpoint(a, b, &mut positions, &mut midpoint_cache);
            let bc = midpoint(b, c, &mut positions, &mut midpoint_cache);
            let ca = midpoint(c, a, &mut positions, &mut midpoint_cache);
            next_faces.push([a, ab, ca]);
            next_faces.push([b, bc, ab]);
            next_faces.push([c, ca, bc]);
            next_faces.push([ab, bc, ca]);
        }
        faces = next_faces;
    }

    let vertices = positions.iter().map(|&p| Vertex { position: p, normal: p }).collect();
    Mesh { vertices, indices: faces }
}

/// Índice del punto medio (normalizado a la esfera) entre `a` y `b`,
/// reutilizando el ya creado por la cara vecina para no duplicar vértices.
fn midpoint(a: u32, b: u32, positions: &mut Vec<Vec3>, cache: &mut HashMap<(u32, u32), u32>) -> u32 {
    let key = if a < b { (a, b) } else { (b, a) };
    if let Some(&idx) = cache.get(&key) {
        return idx;
    }
    let mid = ((positions[a as usize] + positions[b as usize]) * 0.5).normalize();
    positions.push(mid);
    let idx = (positions.len() - 1) as u32;
    cache.insert(key, idx);
    idx
}

/// Recalcula normales por vértice promediando las normales de cara
/// (producto cruz), usado tras desplazar vértices (`displace`).
pub fn recompute_normals(mesh: &mut Mesh) {
    for v in &mut mesh.vertices {
        v.normal = Vec3::ZERO;
    }
    for tri in &mesh.indices {
        let p0 = mesh.vertices[tri[0] as usize].position;
        let p1 = mesh.vertices[tri[1] as usize].position;
        let p2 = mesh.vertices[tri[2] as usize].position;
        let face_normal = (p1 - p0).cross(p2 - p0);
        for &i in tri {
            mesh.vertices[i as usize].normal = mesh.vertices[i as usize].normal + face_normal;
        }
    }
    for v in &mut mesh.vertices {
        v.normal = v.normal.normalize();
    }
}

/// Desplaza cada vértice a lo largo de su propia dirección (desde el origen)
/// según `deform(direccion_unitaria)`, y recalcula las normales resultantes.
/// Usado para la deformación por ruido multioctava del Planeta Fragmentado y
/// de la roca de superficie (Fase 5).
pub fn displace(mesh: &mut Mesh, mut deform: impl FnMut(Vec3) -> f32) {
    for v in &mut mesh.vertices {
        let dir = v.position.normalize();
        let factor = deform(dir);
        v.position = dir * factor;
    }
    recompute_normals(mesh);
}

/// Transforma posiciones (como puntos) y normales (como direcciones,
/// renormalizadas) de una malla por `m`. Usado para componer piezas de los
/// modelos de superficie antes de fusionarlas con `merge`.
pub fn transform(mesh: &Mesh, m: Mat4) -> Mesh {
    let vertices = mesh
        .vertices
        .iter()
        .map(|v| Vertex { position: m.transform_point3(v.position), normal: m.transform_vector3(v.normal).normalize() })
        .collect();
    Mesh { vertices, indices: mesh.indices.clone() }
}

/// Concatena varias mallas en una sola, reindexando cada una por su offset
/// de vértices. Usado para armar los modelos de superficie a partir de
/// piezas simples (tronco+copa, base+techo, etc.).
pub fn merge(meshes: &[Mesh]) -> Mesh {
    let mut vertices = Vec::new();
    let mut indices = Vec::new();
    for mesh in meshes {
        let offset = vertices.len() as u32;
        vertices.extend_from_slice(&mesh.vertices);
        indices.extend(mesh.indices.iter().map(|tri| [tri[0] + offset, tri[1] + offset, tri[2] + offset]));
    }
    Mesh { vertices, indices }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn icosphere_base_has_twelve_vertices_on_the_unit_sphere() {
        let mesh = icosphere(0);
        assert_eq!(mesh.vertices.len(), 12);
        assert_eq!(mesh.indices.len(), 20);
        for v in &mesh.vertices {
            assert!((v.position.length() - 1.0).abs() < 1e-5);
        }
    }

    #[test]
    fn icosphere_subdivision_quadruples_the_faces() {
        let base = icosphere(0);
        let once = icosphere(1);
        assert_eq!(once.indices.len(), base.indices.len() * 4);
        for v in &once.vertices {
            assert!((v.position.length() - 1.0).abs() < 1e-5);
        }
    }

    #[test]
    fn displace_moves_vertices_along_their_own_direction() {
        let mut mesh = icosphere(1);
        displace(&mut mesh, |_dir| 2.0);
        for v in &mesh.vertices {
            assert!((v.position.length() - 2.0).abs() < 1e-4);
            // Tras desplazar uniformemente, la normal recalculada sigue
            // apuntando hacia afuera (misma dirección que la posición).
            assert!(v.normal.dot(v.position.normalize()) > 0.0);
        }
    }

    #[test]
    fn merge_reindexes_each_mesh_by_its_vertex_offset() {
        let a = cuboid(Vec3::splat(0.5));
        let b = cuboid(Vec3::splat(0.5));
        let a_len = a.vertices.len();
        let merged = merge(&[a, b]);
        assert_eq!(merged.vertices.len(), a_len * 2);
        assert!(merged.indices.iter().flatten().all(|&i| (i as usize) < merged.vertices.len()));
        assert!(merged.indices[merged.indices.len() - 1].iter().any(|&i| i as usize >= a_len));
    }

    #[test]
    fn cylinder_and_cone_vertices_sit_within_their_radius_and_height() {
        let cyl = cylinder(6, 0.5, 2.0);
        for v in &cyl.vertices {
            assert!(v.position.x.hypot(v.position.z) <= 0.5 + 1e-4);
            assert!(v.position.y.abs() <= 1.0 + 1e-4);
        }
        let cn = cone(6, 0.5, 2.0);
        for v in &cn.vertices {
            assert!(v.position.y.abs() <= 1.0 + 1e-4);
        }
    }
}
