
use crate::color::Vec3;
use crate::material::Material;
use crate::ray_intersect::{HitInfo, Ray, SceneObject, ObjectId};

pub struct Plane {
    pub y: f32,            
    pub material: Material,
}

impl Plane {
    pub fn new(y: f32, material: Material) -> Self { Self { y, material } }
}

impl SceneObject for Plane {
    fn intersect(&self, ray: &Ray) -> Option<HitInfo> {
    let denom = ray.dir.y;
    if denom.abs() < 1e-6 { return None; }
    let t = (self.y - ray.origin.y) / denom;
    if t < 0.0 { return None; }
    let position = ray.origin + ray.dir * t;

    let normal = Vec3::new(0.0, 1.0, 0.0);

    let u = position.x * 0.2_f32;
    let v = position.z * 0.2_f32;
    Some(HitInfo { t, position, normal, material: self.material, object_id: ObjectId::Plane, u: u.fract(), v: v.fract() })
    }
}
