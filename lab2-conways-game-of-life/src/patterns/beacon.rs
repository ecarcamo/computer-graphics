use crate::game_of_life::GameOfLife;

pub fn create_beacon(game: &mut GameOfLife, center_x: usize, center_y: usize) {
    game.set_alive(center_x, center_y);    
    game.set_alive(center_x, center_y + 1);
    game.set_alive(center_x, center_y - 1);

    

}

pub fn create_multiple_beacons(game: &mut GameOfLife, positions: &[(usize, usize)]) {
    for &(x, y) in positions {
        create_beacon(game, x, y);
    }
}