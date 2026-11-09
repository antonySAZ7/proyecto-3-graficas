import {
  add, basisFromEuler, clamp, cross, dot, length, mat4AlignYToNormal,
  mat4FromTRS, mat4LookAt, mat4Multiply, mat4Perspective, mat4RotationX,
  mat4RotationY, mat4RotationZ, mat4Scale, mat4Translation, normalize,
  rotateAroundY, scale, sub, vec3,
} from './math.js';
import { SoftwareRenderer, makeCamera } from './renderer.js';
import {
  chooseLOD, createCone, createCube, createCylinder, createPlanetLOD,
  createSurfaceModels, createUVSphere,
} from './meshes.js';
import { loadOBJ } from './obj.js';
import {
  earthMaterial, galactusArmorMaterial, galactusSkinMaterial, iceMaterial,
  lavaMaterial, metalWorldMaterial, moonMaterial, rockyMaterial, shipMaterial,
  sunMaterial, surfaceMaterial,
} from './shaders.js';

const canvas = document.querySelector('#screen');
const telemetry = document.querySelector('#telemetry');
const objective = document.querySelector('#objective');
const banner = document.querySelector('#banner');
const errorBox = document.querySelector('#error');
const hud = document.querySelector('.hud');

const renderer = new SoftwareRenderer(canvas, 480, 270);
const keys = new Set();
const justPressed = new Set();
let shipMesh;
let lastTime = performance.now();
let elapsed = 0;
let paused = false;
let showHud = true;
let enableLOD = true;
let orbitAssist = false;
let earthLOD = 'MEDIO';
let bannerTimer = 0;
let beamTimer = 0;
let warpTimer = 0;
let warpCooldown = 0;
let galactusHealth = 100;

const planetLOD = createPlanetLOD(false);
const deformedLOD = createPlanetLOD(true);
const moonLOD = {
  low: createUVSphere(8, 5),
  medium: createUVSphere(14, 8),
  high: createUVSphere(22, 12),
};
const galactusSphere = createUVSphere(18, 12);
const eyeSphere = createUVSphere(8, 5);
const cube = createCube();
const cone = createCone(10);
const cylinder = createCylinder(10);
const surfaceModels = createSurfaceModels();

const materials = {
  sun: sunMaterial(),
  earth: earthMaterial(),
  lava: lavaMaterial(),
  ice: iceMaterial(),
  metal: metalWorldMaterial(),
  rock: rockyMaterial(),
  moon: moonMaterial(),
  ship: shipMaterial(),
  armor: galactusArmorMaterial(),
  skin: galactusSkinMaterial(),
  tree: surfaceMaterial('tree'),
  surfaceRock: surfaceMaterial('rock'),
  tower: surfaceMaterial('tower'),
  beacon: surfaceMaterial('beacon'),
};

const bodies = [
  { id: 'earth', name: 'Tierra', radius: 6, orbitRadius: 36, orbitSpeed: 0.006, phase: -Math.PI / 2, rotationSpeed: 0.12, lod: planetLOD, material: materials.earth },
  { id: 'forge', name: 'Mundo Forja', radius: 4.2, orbitRadius: 21, orbitSpeed: 0.012, phase: 0.65, rotationSpeed: -0.18, lod: planetLOD, material: materials.lava },
  { id: 'ice', name: 'Niflheim', radius: 5, orbitRadius: 52, orbitSpeed: 0.0042, phase: 2.7, rotationSpeed: 0.08, lod: planetLOD, material: materials.ice },
  { id: 'broken', name: 'Planeta Fragmentado', radius: 5.4, orbitRadius: 66, orbitSpeed: 0.0032, phase: 4.35, rotationSpeed: 0.1, lod: deformedLOD, material: materials.rock },
  { id: 'silver', name: 'Mundo de Plata', radius: 4.6, orbitRadius: 80, orbitSpeed: 0.0022, phase: 1.72, rotationSpeed: -0.07, lod: planetLOD, material: materials.metal },
];

const sun = { name: 'Sol', radius: 9, position: vec3(), lod: planetLOD, material: materials.sun };
const moon = { name: 'Luna', radius: 1.45, position: vec3(), lod: moonLOD, material: materials.moon };

const ship = {
  position: vec3(0, 3.2, -20),
  velocity: vec3(),
  rotation: vec3(-0.04, 0, 0),
  angularVelocity: vec3(),
  speed: 0,
};

