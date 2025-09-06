// Cubo axis-aligned sencillo para ray tracing básico.
// Mantiene el estilo docente: claro, compacto y comentado.

use crate::color::Vec3;
use crate::material::Material;
use crate::ray_intersect::{HitInfo, Ray, SceneObject};

pub struct Cube {
    pub center: Vec3,
    pub half: f32,          // mitad del lado
    pub material: Material, // color y propiedades especulares
}

impl Cube {
    pub fn new(center: Vec3, side: f32, material: Material) -> Self {
        Self { center, half: side * 0.5, material }
    }
}

impl SceneObject for Cube {
    fn intersect(&self, ray: &Ray) -> Option<HitInfo> {
        // Extremos del AABB
        let min = self.center - Vec3::new(self.half, self.half, self.half);
        let max = self.center + Vec3::new(self.half, self.half, self.half);

        // Slab method (rápido y estable mientras dir != 0)
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

        // Determinar la cara golpeada: la componente más cercana al borde define la normal
        let bias = self.half - 1e-4; // tolerancia para decidir
        let mut normal = Vec3::new(0.0, 0.0, 0.0);
        if local.x.abs() > bias && local.x.abs() >= local.y.abs() && local.x.abs() >= local.z.abs() {
            normal = Vec3::new(local.x.signum(), 0.0, 0.0);
        } else if local.y.abs() > bias && local.y.abs() >= local.x.abs() && local.y.abs() >= local.z.abs() {
            normal = Vec3::new(0.0, local.y.signum(), 0.0);
        } else if local.z.abs() > bias && local.z.abs() >= local.x.abs() && local.z.abs() >= local.y.abs() {
            normal = Vec3::new(0.0, 0.0, local.z.signum());
        } else {
            // Centro o error numérico: fallback (raro)
            normal = Vec3::new(0.0, 1.0, 0.0);
        }

        Some(HitInfo { t: t_hit, position, normal, material: self.material })
    }
}
