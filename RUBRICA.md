# Matriz de cumplimiento de la rúbrica

La meta técnica es demostrar los 200 puntos disponibles. La nota máxima sigue siendo 100 y cada característica solamente cuenta si funciona durante la demostración y el equipo puede explicarla.

## Requisitos eliminatorios

| Requisito | Implementación / dónde verlo |
|---|---|
| Pipeline 3D por software propio | `src/renderer.js`: framebuffer RGBA, z-buffer, recorte, proyección, rasterizador e interpolación. Canvas solo muestra los píxeles. |
| Sol, nave propia y planeta | Sol procedural; nave OBJ original en `assets/models/fantasticar.obj`; cinco planetas. |
| Nave controlable; órbita y rotación por tiempo | `updateShip`, `updateBodyPositions` y `delta time` en `src/game.js`. |
| Acercarse, rodear y alejarse sin cambiar escena | Vuelo continuo y asistencia orbital con `O`. No hay cargas ni cambio de nivel. |
| Modelo, vista, proyección y profundidad | Matrices en `src/math.js`; z-buffer y rasterización en `src/renderer.js`. |
| Shader de fragmentos propio | Todos los materiales de `src/shaders.js` se evalúan por fragmento en CPU. |
| Código, modelos, recursos, instrucciones y video | Código/modelo/instrucciones incluidos. El video debe grabarse siguiendo `GUION_VIDEO.md`. |

## Menú de puntos del PDF

| Área | Característica | Puntos meta | Evidencia durante la demo |
|---|---|---:|---|
| Planetas | 5 planetas distinguibles, orbitan, rotan y admiten acercamiento | 50 | Tierra, Forja, Niflheim, Fragmentado y Plata; HUD identifica el más cercano. |
| Superficie | 4 tipos de modelo propio | 20 | Árbol, roca, torre y baliza sobre la Tierra. |
| Superficie | Distribución procedural reproducible | 10 | `createSurfaceInstances`, semilla fija `2026`. |
| Superficie | Posición/orientación sobre superficie curva en rotación | 5 | `mat4AlignYToNormal` y transformación padre de la Tierra. |
| Renderizado | 4 shaders visual e internamente diferentes | 24 | Tierra, lava, hielo, metal (además Sol, roca, nave y Galactus). |
| Renderizado | Shader animado | 8 | Plasma solar, nubes, lava, circuitos y armadura dependen de `scene.time`. |
| Renderizado | Mapeo de normales | 8 | `earthMaterial` perturba la normal con un mapa de altura procedural antes de iluminar. |
| Renderizado | Deformación geométrica | 10 | Planeta Fragmentado usa ruido multioctava al crear sus vértices. |
| Renderizado | Generación/subdivisión procedural antes de rasterizar | 10 | `createUVSphere` genera tres mallas de distinta subdivisión. |
| Navegación | Control local para rodear planeta | 8 | Tecla `O`, mantiene trayectoria tangencial y corrige altura. |
| Navegación | Aceleración, frenado y giro progresivos | 8 | Velocidad y velocidad angular integradas con `dt`; respuesta suavizada exponencialmente. |
| Navegación | Gravedad planetaria | 10 | Fuerza inversa a distancia cuadrada modifica `ship.velocity`. |
| Navegación | Colisión básica | 4 | La nave es expulsada al radio seguro y se elimina la velocidad hacia el planeta. |
| Rendimiento | Descartar objetos fuera de vista | 7 | Frustum culling por esfera; comparar contadores con `F2`. |
| Rendimiento | LOD por distancia | 8 | Mallas baja/media/alta; comparar triángulos con `F3`. |
| Presentación | Coherencia visual | 5 | Dirección azul/cian contra púrpura/magenta; Tierra como foco y Galactus como amenaza. |
| Presentación | Claridad y controles | 5 | HUD, telemetría, objetivo, controles en pantalla y mensajes de estado. |
|  | **Total meta** | **200** |  |

## Criterios adicionales de las capturas de Canvas

- Cinco elementos planeta/estrella/luna: superado; hay cinco planetas, Sol y Luna.
- Warping: `H` activa aceleración y estelas animadas en el skybox.
- Nave modelada que sigue la cámara: OBJ original con cámara perseguidora en tercera persona.
- Skybox con estrellas: 680 estrellas reproducibles proyectadas sin traslación de cámara.
- Condicionales para no atravesar elementos: colisión con cinco planetas y Luna.
- Movimiento 3D de cámara: cabeceo, guiñada y alabeo, cámara ligada a la base local.
- Órbitas: todos los planetas y la Luna orbitan mediante tiempo transcurrido.

## Evidencias recomendadas para el profesor

1. Mostrar el código de `rasterizeTriangle` y señalar prueba de profundidad e interpolación en perspectiva.
2. Acercarse a la Tierra hasta ver modelos; activar `O` y rodearla.
3. Presionar `F2` y comparar `OBJ. CULLED` y `TRIÁNGULOS`.
4. Presionar `F3` cerca y lejos de la Tierra y comparar el LOD y triángulos.
5. Presionar `H` para el warp y `Espacio` para combatir.
6. Cambiar en vivo `orbitSpeed`, la semilla o intensidad de normal mapping y explicar el resultado.
