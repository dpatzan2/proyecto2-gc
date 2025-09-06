use crate::color::{Color, Vec3};

pub struct PointLight {
	pub position: Vec3,
	pub intensity: f32,
	pub color: Color,
}
