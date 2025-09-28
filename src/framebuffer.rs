use raylib::prelude::*;
use crate::color::Color as PColor;

pub struct RLFramebuffer {
    pub width: u32,
    pub height: u32,
    img: Image,
    buf: Vec<PColor>,
}

impl RLFramebuffer {
    pub fn new(width: u32, height: u32) -> Self {
        let img = Image::gen_image_color(width as i32, height as i32, Color::BLACK);
        let buf = vec![PColor::black(); (width * height) as usize];
        Self { width, height, img, buf }
    }
    pub fn clear(&mut self, col: PColor) {
        for v in self.buf.iter_mut() { *v = col; }
        let rc: Color = col.into();
        self.img = Image::gen_image_color(self.width as i32, self.height as i32, rc);
    }
    pub fn set_pixel(&mut self, x: u32, y: u32, c: PColor) {
        if x < self.width && y < self.height {
            let idx = (y * self.width + x) as usize;
            self.buf[idx] = c;
        }
    }
   
    pub fn present(&mut self, rl: &mut RaylibHandle, th: &RaylibThread) {
    
        for y in 0..self.height {
            for x in 0..self.width {
                let idx = (y * self.width + x) as usize;
                let rc: Color = self.buf[idx].into();
                self.img.draw_pixel(x as i32, y as i32, rc);
            }
        }
        if let Ok(tex) = rl.load_texture_from_image(th, &self.img) {
            let mut d = rl.begin_drawing(th);
            d.clear_background(Color::BLACK);
            d.draw_texture(&tex, 0, 0, Color::WHITE);
        }
    }

    pub fn width(&self) -> u32 { self.width }
    pub fn height(&self) -> u32 { self.height }
    pub fn save(&self, path: &str) { let _ = self.img.export_image(path); }
  
    pub fn replace_buffer(&mut self, buf: Vec<PColor>) {
        if buf.len() == (self.width * self.height) as usize {
            self.buf = buf;
        }
    }
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
