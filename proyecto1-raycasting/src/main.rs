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
    win_sfx: &Sound, // <-- Elimina audio
) {
    win_sfx.play(); // Reproduce el sonido

    // Carga el fondo de victoria
    let fondo = window
        .load_texture(raylib_thread, "assets/fondo_victoria.jpg")
        .expect("No se pudo cargar el fondo de victoria");

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

        // Dibuja el texto de victoria
        d.draw_text(
            "¡Felicidades, has ganado!",
            window_width / 2 - 250,
            window_height / 2 - 40,
            40,
            Color::new(255, 215, 0, 255), // Amarillo dorado
        );
        d.draw_text(
            "PRESIONA ESC para salir",
            window_width / 2 - 200,
            window_height / 2 + 10,
            30,
            Color::RAYWHITE,
        );

        if d.is_key_pressed(KeyboardKey::KEY_ESCAPE) {
            break;
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
    framebuffer.set_background_color(Color::new(210, 180, 140, 255)); // Color arena/dorado

    // let maze = load_maze("maze.txt"); // <-- Comenta o elimina esta línea
    let mut current_level: u8 = 1; // Empieza en fácil
    let mut maze = get_maze_for_level(current_level);

    let mut player = Player {
        pos: find_starting_position(&maze, block_size), // Usa la nueva función
        a: PI / 3.0,
        fov: PI / 3.0,
    };

    mostrar_pantalla_bienvenida(&mut window, &raylib_thread, window_width, window_height);

    let wall_tex = CpuImage::from_path("assets/texturas/wall.jpg");
    let goal_tex = CpuImage::from_path("assets/texturas/goal.jpg"); 

    let mut mode = "3D"; // Mueve esto fuera del bucle principal

    // Inicializa el dispositivo de audio
    let mut audio = RaylibAudio::init_audio_device().expect("No se pudo inicializar el dispositivo de audio");
    let mut bg_music = audio.new_music("assets/sonidos/bg_music_taylor.wav").expect("No se pudo cargar la música de fondo");
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

        // Cambia de nivel con teclas (ejemplo: N para siguiente nivel)
        if window.is_key_pressed(KeyboardKey::KEY_N) {
            if current_level < 3 {
                current_level += 1;
                maze = get_maze_for_level(current_level);
                // Opcional: reinicia posición del jugador
                player.pos = find_starting_position(&maze, block_size);
            }
        }
        if window.is_key_pressed(KeyboardKey::KEY_B) {
            if current_level > 1 {
                current_level -= 1;
                maze = get_maze_for_level(current_level);
                player.pos = find_starting_position(&maze, block_size);
            }
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
            mostrar_pantalla_win(&mut window, &raylib_thread, window_width, window_height, &win_sfx);
            break;
        }

        thread::sleep(Duration::from_millis(16));
    }
}
