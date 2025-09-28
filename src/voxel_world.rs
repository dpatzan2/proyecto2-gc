use std::collections::{HashMap, HashSet};
use crate::color::Vec3;
use crate::material::{Material, MaterialKind};
use crate::ray_intersect::{HitInfo, ObjectId, Ray, SceneObject};


pub struct VoxelWorld {
    voxels: HashMap<(i32,i32,i32), Material>,
    exposed: HashSet<(i32,i32,i32)>, 
    min: (i32,i32,i32),
    max: (i32,i32,i32),
}

impl VoxelWorld {
    pub fn new() -> Self { Self { voxels: HashMap::new(), exposed: HashSet::new(), min: (i32::MAX,i32::MAX,i32::MAX), max:(i32::MIN,i32::MIN,i32::MIN) } }
    pub fn add_voxel(&mut self, x:i32,y:i32,z:i32, mat: Material) {
        self.voxels.insert((x,y,z), mat);
        self.min.0 = self.min.0.min(x); self.min.1 = self.min.1.min(y); self.min.2 = self.min.2.min(z);
        self.max.0 = self.max.0.max(x); self.max.1 = self.max.1.max(y); self.max.2 = self.max.2.max(z);
    }
    pub fn remove_voxel(&mut self, x:i32,y:i32,z:i32) {
        self.voxels.remove(&(x,y,z));
      
    }
    pub fn has_voxel(&self, x:i32,y:i32,z:i32) -> bool { self.voxels.contains_key(&(x,y,z)) }
    pub fn voxel_material(&self, x:i32,y:i32,z:i32) -> Option<Material> { self.voxels.get(&(x,y,z)).copied() }
    pub fn recompute_exposed(&mut self) {
        self.exposed.clear();
        for &(x,y,z) in self.voxels.keys() {
            if !self.has_voxel(x, y+1, z) { self.exposed.insert((x,y,z)); }
        }
    }
    pub fn is_top_exposed(&self, x:i32,y:i32,z:i32) -> bool { self.exposed.contains(&(x,y,z)) }

    pub fn enforce_water_border(&mut self, terrain_mat: Material) {

        let water_positions: Vec<(i32,i32,i32)> = self.voxels.iter()
            .filter(|(_k, m)| m.kind == MaterialKind::Water)
            .map(|(k,_m)| *k).collect();
        for (x,y,z) in water_positions {
            // abajo
            if !self.has_voxel(x, y-1, z) { self.add_voxel(x, y-1, z, terrain_mat); }
      
            if !self.has_voxel(x+1,y,z) { self.add_voxel(x+1,y,z, terrain_mat); }
            if !self.has_voxel(x-1,y,z) { self.add_voxel(x-1,y,z, terrain_mat); }
            if !self.has_voxel(x,y,z+1) { self.add_voxel(x,y,z+1, terrain_mat); }
            if !self.has_voxel(x,y,z-1) { self.add_voxel(x,y,z-1, terrain_mat); }
        }
        
        self.recompute_exposed();
    }
    fn aabb_bounds(&self) -> (Vec3, Vec3) {
        if self.voxels.is_empty() { return (Vec3::new(0.0,0.0,0.0), Vec3::new(0.0,0.0,0.0)); }
        let min = Vec3::new(self.min.0 as f32 -0.5, self.min.1 as f32 -0.5, self.min.2 as f32 -0.5);
        let max = Vec3::new(self.max.0 as f32 +0.5, self.max.1 as f32 +0.5, self.max.2 as f32 +0.5);
        (min,max)
    }
    fn ray_aabb(ray: &Ray, min: Vec3, max: Vec3) -> Option<f32> {
        let inv = Vec3::new(1.0/ray.dir.x,1.0/ray.dir.y,1.0/ray.dir.z);
        let mut tmin = (min.x - ray.origin.x) * inv.x;
        let mut tmax = (max.x - ray.origin.x) * inv.x;
        if tmin>tmax { std::mem::swap(&mut tmin,&mut tmax); }
        let mut tymin = (min.y - ray.origin.y)*inv.y;
        let mut tymax = (max.y - ray.origin.y)*inv.y;
        if tymin>tymax { std::mem::swap(&mut tymin,&mut tymax); }
        if tmin>tymax || tymin>tmax { return None; }
        if tymin>tmin { tmin = tymin; }
        if tymax<tmax { tmax = tymax; }
        let mut tzmin = (min.z - ray.origin.z)*inv.z;
        let mut tzmax = (max.z - ray.origin.z)*inv.z;
        if tzmin>tzmax { std::mem::swap(&mut tzmin,&mut tzmax); }
        if tmin>tzmax || tzmin>tmax { return None; }
        if tzmin>tmin { tmin = tzmin; }
        if tzmax<tmax { tmax = tzmax; }
    if tmax < 0.0 { return None; }
    Some(tmin)
    }
    fn voxel_hit(&self, ix:i32,iy:i32,iz:i32, ray:&Ray) -> Option<HitInfo> {
        if let Some(mat) = self.voxels.get(&(ix,iy,iz)) {
           
            let min = Vec3::new(ix as f32 -0.5, iy as f32 -0.5, iz as f32 -0.5);
            let max = Vec3::new(ix as f32 +0.5, iy as f32 +0.5, iz as f32 +0.5);
            if let Some(t) = Self::ray_aabb(ray, min, max) {
                if t < 0.0 { return None; }
                let mut pos = ray.origin + ray.dir * t;
                let local = pos - Vec3::new(ix as f32, iy as f32, iz as f32);
                let bias = 0.5 - 1e-4;
                let mut normal = Vec3::new(0.0,0.0,0.0);
                if local.x.abs()>bias && local.x.abs()>=local.y.abs() && local.x.abs()>=local.z.abs() { normal = Vec3::new(local.x.signum(),0.0,0.0); }
                else if local.y.abs()>bias && local.y.abs()>=local.x.abs() && local.y.abs()>=local.z.abs() { normal = Vec3::new(0.0,local.y.signum(),0.0); }
                else if local.z.abs()>bias && local.z.abs()>=local.x.abs() && local.z.abs()>=local.y.abs() { normal = Vec3::new(0.0,0.0,local.z.signum()); }

                if mat.kind == MaterialKind::Water {
                    normal = Vec3::new(0.0, 1.0, 0.0);
                    pos.y = iy as f32 + 0.5 + 1e-4;
                }
                
                let h = 0.5;
                let (u,v) = if normal.x.abs()>0.9 { ((local.z / h +1.0)*0.5, (local.y / h +1.0)*0.5) }
                             else if normal.y.abs()>0.9 { ((local.x / h +1.0)*0.5, (local.z / h +1.0)*0.5) }
                             else { ((local.x / h +1.0)*0.5, (local.y / h +1.0)*0.5) };
                return Some(HitInfo { t, position: pos, normal, material: *mat, object_id: ObjectId::Cube, u: u.fract(), v: v.fract() });
            }
        }
        None
    }