function orbitPosition(radius, angle, y = 0) {
  return vec3(Math.cos(angle) * radius, y, Math.sin(angle) * radius);
}

function updateBodyPositions(time) {
  for (const body of bodies) {
    body.position = orbitPosition(body.orbitRadius, body.phase + time * body.orbitSpeed, Math.sin(body.phase * 2.3) * 2.5);
    body.rotation = time * body.rotationSpeed;
  }
  const earth = bodies[0];
  moon.position = add(earth.position, orbitPosition(9, time * 0.09 + 0.25, 1.2 * Math.sin(time * 0.07)));
  moon.rotation = time * 0.04;
}

function createSurfaceInstances(seed = 2026, count = 28) {
  let state = seed >>> 0;
  const random = () => {
    state = (1664525 * state + 1013904223) >>> 0;
    return state / 4294967296;
  };
  const types = ['tree', 'surfaceRock', 'tower', 'beacon'];
  const instances = [];
  for (let i = 0; i < count; i++) {
    const y = random() * 2 - 1;
    const angle = random() * Math.PI * 2;
    const radial = Math.sqrt(Math.max(0, 1 - y * y));
    const normal = normalize(vec3(Math.cos(angle) * radial, y, Math.sin(angle) * radial));
    instances.push({
      type: types[i % types.length],
      normal,
      spin: random() * Math.PI * 2,
      size: 0.28 + random() * 0.24,
    });
  }
  return instances;
}

const surfaceInstances = createSurfaceInstances();

function resetShip() {
  updateBodyPositions(elapsed);
  const earth = bodies[0];
  ship.position = add(earth.position, vec3(0, 3.2, 16));
  ship.velocity = vec3();
  ship.rotation = vec3(-0.04, 0, 0);
  ship.angularVelocity = vec3();
  orbitAssist = false;
  warpTimer = 0;
  galactusHealth = 100;
  showBanner('SISTEMAS RESTABLECIDOS', 1.4);
}

function showBanner(text, seconds = 1) {
  banner.textContent = text;
  banner.classList.add('visible');
  bannerTimer = seconds;
}

function closestPlanet(position) {
  let closest = bodies[0], best = Infinity;
  for (const body of bodies) {
    const distance = length(sub(position, body.position)) - body.radius;
    if (distance < best) {
      best = distance;
      closest = body;
    }
  }
  return { body: closest, surfaceDistance: best };
}

function inputAxis(positive, negative) {
  return (keys.has(positive) ? 1 : 0) - (keys.has(negative) ? 1 : 0);
}

