import {
  add, clamp, dot, lerp, lerp3, mat4Multiply, normalize, scale, sub,
  transform4, transformDirection, vec3,
} from './math.js';

function edge(a, b, p) {
  return (b.x - a.x) * (p.y - a.y) - (b.y - a.y) * (p.x - a.x);
}

function lerpVertex(a, b, t) {
  return {
    view: lerp3(a.view, b.view, t),
    world: lerp3(a.world, b.world, t),
    normal: normalize(lerp3(a.normal, b.normal, t)),
    u: lerp(a.u, b.u, t),
    v: lerp(a.v, b.v, t),
  };
}

function clipNear(vertices, near) {
  const output = [];
  for (let i = 0; i < vertices.length; i++) {
    const current = vertices[i];
    const previous = vertices[(i + vertices.length - 1) % vertices.length];
    const currentInside = current.view.z <= -near;
    const previousInside = previous.view.z <= -near;
    if (currentInside !== previousInside) {
      const t = (-near - previous.view.z) / (current.view.z - previous.view.z);
      output.push(lerpVertex(previous, current, t));
    }
    if (currentInside) output.push(current);
  }
  return output;
}

function seededStars(count) {
  let state = 0x4f414e54;
  const random = () => {
    state ^= state << 13;
    state ^= state >>> 17;
    state ^= state << 5;
    return (state >>> 0) / 4294967296;
  };
  const stars = [];
  for (let i = 0; i < count; i++) {
    const z = random() * 2 - 1;
    const a = random() * Math.PI * 2;
    const r = Math.sqrt(1 - z * z);
    stars.push({
      direction: vec3(Math.cos(a) * r, z, Math.sin(a) * r),
      brightness: 0.45 + random() * 0.55,
      tint: random(),
    });
  }
  return stars;
}

export class SoftwareRenderer {
  constructor(canvas, width = 480, height = 270) {
    this.canvas = canvas;
    this.context = canvas.getContext('2d', { alpha: false });
    this.width = width;
    this.height = height;
    canvas.width = width;
    canvas.height = height;
    this.image = this.context.createImageData(width, height);
    this.color = this.image.data;
    this.depth = new Float32Array(width * height);
    this.stars = seededStars(680);
    this.enableFrustumCulling = true;
    this.stats = {};
    this.resetStats();
  }

  resetStats() {
    this.stats = {
      objects: 0,
      objectsCulled: 0,
      trianglesSubmitted: 0,
      trianglesClipped: 0,
      trianglesRasterized: 0,
      fragments: 0,
    };
  }

  clear(top = 0x030617, bottom = 0x130b2d) {
    this.resetStats();
    this.depth.fill(Infinity);
    for (let y = 0; y < this.height; y++) {
      const t = y / (this.height - 1);
      const r = (((top >> 16) & 255) * (1 - t) + ((bottom >> 16) & 255) * t) | 0;
      const g = (((top >> 8) & 255) * (1 - t) + ((bottom >> 8) & 255) * t) | 0;
      const b = ((top & 255) * (1 - t) + (bottom & 255) * t) | 0;
      for (let x = 0; x < this.width; x++) {
        const i = (y * this.width + x) * 4;
        this.color[i] = r;
        this.color[i + 1] = g;
        this.color[i + 2] = b;
        this.color[i + 3] = 255;
      }
    }
  }

  putPixel(x, y, packed, z = Infinity) {
    x |= 0;
    y |= 0;
    if (x < 0 || y < 0 || x >= this.width || y >= this.height) return;
    const pixel = y * this.width + x;
    if (z !== Infinity && z >= this.depth[pixel]) return;
    if (z !== Infinity) this.depth[pixel] = z;
    const i = pixel * 4;
    this.color[i] = (packed >> 16) & 255;
    this.color[i + 1] = (packed >> 8) & 255;
    this.color[i + 2] = packed & 255;
    this.color[i + 3] = 255;
  }