    pub fn occluded(&self, origin: Vec3, dir: Vec3, max_t: f32) -> bool {
        if self.voxels.is_empty() { return false; }
        let mut ix = (origin.x + 0.5).floor() as i32;
        let mut iy = (origin.y + 0.5).floor() as i32;
        let mut iz = (origin.z + 0.5).floor() as i32;

        let step_x = if dir.x > 0.0 { 1 } else { -1 };
        let step_y = if dir.y > 0.0 { 1 } else { -1 };
        let step_z = if dir.z > 0.0 { 1 } else { -1 };
        let invx = if dir.x != 0.0 { 1.0/dir.x } else { f32::INFINITY };
        let invy = if dir.y != 0.0 { 1.0/dir.y } else { f32::INFINITY };
        let invz = if dir.z != 0.0 { 1.0/dir.z } else { f32::INFINITY };
        let next_boundary = |p: f32, i: i32, step: i32| -> f32 { let boundary = i as f32 + 0.5 * step as f32; boundary - p };
        let mut t_max_x = if invx.is_finite() { next_boundary(origin.x, ix, step_x) * invx } else { f32::INFINITY };
        let mut t_max_y = if invy.is_finite() { next_boundary(origin.y, iy, step_y) * invy } else { f32::INFINITY };
        let mut t_max_z = if invz.is_finite() { next_boundary(origin.z, iz, step_z) * invz } else { f32::INFINITY };
        let t_delta_x = (step_x as f32 * invx).abs();
        let t_delta_y = (step_y as f32 * invy).abs();
        let t_delta_z = (step_z as f32 * invz).abs();
        let mut t_curr = 0.0;
        for _ in 0..512 {
   
            if t_max_x < t_max_y {
                if t_max_x < t_max_z { ix += step_x; t_curr = t_max_x; t_max_x += t_delta_x; }
                else { iz += step_z; t_curr = t_max_z; t_max_z += t_delta_z; }
            } else {
                if t_max_y < t_max_z { iy += step_y; t_curr = t_max_y; t_max_y += t_delta_y; }
                else { iz += step_z; t_curr = t_max_z; t_max_z += t_delta_z; }
            }
            if t_curr > max_t { break; }
            if ix < self.min.0-1 || ix > self.max.0+1 || iy < self.min.1-1 || iy > self.max.1+1 || iz < self.min.2-1 || iz > self.max.2+1 { break; }
            if let Some(mat) = self.voxels.get(&(ix,iy,iz)) {
        
                if mat.kind != MaterialKind::Cloud {
                    return true;
                }
            }
        }
        false
    }

  
    pub fn occluded_ignore_water(&self, origin: Vec3, dir: Vec3, max_t: f32) -> bool {
        if self.voxels.is_empty() { return false; }
        let mut ix = (origin.x + 0.5).floor() as i32;
        let mut iy = (origin.y + 0.5).floor() as i32;
        let mut iz = (origin.z + 0.5).floor() as i32;

        let step_x = if dir.x > 0.0 { 1 } else { -1 };
        let step_y = if dir.y > 0.0 { 1 } else { -1 };
        let step_z = if dir.z > 0.0 { 1 } else { -1 };
        let invx = if dir.x != 0.0 { 1.0/dir.x } else { f32::INFINITY };
        let invy = if dir.y != 0.0 { 1.0/dir.y } else { f32::INFINITY };
        let invz = if dir.z != 0.0 { 1.0/dir.z } else { f32::INFINITY };
        let next_boundary = |p: f32, i: i32, step: i32| -> f32 { let boundary = i as f32 + 0.5 * step as f32; boundary - p };
        let mut t_max_x = if invx.is_finite() { next_boundary(origin.x, ix, step_x) * invx } else { f32::INFINITY };
        let mut t_max_y = if invy.is_finite() { next_boundary(origin.y, iy, step_y) * invy } else { f32::INFINITY };
        let mut t_max_z = if invz.is_finite() { next_boundary(origin.z, iz, step_z) * invz } else { f32::INFINITY };
        let t_delta_x = (step_x as f32 * invx).abs();
        let t_delta_y = (step_y as f32 * invy).abs();
        let t_delta_z = (step_z as f32 * invz).abs();
        let mut t_curr = 0.0;
        for _ in 0..512 {
            if t_max_x < t_max_y {
                if t_max_x < t_max_z { ix += step_x; t_curr = t_max_x; t_max_x += t_delta_x; }
                else { iz += step_z; t_curr = t_max_z; t_max_z += t_delta_z; }
            } else {
                if t_max_y < t_max_z { iy += step_y; t_curr = t_max_y; t_max_y += t_delta_y; }
                else { iz += step_z; t_curr = t_max_z; t_max_z += t_delta_z; }
            }
            if t_curr > max_t { break; }
            if ix < self.min.0-1 || ix > self.max.0+1 || iy < self.min.1-1 || iy > self.max.1+1 || iz < self.min.2-1 || iz > self.max.2+1 { break; }
            if let Some(mat) = self.voxels.get(&(ix,iy,iz)) {
           
                if mat.kind != MaterialKind::Water && mat.kind != MaterialKind::Cloud { return true; }
            }
        }
        false
    }
}