function updateShip(dt) {
  const yawInput = inputAxis('KeyD', 'KeyA');
  const pitchInput = inputAxis('ArrowDown', 'ArrowUp');
  const rollInput = inputAxis('KeyE', 'KeyQ');
  const throttle = inputAxis('KeyW', 'KeyS');
  const response = 1 - Math.exp(-dt * 5.2);
  ship.angularVelocity.x += (pitchInput * 1.05 - ship.angularVelocity.x) * response;
  ship.angularVelocity.y += (yawInput * 1.15 - ship.angularVelocity.y) * response;
  ship.angularVelocity.z += (rollInput * 1.35 - ship.angularVelocity.z) * response;
  ship.rotation.x = clamp(ship.rotation.x + ship.angularVelocity.x * dt, -1.42, 1.42);
  ship.rotation.y += ship.angularVelocity.y * dt;
  ship.rotation.z += ship.angularVelocity.z * dt;

  const basis = basisFromEuler(ship.rotation);
  const acceleration = keys.has('ShiftLeft') ? 20 : 10;
  ship.velocity = add(ship.velocity, scale(basis.forward, throttle * acceleration * dt));
  ship.velocity = scale(ship.velocity, Math.exp(-0.055 * dt));

  // Every planet exerts a small gravity force that bends the flight path.
  for (const body of bodies) {
    const delta = sub(body.position, ship.position);
    const distanceSq = Math.max(4, dot(delta, delta));
    const gravity = Math.min(2.8, body.radius * body.radius * 0.72 / distanceSq);
    ship.velocity = add(ship.velocity, scale(normalize(delta), gravity * dt));
  }

  if (orbitAssist) {
    const { body, surfaceDistance } = closestPlanet(ship.position);
    const radial = normalize(sub(ship.position, body.position));
    let tangent = normalize(cross(vec3(0, 1, 0), radial));
    if (length(tangent) < 0.1) tangent = normalize(cross(vec3(1, 0, 0), radial));
    const desiredSpeed = clamp(4 + surfaceDistance * 0.12, 4, 9);
    const desired = scale(tangent, desiredSpeed);
    const assist = 1 - Math.exp(-dt * 0.85);
    ship.velocity = add(scale(ship.velocity, 1 - assist), scale(desired, assist));
    const preferredDistance = body.radius + 4.5;
    const correction = clamp((preferredDistance - length(sub(ship.position, body.position))) * 0.9, -2, 2);
    ship.velocity = add(ship.velocity, scale(radial, correction * dt));
  }

  if (warpTimer > 0) {
    ship.velocity = add(ship.velocity, scale(basis.forward, 13 * dt));
    warpTimer = Math.max(0, warpTimer - dt);
  }
  warpCooldown = Math.max(0, warpCooldown - dt);
  ship.position = add(ship.position, scale(ship.velocity, dt));

  // Basic sphere collision: push the ship out and remove inward velocity.
  for (const body of [...bodies, moon]) {
    const delta = sub(ship.position, body.position);
    const safeRadius = body.radius + 0.85;
    const distance = length(delta);
    if (distance < safeRadius) {
      const normal = distance > 1e-5 ? scale(delta, 1 / distance) : vec3(0, 1, 0);
      ship.position = add(body.position, scale(normal, safeRadius));
      const inward = dot(ship.velocity, normal);
      if (inward < 0) ship.velocity = sub(ship.velocity, scale(normal, inward * 1.25));
      showBanner(`COLISIÓN EVITADA: ${body.name.toUpperCase()}`, 0.7);
    }
  }
  ship.speed = length(ship.velocity);
}

function handleActions() {
  if (justPressed.has('KeyO')) {
    orbitAssist = !orbitAssist;
    showBanner(orbitAssist ? 'ASISTENCIA ORBITAL ACTIVADA' : 'VUELO LIBRE', 1.2);
  }
  if (justPressed.has('KeyH') && warpCooldown <= 0) {
    warpTimer = 1.65;
    warpCooldown = 5;
    const basis = basisFromEuler(ship.rotation);
    ship.velocity = add(ship.velocity, scale(basis.forward, 24));
    showBanner('SALTO WARP', 1.1);
  }
  if (keys.has('Space')) beamTimer = 0.11;
  if (justPressed.has('KeyR')) resetShip();
  if (justPressed.has('F1')) {
    showHud = !showHud;
    hud.style.display = showHud ? '' : 'none';
  }
  if (justPressed.has('F2')) {
    renderer.enableFrustumCulling = !renderer.enableFrustumCulling;
    showBanner(`FRUSTUM CULLING ${renderer.enableFrustumCulling ? 'ON' : 'OFF'}`, 1.1);
  }
  if (justPressed.has('F3')) {
    enableLOD = !enableLOD;
    showBanner(`LOD ${enableLOD ? 'DINÁMICO' : 'MÁXIMO'}`, 1.1);
  }
  if (justPressed.has('KeyP')) paused = !paused;
}

function makeGameCamera() {
  const basis = basisFromEuler(ship.rotation);
  const position = add(add(ship.position, scale(basis.forward, -8.2)), scale(basis.up, 2.7));
  const target = add(ship.position, scale(basis.forward, 12));
  const near = 0.22, far = 260, fovY = Math.PI / 3;
  const aspect = renderer.width / renderer.height;
  return makeCamera(position, target, mat4LookAt(position, target, basis.up), mat4Perspective(fovY, aspect, near, far), near, far, fovY, aspect);
}

function bodyModel(body) {
  return mat4Multiply(
    mat4Translation(body.position),
    mat4Multiply(mat4RotationY(body.rotation || 0), mat4Scale(body.radius)),
  );
}

function drawCelestialBody(body, camera, scene) {
  const distance = length(sub(camera.position, body.position));
  const selected = enableLOD ? chooseLOD(body.lod, distance, body.radius) : { mesh: body.lod.high, level: 'ALTO' };
  if (body.id === 'earth') earthLOD = selected.level;
  renderer.drawMesh(selected.mesh, bodyModel(body), body.material, camera, scene);
}

