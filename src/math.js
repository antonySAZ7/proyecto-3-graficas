export const EPSILON = 1e-6;

export function vec3(x = 0, y = 0, z = 0) {
  return { x, y, z };
}

export const add = (a, b) => vec3(a.x + b.x, a.y + b.y, a.z + b.z);
export const sub = (a, b) => vec3(a.x - b.x, a.y - b.y, a.z - b.z);
export const scale = (v, s) => vec3(v.x * s, v.y * s, v.z * s);
export const dot = (a, b) => a.x * b.x + a.y * b.y + a.z * b.z;
export const cross = (a, b) => vec3(
  a.y * b.z - a.z * b.y,
  a.z * b.x - a.x * b.z,
  a.x * b.y - a.y * b.x,
);
export const lengthSq = (v) => dot(v, v);
export const length = (v) => Math.sqrt(lengthSq(v));
export function normalize(v) {
  const len = length(v);
  return len > EPSILON ? scale(v, 1 / len) : vec3(0, 0, 0);
}
export const lerp = (a, b, t) => a + (b - a) * t;
export const lerp3 = (a, b, t) => vec3(lerp(a.x, b.x, t), lerp(a.y, b.y, t), lerp(a.z, b.z, t));
export const clamp = (v, lo, hi) => Math.max(lo, Math.min(hi, v));
export const smoothstep = (a, b, x) => {
  const t = clamp((x - a) / (b - a), 0, 1);
  return t * t * (3 - 2 * t);
};

// Matrices are column-major, matching the usual model * local-vector notation.
export function mat4Identity() {
  return new Float32Array([1, 0, 0, 0, 0, 1, 0, 0, 0, 0, 1, 0, 0, 0, 0, 1]);
}

export function mat4Multiply(a, b) {
  const out = new Float32Array(16);
  for (let c = 0; c < 4; c++) {
    for (let r = 0; r < 4; r++) {
      out[c * 4 + r] =
        a[0 * 4 + r] * b[c * 4 + 0] +
        a[1 * 4 + r] * b[c * 4 + 1] +
        a[2 * 4 + r] * b[c * 4 + 2] +
        a[3 * 4 + r] * b[c * 4 + 3];
    }
  }
  return out;
}

export function transform4(m, v, w = 1) {
  return {
    x: m[0] * v.x + m[4] * v.y + m[8] * v.z + m[12] * w,
    y: m[1] * v.x + m[5] * v.y + m[9] * v.z + m[13] * w,
    z: m[2] * v.x + m[6] * v.y + m[10] * v.z + m[14] * w,
    w: m[3] * v.x + m[7] * v.y + m[11] * v.z + m[15] * w,
  };
}

export function transformDirection(m, v) {
  return normalize(vec3(
    m[0] * v.x + m[4] * v.y + m[8] * v.z,
    m[1] * v.x + m[5] * v.y + m[9] * v.z,
    m[2] * v.x + m[6] * v.y + m[10] * v.z,
  ));
}

export function mat4Translation(v) {
  const out = mat4Identity();
  out[12] = v.x;
  out[13] = v.y;
  out[14] = v.z;
  return out;
}

export function mat4Scale(x, y = x, z = x) {
  const out = mat4Identity();
  out[0] = x;
  out[5] = y;
  out[10] = z;
  return out;
}

export function mat4RotationX(a) {
  const c = Math.cos(a), s = Math.sin(a);
  return new Float32Array([1, 0, 0, 0, 0, c, s, 0, 0, -s, c, 0, 0, 0, 0, 1]);
}

export function mat4RotationY(a) {
  const c = Math.cos(a), s = Math.sin(a);
  return new Float32Array([c, 0, -s, 0, 0, 1, 0, 0, s, 0, c, 0, 0, 0, 0, 1]);
}

