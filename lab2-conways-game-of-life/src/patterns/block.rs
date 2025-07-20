use crate::game_of_life::GameOfLife;

pub fn create_block(game: &mut GameOfLife, center_x: usize, center_y: usize) {
    game.set_alive(center_x, center_y);    
    game.set_alive(center_x + 1, center_y + 1);
    game.set_alive(center_x , center_y + 1);
    game.set_alive(center_x + 1, center_y );

}

pub fn create_multiple_block(game: &mut GameOfLife, positions: &[(usize, usize)]) {
    for &(x, y) in positions {
        create_block(game, x, y);
    }
}