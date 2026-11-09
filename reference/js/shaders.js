import {
  add, clamp, dot, fractalNoise, mixColor, mulColor, normalize, scale,
  smoothstep, sub, vec3,
} from './math.js';

function litColor(base, fragment, scene, options = {}) {
  const lightDirection = normalize(sub(scene.sunPosition, fragment.world));
  const diffuse = Math.max(0, dot(fragment.normal, lightDirection));
  const halfVector = normalize(add(lightDirection, fragment.viewDirection));
  const specular = Math.pow(Math.max(0, dot(fragment.normal, halfVector)), options.shininess || 20) * (options.specular || 0.25);
  const rim = Math.pow(1 - Math.max(0, dot(fragment.normal, fragment.viewDirection)), 2) * (options.rim || 0);
  return mulColor(base, (options.ambient || 0.16) + diffuse * (options.diffuse || 0.84) + specular + rim);
}

export function earthMaterial() {
  return {
    name: 'Tierra: océano, continentes, nubes y normal mapping procedural',
    doubleSided: false,
    shade(fragment, scene) {
      const longitude = fragment.u * Math.PI * 2;
      const latitude = (fragment.v - 0.5) * Math.PI;
      const sample = {
        x: Math.cos(longitude) * Math.cos(latitude) * 2.4,
        y: Math.sin(latitude) * 2.4,
        z: Math.sin(longitude) * Math.cos(latitude) * 2.4,
      };
      const terrain = fractalNoise(sample) + Math.sin(longitude * 3 + Math.sin(latitude * 5)) * 0.12;
      const polar = smoothstep(0.73, 0.95, Math.abs(Math.sin(latitude)));
      let base;
      if (polar > 0.12) base = mixColor(0xb6e8f7, 0xf7fbff, polar);
      else if (terrain > 0.06) {
        const height = smoothstep(0.06, 0.72, terrain);
        base = mixColor(0x23824f, 0xb7a56d, height);
      } else {
        const depth = smoothstep(-0.8, 0.06, terrain);
        base = mixColor(0x041a55, 0x087fbd, depth);
      }

      // CPU normal mapping: a procedural height field perturbs the interpolated
      // geometric normal before the lighting equation.
      const bump = vec3(
        Math.sin(longitude * 28 + scene.time * 0.15),
        Math.sin(latitude * 25 - longitude * 4),
        Math.cos(longitude * 21 + latitude * 17),
      );
      const original = fragment.normal;
      fragment.normal = normalize(add(original, scale(bump, terrain > 0.06 ? 0.09 : 0.025)));
      let color = litColor(base, fragment, scene, { ambient: 0.2, specular: terrain < 0.06 ? 0.5 : 0.1, shininess: 34, rim: 0.22 });
      fragment.normal = original;
      const clouds = fractalNoise({ x: sample.x * 1.6 + scene.time * 0.025, y: sample.y * 1.6, z: sample.z * 1.6 });
      if (clouds > 0.54) color = mixColor(color, 0xe9fbff, smoothstep(0.54, 0.85, clouds) * 0.75);
      return color;
    },
  };
}

export function sunMaterial() {
  return {
    name: 'Sol: plasma animado',
    doubleSided: false,
    noCullObject: true,
    shade(fragment, scene) {
      const flow =
        Math.sin(fragment.u * 48 + scene.time * 3.1) * 0.32 +
        Math.sin(fragment.v * 35 - scene.time * 2.4) * 0.22 +
        fractalNoise({ x: fragment.u * 5, y: fragment.v * 5, z: scene.time * 0.22 }) * 0.45;
      const pulse = 0.5 + 0.5 * Math.sin(scene.time * 2 + fragment.v * 13);
      return mixColor(0xff5b12, 0xfff5a1, clamp(flow * 0.5 + pulse * 0.38 + 0.38, 0, 1));
    },
  };
}

export function lavaMaterial() {
  return {
    name: 'Mundo Forja: grietas de lava',
    doubleSided: false,
    shade(fragment, scene) {
      const ridge = Math.abs(Math.sin(fragment.u * 51 + Math.sin(fragment.v * 19) * 2.2));
      const fissure = smoothstep(0.84, 0.98, ridge + fractalNoise({ x: fragment.u * 5, y: fragment.v * 7, z: 2 }) * 0.18);
      const rock = litColor(mixColor(0x160b14, 0x582130, fractalNoise(fragment.world) * 0.5 + 0.5), fragment, scene, { ambient: 0.18, diffuse: 0.72 });
      const glow = mixColor(0xff3100, 0xffdb43, 0.5 + 0.5 * Math.sin(scene.time * 2.8 + fragment.u * 30));
      return mixColor(rock, glow, fissure);
    },
  };
}