function drawEarthSurface(camera, scene) {
  const earth = bodies[0];
  if (length(sub(camera.position, earth.position)) > 38) return;
  const parent = mat4Multiply(mat4Translation(earth.position), mat4RotationY(earth.rotation));
  const meshes = {
    tree: surfaceModels.tree,
    surfaceRock: surfaceModels.rock,
    tower: surfaceModels.tower,
    beacon: surfaceModels.beacon,
  };
  for (const instance of surfaceInstances) {
    const localPosition = scale(instance.normal, earth.radius + 0.03);
    const aligned = mat4AlignYToNormal(instance.normal, localPosition);
    const local = mat4Multiply(aligned, mat4Multiply(mat4RotationY(instance.spin), mat4Scale(instance.size)));
    renderer.drawMesh(meshes[instance.type], mat4Multiply(parent, local), materials[instance.type], camera, scene);
  }
}

function galactusPosition() {
  const earth = bodies[0];
  return add(earth.position, vec3(8 + Math.sin(elapsed * 0.18) * 2, 8.5 + Math.sin(elapsed * 0.45), -34));
}

function scaledPart(position, scaling, rotation = vec3()) {
  return mat4Multiply(
    mat4Translation(position),
    mat4Multiply(
      mat4Multiply(mat4RotationY(rotation.y), mat4Multiply(mat4RotationX(rotation.x), mat4RotationZ(rotation.z))),
      mat4Scale(scaling.x, scaling.y, scaling.z),
    ),
  );
}

function drawGalactus(camera, scene) {
  const g = galactusPosition();
  // Helmet shell, forward face, torso, shoulders and iconic vertical horns.
  renderer.drawMesh(galactusSphere, scaledPart(g, vec3(7.5, 9.2, 5.6)), materials.armor, camera, scene);
  renderer.drawMesh(galactusSphere, scaledPart(add(g, vec3(0, -0.7, 3.7)), vec3(5.4, 6.9, 3.2)), materials.skin, camera, scene);
  renderer.drawMesh(cube, scaledPart(add(g, vec3(0, -12.5, 0)), vec3(13, 6.5, 5.5)), materials.armor, camera, scene);
  renderer.drawMesh(galactusSphere, scaledPart(add(g, vec3(-10.5, -9.5, 0)), vec3(5.3, 4.2, 4.5)), materials.armor, camera, scene);
  renderer.drawMesh(galactusSphere, scaledPart(add(g, vec3(10.5, -9.5, 0)), vec3(5.3, 4.2, 4.5)), materials.armor, camera, scene);
  renderer.drawMesh(cylinder, scaledPart(add(g, vec3(-6.1, 9, 0)), vec3(1.15, 5.2, 1.15)), materials.armor, camera, scene);
  renderer.drawMesh(cylinder, scaledPart(add(g, vec3(6.1, 9, 0)), vec3(1.15, 5.2, 1.15)), materials.armor, camera, scene);
  renderer.drawMesh(cone, scaledPart(add(g, vec3(-6.1, 14.7, 0)), vec3(1.5, 3.4, 1.5)), materials.armor, camera, scene);
  renderer.drawMesh(cone, scaledPart(add(g, vec3(6.1, 14.7, 0)), vec3(1.5, 3.4, 1.5)), materials.armor, camera, scene);
  const leftEye = add(g, vec3(-1.9, 1.1, 6.5));
  const rightEye = add(g, vec3(1.9, 1.1, 6.5));
  renderer.drawMesh(eyeSphere, scaledPart(leftEye, vec3(0.85, 0.36, 0.3)), materials.beacon, camera, scene);
  renderer.drawMesh(eyeSphere, scaledPart(rightEye, vec3(0.85, 0.36, 0.3)), materials.beacon, camera, scene);
  return { position: g, leftEye, rightEye };
}

function drawCombat(camera, galactus) {
  const basis = basisFromEuler(ship.rotation);
  if (beamTimer > 0) {
    const start = add(ship.position, scale(basis.forward, 1.7));
    const end = add(start, scale(basis.forward, 120));
    renderer.drawLine3D(start, end, 0x69f8ff, camera, 2);
    const toTarget = normalize(sub(galactus.position, ship.position));
    if (dot(toTarget, basis.forward) > 0.975) galactusHealth = Math.max(0, galactusHealth - 0.45);
  }
  const attackPhase = elapsed % 7;
  if (attackPhase < 1.35 && galactusHealth > 0) {
    const impact = add(bodies[0].position, vec3(0, 1.5, 0));
    renderer.drawLine3D(galactus.leftEye, impact, 0xff43df, camera, 1);
    renderer.drawLine3D(galactus.rightEye, impact, 0xba55ff, camera, 1);
  }
}

