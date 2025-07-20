use raylib::prelude::*;

pub struct GameOfLife {
    width: usize,
    height: usize,
    current_generation: Vec<Vec<bool>>,
    next_generation: Vec<Vec<bool>>,
}

impl GameOfLife {
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            width,
            height,
            current_generation: vec![vec![false; width]; height],
            next_generation: vec![vec![false; width]; height],
        }
    }

    
    
    pub fn set_alive(&mut self, x: usize, y: usize) {
        if x < self.width && y < self.height {
            self.current_generation[y][x] = true;
        }
    }
    
    fn count_neighbors(&self, x: usize, y: usize) -> u8 {
        let mut count = 0;
        
        for dx in -1i32..=1i32 {
            for dy in -1i32..=1i32 {
                if dx == 0 && dy == 0 { continue; } // No contamos la célula misma
                
                let nx = x as i32 + dx;
                let ny = y as i32 + dy;
                
                // Verificar límites
                if nx >= 0 && ny >= 0 && nx < self.width as i32 && ny < self.height as i32 {
                    if self.current_generation[ny as usize][nx as usize] {
                        count += 1;
                    }
                }
            }
        }
        count
    }
    
    pub fn update(&mut self) {
        // Calcular la siguiente generación
        for y in 0..self.height {
            for x in 0..self.width {
                let neighbors = self.count_neighbors(x, y);
                let is_alive = self.current_generation[y][x];
                
                self.next_generation[y][x] = match (is_alive, neighbors) {
                    // Célula viva con 2 o 3 vecinos sobrevive
                    (true, 2) | (true, 3) => true,
                    // Célula muerta con exactamente 3 vecinos nace
                    (false, 3) => true,
                    // Cualquier otro caso: muerte
                    _ => false,
                };
            }
        }
        
        // Intercambiar generaciones
        std::mem::swap(&mut self.current_generation, &mut self.next_generation);
    }
    
    pub fn render(&self, framebuffer: &mut crate::framebuffer::Framebuffer, scale: i32) {
        // Calcular offset para centrar el juego en la pantalla
        let game_pixel_width = self.width as i32 * scale;
        let game_pixel_height = self.height as i32 * scale;
        
        // Asumir que el framebuffer es 800x800
        let offset_x = (800 - game_pixel_width) / 2;
        let offset_y = (800 - game_pixel_height) / 2;
        
        for y in 0..self.height {
            for x in 0..self.width {
                if self.current_generation[y][x] {
                    for dy in 0..scale {
                        for dx in 0..scale {
                            framebuffer.point(Vector2 {
                                x: (offset_x + x as i32 * scale + dx) as f32,
                                y: (offset_y + y as i32 * scale + dy) as f32,
                            });
                        }
                    }
                }
            }
        }
    }
    
}