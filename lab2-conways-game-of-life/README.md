# 🎮 Conway's Game of Life - Esteban Edition

## 📹 Video Demostración

<!-- Aquí subirás tu video de demostración -->
![video-funcionamiento-conways-game-of-life](https://github.com/user-attachments/assets/9154974e-53ca-447c-8b3e-c737fdb3c6c8)


---

## 📋 Descripción

Implementación del famoso **Juego de la Vida de Conway** en Rust utilizando Raylib. Esta versión incluye múltiples patrones clásicos, mundo toroidal (wrap-around) y una interfaz gráfica fluida.

### ✨ Características Principales

- 🌍 **Mundo Toroidal**: Los bordes se conectan entre sí, permitiendo que los patrones continúen infinitamente
- 🎨 **Múltiples Patrones**: Implementación de patrones clásicos del Game of Life
- ⚡ **Renderizado en Tiempo Real**: 60 FPS con visualización suave
- 🎯 **Generación Aleatoria**: Algunos patrones se generan aleatoriamente en cada ejecución
- 💾 **Exportación**: Guarda capturas en formato BMP presionando `S`

---

## 🧬 Patrones Implementados

### 🔄 Osciladores
- **Pulsar** (período 3): Oscilador complejo en forma de cruz
- **Blinker** (período 2): Línea vertical que oscila
- **Toad** (período 2): Patrón que se mueve horizontalmente
- **Beacon** (período 2): Dos bloques que parpadean

### 🚀 Naves Espaciales
- **Heavy Weight Spaceship (HWSS)**: Nave pesada que viaja horizontalmente
- **Glider**: Pequeña nave diagonal

### 🔫 Generadores
- **Glider Gun**: Cañón que dispara gliders en 4 direcciones diferentes
  - Hacia arriba ↑
  - Hacia abajo ↓  
  - Hacia izquierda ←
  - Hacia derecha →

### 🟦 Vida Estática
- **Block**: Patrón estático de 2x2 células

---

## 🛠️ Tecnologías Utilizadas

- **Lenguaje**: Rust 🦀
- **Gráficos**: Raylib-rs
- **Números Aleatorios**: rand crate
- **Patrones**: Arquitectura modular con múltiples archivos

---

## 📁 Estructura del Proyecto

```
lab2-conways-game-of-life/
├── src/
│   ├── main.rs                    # Punto de entrada principal
│   ├── framebuffer.rs             # Manejo del buffer de píxeles
│   ├── game_of_life.rs            # Lógica principal del juego
│   └── patterns/                  # Módulo de patrones
│       ├── mod.rs                 # Declaración de módulos
│       ├── blinker.rs             # Oscilador blinker
│       ├── block.rs               # Vida estática
│       ├── glider_creator.rs      # Cañón de gliders
│       ├── heavy_weight_spaceship.rs # Nave pesada
│       ├── pulsar.rs              # Oscilador pulsar
│       └── toad.rs                # Oscilador toad
├── Cargo.toml                     # Dependencias del proyecto
└── README.md                      # Este archivo
```

---

## 🚀 Instalación y Ejecución

### Prerrequisitos

1. **Rust**: Instala desde [rustup.rs](https://rustup.rs/)
2. **Dependencias del sistema** (Linux):
   ```bash
   sudo apt install libasound2-dev mesa-common-dev libx11-dev libxrandr-dev libxi-dev xorg-dev libgl1-mesa-dev libglu1-mesa-dev
   ```

### Ejecutar el Proyecto

```bash
# Clonar el repositorio
git clone [https://github.com/ecarcamo/computer-graphics/tree/main/lab2-conways-game-of-life]
cd lab2-conways-game-of-life

# Compilar y ejecutar
cargo run --release
```

---

## 🎮 Controles

| Tecla | Acción |
|-------|---------|
| `S` | Guardar captura de pantalla (`out.bmp`) |
| `ESC` o cerrar ventana | Salir del programa |

---

## ⚙️ Configuración

### Parámetros Principales (en `main.rs`)

```rust
let window_width = 800;        // Ancho de ventana
let window_height = 800;       // Alto de ventana
let game_width = 100;          // Celdas horizontales
let game_height = 100;         // Celdas verticales
let cell_size = 6;             // Tamaño de cada celda en píxeles
```

### Velocidad de Simulación

```rust
thread::sleep(Duration::from_millis(100)); // 100ms entre frames
```

---

## 🧪 Patrones de Prueba

El programa inicia con una configuración predefinida que incluye:

- **5 Pulsars** en las esquinas y centro
- **1 Glider Gun** disparando hacia arriba
- **8 Heavy Weight Spaceships** en línea horizontal
- **9 Toads** en posiciones aleatorias
- **1 Blinker** en el centro

---

## 🌍 Mundo Toroidal

Esta implementación utiliza un **mundo toroidal**, lo que significa:

- Los bordes izquierdo y derecho están conectados
- Los bordes superior e inferior están conectados  
- Las naves espaciales pueden "salir" por un lado y "entrar" por el otro
- Los patrones pueden interactuar a través de los bordes

---

## 🎯 Reglas del Juego de la Vida

1. **Supervivencia**: Una célula viva con 2 o 3 vecinos sobrevive
2. **Muerte por soledad**: Una célula viva con menos de 2 vecinos muere
3. **Muerte por sobrepoblación**: Una célula viva con más de 3 vecinos muere
4. **Nacimiento**: Una célula muerta con exactamente 3 vecinos nace

---

## 📊 Rendimiento

- **Resolución**: 800x600 píxeles
- **Grid**: 100x100 células
- **FPS Target**: ~10 FPS (100ms por frame)
- **Memoria**: Mínima, solo dos grids de 100x100 booleanos

---

## 🔧 Desarrollo

### Agregar Nuevos Patrones

1. Crear archivo en `src/patterns/nuevo_patron.rs`
2. Implementar función `create_patron()` y `create_multiple_patrones()`
3. Agregar módulo en `src/patterns/mod.rs`
4. Importar y usar en `main.rs`

---

## 📝 Autor

**Esteban** - 23016 , 6to Semestre, Computer Graphics

---

## 📄 Licencia

Este proyecto es parte de un curso académico de Computer Graphics.

---

*¡Disfruta viendo la vida artificial evolucionar en tu pantalla! 🧬✨*
