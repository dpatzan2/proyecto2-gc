
use crate::color::Vec3;
use crate::material::Material;
use crate::ray_intersect::{HitInfo, Ray, SceneObject, ObjectId};

pub struct Cube {
    pub center: Vec3,
    pub half: f32,          
    pub material: Material, 
    pub textured: bool,     
}

impl Cube {
    pub fn new(center: Vec3, side: f32, material: Material) -> Self { Self { center, half: side * 0.5, material, textured: false } }
    pub fn new_textured(center: Vec3, side: f32, material: Material) -> Self { Self { center, half: side * 0.5, material, textured: true } }

    pub fn face_uv(&self, position: Vec3, normal: Vec3) -> (f32, f32) {
        let local = position - self.center;
        let h = self.half;
        let (u, v) = if normal.x.abs() > 0.9 { // caras +/-X: usar (z,y)
            ((local.z / h + 1.0) * 0.5, (local.y / h + 1.0) * 0.5)
        } else if normal.y.abs() > 0.9 { // caras +/-Y: usar (x,z)
            ((local.x / h + 1.0) * 0.5, (local.z / h + 1.0) * 0.5)
        } else { // caras +/-Z: usar (x,y)
            ((local.x / h + 1.0) * 0.5, (local.y / h + 1.0) * 0.5)
        };
        (u.fract(), v.fract())
    }
}

impl SceneObject for Cube {
    fn intersect(&self, ray: &Ray) -> Option<HitInfo> {

        let min = self.center - Vec3::new(self.half, self.half, self.half);
        let max = self.center + Vec3::new(self.half, self.half, self.half);

     
        let inv = Vec3::new(1.0 / ray.dir.x, 1.0 / ray.dir.y, 1.0 / ray.dir.z);

        let mut tmin = (min.x - ray.origin.x) * inv.x;
        let mut tmax = (max.x - ray.origin.x) * inv.x;
        if tmin > tmax { std::mem::swap(&mut tmin, &mut tmax); }

        let mut tymin = (min.y - ray.origin.y) * inv.y;
        let mut tymax = (max.y - ray.origin.y) * inv.y;
        if tymin > tymax { std::mem::swap(&mut tymin, &mut tymax); }

        if tmin > tymax || tymin > tmax { return None; }
        if tymin > tmin { tmin = tymin; }
        if tymax < tmax { tmax = tymax; }

        let mut tzmin = (min.z - ray.origin.z) * inv.z;
        let mut tzmax = (max.z - ray.origin.z) * inv.z;
        if tzmin > tzmax { std::mem::swap(&mut tzmin, &mut tzmax); }

        if tmin > tzmax || tzmin > tmax { return None; }
        if tzmin > tmin { tmin = tzmin; }
        if tzmax < tmax { tmax = tzmax; }

        // Seleccionar el primer impacto positivo
        let t_hit = if tmin < 0.0 { tmax } else { tmin };
        if t_hit < 0.0 { return None; }

        let position = ray.origin + ray.dir * t_hit;
        let local = position - self.center;

      
        let bias = self.half - 1e-4; 
    let mut normal = Vec3::new(0.0, 0.0, 0.0); // will select dominant axis below
        if local.x.abs() > bias && local.x.abs() >= local.y.abs() && local.x.abs() >= local.z.abs() {
            normal = Vec3::new(local.x.signum(), 0.0, 0.0);
        } else if local.y.abs() > bias && local.y.abs() >= local.x.abs() && local.y.abs() >= local.z.abs() {
            normal = Vec3::new(0.0, local.y.signum(), 0.0);
        } else if local.z.abs() > bias && local.z.abs() >= local.x.abs() && local.z.abs() >= local.y.abs() {
            normal = Vec3::new(0.0, 0.0, local.z.signum());
        } else {
     
            normal = Vec3::new(0.0, 1.0, 0.0);
        }

    let (u, v) = self.face_uv(position, normal);
    Some(HitInfo { t: t_hit, position, normal, material: self.material, object_id: ObjectId::Cube, u, v })
    }
}
