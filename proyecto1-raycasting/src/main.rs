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
use maze::{generate_maze_with_goal}; // importa la nueva función
use player::{Player, process_events};

use raylib::audio::{RaylibAudio, Sound};
use raylib::prelude::*;
use std::f32::consts::PI;
use std::thread;
use std::time::Duration;

use crate::maze::Maze;

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

fn draw_cell(framebuffer: &mut Framebuffer, xo: usize, yo: usize, block_size: usize, cell: char, goal_tex: Option<&CpuImage>) {
    if cell == ' ' {
        return;
    }
    if cell == 'g' {
        if let Some(tex) = goal_tex {
            // Dibuja la textura goal.jpg en el bloque
            for x in 0..block_size {
                for y in 0..block_size {
                    let u = x as f32 / block_size as f32;
                    let v = y as f32 / block_size as f32;
                    let color = tex.sample(u, v);
                    framebuffer.set_current_color(color);
                    framebuffer.set_pixel((xo + x) as u32, (yo + y) as u32);
                }
            }
            return;
        }
    }
    let color = cell_to_color(cell);
    framebuffer.set_current_color(color);

    for x in xo..xo + block_size {
        for y in yo..yo + block_size {
            framebuffer.set_pixel(x as u32, y as u32);
        }
    }
}

