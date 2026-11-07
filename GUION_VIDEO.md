# Guion de video de entrega (75-90 segundos)

Grabar la pantalla a 1080p si es posible. Mantener visible el HUD porque sus contadores demuestran el trabajo real del renderizador.

1. **0:00-0:08 - Inicio.** Mostrar Tierra, nave, Galactus, Sol/Luna o planetas visibles. Decir: “La escena completa usa un pipeline 3D por software y shaders de CPU; Canvas solo presenta el framebuffer”.
2. **0:08-0:22 - Vuelo 3D.** Usar `W`, `A/D`, flechas y `Q/E`; señalar velocidad progresiva y cámara perseguidora.
3. **0:22-0:38 - Acercamiento.** Volar hacia la Tierra hasta distinguir océanos, continentes, nubes y modelos sobre la superficie. Intentar acercarse demasiado para mostrar “COLISIÓN EVITADA”.
4. **0:38-0:52 - Órbita.** Presionar `O` y completar una parte visible de la vuelta alrededor del planeta sin cambiar de escena.
5. **0:52-1:04 - Rendimiento.** Alternar `F2` y `F3`; enfocar los contadores de objetos descartados, triángulos y LOD.
6. **1:04-1:15 - Alejamiento y warp.** Desactivar `O`, orientar la nave al espacio y presionar `H`; mostrar las estelas animadas.
7. **1:15-1:25 - Batalla.** Reiniciar con `R`, apuntar hacia Galactus y mantener `Espacio`; mostrar los rayos y su porcentaje de energía.
8. **1:25-1:30 - Cierre.** Mostrar brevemente el repositorio y mencionar el archivo `RUBRICA.md`.

Antes de grabar, practicar el recorrido una vez. Si la computadora baja de 24 FPS, cambiar la resolución interna de `480x270` a `400x225` en `src/game.js`; esto no modifica la resolución visual del canvas en pantalla.
