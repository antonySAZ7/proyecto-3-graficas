//! Matemática propia: vectores, matrices 4x4 y cuaterniones.
//! Convención: matrices columna-mayor (como OpenGL/glam), m[col*4+fila].

use std::ops::{Add, Div, Mul, Sub};

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Vec3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl Vec3 {
    pub const ZERO: Vec3 = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
    pub const X: Vec3 = Vec3 { x: 1.0, y: 0.0, z: 0.0 };
    pub const Y: Vec3 = Vec3 { x: 0.0, y: 1.0, z: 0.0 };
    pub const Z: Vec3 = Vec3 { x: 0.0, y: 0.0, z: 1.0 };

    pub const fn new(x: f32, y: f32, z: f32) -> Self {
        Self { x, y, z }
    }

    pub const fn splat(v: f32) -> Self {
        Self { x: v, y: v, z: v }
    }

    pub fn dot(self, o: Vec3) -> f32 {
        self.x * o.x + self.y * o.y + self.z * o.z
    }

    pub fn cross(self, o: Vec3) -> Vec3 {
        Vec3::new(
            self.y * o.z - self.z * o.y,
            self.z * o.x - self.x * o.z,
            self.x * o.y - self.y * o.x,
        )
    }

    pub fn length_squared(self) -> f32 {
        self.dot(self)
    }

    pub fn length(self) -> f32 {
        self.length_squared().sqrt()
    }

    pub fn distance(self, o: Vec3) -> f32 {
        (self - o).length()
    }

    /// Normaliza; devuelve el vector cero si la longitud es despreciable (en vez de NaN/panic).
    pub fn normalize(self) -> Vec3 {
        let len = self.length();
        if len > 1e-6 { self * (1.0 / len) } else { Vec3::ZERO }
    }

    pub fn lerp(self, o: Vec3, t: f32) -> Vec3 {
        self + (o - self) * t
    }

    pub fn extend(self, w: f32) -> Vec4 {
        Vec4::new(self.x, self.y, self.z, w)
    }
}

impl Add for Vec3 {
    type Output = Vec3;
    fn add(self, o: Vec3) -> Vec3 {
        Vec3::new(self.x + o.x, self.y + o.y, self.z + o.z)
    }
}
impl Sub for Vec3 {
    type Output = Vec3;
    fn sub(self, o: Vec3) -> Vec3 {
        Vec3::new(self.x - o.x, self.y - o.y, self.z - o.z)
    }
}
impl Mul<f32> for Vec3 {
    type Output = Vec3;
    fn mul(self, s: f32) -> Vec3 {
        Vec3::new(self.x * s, self.y * s, self.z * s)
    }
}
impl Div<f32> for Vec3 {
    type Output = Vec3;
    fn div(self, s: f32) -> Vec3 {
        Vec3::new(self.x / s, self.y / s, self.z / s)
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Vec4 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub w: f32,
}

impl Vec4 {
    pub const fn new(x: f32, y: f32, z: f32, w: f32) -> Self {
        Self { x, y, z, w }
    }
}

/// Matriz 4x4 columna-mayor: `m[columna * 4 + fila]`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Mat4 {
    pub m: [f32; 16],
}

impl Mat4 {
    pub const IDENTITY: Mat4 = Mat4 {
        m: [1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0],
    };

    #[inline]
    fn get(&self, r: usize, c: usize) -> f32 {
        self.m[c * 4 + r]
    }

    #[inline]
    fn set(&mut self, r: usize, c: usize, v: f32) {
        self.m[c * 4 + r] = v;
    }

    pub fn from_translation(t: Vec3) -> Mat4 {
        let mut out = Mat4::IDENTITY;
        out.set(0, 3, t.x);
        out.set(1, 3, t.y);
        out.set(2, 3, t.z);
        out
    }

    pub fn from_scale(s: Vec3) -> Mat4 {
        let mut out = Mat4::IDENTITY;
        out.set(0, 0, s.x);
        out.set(1, 1, s.y);
        out.set(2, 2, s.z);
        out
    }

