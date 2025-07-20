use raylib::prelude::*;

pub struct Framebuffer {
    image: Image,
    background_color: Color,
    current_color:    Color,
}

impl Framebuffer {
    pub fn new(width: i32, height: i32) -> Self {
        let bg = Color::BLACK;
        let img = Image::gen_image_color(width, height, bg);
        Framebuffer { image: img, background_color: bg, current_color: Color::WHITE }
    }

    pub fn set_background_color(&mut self, color: Color) {
        self.background_color = color;
    }

    pub fn clear(&mut self) {
        let w = self.image.width();
        let h = self.image.height();
        self.image = Image::gen_image_color(w, h, self.background_color);
    }

    pub fn set_current_color(&mut self, color: Color) {
        self.current_color = color;
    }

    pub fn point(&mut self, pos: Vector2) {
        let x = pos.x as i32;
        let h = self.image.height();
        let y_src = pos.y as i32;
        let y = h - 1 - y_src;
        self.image.draw_pixel(x, y, self.current_color);
    }

    pub fn render_to_file(&self, file: &str) {
        self.image.export_image(file);
    }

    pub fn swap_buffers(
        &self,
        window: &mut RaylibHandle,
        raylib_thread: &RaylibThread
    ){
        if let Ok(texture) = window.load_texture_from_image(raylib_thread, &self.image){
            let mut renderer = window.begin_drawing(raylib_thread);

            renderer.draw_texture(&texture, 0, 0, Color::WHITE);
        }
    }

    // pub fn get_color(&self, x: i32, y:i32) -> Color {
    //     if x<0 || y<0  || x>= self.image.width() || y>= self.image.height() {
    //         return Color::BLACK; 
    //     }
    //     let h = self.image.height();
    //     let y_flipped = h - 1 - y; 

    //     Color::BLACK
    // }

}