  drawStarfield(camera, warp = 0) {
    const center = { x: this.width * 0.5, y: this.height * 0.5 };
    for (const star of this.stars) {
      const view = transform4(camera.view, star.direction, 0);
      if (view.z > -0.04) continue;
      const clip = transform4(camera.projection, view, 1);
      if (Math.abs(clip.w) < 1e-5) continue;
      const x = (clip.x / clip.w * 0.5 + 0.5) * this.width;
      const y = (1 - (clip.y / clip.w * 0.5 + 0.5)) * this.height;
      if (x < 0 || y < 0 || x >= this.width || y >= this.height) continue;
      const warm = star.tint > 0.82;
      const cold = star.tint < 0.25;
      const value = (150 + 105 * star.brightness) | 0;
      const color = warm
        ? (value << 16) | ((value * 0.72) << 8) | (value * 0.55)
        : cold
          ? ((value * 0.58) << 16) | ((value * 0.8) << 8) | value
          : (value << 16) | (value << 8) | value;
      if (warp > 0.02) {
        const dx = x - center.x, dy = y - center.y;
        const inv = 1 / Math.max(1, Math.hypot(dx, dy));
        const streak = 2 + warp * 20 * star.brightness;
        this.drawLine2D(x - dx * inv * streak, y - dy * inv * streak, x, y, color);
      } else {
        this.putPixel(x, y, color);
        if (star.brightness > 0.92) this.putPixel(x + 1, y, color);
      }
    }
  }

  isSphereVisible(center, radius, camera) {
    const v = transform4(camera.view, center, 1);
    const depth = -v.z;
    if (depth + radius < camera.near || depth - radius > camera.far) return false;
    if (depth <= 0) return false;
    const tanY = Math.tan(camera.fovY * 0.5);
    const tanX = tanY * camera.aspect;
    if (Math.abs(v.x) - radius > depth * tanX) return false;
    if (Math.abs(v.y) - radius > depth * tanY) return false;
    return true;
  }

  drawMesh(mesh, model, material, camera, scene) {
    this.stats.objects++;
    const center = vec3(model[12], model[13], model[14]);
    const sx = Math.hypot(model[0], model[1], model[2]);
    const sy = Math.hypot(model[4], model[5], model[6]);
    const sz = Math.hypot(model[8], model[9], model[10]);
    const radius = mesh.boundsRadius * Math.max(sx, sy, sz);
    if (this.enableFrustumCulling && !material.noCullObject && !this.isSphereVisible(center, radius, camera)) {
      this.stats.objectsCulled++;
      return;
    }

    const transformed = new Array(mesh.positions.length);
    for (let i = 0; i < mesh.positions.length; i++) {
      const world4 = transform4(model, mesh.positions[i], 1);
      const world = vec3(world4.x, world4.y, world4.z);
      const view4 = transform4(camera.view, world, 1);
      transformed[i] = {
        world,
        view: vec3(view4.x, view4.y, view4.z),
        normal: transformDirection(model, mesh.normals[i]),
        u: mesh.uvs[i]?.u || 0,
        v: mesh.uvs[i]?.v || 0,
      };
    }

    for (let i = 0; i < mesh.indices.length; i += 3) {
      this.stats.trianglesSubmitted++;
      const polygon = clipNear([
        transformed[mesh.indices[i]],
        transformed[mesh.indices[i + 1]],
        transformed[mesh.indices[i + 2]],
      ], camera.near);
      if (polygon.length < 3) continue;
      if (polygon.length !== 3) this.stats.trianglesClipped++;
      for (let j = 1; j < polygon.length - 1; j++) {
        this.rasterizeTriangle(polygon[0], polygon[j], polygon[j + 1], material, camera, scene);
      }
    }
  }

  projectVertex(vertex, projection) {
    const clip = transform4(projection, vertex.view, 1);
    if (Math.abs(clip.w) < 1e-6) return null;
    const invW = 1 / clip.w;
    return {
      ...vertex,
      invW,
      x: (clip.x * invW * 0.5 + 0.5) * this.width,
      y: (1 - (clip.y * invW * 0.5 + 0.5)) * this.height,
      depth: clip.z * invW * 0.5 + 0.5,
    };
  }

