// main.rs
#![allow(unused_imports)]
#![allow(dead_code)]

mod caster;
mod framebuffer;
mod line;
mod maze;
mod player;

use caster::{Intersect, cast_ray};
use framebuffer::Framebuffer;
use line::line;
use maze::{Maze, load_maze};
use player::{Player, process_events};

use raylib::prelude::*;
use std::f32::consts::PI;
use std::thread;
use std::time::Duration;

fn cell_to_color(cell: char) -> Color {
    match cell {
        '+' => {
            return Color::BLUEVIOLET;
        }
        '-' => {
            return Color::VIOLET;
        }
        '|' => {
            return Color::VIOLET;
        }
        'g' => {
            return Color::GREEN;
        }
        _ => {
            return Color::WHITE;
        }
    }
}

fn draw_cell(framebuffer: &mut Framebuffer, xo: usize, yo: usize, block_size: usize, cell: char) {
    if cell == ' ' {
        return;
    }
    let color = cell_to_color(cell);
    framebuffer.set_current_color(color);

    for x in xo..xo + block_size {
        for y in yo..yo + block_size {
            framebuffer.set_pixel(x as u32, y as u32);
        }
    }
}

pub fn render_maze(framebuffer: &mut Framebuffer, maze: &Maze, block_size: usize, player: &Player) {
    for (row_index, row) in maze.iter().enumerate() {
        for (col_index, &cell) in row.iter().enumerate() {
            let xo = col_index * block_size;
            let yo = row_index * block_size;
            draw_cell(framebuffer, xo, yo, block_size, cell);
        }
    }

    framebuffer.set_current_color(Color::WHITESMOKE);

    // draw what the player sees
    let num_rays = 5;
    for i in 0..num_rays {
        let current_ray = i as f32 / num_rays as f32; // current ray divided by total rays
        let a = player.a - (player.fov / 2.0) + (player.fov * current_ray);
        cast_ray(framebuffer, &maze, &player, a, block_size, true);
    }
}

fn render_world(framebuffer: &mut Framebuffer, maze: &Maze, block_size: usize, player: &Player) {
    let num_rays = framebuffer.width;

    // let hw = framebuffer.width as f32 / 2.0;   // precalculated half width
    let hh = framebuffer.height as f32 / 2.0; // precalculated half height

    framebuffer.set_current_color(Color::WHITESMOKE);

    for i in 0..num_rays {
        let current_ray = i as f32 / num_rays as f32; // current ray divided by total rays
        let a = player.a - (player.fov / 2.0) + (player.fov * current_ray);
        let intersect = cast_ray(framebuffer, &maze, &player, a, block_size, false);

        // Calculate the height of the stake
        let distance_to_wall = intersect.distance; // how far is this wall from the player
        let distance_to_projection_plane = 70.0; // how far is the "player" from the "camera"
        // this ratio doesn't really matter as long as it is a function of distance
        let stake_height = (hh / distance_to_wall) * distance_to_projection_plane;

        // Calculate the position to draw the stake
        let stake_top = (hh - (stake_height / 2.0)) as usize;
        let stake_bottom = (hh + (stake_height / 2.0)) as usize;

        // Draw the stake directly in the framebuffer
        for y in stake_top..stake_bottom {
            framebuffer.set_pixel(i, y as u32); // Assuming white color for the stake
        }
    }
}

fn draw_fps(d: &mut RaylibDrawHandle, fps: u32) {
    d.draw_text(&format!("FPS: {}", fps), 10, 10, 20, Color::RAYWHITE);
}

pub fn render_minimap(
    framebuffer: &mut Framebuffer,
    maze: &Maze,
    block_size: usize,
    player: &Player,
    offset_x: usize,
    offset_y: usize,
) {
    // Dibuja el laberinto con offset
    for (row_index, row) in maze.iter().enumerate() {
        for (col_index, &cell) in row.iter().enumerate() {
            let xo = offset_x + col_index * block_size;
            let yo = offset_y + row_index * block_size;
            draw_cell(framebuffer, xo, yo, block_size, cell);
        }
    }

    // Dibuja el jugador en el minimapa
    framebuffer.set_current_color(Color::YELLOW);
    let px = offset_x + (player.pos.x / 100.0 * block_size as f32) as usize;
    let py = offset_y + (player.pos.y / 100.0 * block_size as f32) as usize;
    for x in px..px + block_size / 4 {
        for y in py..py + block_size / 4 {
            framebuffer.set_pixel(x as u32, y as u32);
        }
    }
}

