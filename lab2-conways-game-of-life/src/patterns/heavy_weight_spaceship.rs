use crate::game_of_life::GameOfLife;

pub fn create_heavy_weight_spaceship(game: &mut GameOfLife, center_x: usize, center_y: usize) {
    game.set_alive(center_x , center_y + 1);    
    game.set_alive(center_x, center_y - 1);  

    game.set_alive(center_x + 1 , center_y + 2);    

    game.set_alive(center_x + 2 , center_y + 2);    
    game.set_alive(center_x + 3 , center_y + 2);    
    game.set_alive(center_x + 4 , center_y + 2);    
    game.set_alive(center_x + 5 , center_y + 2);    
    game.set_alive(center_x + 6 , center_y + 2);   
    game.set_alive(center_x + 6 , center_y + 1);    
    game.set_alive(center_x + 6 , center_y );     
    game.set_alive(center_x + 5 , center_y - 1);  
    game.set_alive(center_x + 3 , center_y - 2);   
    game.set_alive(center_x + 2 , center_y - 2);   

}

pub fn create_multiple_heavy_weight_spaceships(game: &mut GameOfLife, positions: &[(usize, usize)]) {
    for &(x, y) in positions {
        create_heavy_weight_spaceship(game, x, y);
    }
}