export function mat4RotationZ(a) {
  const c = Math.cos(a), s = Math.sin(a);
  return new Float32Array([c, s, 0, 0, -s, c, 0, 0, 0, 0, 1, 0, 0, 0, 0, 1]);
}

export function mat4FromTRS(position, rotation, uniformScale = 1) {
  const rz = mat4RotationZ(rotation.z);
  const rx = mat4RotationX(rotation.x);
  const ry = mat4RotationY(rotation.y);
  const r = mat4Multiply(ry, mat4Multiply(rx, rz));
  return mat4Multiply(mat4Translation(position), mat4Multiply(r, mat4Scale(uniformScale)));
}

export function mat4Perspective(fovY, aspect, near, far) {
  const f = 1 / Math.tan(fovY * 0.5);
  const nf = 1 / (near - far);
  return new Float32Array([
    f / aspect, 0, 0, 0,
    0, f, 0, 0,
    0, 0, (far + near) * nf, -1,
    0, 0, 2 * far * near * nf, 0,
  ]);
}

export function mat4LookAt(eye, target, upHint = vec3(0, 1, 0)) {
  const z = normalize(sub(eye, target));
  let x = normalize(cross(upHint, z));
  if (lengthSq(x) < EPSILON) x = vec3(1, 0, 0);
  const y = cross(z, x);
  return new Float32Array([
    x.x, y.x, z.x, 0,
    x.y, y.y, z.y, 0,
    x.z, y.z, z.z, 0,
    -dot(x, eye), -dot(y, eye), -dot(z, eye), 1,
  ]);
}

export function basisFromEuler(rotation) {
  const matrix = mat4Multiply(mat4RotationY(rotation.y), mat4Multiply(mat4RotationX(rotation.x), mat4RotationZ(rotation.z)));
  return {
    right: transformDirection(matrix, vec3(1, 0, 0)),
    up: transformDirection(matrix, vec3(0, 1, 0)),
    forward: transformDirection(matrix, vec3(0, 0, -1)),
    matrix,
  };
}

export function mat4AlignYToNormal(normal, position = vec3()) {
  const y = normalize(normal);
  const helper = Math.abs(y.y) > 0.92 ? vec3(1, 0, 0) : vec3(0, 1, 0);
  const x = normalize(cross(helper, y));
  const z = normalize(cross(x, y));
  return new Float32Array([
    x.x, x.y, x.z, 0,
    y.x, y.y, y.z, 0,
    z.x, z.y, z.z, 0,
    position.x, position.y, position.z, 1,
  ]);
}

export function rotateAroundY(v, angle) {
  const c = Math.cos(angle), s = Math.sin(angle);
  return vec3(v.x * c + v.z * s, v.y, -v.x * s + v.z * c);
}

export function hash3(x, y, z) {
  const n = Math.sin(x * 127.1 + y * 311.7 + z * 74.7) * 43758.5453123;
  return n - Math.floor(n);
}

export function fractalNoise(v) {
  let sum = 0, amp = 0.5, freq = 1;
  for (let i = 0; i < 4; i++) {
    sum += amp * Math.sin((v.x * 1.7 + v.y * 2.3 - v.z * 1.1) * freq + i * 2.17);
    amp *= 0.5;
    freq *= 2.03;
  }
  return sum;
}

export function mulColor(color, factor) {
  const r = clamp(((color >> 16) & 255) * factor, 0, 255) | 0;
  const g = clamp(((color >> 8) & 255) * factor, 0, 255) | 0;
  const b = clamp((color & 255) * factor, 0, 255) | 0;
  return (r << 16) | (g << 8) | b;
}

export function mixColor(a, b, t) {
  t = clamp(t, 0, 1);
  const r = lerp((a >> 16) & 255, (b >> 16) & 255, t) | 0;
  const g = lerp((a >> 8) & 255, (b >> 8) & 255, t) | 0;
  const bl = lerp(a & 255, b & 255, t) | 0;
  return (r << 16) | (g << 8) | bl;
}