    /// Proyección en perspectiva de mano derecha (cámara mirando hacia -Z), z' en [-1, 1].
    pub fn perspective_rh(fov_y_radians: f32, aspect: f32, near: f32, far: f32) -> Mat4 {
        let f = 1.0 / (fov_y_radians * 0.5).tan();
        let nf = 1.0 / (near - far);
        Mat4 {
            m: [
                f / aspect, 0.0, 0.0, 0.0,
                0.0, f, 0.0, 0.0,
                0.0, 0.0, (far + near) * nf, -1.0,
                0.0, 0.0, 2.0 * far * near * nf, 0.0,
            ],
        }
    }

    /// Matriz de vista de mano derecha.
    pub fn look_at_rh(eye: Vec3, target: Vec3, up_hint: Vec3) -> Mat4 {
        let z = (eye - target).normalize();
        let mut x = up_hint.cross(z).normalize();
        if x.length_squared() < 1e-6 {
            x = Vec3::X;
        }
        let y = z.cross(x);
        Mat4 {
            m: [
                x.x, y.x, z.x, 0.0,
                x.y, y.y, z.y, 0.0,
                x.z, y.z, z.z, 0.0,
                -x.dot(eye), -y.dot(eye), -z.dot(eye), 1.0,
            ],
        }
    }

    /// Base ortonormal que alinea el eje Y local a `normal`, ubicada en `position`.
    /// Útil para orientar objetos sobre la superficie curva de un planeta (sin gimbal lock).
    pub fn align_y_to_normal(normal: Vec3, position: Vec3) -> Mat4 {
        let y = normal.normalize();
        let helper = if y.y.abs() > 0.92 { Vec3::X } else { Vec3::Y };
        let x = helper.cross(y).normalize();
        let z = x.cross(y).normalize();
        Mat4 {
            m: [
                x.x, x.y, x.z, 0.0,
                y.x, y.y, y.z, 0.0,
                z.x, z.y, z.z, 0.0,
                position.x, position.y, position.z, 1.0,
            ],
        }
    }

    pub fn transpose(&self) -> Mat4 {
        let mut out = Mat4::IDENTITY;
        for r in 0..4 {
            for c in 0..4 {
                out.set(c, r, self.get(r, c));
            }
        }
        out
    }

    /// Inversa general por eliminación Gauss-Jordan con pivoteo parcial.
    /// Los índices son el algoritmo en sí (pivote, fila, columna aumentada);
    /// forzarlo a iteradores lo haría menos legible.
    #[allow(clippy::needless_range_loop)]
    pub fn inverse(&self) -> Mat4 {
        let mut a = [[0f32; 8]; 4];
        for r in 0..4 {
            for c in 0..4 {
                a[r][c] = self.get(r, c);
            }
            a[r][4 + r] = 1.0;
        }

        for col in 0..4 {
            let mut pivot_row = col;
            let mut max_val = a[col][col].abs();
            for r in (col + 1)..4 {
                if a[r][col].abs() > max_val {
                    max_val = a[r][col].abs();
                    pivot_row = r;
                }
            }
            if pivot_row != col {
                a.swap(col, pivot_row);
            }
            let pivot = a[col][col];
            if pivot.abs() < 1e-12 {
                continue; // matriz singular: deja esa columna como está (mejor que dividir por cero)
            }
            for c2 in 0..8 {
                a[col][c2] /= pivot;
            }
            for r in 0..4 {
                if r != col {
                    let factor = a[r][col];
                    for c2 in 0..8 {
                        a[r][c2] -= factor * a[col][c2];
                    }
                }
            }
        }

        let mut out = Mat4::IDENTITY;
        for r in 0..4 {
            for c in 0..4 {
                out.set(r, c, a[r][4 + c]);
            }
        }
        out
    }

    /// Transforma un punto (w implícito = 1); asume matriz afín.
    pub fn transform_point3(&self, v: Vec3) -> Vec3 {
        Vec3::new(
            self.get(0, 0) * v.x + self.get(0, 1) * v.y + self.get(0, 2) * v.z + self.get(0, 3),
            self.get(1, 0) * v.x + self.get(1, 1) * v.y + self.get(1, 2) * v.z + self.get(1, 3),
            self.get(2, 0) * v.x + self.get(2, 1) * v.y + self.get(2, 2) * v.z + self.get(2, 3),
        )
    }

    /// Transforma una dirección (w implícito = 0): ignora traslación.
    pub fn transform_vector3(&self, v: Vec3) -> Vec3 {
        Vec3::new(
            self.get(0, 0) * v.x + self.get(0, 1) * v.y + self.get(0, 2) * v.z,
            self.get(1, 0) * v.x + self.get(1, 1) * v.y + self.get(1, 2) * v.z,
            self.get(2, 0) * v.x + self.get(2, 1) * v.y + self.get(2, 2) * v.z,
        )
    }
}

