use std::ops::{Add, Mul, Sub};

// --- Color -------------------------------------------------------------

#[derive(Clone, Copy, Debug)]
pub struct Color {
    pub r: f32,
    pub g: f32,
    pub b: f32,
}

impl Color {
    pub fn new(r: f32, g: f32, b: f32) -> Self { Self { r, g, b } }
    pub fn black() -> Self { Self::new(0.0, 0.0, 0.0) }
    pub fn white() -> Self { Self::new(1.0, 1.0, 1.0) }
    pub fn clamped(self) -> Self {
        Self::new(
            self.r.clamp(0.0, 1.0),
            self.g.clamp(0.0, 1.0),
            self.b.clamp(0.0, 1.0),
        )
    }
    pub fn to_rgb8(self) -> [u8;3] { [(self.r * 255.0) as u8, (self.g * 255.0) as u8, (self.b * 255.0) as u8] }
}

impl Add for Color {
    type Output = Color;
    fn add(self, o: Color) -> Color { Color::new(self.r + o.r, self.g + o.g, self.b + o.b) }
}
impl Mul<f32> for Color {
    type Output = Color;
    fn mul(self, s: f32) -> Color { Color::new(self.r * s, self.g * s, self.b * s) }
}
impl Mul<Color> for Color {
    type Output = Color;
    fn mul(self, o: Color) -> Color { Color::new(self.r * o.r, self.g * o.g, self.b * o.b) }
}
impl Sub for Color {
    type Output = Color;
    fn sub(self, o: Color) -> Color { Color::new(self.r - o.r, self.g - o.g, self.b - o.b) }
}

// --- Vec3 --------------------------------------------------------------

#[derive(Clone, Copy, Debug)]
pub struct Vec3 { pub x: f32, pub y: f32, pub z: f32 }

impl Vec3 {
    pub fn new(x: f32, y: f32, z: f32) -> Self { Self { x, y, z } }
    pub fn dot(self, o: Vec3) -> f32 { self.x * o.x + self.y * o.y + self.z * o.z }
    pub fn length(self) -> f32 { self.dot(self).sqrt() }
    pub fn normalized(self) -> Self {
        let l = self.length();
        if l > 0.0 { Self::new(self.x / l, self.y / l, self.z / l) } else { self }
    }
    pub fn cross(self, o: Vec3) -> Vec3 {
        Vec3::new(
            self.y * o.z - self.z * o.y,
            self.z * o.x - self.x * o.z,
            self.x * o.y - self.y * o.x,
        )
    }
}

use std::ops::{AddAssign, Mul as StdMul, Neg};
impl Add for Vec3 { type Output = Vec3; fn add(self, o: Vec3) -> Vec3 { Vec3::new(self.x + o.x, self.y + o.y, self.z + o.z) } }
impl AddAssign for Vec3 { fn add_assign(&mut self, o: Vec3) { self.x += o.x; self.y += o.y; self.z += o.z; } }
impl Sub for Vec3 { type Output = Vec3; fn sub(self, o: Vec3) -> Vec3 { Vec3::new(self.x - o.x, self.y - o.y, self.z - o.z) } }
impl StdMul<f32> for Vec3 { type Output = Vec3; fn mul(self, s: f32) -> Vec3 { Vec3::new(self.x * s, self.y * s, self.z * s) } }
impl Neg for Vec3 { type Output = Vec3; fn neg(self) -> Vec3 { Vec3::new(-self.x, -self.y, -self.z) } }

// scalar * Vec3 (simétrico)
impl std::ops::Mul<Vec3> for f32 { type Output = Vec3; fn mul(self, v: Vec3) -> Vec3 { Vec3::new(v.x * self, v.y * self, v.z * self) } }

// Conversión rápida (ej. para debug)
impl From<Vec3> for Color { fn from(v: Vec3) -> Self { Color::new(v.x, v.y, v.z) } }
