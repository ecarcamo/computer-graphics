// maze.rs

use rand::Rng;
use rand::seq::SliceRandom;
use std::fs::File;
use std::io::{BufRead, BufReader};

pub type Maze = Vec<Vec<char>>;

pub fn load_maze(filename: &str) -> Maze {
    let file = File::open(filename).unwrap();
    let reader = BufReader::new(file);

    reader
        .lines()
        .map(|line| line.unwrap().chars().collect())
        .collect()
}

pub fn generate_random_maze(width: usize, height: usize) -> Vec<Vec<char>> {
    let mut maze = vec![vec!['+'; width]; height];
    let mut rng = rand::rng();

    for y in 1..height - 1 {
        for x in 1..width - 1 {
            maze[y][x] = if rng.random_bool(0.7) { ' ' } else { '+' };
        }
    }
    maze
}

pub fn generate_maze_with_goal(width: usize, height: usize) -> Maze {
    let mut maze = vec![vec!['+'; width]; height];

    fn carve(x: usize, y: usize, maze: &mut Vec<Vec<char>>) {
        maze[y][x] = ' ';
        let mut dirs = [(0, 1), (1, 0), (0, -1), (-1, 0)];
        let mut rng = rand::rng();
        dirs.shuffle(&mut rng);
        for (dx, dy) in dirs {
            let nx = x as isize + dx * 2;
            let ny = y as isize + dy * 2;
            if nx > 0
                && ny > 0
                && nx < (maze[0].len() as isize) - 1
                && ny < (maze.len() as isize) - 1
            {
                if maze[ny as usize][nx as usize] == '+' {
                    maze[(y as isize + dy) as usize][(x as isize + dx) as usize] = ' ';
                    carve(nx as usize, ny as usize, maze);
                }
            }
        }
    }

    carve(1, 1, &mut maze);

    maze[height - 2][width - 2] = 'g';

    maze
}
