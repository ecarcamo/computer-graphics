use crate::game_of_life::GameOfLife;
use super::block::{create_block}; // Import desde el módulo hermano

#[derive(Clone, Copy)]
pub enum Direction {
    Right,  // Original (→)
    Left,   // Espejado horizontal (←)
    Down,   // Rotado 90° (↓)
    Up,     // Rotado 270° (↑)
}

pub fn create_creators_glider_with_direction(
    game: &mut GameOfLife, 
    center_x: usize, 
    center_y: usize,
    direction: Direction
) {
    // Definir todas las posiciones relativas al centro (0,0)
    let mut positions = vec![
        // Bloque inicial
        (0, 0), (1, 0), (0, 1), (1, 1),
        
        // Centro del cañón
        (10, 0), (14, 0), (16, 0), (17, 0),
        
        // Parte arriba
        (10, 1), (11, 2), (12, 3), (13, 3), (15, 2), (16, 1),
        
        // Parte abajo  
        (10, -1), (11, -2), (12, -3), (13, -3), (15, -2), (16, -1),
        
        // Sección derecha
        (20, 1), (20, 2), (20, 3), (21, 1), (21, 2), (21, 3),
        (22, 4), (22, 0), (24, 4), (24, 5), (24, 0), (24, -1),
        
        // Bloque final
        (34, 2), (35, 2), (34, 3), (35, 3),
    ];
    
    // Aplicar transformación según la dirección
    for (dx, dy) in positions {
        let (final_x, final_y) = match direction {
            Direction::Right => (center_x as i32 + dx, center_y as i32 + dy),
            Direction::Left => (center_x as i32 - dx, center_y as i32 + dy),
            Direction::Down => (center_x as i32 + dy, center_y as i32 + dx),
            Direction::Up => (center_x as i32 - dy, center_y as i32 - dx),
        };
        
        // Verificar límites antes de colocar
        if final_x >= 0 && final_y >= 0 {
            let x = final_x as usize;
            let y = final_y as usize;
            if x < 100 && y < 100 { // Límites del juego
                game.set_alive(x, y);
            }
        }
    }
}

// Mantener la función original para compatibilidad
pub fn create_creators_glider(game: &mut GameOfLife, center_x: usize, center_y: usize) {
    create_creators_glider_with_direction(game, center_x, center_y, Direction::Right);
}

// Funciones de conveniencia para cada dirección
pub fn create_glider_gun_right(game: &mut GameOfLife, center_x: usize, center_y: usize) {
    create_creators_glider_with_direction(game, center_x, center_y, Direction::Right);
}

pub fn create_glider_gun_left(game: &mut GameOfLife, center_x: usize, center_y: usize) {
    create_creators_glider_with_direction(game, center_x, center_y, Direction::Left);
}

pub fn create_glider_gun_down(game: &mut GameOfLife, center_x: usize, center_y: usize) {
    create_creators_glider_with_direction(game, center_x, center_y, Direction::Down);
}

pub fn create_glider_gun_up(game: &mut GameOfLife, center_x: usize, center_y: usize) {
    create_creators_glider_with_direction(game, center_x, center_y, Direction::Up);
}

// Crear múltiples con direcciones diferentes
pub fn create_multiple_creators_gliders_with_directions(
    game: &mut GameOfLife, 
    configs: &[(usize, usize, Direction)]
) {
    for &(x, y, direction) in configs {
        create_creators_glider_with_direction(game, x, y, direction);
    }
}

// Mantener función original
pub fn create_multiple_creators_gliders(game: &mut GameOfLife, positions: &[(usize, usize)]) {
    for &(x, y) in positions {
        create_creators_glider(game, x, y);
    }
}

