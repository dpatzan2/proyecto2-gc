use crate::color::Color;

#[derive(Clone, Copy)]
pub struct Material {
	pub color: Color,
	pub specular: f32,
	pub shininess: f32,
}
