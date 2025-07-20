use crate::game_of_life::GameOfLife;

pub fn create_toad(game: &mut GameOfLife, center_x: usize, center_y: usize) {
    // Primera fila
    game.set_alive(center_x + 1, center_y);
    game.set_alive(center_x + 2, center_y);
    game.set_alive(center_x + 3, center_y);
    
    // Segunda fila
    game.set_alive(center_x, center_y + 1);
    game.set_alive(center_x + 1, center_y + 1);
    game.set_alive(center_x + 2, center_y + 1);
}

pub fn create_multiple_toads(game: &mut GameOfLife, positions: &[(usize, usize)]) {
    for &(x, y) in positions {
        create_toad(game, x, y);
    }
}