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
    
    // ✅ Función auxiliar para wrap-around
    fn wrap_coordinates(&self, x: i32, y: i32) -> (usize, usize) {
        let wrapped_x = if x < 0 {
            (self.width as i32 + x) as usize
        } else if x >= self.width as i32 {
            (x - self.width as i32) as usize
        } else {
            x as usize
        };

        let wrapped_y = if y < 0 {
            (self.height as i32 + y) as usize
        } else if y >= self.height as i32 {
            (y - self.height as i32) as usize
        } else {
            y as usize
        };

        (wrapped_x, wrapped_y)
    }

    // ✅ Modificar count_neighbors para usar wrap-around
    fn count_neighbors(&self, x: usize, y: usize) -> usize {
        let mut count = 0;
        
        // Verificar las 8 celdas vecinas
        for dx in -1..=1 {
            for dy in -1..=1 {
                if dx == 0 && dy == 0 {
                    continue; // Saltar la celda central
                }
                
                let neighbor_x = x as i32 + dx;
                let neighbor_y = y as i32 + dy;
                
                // ✅ Usar wrap-around en lugar de verificar límites
                let (wrapped_x, wrapped_y) = self.wrap_coordinates(neighbor_x, neighbor_y);
                
                if self.current_generation[wrapped_y][wrapped_x] {
                    count += 1;
                }
            }
        }
        
        count
    }
    
    pub fn update(&mut self) {
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