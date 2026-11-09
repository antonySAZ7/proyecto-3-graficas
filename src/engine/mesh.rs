use glam::Vec3;

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
