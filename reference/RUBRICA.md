# Matriz de cumplimiento de la rúbrica

La meta técnica es demostrar los 200 puntos disponibles. La nota máxima sigue siendo 100 y cada característica solamente cuenta si funciona durante la demostración y el equipo puede explicarla.

Todas las rutas son relativas a la raíz del repo. El proyecto es Rust puro
(`cargo run --release`); `reference/js/` es el proyecto original en
JavaScript, conservado solo como referencia de comportamiento y que ya no se
compila ni cuenta como evidencia.

## Requisitos eliminatorios

| Requisito | Implementación / dónde verlo |
|---|---|
| Pipeline 3D por software propio | `src/engine/framebuffer.rs` (framebuffer + z-buffer), `src/engine/rasterizer.rs` (transformación objeto→mundo→clip→NDC→pantalla, interpolación con corrección de perspectiva, prueba de profundidad). `minifb` solo copia el buffer de píxeles ya resuelto a la ventana. |
| Sol, nave propia y planeta | Sol procedural (`src/shaders/star.rs`); nave OBJ original en `assets/models/fantasticar.obj`, leída por `src/engine/obj.rs`; cinco planetas definidos como datos en `src/game/bodies.rs`. |
| Nave controlable; órbita y rotación por tiempo | Entrada en `src/game/input.rs`, integración de vuelo en `Ship::update` (`src/game/ship.rs`); posiciones orbitales y rotación de cada cuerpo en `World::update` (`src/game/world.rs`), todo en función de `time`/`dt`, nunca de frames. |
| Acercarse, rodear y alejarse sin cambiar escena | Vuelo continuo en un único mundo persistente; asistencia de órbita con `O` (`Ship::update`, rama de `orbit_assist`). No hay pantallas de carga ni cambio de escena al acercarse a un cuerpo. |
| Modelo, vista, proyección y profundidad | Matrices en `src/engine/math.rs` (`Mat4::perspective_rh`, `look_at`, etc.); z-buffer y prueba de profundidad en `src/engine/rasterizer.rs` / `src/engine/framebuffer.rs`. |
| Shader de fragmentos propio | Trait `FragmentShader` en `src/shaders/mod.rs`; cada archivo de `src/shaders/*.rs` lo implementa y se evalúa por fragmento en CPU dentro del rasterizador. |
| Código, modelos, recursos, instrucciones y video | Código y modelo incluidos en el repo; instrucciones en este README y en `README.md`; guion de grabación en `GUION_VIDEO.md`. |

## Menú de puntos del PDF

