use crate::color::Vec3;
use crate::material::Material;

// Identificador simple del objeto impactado para aplicar lógica específica (texturas, etc.).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ObjectId { Cube, Plane }

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
	pub object_id: ObjectId,
	pub u: f32,
	pub v: f32,
}

pub trait SceneObject {
	fn intersect(&self, ray: &Ray) -> Option<HitInfo>;
}
