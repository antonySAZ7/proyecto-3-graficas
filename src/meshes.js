import {
  fractalNoise, mat4Multiply, mat4Scale, mat4Translation, normalize,
  transform4, transformDirection, vec3,
} from './math.js';

function finishMesh(name, positions, normals, uvs, indices) {
  let boundsRadius = 0;
  for (const p of positions) boundsRadius = Math.max(boundsRadius, Math.hypot(p.x, p.y, p.z));
  return { name, positions, normals, uvs, indices, boundsRadius };
}

export function createUVSphere(segments = 16, rings = 10, radius = 1, deform = null) {
  const positions = [], normals = [], uvs = [], indices = [];
  for (let y = 0; y <= rings; y++) {
    const v = y / rings;
    const phi = v * Math.PI;
    for (let x = 0; x <= segments; x++) {
      const u = x / segments;
      const theta = u * Math.PI * 2;
      const n = vec3(Math.sin(phi) * Math.cos(theta), Math.cos(phi), Math.sin(phi) * Math.sin(theta));
      const localRadius = radius * (deform ? deform(n, u, v) : 1);
      positions.push(vec3(n.x * localRadius, n.y * localRadius, n.z * localRadius));
      normals.push(normalize(n));
      uvs.push({ u, v });
    }
  }
  const stride = segments + 1;
  for (let y = 0; y < rings; y++) {
    for (let x = 0; x < segments; x++) {
      const a = y * stride + x;
      const b = (y + 1) * stride + x;
      const c = b + 1;
      const d = a + 1;
      if (y > 0) indices.push(a, d, b);
      if (y < rings - 1) indices.push(d, c, b);
    }
  }
  return finishMesh(`UV sphere ${segments}x${rings}`, positions, normals, uvs, indices);
}

export function createPlanetLOD(deformed = false) {
  const modifier = deformed
    ? (n) => 1 + 0.12 * fractalNoise({ x: n.x * 2.3, y: n.y * 2.3, z: n.z * 2.3 })
    : null;
  return {
    low: createUVSphere(10, 6, 1, modifier),
    medium: createUVSphere(18, 10, 1, modifier),
    high: createUVSphere(30, 18, 1, modifier),
  };
}

export function createCube(size = 1) {
  const h = size * 0.5;
  const positions = [], normals = [], uvs = [], indices = [];
  const faces = [
    [vec3(0, 0, 1), [vec3(-h, -h, h), vec3(h, -h, h), vec3(h, h, h), vec3(-h, h, h)]],
    [vec3(0, 0, -1), [vec3(h, -h, -h), vec3(-h, -h, -h), vec3(-h, h, -h), vec3(h, h, -h)]],
    [vec3(1, 0, 0), [vec3(h, -h, h), vec3(h, -h, -h), vec3(h, h, -h), vec3(h, h, h)]],
    [vec3(-1, 0, 0), [vec3(-h, -h, -h), vec3(-h, -h, h), vec3(-h, h, h), vec3(-h, h, -h)]],
    [vec3(0, 1, 0), [vec3(-h, h, h), vec3(h, h, h), vec3(h, h, -h), vec3(-h, h, -h)]],
    [vec3(0, -1, 0), [vec3(-h, -h, -h), vec3(h, -h, -h), vec3(h, -h, h), vec3(-h, -h, h)]],
  ];
  for (const [normal, corners] of faces) {
    const base = positions.length;
    positions.push(...corners);
    normals.push(normal, normal, normal, normal);
    uvs.push({ u: 0, v: 1 }, { u: 1, v: 1 }, { u: 1, v: 0 }, { u: 0, v: 0 });
    indices.push(base, base + 1, base + 2, base, base + 2, base + 3);
  }
  return finishMesh('cube', positions, normals, uvs, indices);
}

export function createCone(segments = 10, radius = 1, height = 2) {
  const positions = [], normals = [], uvs = [], indices = [];
  const half = height * 0.5;
  for (let i = 0; i < segments; i++) {
    const a0 = i / segments * Math.PI * 2;
    const a1 = (i + 1) / segments * Math.PI * 2;
    const p0 = vec3(Math.cos(a0) * radius, -half, Math.sin(a0) * radius);
    const p1 = vec3(Math.cos(a1) * radius, -half, Math.sin(a1) * radius);
    const sideNormal0 = normalize(vec3(Math.cos(a0), radius / height, Math.sin(a0)));
    const sideNormal1 = normalize(vec3(Math.cos(a1), radius / height, Math.sin(a1)));
    let base = positions.length;
    positions.push(p0, p1, vec3(0, half, 0));
    normals.push(sideNormal0, sideNormal1, normalize(vec3(Math.cos((a0 + a1) * 0.5), radius / height, Math.sin((a0 + a1) * 0.5))));
    uvs.push({ u: i / segments, v: 1 }, { u: (i + 1) / segments, v: 1 }, { u: (i + 0.5) / segments, v: 0 });
    indices.push(base, base + 1, base + 2);
    base = positions.length;
    positions.push(vec3(0, -half, 0), p1, p0);
    normals.push(vec3(0, -1, 0), vec3(0, -1, 0), vec3(0, -1, 0));
    uvs.push({ u: 0.5, v: 0.5 }, { u: 1, v: 1 }, { u: 0, v: 1 });
    indices.push(base, base + 1, base + 2);
  }
  return finishMesh('cone', positions, normals, uvs, indices);
}

