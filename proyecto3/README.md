# 🌌 **Laboratorio 5 **

Este proyecto implementa un **software renderer** escrito completamente en Rust, capaz de generar un **sistema solar procedural** utilizando **únicamente shaders de color** (sin texturas ni materiales).
Cada planeta, estrella, luna y anillo es generado mediante funciones matemáticas, ruido y capas de color aplicadas directamente en el fragment shader.

Incluye animaciones, rotación, órbitas, controles manuales, captura de pantalla y paralelización para mejorar el desempeño.

---

# 🎥 Video demostrativo

👉 **[https://youtu.be/i75bjKmrTxE](https://youtu.be/i75bjKmrTxE)**

---

# ⭐ Características del proyecto

## ✔ Planetas requeridos

* 🌞 **Sol**
* 🌍 **Planeta Rocoso**
* 🪐 **Gigante Gaseoso**

## ✔ Planetas extra (30 pts)

* 🌋 **Planeta Volcánico – “Vulkar”**
* 🌊 **Planeta Azul Oceánico – “Aquahelion”**
* ❄ **Gigante de Hielo – “Glaciaron”**

## ✔ Luna (20 pts)

* 🌘 **Lunaris** orbitando el planeta rocoso.

## ✔ Sistema de anillos (20 pts)

* 🪐 **Jovarik**, el gigante gaseoso, contiene **12 rocas** orbitando como anillos.

---

# 🎨 Complejidad de shaders

Cada planeta se generó mediante **capas matemáticas de color**, logrando entre **3 y 5 capas**, lo que lo coloca en la categoría de **40 puntos (máxima complejidad)**.

Ejemplos de capas utilizadas:

* Ruido fractal animado
* Gradiente radial
* Bandas atmosféricas
* Patrones sinusoidales
* Pulsos de fuego (en Vulkar)
* Líneas diagonales dinámicas (Aquahelion)
* Degradado frío con bandas verticales (Gigante de Hielo)

---

# 🖼 Capturas del sistema

Todas las imágenes están generadas desde el renderer:

| Nombre                                    | Imagen                        |
| ----------------------------------------- | ----------------------------- |
| **Sol**                                   | `sol.png`                     |
| **Planeta Rocoso (Terranis)**             | `planeta_rocoso.png`          |
| **Luna (Lunaris)**                        | `luna.png`                    |
| **Planeta Volcánico (Vulkar)**            | `lava.png`                    |
| **Planeta Azul (Aquahelion)**             | `planeta_azul.png`            |
| **Gigante Gaseoso con Anillos (Jovarik)** | `planeta_gaseoso_anillos.png` |
| **Sistema completo**                      | `planetas_general.png`        |

---

# 🛰 Video de Orbitas, Rotación y Toma de Capturas

Mira el video completo del funcionamiento aquí:
👉 **[https://youtu.be/i75bjKmrTxE](https://youtu.be/i75bjKmrTxE)**

---

# 🎮 Controles del sistema

### 🚀 Movimiento de cámara

| Tecla | Acción                      |
| ----- | --------------------------- |
| **W** | Mover cámara hacia arriba   |
| **S** | Mover cámara hacia abajo    |
| **A** | Mover cámara a la izquierda |
| **D** | Mover cámara a la derecha   |

---

### 🪐 Selección de planetas

| Tecla | Selecciona                    |
| ----- | ----------------------------- |
| **1** | Sol                           |
| **2** | Vulkar (lava)                 |
| **3** | Terranis (rocoso)             |
| **4** | Lunaris (luna)                |
| **5** | Jovarik (gaseoso con anillos) |
| **6** | Glaciaron (gigante de hielo)  |

---

### 🔁 Rotar planeta seleccionado

| Tecla | Acción     |
| ----- | ---------- |
| **Z** | Rotación − |
| **X** | Rotación + |

---

### 🔍 Cambiar escala del planeta seleccionado

| Tecla | Acción           |
| ----- | ---------------- |
| **C** | Aumentar tamaño  |
| **V** | Disminuir tamaño |

---

### ⏸ Pausar / Reanudar animación

| Tecla | Acción                   |
| ----- | ------------------------ |
| **P** | Toggle de pausa/reanudar |

---

### 📸 Captura de pantalla

| Tecla | Acción                                  |
| ----- | --------------------------------------- |
| **O** | Guardar captura como `screenshot_X.png` |

---

# ⚙ Cómo correr el proyecto

Requisitos:

* Rust instalado
* Cargo instalado

Ejecutar:

```bash
cargo run --release
```

(Muy importante usar `--release`, ya que el renderer utiliza **Rayon** para paralelizar y acelerar el proceso).

---

# 📁 Estructura del proyecto

```
/src
 ├── main.rs           # Lógica principal del sistema solar
 ├── shaders.rs        # Shaders procedurales de colores
 ├── fragment.rs       # Estructura de fragmentos
 ├── framebuffer.rs    # Framebuffer y Z-buffer
 ├── triangle.rs       # Rasterización
 ├── vertex.rs         # Vertex shader
 ├── screenshot.rs     # Utilidad para guardar imágenes PNG
/assets/models
 └── sphere.obj        # Modelo base para todos los planetas
```

---

# 🧠 Explicación técnica del render

El pipeline implementado:

1. **Vertex Shader**
   Transforma cada vértice aplicando matriz de modelo y animaciones.

2. **Primitive Assembly**
   Agrupa vértices en triángulos.

3. **Rasterización**
   Conversión del triángulo a fragmentos individuales (pixel shader).

4. **Fragment Shader**
   Combina capas de color, funciones matemáticas y animaciones para generar el resultado final.

5. **Z-Buffer**
   Evita que los planetas se sobreescriban incorrectamente.

6. **Paralelización con Rayon**

   * Vertex shader en paralelo
   * Fragmentos en paralelo
     Aumentando significativamente la velocidad.

---

# 👤 Autor

**Esteban Cárcamo**
UVG
Laboratorio de Gráficas por Computadora

---
