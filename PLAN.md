# Plan por fases

Una fase por sesión de Claude Code. Al terminar: `cargo test`, `cargo run --release`, commit local, marcar las casillas y `/clear`. Reglas y arquitectura en `CLAUDE.md` (sin push, sin coautoría).

Referencia JS: `reference/js/`. Imagen de referencia de Galactus: `reference/galactus.png`.

## Fases 0 a 8: completadas
Preparación, núcleo del motor, nave y cámara, sistema solar, rendimiento (culling, LOD, rayon), superficie y geometría, shaders, estética y combate, entrega. Evidencia de rúbrica en `RUBRICA.md`. No tocar su lógica salvo lo que pidan las fases de abajo.

## Reglas de las fases 9 a 12
- El proyecto aún no se entrega. No se usan ramas: cada fase termina en **un commit local**, y si algo se complica se vuelve al commit de la fase anterior con `git reset --hard <hash>`.
- Antes de empezar, anota el hash del commit actual (`git log -1 --oneline`).
- Cada fase debe poder quitarse sin romper las demás: lo nuevo va detrás de una constante en `game/config.rs` (por ejemplo `GALACTUS_DETALLADO`) y el comportamiento anterior sigue disponible.
- No bajar de 60 FPS a la resolución objetivo. Si una fase baja el rendimiento, se simplifica antes de continuar.
- No cambiar el motor (`engine/`) salvo que sea imprescindible (por ejemplo, mezcla aditiva para fuego); si se cambia, decirlo en el commit.
- Nada fuera de lo listado. Sin audio, guardado, IA de drones, misiones múltiples ni armas extra.

## Fase 9: Combate legible (tareas simples, juntas)
- [x] **Ataque de fuego:** el disparo pasa del rayo azul actual a un proyectil rojo/naranja con núcleo amarillo claro, brillo y parpadeo animado con el tiempo; estela corta de chispas.
- [x] **Destello de impacto** al golpear a Galactus: flash rojo/blanco breve y chispas.
- [x] **Salud de Galactus:** valor configurable en `config.rs` (por defecto unos 30 impactos), baja con cada golpe.
- [x] **Enemigo visible en el HUD:** nombre "GALACTUS" con barra de vida arriba al centro, y una flecha en el borde de la pantalla que apunta hacia él cuando está fuera de vista.
- [x] **Reacción al daño:** Galactus parpadea o se tiñe de blanco/rojo un instante al recibir un impacto.

**Listo cuando:** se entiende de un vistazo quién es el enemigo, dónde está y que el ataque le hace daño.

## Fase 10: Galactus elaborado (tarea compleja, aislada)
Objetivo: que vuelva a verse como el villano. Referencia: `reference/galactus.png` y las partes de Galactus en `reference/js/game.js` y `meshes.js`.
- [ ] Modelo compuesto por partes con una transformación padre: casco/cabeza ovalada con **dos espirales altas** a los lados (como cuernos), **cara rosada** con **dos ojos cian** brillantes, **hombros esféricos** a ambos lados, **torso en forma de bloque** y base.
- [ ] Shader púrpura con **bandas/rayas animadas** (mismo lenguaje visual de la referencia) y brillo magenta en los hombros; la cara con shader aparte, suave y rosado.
- [ ] Escala colosal frente a los planetas; rotación lenta y un pulso de energía en los ojos.
- [ ] Culling con una sola esfera englobante para todo el conjunto, y LOD (versión simple a distancia) para no pasar el presupuesto de triángulos.
- [ ] Respetar la barra de vida y el parpadeo de la Fase 9 sobre todas las partes.
- [ ] `GALACTUS_DETALLADO` en `config.rs`: en `false` vuelve a la esfera actual.

**Listo cuando:** Galactus se reconoce a distancia, mantiene los FPS y se puede desactivar con una constante.

## Fase 11: Pantallas y objetivo del juego (tarea compleja, aislada)
Problema actual: al abrir el juego no se sabe de qué trata ni qué hacer.
- [ ] **Estados de juego** (`Menu`, `Jugando`, `Pausa`, `Victoria`, `Derrota`) en `game/state.rs`; el bucle principal decide qué dibujar y actualizar según el estado.
- [ ] **Menú principal:** fondo con el skybox y la Tierra girando (reutiliza la escena existente), título "FANTASTICAR: DEFENSA DE LA TIERRA", historia en 2 o 3 líneas ("Galactus se acerca a la Tierra. Los Cuatro Fantásticos son su última defensa."), controles principales y "ENTER para jugar".
- [ ] **Objetivo guiado en el HUD**, en pasos: "1. Vuela hacia Galactus" → "2. ¡Dispara hasta destruirlo!" con la distancia y la barra de su vida.
- [ ] **Amenaza simple:** Galactus avanza despacio hacia la Tierra; si llega, la barra de "Salud de la Tierra" baja con el tiempo. Velocidades y valores en `config.rs`.
- [ ] **Victoria** al llegar su salud a 0 y **derrota** si la Tierra llega a 0 o la nave choca; pantalla con texto claro y `ENTER` para reiniciar, `Esc` para salir.
- [ ] `P` pausa y vuelve; `R` reinicia partida desde cualquier estado de juego.

**Listo cuando:** una persona que nunca vio el juego entiende la historia y qué hacer, puede ganar y perder, y reinicia sin cerrar el programa.

## Fase 12: Cierre y entrega
- [ ] Actualizar `README.md`: nuevos controles, menú, objetivo, cómo ganar, capturas nuevas.
- [ ] Actualizar `RUBRICA.md` y `GUION_VIDEO.md` con el flujo nuevo (menú → vuelo → acercamiento → combate → victoria).
- [ ] Revisar rendimiento y contadores (`F2`/`F3`) para confirmar que siguen funcionando.
- [ ] `cargo test`, `cargo clippy` y build limpio desde cero.
- [ ] Declarar el uso de IA y los integrantes y contribuciones reales.

## Futuro (no hacer ahora)
Más armas, drones enemigos con IA simple, misiones encadenadas, guardado de partida, audio, gamepad, galaxia generada por semilla y escenas desde archivos de datos.
