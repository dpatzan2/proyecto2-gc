use crate::color::Color;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum MaterialKind {
	Terrain, // tierra/césped
	Trunk,   // tronco
	Leaves,  // hojas
}

#[derive(Clone, Copy)]
pub struct Material {
	pub color: Color,
	pub specular: f32,
	pub shininess: f32,
	pub kind: MaterialKind,
}
