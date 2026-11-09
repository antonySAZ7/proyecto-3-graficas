//! Lector de modelos OBJ (solo `v` y `f`): triangula caras con más de 3
//! vértices (abanico) y calcula normales por vértice si el archivo no las trae.

use std::fs;

use crate::engine::math::Vec3;
use crate::engine::mesh::{Mesh, Vertex};

pub fn load_obj(path: &str) -> Result<Mesh, String> {
    let text = fs::read_to_string(path).map_err(|e| format!("no se pudo leer '{path}': {e}"))?;
    parse_obj(&text)
}

/// Índice de un token `f`: toma solo la posición (`v`, ignora `/vt/vn`).
fn parse_face_index(token: &str, vertex_count: usize) -> Option<usize> {
    let vi: i64 = token.split('/').next()?.parse().ok()?;
    if vi > 0 {
        Some((vi - 1) as usize)
    } else if vi < 0 {
        Some((vertex_count as i64 + vi) as usize)
    } else {
        None
    }
}

pub fn parse_obj(text: &str) -> Result<Mesh, String> {
    let mut positions: Vec<Vec3> = Vec::new();
    let mut triangles: Vec<[usize; 3]> = Vec::new();

    for line in text.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let mut tokens = line.split_whitespace();
        match tokens.next() {
            Some("v") => {
                let coords: Vec<f32> = tokens.filter_map(|t| t.parse::<f32>().ok()).collect();
                if coords.len() < 3 {
                    return Err(format!("vértice inválido: '{line}'"));
                }
                positions.push(Vec3::new(coords[0], coords[1], coords[2]));
            }
            Some("f") => {
                let idx: Vec<usize> = tokens
                    .filter_map(|t| parse_face_index(t, positions.len()))
                    .collect();
                if idx.len() < 3 {
                    return Err(format!("cara inválida: '{line}'"));
                }
                // Triangulación en abanico para caras con más de 3 vértices.
                for k in 1..idx.len() - 1 {
                    triangles.push([idx[0], idx[k], idx[k + 1]]);
                }
            }
            _ => {}
        }
    }

    if positions.is_empty() {
        return Err("OBJ sin vértices".to_string());
    }
    if triangles.is_empty() {
        return Err("OBJ sin caras".to_string());
    }

    let mut normals = vec![Vec3::ZERO; positions.len()];
    for tri in &triangles {
        for &i in tri {
            if i >= positions.len() {
                return Err(format!("índice de cara fuera de rango: {i}"));
            }
        }
        let (a, b, c) = (positions[tri[0]], positions[tri[1]], positions[tri[2]]);
        let face_normal = (b - a).cross(c - a);
        normals[tri[0]] = normals[tri[0]] + face_normal;
        normals[tri[1]] = normals[tri[1]] + face_normal;
        normals[tri[2]] = normals[tri[2]] + face_normal;
    }

    let vertices = positions
        .iter()
        .zip(normals.iter())
        .map(|(&position, &n)| {
            let normal = if n.length_squared() > 1e-12 { n.normalize() } else { Vec3::Y };
            Vertex { position, normal }
        })
        .collect();

    let indices = triangles.into_iter().map(|t| [t[0] as u32, t[1] as u32, t[2] as u32]).collect();

    Ok(Mesh { vertices, indices })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_single_triangle() {
        let obj = "v 0 0 0\nv 1 0 0\nv 0 1 0\nf 1 2 3\n";
        let mesh = parse_obj(obj).unwrap();
        assert_eq!(mesh.vertices.len(), 3);
        assert_eq!(mesh.indices, vec![[0, 1, 2]]);
        // Normal calculada: +Z para este triángulo en el plano XY con este orden.
        assert!(mesh.vertices[0].normal.z > 0.9);
    }

    #[test]
    fn triangulates_quad_as_fan() {
        let obj = "v 0 0 0\nv 1 0 0\nv 1 1 0\nv 0 1 0\nf 1 2 3 4\n";
        let mesh = parse_obj(obj).unwrap();
        assert_eq!(mesh.indices.len(), 2);
        assert_eq!(mesh.indices[0], [0, 1, 2]);
        assert_eq!(mesh.indices[1], [0, 2, 3]);
    }

    #[test]
    fn rejects_empty_input() {
        assert!(parse_obj("").is_err());
        assert!(parse_obj("v 0 0 0\n").is_err());
    }

    #[test]
    fn loads_fantasticar_asset() {
        let path = concat!(env!("CARGO_MANIFEST_DIR"), "/assets/models/fantasticar.obj");
        let mesh = load_obj(path).expect("el modelo de la nave debe cargar");
        assert!(!mesh.vertices.is_empty());
        assert!(!mesh.indices.is_empty());
        for v in &mesh.vertices {
            assert!((v.normal.length() - 1.0).abs() < 1e-3, "normal no normalizada: {:?}", v.normal);
        }
    }
}
