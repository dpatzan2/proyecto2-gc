use crate::color::Vec3;
use crate::material::Material;

#[derive(Clone, Copy, Debug)]
pub struct Ray {
	pub origin: Vec3,
	pub dir: Vec3,
}

pub struct HitInfo {
	pub t: f32,
	pub position: Vec3,
	pub normal: Vec3,
	pub material: Material,
}

pub trait SceneObject {
	fn intersect(&self, ray: &Ray) -> Option<HitInfo>;
}