| Área | Característica | Puntos meta | Evidencia durante la demo |
|---|---|---:|---|
| Planetas | 5 planetas distinguibles, orbitan, rotan y admiten acercamiento | 50 | Tierra, Mundo Forja, Niflheim, Planeta Fragmentado y Mundo de Plata (`src/game/bodies.rs`), cada uno con shader propio; el panel de telemetría (`src/game/hud.rs`) identifica el cuerpo más cercano y su distancia. |
| Superficie | 4 tipos de modelo propio | 20 | Árbol cristalino, roca, torre y baliza sobre la Tierra, generados en `src/game/surface.rs` con mallas de `src/engine/mesh.rs`. |
| Superficie | Distribución procedural reproducible | 10 | `surface::scatter` con semilla fija `SURFACE_SCATTER_SEED` (`src/game/config.rs`); prueba `scatter_is_reproducible_with_same_seed` en `src/game/surface.rs`. |
| Superficie | Posición/orientación sobre superficie curva en rotación | 5 | Cada instancia se orienta a la normal del punto de la esfera y se transforma junto con la rotación de la Tierra (`src/game/surface.rs`, prueba `instance_sits_at_body_radius_plus_offset_from_the_planet_center`). |
| Renderizado | 4 shaders visual e internamente diferentes | 24 | Tierra (`earth.rs`), lava (`lava.rs`), hielo (`ice.rs`), metal (`metal.rs`); además sol (`star.rs`), roca (`rocky.rs`), nave (`ship.rs`) y Galactus (`galactus.rs`). |
| Renderizado | Shader animado | 8 | Todos reciben `Uniforms::time`: plasma solar (`star.rs`), nubes de la Tierra (`earth.rs`), lava fluyendo (`lava.rs`), circuitos del Mundo de Plata (`metal.rs`). |
| Renderizado | Mapeo de normales | 8 | `earth.rs` perturba la normal con el gradiente del ruido de terreno por diferencias finitas, calculado en el shader (software, no GPU). |
| Renderizado | Deformación geométrica | 10 | Planeta Fragmentado y Galactus parten de un icosaedro subdividido y desplazan cada vértice con ruido multioctava (`src/engine/mesh.rs::displace`, usado desde `main.rs` al construir sus LOD). |
| Renderizado | Generación/subdivisión procedural antes de rasterizar | 10 | `src/engine/mesh.rs` genera la esfera UV y el icosaedro subdividido (3 niveles de detalle cada uno) antes de que el rasterizador los reciba. |
| Navegación | Control local para rodear planeta | 8 | Tecla `O` (`Action::OrbitAssist`), mantiene una distancia objetivo al cuerpo más cercano y corrige la velocidad tangencial en `Ship::update`. |
| Navegación | Aceleración, frenado y giro progresivos | 8 | Velocidad y velocidad angular suavizadas con `dt` vía `physics::smooth_towards_vec3`/`drag_factor` (`src/game/physics.rs`), usadas en `Ship::update`. |
| Navegación | Gravedad planetaria | 10 | `World::gravity_accel`: inversa al cuadrado de la distancia, con piso y techo configurables (`src/game/world.rs`, `src/game/config.rs`). |
| Navegación | Colisión básica | 4 | `World::resolve_collision` expulsa la nave al radio + margen de seguridad y anula la velocidad hacia el cuerpo (`src/game/world.rs`). |
| Rendimiento | Descartar objetos fuera de vista | 7 | `Frustum::sphere_visible` (`src/engine/culling.rs`); comparar contadores de objetos visibles/descartados con `F2`. |
| Rendimiento | LOD por distancia | 8 | `select_lod` (`src/engine/lod.rs`), 3 mallas por cuerpo; comparar triángulos procesados con `F3`. |
| Presentación | Coherencia visual | 5 | Dirección cian/blanco (héroe) contra magenta/púrpura (amenaza) en `HERO_COLOR`/`THREAT_COLOR` (`src/game/config.rs`); Tierra como foco, Galactus como amenaza colosal. |
| Presentación | Claridad y controles | 5 | HUD con fuente bitmap propia (`src/engine/font.rs`): manual de controles, telemetría, objetivo y mensajes de estado (`src/game/hud.rs`); `F1` lo oculta para capturas limpias. |
|  | **Total meta** | **200** |  |

## Criterios adicionales de las capturas de Canvas

- Cinco elementos planeta/estrella/luna: superado; cinco planetas, Sol y Luna (`src/game/bodies.rs`).
- Warping: `H` activa impulso extra y estelas animadas en el skybox (`src/game/skybox.rs`, `config::WARP_*`).
- Nave modelada que sigue la cámara: OBJ original con cámara perseguidora en tercera persona (`src/game/camera.rs`).
- Skybox con estrellas: estrellas reproducibles (semilla `STARFIELD_SEED`) proyectadas sin traslación de cámara (`src/game/skybox.rs`).
- Condicionales para no atravesar elementos: colisión esférica con los cinco planetas, la Luna y Galactus (`World::resolve_collision`).
- Movimiento 3D de cámara: cabeceo, guiñada y alabeo con base ortonormal propia (sin gimbal lock) en `src/engine/math.rs`.
- Órbitas: todos los planetas y la Luna orbitan en función del tiempo transcurrido (`World::update`).

## Evidencias recomendadas para el profesor

1. Mostrar `src/engine/rasterizer.rs` y señalar la prueba de profundidad e interpolación en perspectiva.
2. Acercarse a la Tierra hasta ver los modelos de superficie; activar `O` y rodearla.
3. Presionar `F2` y comparar `OBJETOS VISIBLES` en el panel de rendimiento.
4. Presionar `F3` cerca y lejos de un planeta y comparar `TRIANGULOS`.
5. Presionar `H` para el warp y `Espacio` para disparar contra Galactus.
6. Cambiar en vivo un valor de `src/game/config.rs` (por ejemplo `SURFACE_SCATTER_SEED` o `FRACTURED_DEFORM_AMPLITUDE`) y recompilar para mostrar el efecto.