export function createCylinder(segments = 10, radius = 1, height = 2) {
  const positions = [], normals = [], uvs = [], indices = [];
  const half = height * 0.5;
  for (let i = 0; i < segments; i++) {
    const a0 = i / segments * Math.PI * 2;
    const a1 = (i + 1) / segments * Math.PI * 2;
    const n0 = vec3(Math.cos(a0), 0, Math.sin(a0));
    const n1 = vec3(Math.cos(a1), 0, Math.sin(a1));
    const p00 = vec3(n0.x * radius, -half, n0.z * radius);
    const p01 = vec3(n0.x * radius, half, n0.z * radius);
    const p10 = vec3(n1.x * radius, -half, n1.z * radius);
    const p11 = vec3(n1.x * radius, half, n1.z * radius);
    let base = positions.length;
    positions.push(p00, p10, p11, p01);
    normals.push(n0, n1, n1, n0);
    uvs.push({ u: i / segments, v: 1 }, { u: (i + 1) / segments, v: 1 }, { u: (i + 1) / segments, v: 0 }, { u: i / segments, v: 0 });
    indices.push(base, base + 1, base + 2, base, base + 2, base + 3);
    base = positions.length;
    positions.push(vec3(0, half, 0), p01, p11, vec3(0, -half, 0), p10, p00);
    normals.push(vec3(0, 1, 0), vec3(0, 1, 0), vec3(0, 1, 0), vec3(0, -1, 0), vec3(0, -1, 0), vec3(0, -1, 0));
    uvs.push({ u: 0.5, v: 0.5 }, { u: 0, v: 1 }, { u: 1, v: 1 }, { u: 0.5, v: 0.5 }, { u: 1, v: 1 }, { u: 0, v: 1 });
    indices.push(base, base + 1, base + 2, base + 3, base + 4, base + 5);
  }
  return finishMesh('cylinder', positions, normals, uvs, indices);
}

export function transformMesh(mesh, matrix) {
  return finishMesh(
    mesh.name,
    mesh.positions.map((p) => {
      const q = transform4(matrix, p, 1);
      return vec3(q.x, q.y, q.z);
    }),
    mesh.normals.map((n) => transformDirection(matrix, n)),
    mesh.uvs.map((uv) => ({ ...uv })),
    [...mesh.indices],
  );
}

export function mergeMeshes(name, meshes) {
  const positions = [], normals = [], uvs = [], indices = [];
  for (const mesh of meshes) {
    const offset = positions.length;
    positions.push(...mesh.positions);
    normals.push(...mesh.normals);
    uvs.push(...mesh.uvs);
    for (const i of mesh.indices) indices.push(i + offset);
  }
  return finishMesh(name, positions, normals, uvs, indices);
}

export function createSurfaceModels() {
  const trunk = transformMesh(createCylinder(6, 0.16, 0.8), mat4Translation(vec3(0, 0.4, 0)));
  const crown = transformMesh(createCone(8, 0.55, 1.25), mat4Translation(vec3(0, 1.15, 0)));
  const tree = mergeMeshes('crystal tree', [trunk, crown]);

  const rock = createUVSphere(7, 4, 0.65, (n) => 1 + fractalNoise({ x: n.x * 3, y: n.y * 3, z: n.z * 3 }) * 0.22);

  const towerBase = transformMesh(createCube(1), mat4Multiply(mat4Translation(vec3(0, 0.55, 0)), mat4Scale(0.62, 1.1, 0.62)));
  const towerTop = transformMesh(createCone(6, 0.42, 0.7), mat4Translation(vec3(0, 1.45, 0)));
  const tower = mergeMeshes('Baxter tower', [towerBase, towerTop]);

  const beaconStem = transformMesh(createCylinder(6, 0.12, 0.9), mat4Translation(vec3(0, 0.45, 0)));
  const beaconCore = transformMesh(createUVSphere(8, 5, 0.32), mat4Translation(vec3(0, 1.05, 0)));
  const beacon = mergeMeshes('shield beacon', [beaconStem, beaconCore]);

  return { tree, rock, tower, beacon };
}

export function chooseLOD(lod, distance, radius) {
  const ratio = distance / Math.max(radius, 0.1);
  if (ratio < 5.5) return { mesh: lod.high, level: 'ALTO' };
  if (ratio < 12) return { mesh: lod.medium, level: 'MEDIO' };
  return { mesh: lod.low, level: 'BAJO' };
}
