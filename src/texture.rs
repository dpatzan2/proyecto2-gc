
use crate::color::Color;
use std::path::Path;

pub struct LoadedTexture {
    pub w: u32,
    pub h: u32,
    pub data: Vec<Color>,
}

impl LoadedTexture {
    pub fn sample(&self, u: f32, v: f32) -> Color {
    if self.w == 0 || self.h == 0 { return Color::new(1.0,0.0,1.0); }
        let mut uu = u.fract(); if uu < 0.0 { uu += 1.0; }
        let mut vv = v.fract(); if vv < 0.0 { vv += 1.0; }
        let x = (uu * (self.w - 1) as f32).clamp(0.0, (self.w - 1) as f32) as u32;
        let y = (vv * (self.h - 1) as f32).clamp(0.0, (self.h - 1) as f32) as u32;
        self.data[(y * self.w + x) as usize]
    }
}

pub struct Textures {
    pub grass_top: LoadedTexture,
    pub grass_side: LoadedTexture,
    pub dirt: LoadedTexture,
    pub trunk: LoadedTexture,
    pub leaves: LoadedTexture,
    pub stone: LoadedTexture,
    pub water: LoadedTexture,
}

pub fn load_png(path: &str) -> LoadedTexture {
    if !Path::new(path).exists() {
        eprintln!("[textures] missing file: {}", path);
        return LoadedTexture { w:1, h:1, data: vec![Color::new(1.0,0.0,1.0)] };
    }
    match image::open(path) {
        Ok(img_any) => {
            let img = img_any.to_rgba8();
            let (w,h) = img.dimensions();
            let mut data = Vec::with_capacity((w*h) as usize);
            for p in img.pixels() { let [r,g,b,_a] = p.0; data.push(Color::new(r as f32/255.0, g as f32/255.0, b as f32/255.0)); }
            eprintln!("[textures] loaded {} ({}x{})", path, w, h);
            LoadedTexture { w, h, data }
        }
        Err(err) => {
            eprintln!("[textures] error loading {}: {}", path, err);
            LoadedTexture { w:1, h:1, data: vec![Color::new(1.0,0.0,1.0)] }
        }
    }
}

impl Textures {
    pub fn load_folder(folder: &str) -> Self {
        Self {
            grass_top: load_png(&format!("{}/arriba-cesped.png", folder)),
            grass_side: load_png(&format!("{}/cesped.png", folder)),
            dirt: load_png(&format!("{}/tierra.png", folder)),
            trunk: load_png(&format!("{}/tronco.png", folder)),
            leaves: load_png(&format!("{}/hojas.png", folder)),
            stone: load_png(&format!("{}/stone.png", folder)),
            water: load_png(&format!("{}/agua.png", folder)),
        }
    }
}


pub fn sample_grass_from_textures(normal: crate::color::Vec3, u: f32, v: f32, tex: &Textures, is_top_exposed: bool) -> Color {
    let ax = normal.x.abs(); let ay = normal.y.abs(); let az = normal.z.abs();
    if !is_top_exposed { return tex.dirt.sample(u,v); }
    if ay >= ax && ay >= az { 
        if normal.y > 0.0 { tex.grass_top.sample(u,v) } else { tex.dirt.sample(u,v) }
    } else {
      
        let v_flipped = 1.0 - v;
        tex.grass_side.sample(u, v_flipped)
    }
}

pub fn sample_trunk_from_textures(_normal: crate::color::Vec3, u: f32, v: f32, tex: &Textures) -> Color { tex.trunk.sample(u,v) }
pub fn sample_leaves_from_textures(u: f32, v: f32, tex: &Textures) -> Color { tex.leaves.sample(u,v) }
pub fn sample_stone_from_textures(u: f32, v: f32, tex: &Textures) -> Color { tex.stone.sample(u,v) }
pub fn sample_water_from_textures(u: f32, v: f32, tex: &Textures) -> Color { tex.water.sample(u,v) }