export function iceMaterial() {
  return {
    name: 'Niflheim: hielo iridiscente',
    doubleSided: false,
    shade(fragment, scene) {
      const bands = 0.5 + 0.5 * Math.sin(fragment.v * 68 + fractalNoise(fragment.world) * 3);
      const base = mixColor(0x173a78, 0x9beaff, bands * 0.72);
      return litColor(base, fragment, scene, { ambient: 0.22, specular: 1.25, shininess: 60, rim: 0.42 });
    },
  };
}

export function metalWorldMaterial() {
  return {
    name: 'Mundo de Plata: circuitos',
    doubleSided: false,
    shade(fragment, scene) {
      const gridU = smoothstep(0.89, 0.98, Math.abs(Math.sin(fragment.u * Math.PI * 24)));
      const gridV = smoothstep(0.89, 0.98, Math.abs(Math.sin(fragment.v * Math.PI * 18)));
      const circuit = Math.max(gridU, gridV) * (0.6 + 0.4 * Math.sin(scene.time * 1.4 + fragment.v * 20));
      const metal = litColor(0x6b7899, fragment, scene, { ambient: 0.2, specular: 1.1, shininess: 70, rim: 0.25 });
      return mixColor(metal, 0x52f5ff, circuit);
    },
  };
}

export function rockyMaterial() {
  return {
    name: 'Planeta Fragmentado: geometría deformada',
    doubleSided: false,
    shade(fragment, scene) {
      const strata = 0.5 + 0.5 * Math.sin(fragment.world.y * 4.5 + fractalNoise(fragment.world) * 5);
      const base = mixColor(0x402a59, 0xd07a7f, strata);
      return litColor(base, fragment, scene, { ambient: 0.2, diffuse: 0.85, specular: 0.08 });
    },
  };
}

export function moonMaterial() {
  return {
    name: 'Luna: regolito',
    doubleSided: false,
    shade(fragment, scene) {
      const dust = fractalNoise({ x: fragment.u * 8, y: fragment.v * 8, z: 4 }) * 0.5 + 0.5;
      return litColor(mixColor(0x565d70, 0xcbd2dc, dust), fragment, scene, { ambient: 0.17, specular: 0.05 });
    },
  };
}

export function shipMaterial() {
  return {
    name: 'Fantasticar: aleación azul y paneles blancos',
    doubleSided: true,
    shade(fragment, scene) {
      const stripe = Math.abs(fragment.world.y * 2.1 + fragment.world.x * 0.7) % 1;
      const panels = stripe > 0.72 ? 0xf4f7ff : 0x176bd1;
      const edge = Math.pow(1 - Math.max(0, dot(fragment.normal, fragment.viewDirection)), 3);
      const lit = litColor(panels, fragment, scene, { ambient: 0.3, specular: 1.1, shininess: 70, rim: 0.3 });
      return mixColor(lit, 0x70f8ff, edge * 0.55);
    },
  };
}

export function galactusArmorMaterial() {
  return {
    name: 'Galactus: armadura cósmica animada',
    doubleSided: true,
    shade(fragment, scene) {
      const channel = 0.5 + 0.5 * Math.sin(fragment.u * 42 + fragment.v * 18 - scene.time * 2.2);
      const armor = litColor(mixColor(0x35106d, 0x9a32d4, channel * 0.5), fragment, scene, { ambient: 0.32, specular: 0.7, shininess: 46, rim: 0.55 });
      const energy = smoothstep(0.9, 0.99, Math.abs(Math.sin(fragment.v * Math.PI * 12)));
      return mixColor(armor, 0xff54ef, energy * (0.45 + 0.35 * Math.sin(scene.time * 4)));
    },
  };
}

export function galactusSkinMaterial() {
  return {
    name: 'Galactus: rostro',
    doubleSided: true,
    shade(fragment, scene) {
      const face = mixColor(0xa66b75, 0xe3a9a8, 0.55 + fractalNoise(fragment.world) * 0.18);
      return litColor(face, fragment, scene, { ambient: 0.28, specular: 0.18, shininess: 22, rim: 0.18 });
    },
  };
}

export function surfaceMaterial(type) {
  const palettes = {
    tree: [0x0d5264, 0x58f5df],
    rock: [0x425166, 0xa9b1c4],
    tower: [0xdde8ff, 0x2379cf],
    beacon: [0x173071, 0x62ffff],
  };
  const [dark, bright] = palettes[type];
  return {
    name: `Superficie: ${type}`,
    doubleSided: true,
    shade(fragment, scene) {
      const pattern = 0.5 + 0.5 * Math.sin(fragment.world.y * 8 + fragment.world.x * 4);
      const base = mixColor(dark, bright, pattern * 0.45);
      const lit = litColor(base, fragment, scene, { ambient: 0.25, specular: type === 'beacon' ? 1 : 0.2, rim: 0.18 });
      return type === 'beacon' ? mixColor(lit, 0x82ffff, 0.38 + 0.22 * Math.sin(scene.time * 5)) : lit;
    },
  };
}