impl Mul for Mat4 {
    type Output = Mat4;
    fn mul(self, o: Mat4) -> Mat4 {
        let mut out = [0f32; 16];
        for c in 0..4 {
            for r in 0..4 {
                out[c * 4 + r] = (0..4).map(|k| self.get(r, k) * o.get(k, c)).sum();
            }
        }
        Mat4 { m: out }
    }
}

impl Mul<Vec4> for Mat4 {
    type Output = Vec4;
    fn mul(self, v: Vec4) -> Vec4 {
        Vec4::new(
            self.get(0, 0) * v.x + self.get(0, 1) * v.y + self.get(0, 2) * v.z + self.get(0, 3) * v.w,
            self.get(1, 0) * v.x + self.get(1, 1) * v.y + self.get(1, 2) * v.z + self.get(1, 3) * v.w,
            self.get(2, 0) * v.x + self.get(2, 1) * v.y + self.get(2, 2) * v.z + self.get(2, 3) * v.w,
            self.get(3, 0) * v.x + self.get(3, 1) * v.y + self.get(3, 2) * v.z + self.get(3, 3) * v.w,
        )
    }
}

/// Cuaternión para orientación sin gimbal lock (nave, cámara).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Quat {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub w: f32,
}

impl Quat {
    pub const IDENTITY: Quat = Quat { x: 0.0, y: 0.0, z: 0.0, w: 1.0 };

    pub fn from_axis_angle(axis: Vec3, angle: f32) -> Quat {
        let a = axis.normalize();
        let half = angle * 0.5;
        let s = half.sin();
        Quat { x: a.x * s, y: a.y * s, z: a.z * s, w: half.cos() }
    }

    pub fn length(self) -> f32 {
        (self.x * self.x + self.y * self.y + self.z * self.z + self.w * self.w).sqrt()
    }

    pub fn normalize(self) -> Quat {
        let len = self.length();
        if len > 1e-6 {
            Quat { x: self.x / len, y: self.y / len, z: self.z / len, w: self.w / len }
        } else {
            Quat::IDENTITY
        }
    }

    /// Parte del API de cuaterniones; hoy solo la ejercita el round-trip de prueba.
    #[allow(dead_code)]
    pub fn conjugate(self) -> Quat {
        Quat { x: -self.x, y: -self.y, z: -self.z, w: self.w }
    }

    /// Compone rotaciones: `(self * other)` rota primero por `other`, luego por `self`.
    pub fn mul(self, o: Quat) -> Quat {
        Quat {
            x: self.w * o.x + self.x * o.w + self.y * o.z - self.z * o.y,
            y: self.w * o.y - self.x * o.z + self.y * o.w + self.z * o.x,
            z: self.w * o.z + self.x * o.y - self.y * o.x + self.z * o.w,
            w: self.w * o.w - self.x * o.x - self.y * o.y - self.z * o.z,
        }
    }

    pub fn rotate_vec3(self, v: Vec3) -> Vec3 {
        let u = Vec3::new(self.x, self.y, self.z);
        let s = self.w;
        u * (2.0 * u.dot(v)) + v * (s * s - u.dot(u)) + u.cross(v) * (2.0 * s)
    }

    pub fn to_mat4(self) -> Mat4 {
        let right = self.rotate_vec3(Vec3::X);
        let up = self.rotate_vec3(Vec3::Y);
        let fwd = self.rotate_vec3(Vec3::Z);
        Mat4 {
            m: [
                right.x, right.y, right.z, 0.0,
                up.x, up.y, up.z, 0.0,
                fwd.x, fwd.y, fwd.z, 0.0,
                0.0, 0.0, 0.0, 1.0,
            ],
        }
    }
}

