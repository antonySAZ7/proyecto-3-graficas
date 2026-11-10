# Guion de video de entrega (75-90 segundos)

Grabar la pantalla (`cargo run --release`) a la resolución de ventana normal.
Mantener visible el HUD (`F1`) durante casi todo el video: sus paneles
(rendimiento arriba a la izquierda, manual arriba a la derecha, telemetría
abajo a la izquierda) son la evidencia de que el trabajo es real, no una
animación pregrabada.

1. **0:00-0:08 — Inicio.** La nave aparece cerca de la Tierra. Decir: "Todo
   lo que se ve es un pipeline 3D por software propio en Rust: framebuffer,
   z-buffer y shaders de fragmento en CPU; `minifb` solo abre la ventana".
2. **0:08-0:22 — Vuelo 3D.** Usar flechas (cabeceo/guiñada), `A`/`D`
   (alabeo) y `W` (acelerar); señalar la cámara perseguidora y la respuesta
   progresiva de giro y aceleración.
3. **0:22-0:38 — Acercamiento a la Tierra.** Volar hasta distinguir océanos,
   continentes, nubes animadas y los 4 modelos de superficie (árbol, roca,
   torre, baliza). Acercarse demasiado para mostrar el mensaje de colisión y
   que la nave se detiene en vez de atravesar el planeta.
4. **0:38-0:52 — Órbita.** Presionar `O` y completar una vuelta visible
   alrededor de la Tierra sin cambiar de escena ni cortar la toma.
5. **0:52-1:04 — Rendimiento.** Alternar `F2` (frustum culling) y `F3` (LOD
   máximo); señalar en el panel cómo cambian los triángulos procesados y los
   objetos descartados.
6. **1:04-1:15 — Alejamiento y warp.** Salir de la órbita, apuntar al
   espacio abierto y presionar `H`; mostrar las estelas animadas del skybox
   y, de paso, otro planeta (p. ej. Mundo Forja o Niflheim) para evidenciar
   que los 5 son distinguibles entre sí.
7. **1:15-1:28 — Galactus.** Volar hacia Galactus, mantener `Espacio` para
   disparar y mostrar los impactos y el mensaje de repliegue.
8. **1:28-1:30 — Cierre.** Mostrar brevemente `reference/RUBRICA.md` o el
   repositorio.

Practicar el recorrido una vez antes de grabar. Si el equipo no sostiene 60
FPS, bajar `WIDTH`/`HEIGHT` en `src/game/config.rs` y volver a compilar en
release.