fn mostrar_pantalla_bienvenida(
    window: &mut RaylibHandle,
    raylib_thread: &RaylibThread,
    window_width: i32,
    window_height: i32,
) {
    // Carga el fondo de bienvenida
    let fondo = window
        .load_texture(raylib_thread, "assets/fondo_bienvenida.jpg")
        .expect("No se pudo cargar el fondo de bienvenida");

    let fondo_width = fondo.width();
    let fondo_height = fondo.height();

    loop {
        let mut d = window.begin_drawing(raylib_thread);
        d.clear_background(Color::BLACK);

        // Dibuja el fondo
        let scale_x = window_width as f32 / fondo_width as f32;
        let scale_y = window_height as f32 / fondo_height as f32;
        let scale = scale_x.max(scale_y);

        d.draw_texture_ex(
            &fondo,
            Vector2::new(0.0, 0.0),
            0.0,
            scale,
            Color::WHITE,
        );

        // Dibuja el texto de bienvenida
        d.draw_text(
            "PRESIONA ENTER para jugar",
            window_width / 2 - 250,
            window_height / 2 - 40,
            40,
            Color::new(200, 30, 30, 255), // Rojo estilo Rayo McQueen
        );
        d.draw_text(
            "¡Bienvenido!",
            window_width / 2 - 150,
            window_height / 2 - 100,
            50,
            Color::new(255, 215, 0, 255), // Amarillo dorado
        );

        // Panel de instrucciones estilo DOOM
        let panel_width = 600;
        let panel_height = 180;
        let panel_x = window_width / 2 - panel_width / 2;
        let panel_y = window_height / 2 + 40;

        // Fondo del panel (semi-transparente)
        d.draw_rectangle(panel_x, panel_y, panel_width, panel_height, Color::new(30, 30, 30, 200));

        // Borde del panel
        d.draw_rectangle_lines(panel_x, panel_y, panel_width, panel_height, Color::new(255, 215, 0, 255));

        // Título del panel
        d.draw_text(
            "INSTRUCCIONES",
            panel_x + 20,
            panel_y + 10,
            30,
            Color::new(255, 215, 0, 255),
        );

        // Contenido del panel
        d.draw_text(
            "W / Up : Avanzar\nS / Down : Retroceder\nA / Left : Girar izquierda\nD / Right : Girar derecha\nM : Cambiar modo 2D/3D",
            panel_x + 20,
            panel_y + 50,
            24,
            Color::RAYWHITE,
        );

        // Tu nombre en la esquina inferior derecha
        d.draw_text(
            "Esteban Cárcamo",
            window_width - 260,
            window_height - 40,
            32,
            Color::new(255, 215, 0, 255),
        );

        if d.is_key_pressed(KeyboardKey::KEY_ENTER) {
            break;
        }
    }
}

fn main() {
    let window_width = 1300;
    let window_height = 900;
    let block_size = 100;

    let (mut window, raylib_thread) = raylib::init()
        .size(window_width, window_height)
        .title("DOOM")
        .log_level(TraceLogLevel::LOG_WARNING)
        .build();

    let mut framebuffer = Framebuffer::new(window_width as u32, window_height as u32);
    framebuffer.set_background_color(Color::new(50, 50, 100, 255));

    let maze = load_maze("maze.txt");
    let mut player = Player {
        pos: Vector2::new(150.0, 150.0),
        a: PI / 3.0,
        fov: PI / 3.0,
    };

    mostrar_pantalla_bienvenida(&mut window, &raylib_thread, window_width, window_height);

    while !window.window_should_close() {
        // 1. clear framebuffer
        framebuffer.clear();

        let delta_time = window.get_frame_time();

        // 2. move the player on user input
        process_events(&mut player, &window, delta_time, &maze, block_size);

        let mut mode = "3D";

        if window.is_key_down(KeyboardKey::KEY_M) {
            mode = if mode == "2D" { "3D" } else { "2D" };
        }


        if mode == "2D" {
            render_maze(&mut framebuffer, &maze, block_size, &player);
        } else {
            render_world(&mut framebuffer, &maze, block_size, &player);
        }

        let fps = window.get_fps();


        // 3. draw stuff

        // --- MINIMAPA ---
        let minimap_block_size: usize = 20;
        let minimap_offset_x: usize = (window_width as usize)
            .saturating_sub(maze[0].len() * minimap_block_size)
            .saturating_sub(20);
        let minimap_offset_y: usize = 20;
        render_minimap(
            &mut framebuffer,
            &maze,
            minimap_block_size,
            &player,
            minimap_offset_x,
            minimap_offset_y,
        );
        // --- FIN MINIMAPA ---

        framebuffer.swap_buffers(&mut window, &raylib_thread, draw_fps, fps);

        thread::sleep(Duration::from_millis(16));
    }
}
