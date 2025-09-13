use crate::color::Vec3;
use crate::ray_intersect::Ray;

pub struct OrbitCamera {
    target: Vec3,
    radius: f32,
    yaw: f32,
    pitch: f32,
}

impl OrbitCamera {
    pub fn new(target: Vec3, radius: f32) -> Self {
       
        Self { target, radius, yaw: 0.9, pitch: 0.25 }
    }

    pub fn set_orbit(&mut self, yaw: f32, pitch: f32, radius: f32) {
        self.yaw = yaw; self.pitch = pitch.clamp(-1.2, 1.2); self.radius = radius.clamp(5.0, 25.0);
    }
    pub fn orbit_delta(&mut self, dyaw: f32, dpitch: f32) {
        self.yaw += dyaw;
        self.pitch = (self.pitch + dpitch).clamp(-1.2, 1.2);
    }
    pub fn zoom_mul(&mut self, factor: f32) { self.radius = (self.radius * factor).clamp(5.0, 25.0); }

    pub fn position(&self) -> Vec3 {
        let x = self.radius * self.yaw.cos() * self.pitch.cos();
        let y = self.radius * self.pitch.sin();
        let z = self.radius * self.yaw.sin() * self.pitch.cos();
        Vec3::new(x, y, z) + self.target
    }

    pub fn generate_ray(&self, u: f32, v: f32, aspect: f32) -> Ray {
        // u,v en [0,1]
        let fov = 60.0_f32.to_radians();
        let px = (2.0 * u - 1.0) * aspect * (fov * 0.5).tan();
        let py = (1.0 - 2.0 * v) * (fov * 0.5).tan();
        let cam_pos = self.position();
        let forward = (self.target - cam_pos).normalized();
        let world_up = Vec3::new(0.0, 1.0, 0.0);
        let right = forward.cross(world_up).normalized();
        let up = right.cross(forward).normalized();
        let dir = (forward + right * px + up * py).normalized();
        Ray { origin: cam_pos, dir }
    }
}
