// player.rs

use raylib::prelude::*;
use std::f32::consts::PI;

pub struct Player {
    pub pos: Vector2,
    pub a: f32,
    pub fov: f32, // field of view
}

fn is_position_valid(x: f32, y: f32, maze: &crate::maze::Maze, block_size: usize) -> bool {
    let i = (x / block_size as f32) as usize; // columna
    let j = (y / block_size as f32) as usize; // fila

    // Verifica que j (fila) esté dentro del rango de filas
    if j >= maze.len() {
        return false;
    }
    // Verifica que i (columna) esté dentro del rango de columnas
    if i >= maze[j].len() {
        return false;
    }

    maze[j][i] == ' ' // ' ' representa espacio vacío
}

pub fn process_events(
    player: &mut Player,
    rl: &RaylibHandle,
    delta_time: f32,
    maze: &crate::maze::Maze,
    block_size: usize,
    steps_sfx: &Sound,
) {
    const MOVE_SPEED: f32 = 200.0;
    const ROTATION_SPEED: f32 = PI;

    // Rotación con teclado (flechas y A/D)
    if rl.is_key_down(KeyboardKey::KEY_RIGHT) || rl.is_key_down(KeyboardKey::KEY_D) {
        player.a += ROTATION_SPEED * delta_time;
    }
    if rl.is_key_down(KeyboardKey::KEY_LEFT) || rl.is_key_down(KeyboardKey::KEY_A) {
        player.a -= ROTATION_SPEED * delta_time;
    }

    // Rotación con mouse (horizontal)
    let mouse_delta = rl.get_mouse_delta().x;
    player.a += mouse_delta * 0.01; // Ajusta el factor para sensibilidad

    let mut new_x = player.pos.x;
    let mut new_y = player.pos.y;

    let mut intento_mover = false;
    let mut choco_pared = false;

    // Movimiento con teclado (flechas y W/S)
    if rl.is_key_down(KeyboardKey::KEY_DOWN) || rl.is_key_down(KeyboardKey::KEY_S) {
        new_x -= MOVE_SPEED * player.a.cos() * delta_time;
        new_y -= MOVE_SPEED * player.a.sin() * delta_time;
        intento_mover = true;
    }
    if rl.is_key_down(KeyboardKey::KEY_UP) || rl.is_key_down(KeyboardKey::KEY_W) {
        new_x += MOVE_SPEED * player.a.cos() * delta_time;
        new_y += MOVE_SPEED * player.a.sin() * delta_time;
        intento_mover = true;
    }

    // Validación de movimiento
    if is_position_valid(new_x, player.pos.y, maze, block_size) {
        player.pos.x = new_x;
    } else if intento_mover {
        choco_pared = true;
    }
    if is_position_valid(player.pos.x, new_y, maze, block_size) {
        player.pos.y = new_y;
    } else if intento_mover {
        choco_pared = true;
    }

    // Si intentó moverse y chocó con una pared, reproduce el sonido
    if choco_pared {
        steps_sfx.play();
    }
}
