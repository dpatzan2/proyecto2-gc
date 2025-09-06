use crate::color::Color;

pub struct FrameBuffer {
	pub w: usize,
	pub h: usize,
	data: Vec<Color>,
}

impl FrameBuffer {
	pub fn new(w: usize, h: usize) -> Self {
		Self { w, h, data: vec![Color::black(); w * h] }
	}
	pub fn set(&mut self, x: usize, y: usize, c: Color) {
		if x < self.w && y < self.h {
			self.data[y * self.w + x] = c;
		}
	}
	pub fn get(&self, x: usize, y: usize) -> Color { self.data[y * self.w + x] }
	pub fn data_mut(&mut self) -> &mut [Color] { &mut self.data }
	pub fn iter(&self) -> impl Iterator<Item=&Color> { self.data.iter() }
}
