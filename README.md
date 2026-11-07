# Fantasticar: Defensa de la Tierra

Proyecto final de Gráficas por Computadora. Es una batalla espacial académica inspirada en la estética de *Fantastic Four*: una nave azul y blanca defiende la Tierra de un devorador cósmico colosal.

La escena usa un **renderizador 3D por software propio**. Canvas 2D solamente presenta el arreglo de píxeles mediante `putImageData`; no se usa WebGL, Three.js, un motor de juegos, funciones 3D del navegador ni shaders de GPU. El framebuffer, z-buffer, rasterizador de triángulos, cámara, matrices, iluminación y shaders se ejecutan en CPU con JavaScript.

## Ejecutar

Requisitos: Python 3 y un navegador moderno. No hay paquetes que instalar.

```powershell
cd "C:\Users\tempo\Desktop\UVG\ciclo 6\Grafica\proyecto 3"
python -m http.server 8080
```

Abrir [http://localhost:8080](http://localhost:8080). No conviene abrir `index.html` con doble clic porque el navegador bloquea la carga del modelo OBJ desde `file://`.

Pruebas opcionales:

```powershell
npm run check
npm test
```

## Controles

| Tecla | Acción |
|---|---|
| `W` / `S` | Acelerar / frenar o retroceder |
| `A` / `D` | Guiñada izquierda / derecha |
| `↑` / `↓` | Cabeceo |
| `Q` / `E` | Alabeo |
| `Shift` + `W` | Aceleración fuerte |
| `O` | Activar/desactivar asistencia para rodear el planeta más cercano |
| `H` | Salto warp animado |
| `Espacio` | Disparar el rayo de la nave |
| `R` | Reiniciar cerca de la Tierra |
| `F1` | Ocultar/mostrar HUD |
| `F2` | Comparar frustum culling encendido/apagado |
| `F3` | Comparar LOD dinámico con máximo detalle |
| `P` | Pausa |

## Qué contiene

- Cinco planetas distinguibles: Tierra, Mundo Forja, Niflheim, Planeta Fragmentado y Mundo de Plata.
- Sol animado, Luna, órbitas y rotaciones basadas en tiempo transcurrido.
- Nave original cargada desde `assets/models/fantasticar.obj` y cámara perseguidora 3D.
- Acercamiento continuo a la superficie, vuelo libre y modo orbital sin cambiar de escena.
- Gravedad planetaria, aceleración/giro progresivos y colisiones esféricas.
- Cuatro modelos de superficie propios: árbol cristalino, roca, torre y baliza.
- Distribución procedural reproducible sobre la esfera con semilla `2026`.
- Shaders CPU diferentes para Tierra, Sol, lava, hielo, metal, roca, nave y Galactus.
- Normal mapping procedural en la Tierra, geometría deformada y tres niveles de subdivisión/LOD.
- Frustum culling, contadores de trabajo y comparación interactiva con `F2`/`F3`.
- Skybox estrellado orientado con la cámara, efecto warp y combate con rayos.

La correspondencia exacta con cada punto se encuentra en [RUBRICA.md](./RUBRICA.md).

## Arquitectura para explicar en la defensa

- `src/math.js`: vectores, matrices, cámara, transformaciones y ruido procedural.
- `src/obj.js`: lector OBJ y triangulación de caras.
- `src/meshes.js`: esferas subdivididas, primitivas, deformación y LOD.
- `src/renderer.js`: framebuffer, z-buffer, recorte cercano, rasterización, interpolación con corrección de perspectiva, estrellas y descarte.
- `src/shaders.js`: shaders de fragmento ejecutados por cada píxel en CPU.
- `src/game.js`: simulación, controles, órbitas, gravedad, colisiones, cámara, Galactus y composición de escena.

Parámetros fáciles de modificar en vivo:

- Radio de la Tierra: `radius` del objeto `earth` en `src/game.js`.
- Velocidad orbital: `orbitSpeed` de cualquier planeta.
- Semilla procedural: argumento de `createSurfaceInstances(2026, 28)`.
- Intensidad de normal mapping: factores `0.09` y `0.025` de `earthMaterial()`.
- Resolución interna: `new SoftwareRenderer(canvas, 480, 270)`.

## Entrega

Antes de subir el repositorio:

1. Cambiar esta sección para incluir nombres, carnés y contribuciones reales de cada integrante.
2. Grabar el recorrido indicado en [GUION_VIDEO.md](./GUION_VIDEO.md).
3. Ejecutar `npm test` y comprobar que las cuatro pruebas pasan.
4. Abrir el proyecto en el equipo de la demostración y comprobar rendimiento.
5. Confirmar con el profesor que Canvas 2D como presentador del framebuffer es aceptable; la rúbrica permite una biblioteca de ventana, pero el curso puede tener una preferencia de lenguaje no escrita en el PDF.

## Integrantes y contribuciones

- **[Nombre / carné]**: [describir contribución técnica concreta].
- **[Nombre / carné, si aplica]**: [describir contribución técnica concreta].

## Recursos y asistencia

- Modelo de nave, geometría procedural, shaders y código: originales para este proyecto.
- No se usan texturas, modelos ni audios externos.
- Se utilizó OpenAI Codex como asistencia para planificación, programación, documentación y pruebas. Los integrantes deben revisar, comprender y poder explicar todo el código entregado.
- Los nombres y conceptos inspirados en *Fantastic Four* pertenecen a sus respectivos titulares; esta es una obra académica sin fines comerciales y no incorpora recursos oficiales.