pub fn render_maze(framebuffer: &mut Framebuffer, maze: &Maze, block_size: usize, player: &Player, goal_tex: Option<&CpuImage>) {
    for (row_index, row) in maze.iter().enumerate() {
        for (col_index, &cell) in row.iter().enumerate() {
            let xo = col_index * block_size;
            let yo = row_index * block_size;
            draw_cell(framebuffer, xo, yo, block_size, cell, goal_tex);
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

fn render_world(
    framebuffer: &mut Framebuffer,
    maze: &Maze,
    block_size: usize,
    player: &Player,
    wall_tex: &CpuImage,
    goal_tex: &CpuImage,
) {
    let num_rays = framebuffer.width;
    let hh = framebuffer.height as f32 / 2.0;

    // --- Fondo cielo y piso ---
    let color_cielo = Color::new(180, 210, 255, 255); // azul claro
    let color_piso  = Color::new(210, 180, 140, 255); // arena/dorado

    for y in 0..framebuffer.height {
        let fondo_color = if (y as f32) < hh { color_cielo } else { color_piso };
        for x in 0..framebuffer.width {
            framebuffer.set_current_color(fondo_color);
            framebuffer.set_pixel(x, y);
        }
    }
    // --- Fin fondo cielo y piso ---

    framebuffer.set_current_color(Color::WHITESMOKE);

    for i in 0..num_rays {
        let current_ray = i as f32 / num_rays as f32;
        let a = player.a - (player.fov / 2.0) + (player.fov * current_ray);
        let intersect = cast_ray(framebuffer, &maze, &player, a, block_size, false);

        let distance_to_wall = intersect.distance.max(0.0001);
        let distance_to_projection_plane = 70.0;
        let stake_height = (hh / distance_to_wall) * distance_to_projection_plane;

        let stake_top = (hh - (stake_height / 2.0)) as usize;
        let stake_bottom = (hh + (stake_height / 2.0)) as usize;

        let ray_length = distance_to_wall;
        let hit_x = player.pos.x + ray_length * a.cos();
        let hit_y = player.pos.y + ray_length * a.sin();

        let wall_u = match intersect.impact {
            '|' => (hit_y / block_size as f32).fract(),
            '-' => (hit_x / block_size as f32).fract(),
            _   => 0.0,
        };

        // Selecciona la textura según el tipo de pared
        let tex = if intersect.impact == 'g' {
            goal_tex
        } else {
            wall_tex
        };

        for y in stake_top..stake_bottom {
            let v = (y - stake_top) as f32 / (stake_bottom - stake_top).max(1) as f32;
            let color = tex.sample(wall_u, v);
            framebuffer.set_current_color(color);
            framebuffer.set_pixel(i, y as u32);
        }
    }
}

fn draw_fps(d: &mut RaylibDrawHandle, fps: u32, nivel_texto: &str, window_height: i32) {
    d.draw_text(&format!("FPS: {}", fps), 10, 10, 20, Color::RAYWHITE);
    d.draw_text(
        nivel_texto,
        20,
        window_height - 40,
        32,
        Color::RAYWHITE,
    );
}

pub fn render_minimap(
    framebuffer: &mut Framebuffer,
    maze: &Maze,
    block_size: usize,
    player: &Player,
    offset_x: usize,
    offset_y: usize,
    goal_tex: Option<&CpuImage>,
) {
    // Dibuja el laberinto con offset
    for (row_index, row) in maze.iter().enumerate() {
        for (col_index, &cell) in row.iter().enumerate() {
            let xo = offset_x + col_index * block_size;
            let yo = offset_y + row_index * block_size;
            draw_cell(framebuffer, xo, yo, block_size, cell, goal_tex);
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
) -> u8 {
    let fondo = window
        .load_texture(raylib_thread, "assets/fondo_bienvenida.jpg")
        .expect("No se pudo cargar el fondo de bienvenida");

    let fondo_width = fondo.width();
    let fondo_height = fondo.height();

    let mut nivel_seleccionado: u8 = 1;

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

        // Título y texto
        d.draw_text(
            "¡Bienvenido!",
            window_width / 2 - 150,
            window_height / 2 - 120,
            50,
            Color::new(255, 215, 0, 255),
        );
        d.draw_text(
            "Selecciona nivel:",
            window_width / 2 - 180,
            window_height / 2 - 40,
            40,
            Color::RAYWHITE,
        );

        // Opciones de nivel
        let color_facil = if nivel_seleccionado == 1 { Color::YELLOW } else { Color::RAYWHITE };
        let color_medio = if nivel_seleccionado == 2 { Color::YELLOW } else { Color::RAYWHITE };
        let color_dificil = if nivel_seleccionado == 3 { Color::YELLOW } else { Color::RAYWHITE };

        d.draw_text("1 - Fácil", window_width / 2 - 100, window_height / 2 + 10, 32, color_facil);
        d.draw_text("2 - Medio", window_width / 2 - 100, window_height / 2 + 50, 32, color_medio);
        d.draw_text("3 - Difícil", window_width / 2 - 100, window_height / 2 + 90, 32, color_dificil);

        d.draw_text(
            "Presiona ENTER para comenzar",
            window_width / 2 - 200,
            window_height / 2 + 150,
            28,
            Color::new(200, 30, 30, 255),
        );

        // Cambia selección con teclas
        if d.is_key_pressed(KeyboardKey::KEY_ONE) { nivel_seleccionado = 1; }
        if d.is_key_pressed(KeyboardKey::KEY_TWO) { nivel_seleccionado = 2; }
        if d.is_key_pressed(KeyboardKey::KEY_THREE) { nivel_seleccionado = 3; }

        if d.is_key_pressed(KeyboardKey::KEY_ENTER) {
            break;
        }
    }
    nivel_seleccionado
}

pub struct CpuImage {
    pub w: usize,
    pub h: usize,
    pub pixels: Vec<Color>,
}

impl CpuImage {
    pub fn from_path(path: &str) -> Self {
        let img = Image::load_image(path).expect("No se pudo cargar la imagen");
        let w = img.width as usize;
        let h = img.height as usize;
        let pixels = img.get_image_data().to_vec();
        Self { w, h, pixels }
    }

    pub fn sample(&self, u: f32, v: f32) -> Color {
        let x = ((u.clamp(0.0, 1.0)) * (self.w as f32 - 1.0)) as usize;
        let y = ((v.clamp(0.0, 1.0)) * (self.h as f32 - 1.0)) as usize;
        self.pixels[y * self.w + x]
    }
}

fn get_maze_for_level(level: u8) -> Maze {
    match level {
        1 => generate_maze_with_goal(11, 11), // Fácil
        2 => generate_maze_with_goal(21, 21), // Medio
        3 => generate_maze_with_goal(31, 31), // Difícil
        _ => generate_maze_with_goal(11, 11), // Por defecto fácil
    }
}

fn player_reached_goal(player: &Player, maze: &Maze, block_size: usize) -> bool {
    // Busca la posición de la meta
    let mut goal_pos = None;
    for (j, row) in maze.iter().enumerate() {
        for (i, &cell) in row.iter().enumerate() {
            if cell == 'g' {
                goal_pos = Some((i, j));
                break;
            }
        }
        if goal_pos.is_some() { break; }
    }
    if let Some((goal_i, goal_j)) = goal_pos {
        let goal_x = goal_i as f32 * block_size as f32 + block_size as f32 / 2.0;
        let goal_y = goal_j as f32 * block_size as f32 + block_size as f32 / 2.0;
        let dx = player.pos.x - goal_x;
        let dy = player.pos.y - goal_y;
        let distancia = (dx * dx + dy * dy).sqrt();
        // Cambia el rango aquí, por ejemplo, al tamaño completo del bloque
        return distancia < block_size as f32;
    }
    false
}

fn mostrar_pantalla_win(
    window: &mut RaylibHandle,
    raylib_thread: &RaylibThread,
    window_width: i32,
    window_height: i32,
    win_sfx: &Sound,
) -> bool {
    win_sfx.play();

    let fondo = window
        .load_texture(raylib_thread, "assets/fondo_victoria.jpg")
        .expect("No se pudo cargar el fondo de victoria");

    let fondo_width = fondo.width();
    let fondo_height = fondo.height();

    loop {
        let mut d = window.begin_drawing(raylib_thread);
        d.clear_background(Color::BLACK);

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

        d.draw_text(
            "¡Felicidades, has ganado!",
            window_width / 2 - 250,
            window_height / 2 - 40,
            40,
            Color::new(255, 215, 0, 255),
        );
        d.draw_text(
            "PRESIONA ENTER para repetir el nivel",
            window_width / 2 - 220,
            window_height / 2 + 10,
            30,
            Color::RAYWHITE,
        );
        d.draw_text(
            "PRESIONA ESC para salir",
            window_width / 2 - 200,
            window_height / 2 + 50,
            30,
            Color::RAYWHITE,
        );

        if d.is_key_pressed(KeyboardKey::KEY_ENTER) {
            return true; // Volver al menú
        }
        if d.is_key_pressed(KeyboardKey::KEY_ESCAPE) {
            return false; // Salir
        }
    }
}

fn find_starting_position(maze: &Maze, block_size: usize) -> Vector2 {
    for (j, row) in maze.iter().enumerate() {
        for (i, &cell) in row.iter().enumerate() {
            if cell == ' ' {
                return Vector2::new(i as f32 * block_size as f32 + block_size as f32 / 2.0,
                                    j as f32 * block_size as f32 + block_size as f32 / 2.0);
            }
        }
    }
    // Si no encuentra, usa el centro
    Vector2::new(150.0, 150.0)
}

fn mostrar_pantalla_inicio(
    window: &mut RaylibHandle,
    raylib_thread: &RaylibThread,
    window_width: i32,
    window_height: i32,
) {
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

        // Título y datos
        d.draw_text(
            "Proyecto Raycaster - Rayo McQueen",
            window_width / 2 - 450,
            window_height / 2 - 180,
            48,
            Color::new(255, 215, 0, 255),
        );
        d.draw_text(
            "Autor: Esteban Cárcamo",
            window_width / 2 - 180,
            window_height / 2 - 120,
            32,
            Color::RAYWHITE,
        );
        d.draw_text(
            "Instrucciones:",
            window_width / 2 - 180,
            window_height / 2 - 60,
            32,
            Color::RAYWHITE,
        );
        d.draw_text(
            "- Usa las flechas del teclado para moverte y girar",
            window_width / 2 - 180,
            window_height / 2 - 20,
            28,
            Color::RAYWHITE,
        );
        d.draw_text(
            "- Mouse para mirar horizontalmente",
            window_width / 2 - 180,
            window_height / 2 + 10,
            28,
            Color::RAYWHITE,
        );
        d.draw_text(
            "- M para cambiar entre 2D/3D",
            window_width / 2 - 180,
            window_height / 2 + 40,
            28,
            Color::RAYWHITE,
        );
        d.draw_text(
            "Presiona ENTER para seleccionar nivel",
            window_width / 2 - 220,
            window_height / 2 + 100,
            28,
            Color::new(200, 30, 30, 255),
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
    framebuffer.set_background_color(Color::new(210, 180, 140, 255));

    // Pantalla inicial con instrucciones y datos
    mostrar_pantalla_inicio(&mut window, &raylib_thread, window_width, window_height);

    // Pantalla de selección de nivel
    let nivel_inicial = mostrar_pantalla_bienvenida(&mut window, &raylib_thread, window_width, window_height);
    let current_level: u8 = nivel_inicial;
    let maze = get_maze_for_level(current_level);

    let mut player = Player {
        pos: find_starting_position(&maze, block_size), // Usa la nueva función
        a: PI / 3.0,
        fov: PI / 3.0,
    };


    let wall_tex = CpuImage::from_path("assets/texturas/wall.jpg");
    let goal_tex = CpuImage::from_path("assets/texturas/goal.jpg"); 

    let mut mode = "3D"; // Mueve esto fuera del bucle principal

    // Inicializa el dispositivo de audio
    let audio = RaylibAudio::init_audio_device().expect("No se pudo inicializar el dispositivo de audio");
    let bg_music = audio.new_music("assets/sonidos/bg_music_taylor.wav").expect("No se pudo cargar la música de fondo");
    bg_music.set_volume(1.0);
    bg_music.play_stream();

    // Sonido de victoria
    let win_sfx = audio.new_sound("assets/sonidos/victoria.wav").expect("No se pudo cargar el sonido de victoria");

    while !window.window_should_close() {
        bg_music.update_stream();

        // 1. clear framebuffer
        framebuffer.clear();

        let delta_time = window.get_frame_time();

        // 2. move the player on user input
        process_events(&mut player, &window, delta_time, &maze, block_size);

        // Cambia el modo solo si se presiona la tecla
        if window.is_key_pressed(KeyboardKey::KEY_M) {
            mode = if mode == "2D" { "3D" } else { "2D" };
        }

        if mode == "2D" {
            render_maze(&mut framebuffer, &maze, block_size, &player, Some(&goal_tex));
        } else {
            render_world(&mut framebuffer, &maze, block_size, &player, &wall_tex, &goal_tex);
        }

        let fps = window.get_fps();


        // 3. draw stuff

        // --- MINIMAPA ---
        let minimap_block_size: usize = 12; // <-- tamaño reducido
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
            Some(&goal_tex),
        );
        // --- FIN MINIMAPA ---

        // --- Mostrar nivel actual ---
        let nivel_texto = match current_level {
            1 => "Nivel: Fácil",
            2 => "Nivel: Medio",
            3 => "Nivel: Difícil",
            _ => "Nivel: Fácil",
        };

        framebuffer.swap_buffers(
            &mut window,
            &raylib_thread,
            |d, fps| draw_fps(d, fps, nivel_texto, window_height),
            fps,
        );

        // Verifica si el jugador ha alcanzado el objetivo
        if player_reached_goal(&player, &maze, block_size) {
            if !mostrar_pantalla_win(&mut window, &raylib_thread, window_width, window_height, &win_sfx) {
                break; // Salir del juego si elige escapar
            }
            // Si vuelve al menú, recarga el laberinto y reinicia el jugador
            let maze = get_maze_for_level(current_level);
            player.pos = find_starting_position(&maze, block_size);
        }

        thread::sleep(Duration::from_millis(16));
    }
}