function drawScene(camera) {
  const scene = { time: elapsed, sunPosition: sun.position };
  renderer.clear();
  renderer.drawStarfield(camera, warpTimer / 1.65);
  drawCelestialBody(sun, camera, scene);
  for (const body of bodies) drawCelestialBody(body, camera, scene);
  drawCelestialBody(moon, camera, scene);
  drawEarthSurface(camera, scene);
  const galactus = drawGalactus(camera, scene);
  const shipModel = mat4FromTRS(ship.position, ship.rotation, 0.58);
  renderer.drawMesh(shipMesh, shipModel, materials.ship, camera, scene);
  drawCombat(camera, galactus);
  renderer.present();
}

function updateHUD(fps) {
  const nearby = closestPlanet(ship.position);
  const stats = renderer.stats;
  telemetry.textContent =
    `VELOCIDAD     ${ship.speed.toFixed(1)} u/s\n` +
    `CERCA DE      ${nearby.body.name}\n` +
    `ALTITUD       ${Math.max(0, nearby.surfaceDistance).toFixed(1)} u\n` +
    `ÓRBITA        ${orbitAssist ? 'ASISTIDA' : 'LIBRE'}\n` +
    `WARP          ${warpCooldown > 0 ? `${warpCooldown.toFixed(1)} s` : 'LISTO'}\n` +
    `GALACTUS      ${galactusHealth.toFixed(0)} %\n` +
    `FPS           ${fps.toFixed(0)}\n` +
    `LOD TIERRA    ${earthLOD}\n` +
    `TRIÁNGULOS    ${stats.trianglesSubmitted}\n` +
    `RASTERIZADOS  ${stats.trianglesRasterized}\n` +
    `OBJ. CULLED   ${stats.objectsCulled}\n` +
    `F2 CULLING    ${renderer.enableFrustumCulling ? 'ON' : 'OFF'}\n` +
    `F3 LOD        ${enableLOD ? 'DINÁMICO' : 'MÁXIMO'}`;
  if (galactusHealth <= 0) objective.textContent = 'Objetivo cumplido: Galactus ha sido repelido.';
  else if (nearby.body.id === 'earth' && nearby.surfaceDistance < 8) objective.textContent = 'Excelente: superficie visible. Activa O para rodear la Tierra.';
  else objective.textContent = 'Objetivo: acércate a la Tierra, orbítala y dispara a Galactus.';
}

function frame(now) {
  const dt = Math.min(0.04, Math.max(0.001, (now - lastTime) / 1000));
  lastTime = now;
  handleActions();
  if (!paused) {
    elapsed += dt;
    updateBodyPositions(elapsed);
    updateShip(dt);
    beamTimer = Math.max(0, beamTimer - dt);
    if (bannerTimer > 0) {
      bannerTimer -= dt;
      if (bannerTimer <= 0) banner.classList.remove('visible');
    }
  }
  const camera = makeGameCamera();
  drawScene(camera);
  updateHUD(1 / dt);
  justPressed.clear();
  requestAnimationFrame(frame);
}

window.addEventListener('keydown', (event) => {
  if (!keys.has(event.code)) justPressed.add(event.code);
  keys.add(event.code);
  if (event.code.startsWith('Arrow') || event.code === 'Space') event.preventDefault();
});
window.addEventListener('keyup', (event) => keys.delete(event.code));
window.addEventListener('blur', () => keys.clear());

async function start() {
  try {
    shipMesh = await loadOBJ('./assets/models/fantasticar.obj');
    updateBodyPositions(0);
    resetShip();
    requestAnimationFrame(frame);
  } catch (error) {
    console.error(error);
    errorBox.style.display = 'block';
    errorBox.innerHTML = `<h2>No se pudo iniciar</h2><p>${error.message}</p><p>Ejecuta el proyecto mediante un servidor local, como se indica en README.md.</p>`;
  }
}

start();