impl SceneObject for VoxelWorld {
    fn intersect(&self, ray: &Ray) -> Option<HitInfo> {
        if self.voxels.is_empty() { return None; }
        let (bb_min, bb_max) = self.aabb_bounds();

        let mut t_entry = match Self::ray_aabb(ray, bb_min, bb_max) { Some(t)=> t, None => return None };
        if t_entry < 0.0 { t_entry = 0.0; }
    let mut pos = ray.origin + ray.dir * t_entry;

        let mut ix = (pos.x + 0.5).floor() as i32;
        let mut iy = (pos.y + 0.5).floor() as i32;
        let mut iz = (pos.z + 0.5).floor() as i32;

        let step_x = if ray.dir.x > 0.0 { 1 } else { -1 };
        let step_y = if ray.dir.y > 0.0 { 1 } else { -1 };
        let step_z = if ray.dir.z > 0.0 { 1 } else { -1 };

        let invx = if ray.dir.x != 0.0 { 1.0/ray.dir.x } else { f32::INFINITY };
        let invy = if ray.dir.y != 0.0 { 1.0/ray.dir.y } else { f32::INFINITY };
        let invz = if ray.dir.z != 0.0 { 1.0/ray.dir.z } else { f32::INFINITY };


    let next_boundary = |p: f32, i: i32, step: i32| -> f32 { let boundary = i as f32 + 0.5 * step as f32; boundary - p }; // along axis
        let mut tMaxX = if invx.is_finite() { t_entry + next_boundary(pos.x, ix, step_x) * invx } else { f32::INFINITY };
        let mut tMaxY = if invy.is_finite() { t_entry + next_boundary(pos.y, iy, step_y) * invy } else { f32::INFINITY };
        let mut tMaxZ = if invz.is_finite() { t_entry + next_boundary(pos.z, iz, step_z) * invz } else { f32::INFINITY };
        let tDeltaX = (step_x as f32 * invx).abs();
        let tDeltaY = (step_y as f32 * invy).abs();
        let tDeltaZ = (step_z as f32 * invz).abs();

        let max_t = 200.0;
        for _ in 0..512 { 
            
            if ix < self.min.0-1 || ix > self.max.0+1 || iy < self.min.1-1 || iy > self.max.1+1 || iz < self.min.2-1 || iz > self.max.2+1 { break; }
            if let Some(hit) = self.voxel_hit(ix,iy,iz, ray) { return Some(hit); }
            if tMaxX < tMaxY {
                if tMaxX < tMaxZ { ix += step_x; t_entry = tMaxX; tMaxX += tDeltaX; }
                else { iz += step_z; t_entry = tMaxZ; tMaxZ += tDeltaZ; }
            } else {
                if tMaxY < tMaxZ { iy += step_y; t_entry = tMaxY; tMaxY += tDeltaY; }
                else { iz += step_z; t_entry = tMaxZ; tMaxZ += tDeltaZ; }
            }
            if t_entry > max_t { break; }
        }
        None
    }
}
