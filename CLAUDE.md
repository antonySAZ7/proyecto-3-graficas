# Fantasticar: Defensa de la Tierra (Rust)

Proyecto final de Gráficas por Computadora (UVG). Sistema solar explorable con nave, renderer 3D **por software propio** en Rust. Puerto y mejora del proyecto JS original (en `reference/js/`) sobre la base del lab de shaders (`minifb`, framebuffer, z-buffer, rasterizador, trait `FragmentShader`).

Objetivos, en orden: (1) cumplir requisitos eliminatorios y llegar a los 200 pts de la rúbrica, (2) rendimiento, (3) estética y facilidad de juego, (4) código escalable a un juego completo.

Plan por fases y checklist de rúbrica: `PLAN.md`. Rúbrica detallada: `reference/RUBRICA.md` y `reference/instrucciones.pdf`.

## Reglas inamovibles

- **Prohibido**: `git push`, y cualquier coautoría. Ningún commit lleva `Co-Authored-By` ni menciona a Claude o IA. Sí puedes hacer commits locales al terminar cada fase, con mensaje corto en español.
- **Renderer propio**: sin motor de juego/física, sin funciones 3D, cámara, materiales ni shaders de GPU de raylib u otras libs. Las libs solo pueden dar ventana/entrada (`minifb`), paralelismo (`rayon`) y lectura de archivos. Matemática (vectores, matrices, cámara), OBJ, ruido, rasterización y shaders son código propio.
- **Sin texturas ni modelos externos**: todo procedural u original (la nave está en `assets/models/fantasticar.obj`).
- Movimiento y animación dependen de **delta time**, nunca del número de frames.
- Sin pantallas de carga ni cambio de escena al acercarse a un planeta.

## Arquitectura (mantenla)

```
src/
  main.rs              ventana, bucle, delta time
  engine/              NO depende de game/
    math.rs  framebuffer.rs  rasterizer.rs  mesh.rs  obj.rs  noise.rs  culling.rs  lod.rs
  shaders/             un archivo por shader; trait FragmentShader en mod.rs
  game/
    config.rs          TODOS los números ajustables (radios, órbitas, semilla, resolución)
    bodies.rs          definición de cuerpos celestes como DATOS (fácil agregar planetas)
    ship.rs  camera.rs  physics.rs  input.rs  world.rs  surface.rs  combat.rs  hud.rs
assets/models/
reference/js/          código JS original, SOLO lectura, no se compila
```

Principios para escalar: cuerpos y shaders definidos por datos, no por `if` repartidos; `engine` reutilizable; un solo `config.rs`; nada de números mágicos en la lógica.

## Rendimiento (meta: 60 FPS en 640x360 escalado a la ventana)

- Rasterización por bandas/tiles con `rayon`; sin asignaciones por frame ni por píxel (buffers reutilizados).
- `f32` en todo; ruido con tabla de permutación precalculada.
- Culling (frustum + backface) y LOD siempre activos, con contadores de triángulos expuestos para el HUD.
- `Cargo.toml` release: `opt-level = 3`, `lto = "fat"`, `codegen-units = 1`, `panic = "abort"`.
- Resolución interna configurable en `config.rs`.

## Ahorro de tokens

- Lee solo el archivo o rango de líneas que necesitas; no releas lo que acabas de editar.
- Del JS original, abre **un módulo a la vez** y solo mientras lo portas. No lo pegues en el chat.
- Usa `cargo check` mientras desarrollas; `cargo run --release` solo para validar visualmente.
- Respuestas cortas: qué cambiaste y qué sigue. Sin recapitular código ni explicar lo obvio.
- Trabaja **una fase de `PLAN.md` por sesión**; haz `/clear` al terminar cada fase y arranca la siguiente leyendo solo `CLAUDE.md` y esa fase.
- Antes de una tarea grande o ambigua, pregunta lo mínimo necesario en un solo mensaje; si no, decide y sigue.

## Código

- Rust idiomático, sin `unsafe` salvo en bucles de píxeles justificados con comentario.
- Comentarios breves en español solo donde la intención no es obvia (el equipo debe poder explicar cada parte en la defensa).
- Pruebas (`cargo test`) para: matemática, OBJ, reproducibilidad de la semilla, culling y colisión.