  rasterizeTriangle(a, b, c, material, camera, scene) {
    const p0 = this.projectVertex(a, camera.projection);
    const p1 = this.projectVertex(b, camera.projection);
    const p2 = this.projectVertex(c, camera.projection);
    if (!p0 || !p1 || !p2) return;
    const area = edge(p0, p1, p2);
    if (Math.abs(area) < 0.01) return;
    // Canvas Y points down, therefore front-facing CCW triangles have negative area.
    if (!material.doubleSided && area > 0) return;

    const minX = Math.max(0, Math.floor(Math.min(p0.x, p1.x, p2.x)));
    const maxX = Math.min(this.width - 1, Math.ceil(Math.max(p0.x, p1.x, p2.x)));
    const minY = Math.max(0, Math.floor(Math.min(p0.y, p1.y, p2.y)));
    const maxY = Math.min(this.height - 1, Math.ceil(Math.max(p0.y, p1.y, p2.y)));
    if (minX > maxX || minY > maxY) return;
    this.stats.trianglesRasterized++;

    for (let y = minY; y <= maxY; y++) {
      for (let x = minX; x <= maxX; x++) {
        const sample = { x: x + 0.5, y: y + 0.5 };
        const l0 = edge(p1, p2, sample) / area;
        const l1 = edge(p2, p0, sample) / area;
        const l2 = 1 - l0 - l1;
        if (l0 < -1e-6 || l1 < -1e-6 || l2 < -1e-6) continue;
        const depth = l0 * p0.depth + l1 * p1.depth + l2 * p2.depth;
        if (depth < 0 || depth > 1) continue;
        const pixel = y * this.width + x;
        if (depth >= this.depth[pixel]) continue;

        const denominator = l0 * p0.invW + l1 * p1.invW + l2 * p2.invW;
        if (Math.abs(denominator) < 1e-9) continue;
        const w0 = l0 * p0.invW / denominator;
        const w1 = l1 * p1.invW / denominator;
        const w2 = l2 * p2.invW / denominator;
        const fragment = {
          world: vec3(
            p0.world.x * w0 + p1.world.x * w1 + p2.world.x * w2,
            p0.world.y * w0 + p1.world.y * w1 + p2.world.y * w2,
            p0.world.z * w0 + p1.world.z * w1 + p2.world.z * w2,
          ),
          normal: normalize(vec3(
            p0.normal.x * w0 + p1.normal.x * w1 + p2.normal.x * w2,
            p0.normal.y * w0 + p1.normal.y * w1 + p2.normal.y * w2,
            p0.normal.z * w0 + p1.normal.z * w1 + p2.normal.z * w2,
          )),
          u: p0.u * w0 + p1.u * w1 + p2.u * w2,
          v: p0.v * w0 + p1.v * w1 + p2.v * w2,
          viewDirection: normalize(sub(camera.position, vec3(
            p0.world.x * w0 + p1.world.x * w1 + p2.world.x * w2,
            p0.world.y * w0 + p1.world.y * w1 + p2.world.y * w2,
            p0.world.z * w0 + p1.world.z * w1 + p2.world.z * w2,
          ))),
        };
        const packed = material.shade(fragment, scene);
        this.depth[pixel] = depth;
        const out = pixel * 4;
        this.color[out] = (packed >> 16) & 255;
        this.color[out + 1] = (packed >> 8) & 255;
        this.color[out + 2] = packed & 255;
        this.color[out + 3] = 255;
        this.stats.fragments++;
      }
    }
  }

  projectWorld(point, camera) {
    const view = transform4(camera.view, point, 1);
    if (view.z > -camera.near) return null;
    const clip = transform4(camera.projection, view, 1);
    if (Math.abs(clip.w) < 1e-6) return null;
    const invW = 1 / clip.w;
    return {
      x: (clip.x * invW * 0.5 + 0.5) * this.width,
      y: (1 - (clip.y * invW * 0.5 + 0.5)) * this.height,
      z: clip.z * invW * 0.5 + 0.5,
    };
  }

  drawLine3D(a, b, color, camera, glow = 0) {
    const p0 = this.projectWorld(a, camera);
    const p1 = this.projectWorld(b, camera);
    if (!p0 || !p1) return;
    for (let o = -glow; o <= glow; o++) {
      this.drawLine2D(p0.x + o, p0.y, p1.x + o, p1.y, color, p0.z, p1.z);
    }
  }

  drawLine2D(x0, y0, x1, y1, color, z0 = Infinity, z1 = Infinity) {
    const steps = Math.max(1, Math.ceil(Math.max(Math.abs(x1 - x0), Math.abs(y1 - y0))));
    for (let i = 0; i <= steps; i++) {
      const t = i / steps;
      this.putPixel(lerp(x0, x1, t), lerp(y0, y1, t), color, lerp(z0, z1, t));
    }
  }

  present() {
    this.context.putImageData(this.image, 0, 0);
  }
}

export function makeCamera(position, target, view, projection, near, far, fovY, aspect) {
  return { position, target, view, projection, near, far, fovY, aspect };
}