impl Mul for Quat {
    type Output = Quat;
    fn mul(self, o: Quat) -> Quat {
        Quat::mul(self, o)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn approx(a: f32, b: f32, eps: f32) -> bool {
        (a - b).abs() < eps
    }

    fn vec3_approx(a: Vec3, b: Vec3, eps: f32) -> bool {
        approx(a.x, b.x, eps) && approx(a.y, b.y, eps) && approx(a.z, b.z, eps)
    }

    #[test]
    fn vec3_dot_cross_basic() {
        let a = Vec3::new(1.0, 0.0, 0.0);
        let b = Vec3::new(0.0, 1.0, 0.0);
        assert_eq!(a.dot(b), 0.0);
        assert!(vec3_approx(a.cross(b), Vec3::Z, 1e-6));
    }

    #[test]
    fn vec3_normalize_and_lerp() {
        let v = Vec3::new(3.0, 0.0, 4.0);
        assert!(approx(v.normalize().length(), 1.0, 1e-5));
        assert!(vec3_approx(Vec3::ZERO.lerp(Vec3::new(10.0, 0.0, 0.0), 0.5), Vec3::new(5.0, 0.0, 0.0), 1e-6));
        assert_eq!(Vec3::ZERO.normalize(), Vec3::ZERO);
    }

    #[test]
    fn mat4_identity_leaves_point_unchanged() {
        let p = Vec3::new(1.0, 2.0, 3.0);
        assert!(vec3_approx(Mat4::IDENTITY.transform_point3(p), p, 1e-6));
    }

    #[test]
    fn mat4_translation_and_scale() {
        let m = Mat4::from_translation(Vec3::new(1.0, 2.0, 3.0)) * Mat4::from_scale(Vec3::splat(2.0));
        let p = m.transform_point3(Vec3::new(1.0, 1.0, 1.0));
        assert!(vec3_approx(p, Vec3::new(3.0, 4.0, 5.0), 1e-5));
    }

    #[test]
    fn mat4_inverse_round_trip() {
        let m = Mat4::from_translation(Vec3::new(2.0, -1.0, 5.0))
            * Quat::from_axis_angle(Vec3::Y, 0.7).to_mat4()
            * Mat4::from_scale(Vec3::new(1.5, 0.5, 2.0));
        let identity = m * m.inverse();
        for i in 0..16 {
            let expected = Mat4::IDENTITY.m[i];
            assert!(approx(identity.m[i], expected, 1e-4), "índice {i}: {} vs {}", identity.m[i], expected);
        }
    }

    #[test]
    fn mat4_inverse_transpose_keeps_normal_perpendicular() {
        // El vector normal transformado con (model^-1)^T debe seguir siendo
        // perpendicular a un vector tangente transformado con `model` directamente.
        let model = Mat4::from_translation(Vec3::new(1.0, 0.0, 0.0))
            * Quat::from_axis_angle(Vec3::new(0.3, 1.0, 0.2), 1.1).to_mat4()
            * Mat4::from_scale(Vec3::new(2.0, 0.5, 3.0));
        let normal_mat = model.inverse().transpose();

        let n = Vec3::new(0.0, 1.0, 0.0);
        let t = Vec3::new(1.0, 0.0, 0.0); // perpendicular a n en espacio local

        let world_n = normal_mat.transform_vector3(n).normalize();
        let world_t = model.transform_vector3(t).normalize();
        assert!(approx(world_n.dot(world_t), 0.0, 1e-3));
    }

    #[test]
    fn quat_round_trip_rotation() {
        let q = Quat::from_axis_angle(Vec3::Y, 1.234);
        let v = Vec3::new(1.0, 2.0, 3.0);
        let rotated = q.rotate_vec3(v);
        let back = q.conjugate().rotate_vec3(rotated);
        assert!(vec3_approx(back, v, 1e-4));
        assert!(approx(rotated.length(), v.length(), 1e-4));
    }

    #[test]
    fn align_y_to_normal_maps_y_axis_to_normal() {
        let normal = Vec3::new(1.0, 1.0, 1.0).normalize();
        let m = Mat4::align_y_to_normal(normal, Vec3::ZERO);
        let mapped = m.transform_vector3(Vec3::Y).normalize();
        assert!(vec3_approx(mapped, normal, 1e-4));
    }

    #[test]
    fn perspective_point_in_front_has_positive_w() {
        let proj = Mat4::perspective_rh(45f32.to_radians(), 16.0 / 9.0, 0.1, 100.0);
        let view = Mat4::look_at_rh(Vec3::new(0.0, 0.0, 5.0), Vec3::ZERO, Vec3::Y);
        let clip = (proj * view) * Vec3::ZERO.extend(1.0);
        assert!(clip.w > 0.0);
    }
}
