import test from 'node:test';
import assert from 'node:assert/strict';
import { mat4Translation, transform4, vec3 } from '../src/math.js';
import { createPlanetLOD, createUVSphere } from '../src/meshes.js';
import { parseOBJ } from '../src/obj.js';

test('la matriz de traslación transforma un punto', () => {
  const result = transform4(mat4Translation(vec3(1, 2, 3)), vec3(4, 5, 6));
  assert.deepEqual({ x: result.x, y: result.y, z: result.z }, { x: 5, y: 7, z: 9 });
});

test('la esfera procedural genera triángulos y normales', () => {
  const sphere = createUVSphere(12, 8);
  assert.ok(sphere.positions.length > 80);
  assert.equal(sphere.positions.length, sphere.normals.length);
  assert.equal(sphere.indices.length % 3, 0);
  assert.ok(sphere.boundsRadius > 0.99 && sphere.boundsRadius < 1.01);
});

test('los tres niveles de detalle cambian el trabajo del renderer', () => {
  const lod = createPlanetLOD();
  assert.ok(lod.low.indices.length < lod.medium.indices.length);
  assert.ok(lod.medium.indices.length < lod.high.indices.length);
});

test('el cargador OBJ triangula polígonos', () => {
  const mesh = parseOBJ(`
v -1 -1 0
v 1 -1 0
v 1 1 0
v -1 1 0
f 1 2 3 4
`);
  assert.equal(mesh.indices.length, 6);
  assert.equal(mesh.positions.length, 6);
});
