// Plano infinito horizontal (y = const) para proyectar sombras del cubo.
use crate::color::Vec3;
use crate::material::Material;
use crate::ray_intersect::{HitInfo, Ray, SceneObject};

pub struct Plane {
    pub y: f32,            // altura del plano (y = this)
    pub material: Material,
}

impl Plane {
    pub fn new(y: f32, material: Material) -> Self { Self { y, material } }
}

impl SceneObject for Plane {
    fn intersect(&self, ray: &Ray) -> Option<HitInfo> {
        // Plano con normal (0,1,0): ecuación y = self.y
    let denom = ray.dir.y;
    if denom.abs() < 1e-6 { return None; }
    let t = (self.y - ray.origin.y) / denom;
    if t < 0.0 { return None; }
    let position = ray.origin + ray.dir * t;
    // Normal fija hacia arriba (0,1,0)
    let normal = Vec3::new(0.0, 1.0, 0.0);
    Some(HitInfo { t, position, normal, material: self.material })
    }
}
