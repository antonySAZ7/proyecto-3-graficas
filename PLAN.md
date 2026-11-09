# Plan por fases

Una fase por sesión de Claude Code. Al terminar: `cargo test`, `cargo run --release`, commit local, actualizar la casilla correspondiente y `/clear`. Detalles de reglas y arquitectura en `CLAUDE.md`.

Referencia JS: `reference/js/{math,obj,meshes,renderer,shaders,game}.js`. Úsalo como guía de comportamiento, no como molde: donde pueda quedar mejor, mejóralo.

## Fase 0: Preparación
- [x] Clonar el repo del lab a un repo nuevo; renombrar el crate; conservar la historia útil.
- [x] Copiar el JS a `reference/js/`, `fantasticar.obj` a `assets/models/`, rúbrica y PDF a `reference/`.
- [x] Reorganizar en `engine/`, `shaders/`, `game/` según `CLAUDE.md`; el lab debe seguir compilando y mostrando sus 3 cuerpos.
- [x] Perfil release en `Cargo.toml`, `config.rs` inicial, README mínimo.

**Listo cuando:** `cargo run --release` muestra lo mismo que el lab, con la nueva estructura.

## Fase 1: Núcleo del motor
- [x] `math.rs`: Vec3, Mat4 (modelo, vista, proyección), orientación con cuaternión o base ortonormal (sin gimbal lock), alinear Y a una normal.
- [x] `noise.rs`: valor/Perlin/fbm con tabla de permutación y semilla.
- [x] `obj.rs`: lector OBJ con triangulación de caras y normales calculadas si faltan.
- [x] Bucle con delta time acotado.
- [x] Pruebas de math, OBJ y ruido determinista.

**Listo cuando:** la nave OBJ se dibuja con z-buffer y un shader propio.

## Fase 2: Nave, cámara y vuelo (facilidad de juego)
- [x] Cámara perseguidora 3D con suavizado; cabeceo, guiñada y alabeo.
- [x] Aceleración, frenado y giro progresivos con respuesta suave e independiente del equipo.
- [x] Mejoras sobre el JS: estabilización asistida, indicador de dirección al objetivo, velocidad crucero, límite de velocidad según cercanía a planetas.
- [x] Skybox de estrellas orientado con la cámara (sin traslación).

**Listo cuando:** se vuela cómodo en el vacío, sin marearse.

## Fase 3: Sistema solar
- [x] `bodies.rs` como datos: radio, órbita, velocidad, inclinación, rotación, shader, luna opcional.
- [x] Sol + 5 planetas (Tierra, Mundo Forja, Niflheim, Planeta Fragmentado, Mundo de Plata) + Luna, órbitas y rotación por tiempo.
- [x] Gravedad por inversa del cuadrado, colisión esférica, modo `O` para rodear un planeta.
- [x] Acercamiento continuo sin cortes.

**Listo cuando:** se puede ir de lejos a la superficie y rodear un planeta sin saltos ni atravesarlo.

## Fase 4: Rendimiento
- [x] Rasterización paralela por bandas con `rayon`; buffers reutilizados.
- [x] Frustum culling por esfera (`F2` compara encendido/apagado).
- [x] LOD de 3 niveles por distancia (`F3` compara con máximo detalle).
- [x] HUD con FPS, triángulos procesados y objetos descartados.

**Listo cuando:** 60 FPS estables a la resolución objetivo y los contadores cambian de verdad con `F2`/`F3`.

## Fase 5: Superficie y geometría
- [x] 4 modelos propios sobre la Tierra (árbol cristalino, roca, torre, baliza).
- [x] Distribución procedural con semilla fija configurable, sobre toda la esfera, con prueba de reproducibilidad.
- [x] Posición y orientación correctas sobre la superficie mientras el planeta rota.
- [x] Deformación de geometría por ruido multioctava (Planeta Fragmentado).
- [x] Generación/subdivisión procedural de mallas antes de rasterizar.

**Listo cuando:** la superficie se ve poblada y coherente desde cerca y sin parpadeos.

## Fase 6: Shaders
- [x] Mínimo 4 shaders visual e internamente distintos (Tierra, lava, hielo, metal) más sol, roca y nave. Galactus queda para la Fase 7 (ahí se agrega como entidad).
- [x] Al menos uno animado con el tiempo (plasma solar, nubes de la Tierra, lava, circuitos del Mundo de Plata).
- [x] Mapeo de normales procedural en la Tierra, calculado en el renderer por software (gradiente del ruido de terreno por diferencias finitas).
- [x] Reutilizar lo ya hecho en el lab (estrella, rocoso, luna) donde encaje; `gas_giant.rs` se retiró porque ningún cuerpo quedó con esa identidad visual.

**Listo cuando:** cada cuerpo se identifica de un vistazo.

## Fase 7: Estética y combate
- [ ] Dirección artística: azul/cian/blanco (héroes) contra púrpura/magenta (amenaza); Tierra como foco.
- [ ] Efectos baratos: brillo del sol, atmósfera fresnel, estelas del warp (`H`).
- [ ] Galactus como amenaza colosal con shader propio; rayo de la nave (`Espacio`).
- [ ] HUD claro con fuente bitmap propia: telemetría, objetivo, controles, mensajes. `F1` oculta el HUD.

**Listo cuando:** una captura de pantalla cualquiera se ve intencional y legible.

## Fase 8: Entrega
- [ ] README: ejecución, controles, integrantes y contribuciones, tabla rúbrica → dónde verlo, uso de IA declarado, créditos.
- [ ] `RUBRICA.md` actualizado con evidencia real de cada punto.
- [ ] `GUION_VIDEO.md`: vuelo, acercamiento, vuelta alrededor de un planeta y alejamiento.
- [ ] Screenshots y revisión final: `cargo test`, `cargo clippy`, build limpio desde cero.

## Fase 9 (después de entregar): escalar a juego
Ideas, sin hacerlas antes de la entrega:
- Sistema de misiones y progresión; enemigos con IA simple; más modos de arma.
- Guardado de partida; menú principal y pausa.
- Audio (crate `rodio`), gamepad.
- Galaxia generada por semilla con más sistemas.
- Cargar cuerpos y escenas desde archivos de datos (RON/TOML).

## Checklist de rúbrica (200 pts, nota máx 100)

| Área | Característica | Pts | Fase |
|---|---|---:|---|
| Planetas | 5 planetas distinguibles | 50 | 3 |
| Superficie | 4 tipos de modelo | 20 | 5 |
| Superficie | Distribución procedural reproducible | 10 | 5 |
| Superficie | Orientación sobre superficie curva rotando | 5 | 5 |
| Render | 4 shaders distintos | 24 | 6 |
| Render | Shader animado | 8 | 6 |
| Render | Mapeo de normales | 8 | 6 |
| Render | Deformación de geometría | 10 | 5 |
| Render | Generación/subdivisión procedural | 10 | 5 |
| Navegación | Modo local para rodear planeta | 8 | 3 |
| Navegación | Aceleración y giro progresivos | 8 | 2 |
| Navegación | Gravedad | 10 | 3 |
| Navegación | Colisión | 4 | 3 |
| Rendimiento | Descarte de objetos fuera de vista | 7 | 4 |
| Rendimiento | LOD por distancia | 8 | 4 |
| Presentación | Coherencia visual | 5 | 7 |
| Presentación | Claridad y controles | 5 | 7 |

Extras de las capturas de Canvas: warping, nave que sigue la cámara, skybox con estrellas, no atravesar elementos, cámara con movimiento 3D, órbitas.
