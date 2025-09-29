# Proyecto Raytracer CPU – Diorama Skyblock

Este proyecto muestra un pequeño diorama inspirado en Minecraft renderizado íntegramente en CPU usando Rust. El objetivo es exhibir iluminación básica (Phong + reflejos/refracciones), materiales texturizados y dos mundos intercambiables (Overworld/Nether) dentro de una escena tipo skyblock.

## Requisitos

- [Rust](https://www.rust-lang.org/tools/install) 1.70 o superior (incluye `cargo`).
- Dependencias nativas de `raylib` (en Debian/Ubuntu: `sudo apt install libraylib-dev`).
- La carpeta `assets/` incluida en este repositorio (texturas y skyboxes).

## Ejecución rápida

```bash
cargo run --release
```

El modo `--release` es recomendable porque el trazador ejecuta muchos rayos por cuadro. Si deseas compilar una vez y reutilizar el binario:

```bash
cargo build --release
./target/release/proyecto2-raytracer
```

## Controles

- **Flechas**: orbitan la cámara alrededor de la isla.
- **Q / E**: acercan o alejan la cámara.
- **W / A / S / D**: desplazan la luz principal sobre el plano XZ.
- **R / F**: suben o bajan la luz.
- **N**: alterna entre Overworld y Nether.

Los textos en pantalla resumen los atajos disponibles. Se puede cerrar la ventana con `Esc` o el botón de la ventana.

## Personalización rápida

- **Resolución**: cambia `(fb_w, fb_h)` en `src/main.rs`.
- **Texturas**: reemplaza imágenes dentro de `assets/`. Los nombres se cargan directamente según el archivo (por ejemplo `hierba.jpg` para el césped superior).
- **Materiales**: ajusta parámetros en `src/rendering/raytracer.rs` dentro de la función `build_scene` (albedo, especular, reflectividad, transparencia, etcétera).
- **Skyboxes**: coloca un cubemap en `assets/skybox` (Overworld) y `assets/skybox_nether` (Nether). Si el segundo no existe, se reutiliza el primero con un tinte rojizo.

## Estructura del código

- `src/main.rs`: punto de entrada, carga de assets y bucle principal (input + render).
- `src/rendering/`: contiene el raytracer y utilidades de iluminación (`lighting.rs`).
- `src/geometry/`: primitivas de bloque (sólidas y texturizadas).
- `src/scene/`: definición de materiales e interfaz `Intersectable`.
- `src/math/`: utilidades matemáticas (por ahora solo `Vec3`).
- `src/camera.rs`: cámara orbital simple que genera los rayos primarios.

El trazado se paraleliza por filas utilizando `std::thread::scope`, por lo que cada CPU disponible procesa un bloque de la imagen.

## Notas sobre rendimiento

- Usa `--release` para obtener la máxima velocidad.
- La escena se construye una única vez al arrancar (Overworld y Nether se cachean), por lo que el trabajo por frame se reduce a lanzar rayos y sombrear.
- Si modificas la geometría en tiempo de ejecución, vuelve a llamar a `build_scene` para regenerar el `SceneData` antes de renderizar.

## Capturas

Puedes generar un `PNG` desde la propia ventana usando el menú de tu SO o adaptando el código para escribir el `frame` en disco (por defecto se actualiza una textura en pantalla cada tick).

---


