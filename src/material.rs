use crate::color::Color;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum MaterialKind {
	Terrain, 
	Trunk,  
	Leaves,  
	Glass,  
	Water,  
	Stone,
}

#[derive(Clone, Copy)]
pub struct Material {
	pub color: Color,
	pub specular: f32,
	pub shininess: f32,
	pub kind: MaterialKind,
	
	pub reflectivity: f32,   
	pub transparency: f32,  
	pub ior: f32,           
}

impl Material {
	pub fn new_basic(color: Color, specular: f32, shininess: f32, kind: MaterialKind) -> Self {
		Self { color, specular, shininess, kind, reflectivity: 0.0, transparency: 0.0, ior: 1.0 }
	}
	pub fn new_glass(color: Color, ior: f32, reflectivity: f32, transparency: f32) -> Self {
		Self { color, specular: 0.9, shininess: 180.0, kind: MaterialKind::Glass, reflectivity, transparency, ior }
	}
	pub fn new_water(color: Color) -> Self {
		Self { color, specular: 0.5, shininess: 64.0, kind: MaterialKind::Water, reflectivity: 0.0, transparency: 0.80, ior: 1.33 }
	}
	pub fn new_stone(color: Color) -> Self {
		Self { color, specular: 0.15, shininess: 18.0, kind: MaterialKind::Stone, reflectivity: 0.0, transparency: 0.0, ior: 1.0 }
	}
}
