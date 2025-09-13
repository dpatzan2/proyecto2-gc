use raylib::prelude::*;
use crate::color::Color as PColor;

pub struct RLFramebuffer {
    pub width: u32,
    pub height: u32,
    img: Image,
}

impl RLFramebuffer {
    pub fn new(width: u32, height: u32) -> Self {
        let img = Image::gen_image_color(width as i32, height as i32, Color::BLACK);
        Self { width, height, img }
    }
    pub fn clear(&mut self, col: PColor) {
        let rc: Color = col.into();
        self.img = Image::gen_image_color(self.width as i32, self.height as i32, rc);
    }
    pub fn set_pixel(&mut self, x: u32, y: u32, c: PColor) {
        if x < self.width && y < self.height {
            let rc: Color = c.into();
            self.img.draw_pixel(x as i32, y as i32, rc);
        }
    }
    pub fn present(&self, rl: &mut RaylibHandle, th: &RaylibThread) {
        if let Ok(tex) = rl.load_texture_from_image(th, &self.img) {
            let mut d = rl.begin_drawing(th);
            d.clear_background(Color::BLACK);
            d.draw_texture(&tex, 0, 0, Color::WHITE);
        }
    }

    pub fn width(&self) -> u32 { self.width }
    pub fn height(&self) -> u32 { self.height }
    pub fn save(&self, path: &str) { let _ = self.img.export_image(path); }
}

impl From<PColor> for Color {
    fn from(c: PColor) -> Self {
        Color::new(
            (c.r.clamp(0.0,1.0)*255.0) as u8,
            (c.g.clamp(0.0,1.0)*255.0) as u8,
            (c.b.clamp(0.0,1.0)*255.0) as u8,
            255
        )
    }
}
