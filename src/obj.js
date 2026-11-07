import { cross, normalize, sub, vec3 } from './math.js';

// Minimal OBJ loader written for this project. It supports v, vt, vn and
// polygonal faces. Faces are triangulated and expanded to keep flat normals
// correct even when an OBJ does not provide vn records.
export function parseOBJ(text) {
  const sourcePositions = [vec3()];
  const sourceUVs = [{ u: 0, v: 0 }];
  const sourceNormals = [vec3(0, 1, 0)];
  const positions = [];
  const normals = [];
  const uvs = [];
  const indices = [];

  const resolve = (value, size) => {
    const index = Number(value || 0);
    return index < 0 ? size + index : index;
  };

  for (const raw of text.split(/\r?\n/)) {
    const line = raw.trim();
    if (!line || line.startsWith('#')) continue;
    const parts = line.split(/\s+/);
    const command = parts.shift();
    if (command === 'v') {
      sourcePositions.push(vec3(Number(parts[0]), Number(parts[1]), Number(parts[2])));
    } else if (command === 'vt') {
      sourceUVs.push({ u: Number(parts[0]), v: 1 - Number(parts[1]) });
    } else if (command === 'vn') {
      sourceNormals.push(normalize(vec3(Number(parts[0]), Number(parts[1]), Number(parts[2]))));
    } else if (command === 'f') {
      const face = parts.map((token) => token.split('/'));
      for (let i = 1; i < face.length - 1; i++) {
        const triangle = [face[0], face[i], face[i + 1]];
        const triPositions = triangle.map((v) => sourcePositions[resolve(v[0], sourcePositions.length)]);
        const flatNormal = normalize(cross(sub(triPositions[1], triPositions[0]), sub(triPositions[2], triPositions[0])));
        for (let corner = 0; corner < 3; corner++) {
          const spec = triangle[corner];
          const position = triPositions[corner];
          const uv = sourceUVs[resolve(spec[1], sourceUVs.length)] || sourceUVs[0];
          const normal = spec[2] ? sourceNormals[resolve(spec[2], sourceNormals.length)] : flatNormal;
          positions.push(position);
          normals.push(normal);
          uvs.push(uv);
          indices.push(indices.length);
        }
      }
    }
  }
  let boundsRadius = 0;
  for (const p of positions) boundsRadius = Math.max(boundsRadius, Math.hypot(p.x, p.y, p.z));
  return { positions, normals, uvs, indices, boundsRadius, name: 'OBJ mesh' };
}

export async function loadOBJ(url) {
  const response = await fetch(url);
  if (!response.ok) throw new Error(`No se pudo cargar ${url}: ${response.status}`);
  return parseOBJ(await response.text());
}
