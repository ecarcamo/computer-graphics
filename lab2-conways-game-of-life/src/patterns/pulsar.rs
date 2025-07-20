use crate::game_of_life::GameOfLife;

pub fn create_pulsar(game: &mut GameOfLife, center_x: usize, center_y: usize) {
    //Centro izquierda arriba
    game.set_alive(center_x - 1 , center_y + 2 );
    game.set_alive(center_x - 1 , center_y + 3 );
    game.set_alive(center_x - 2 , center_y + 3 );
    game.set_alive(center_x - 3 , center_y + 2 );
    game.set_alive(center_x - 3 , center_y + 1 );
    game.set_alive(center_x - 2 , center_y + 1 );
    
    //Centro izquierda abajo
    game.set_alive(center_x - 1 , center_y - 2 );
    game.set_alive(center_x - 1 , center_y - 3 );
    game.set_alive(center_x - 2 , center_y - 3 );
    game.set_alive(center_x - 3 , center_y - 2 );
    game.set_alive(center_x - 3 , center_y - 1 );
    game.set_alive(center_x - 2 , center_y - 1 );

    //Centro derecha arriba
    game.set_alive(center_x + 1 , center_y + 2 );
    game.set_alive(center_x + 1 , center_y + 3 );
    game.set_alive(center_x + 2 , center_y + 3 );
    game.set_alive(center_x + 3 , center_y + 2 );
    game.set_alive(center_x + 3 , center_y + 1 );
    game.set_alive(center_x + 2 , center_y + 1 );
    
    //Centro derecha abajo
    game.set_alive(center_x + 1 , center_y - 2 );
    game.set_alive(center_x + 1 , center_y - 3 );
    game.set_alive(center_x + 2 , center_y - 3 );
    game.set_alive(center_x + 3 , center_y - 2 );
    game.set_alive(center_x + 3 , center_y - 1 );
    game.set_alive(center_x + 2 , center_y - 1 );


    //antena arriba izquierda
    game.set_alive(center_x - 2 , center_y + 5 );
    game.set_alive(center_x - 3 , center_y + 5 );
    game.set_alive(center_x - 3 , center_y + 6 );
    game.set_alive(center_x - 3 , center_y + 7 );

    //antena arriba derecha
    game.set_alive(center_x + 2 , center_y + 5 );
    game.set_alive(center_x + 3 , center_y + 5 );
    game.set_alive(center_x + 3 , center_y + 6 );
    game.set_alive(center_x + 3 , center_y + 7 );

    //antena abajo izquierda
    game.set_alive(center_x - 2 , center_y - 5 );
    game.set_alive(center_x - 3 , center_y - 5 );
    game.set_alive(center_x - 3 , center_y - 6 );
    game.set_alive(center_x - 3 , center_y - 7 );

    //antena abajo derecha
    game.set_alive(center_x + 2 , center_y - 5 );
    game.set_alive(center_x + 3 , center_y - 5 );
    game.set_alive(center_x + 3 , center_y - 6 );
    game.set_alive(center_x + 3 , center_y - 7 );


    //antena media izquierda arriba
    game.set_alive(center_x - 5 , center_y + 2 );
    game.set_alive(center_x - 5 , center_y + 3 );
    game.set_alive(center_x - 6 , center_y + 3 );
    game.set_alive(center_x - 7 , center_y + 3 );

    //antena media izquierda abajo
    game.set_alive(center_x - 5 , center_y - 2 );
    game.set_alive(center_x - 5 , center_y - 3 );
    game.set_alive(center_x - 6 , center_y - 3 );
    game.set_alive(center_x - 7 , center_y - 3 );


    //antena media derecha arriba
    game.set_alive(center_x + 5 , center_y + 2 );
    game.set_alive(center_x + 5 , center_y + 3 );
    game.set_alive(center_x + 6 , center_y + 3 );
    game.set_alive(center_x + 7 , center_y + 3 );

    //antena media derecha abajo
    game.set_alive(center_x + 5 , center_y - 2 );
    game.set_alive(center_x + 5 , center_y - 3 );
    game.set_alive(center_x + 6 , center_y - 3 );
    game.set_alive(center_x + 7 , center_y - 3 );

}

pub fn create_multiple_pulsars(game: &mut GameOfLife, positions: &[(usize, usize)]) {
    for &(x, y) in positions {
        create_pulsar(game, x, y);
    }
}