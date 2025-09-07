use crate::color::Vec3;
use crate::ray_intersect::Ray;
use raylib::prelude::*;

pub struct OrbitCamera {
    target: Vec3,
    radius: f32,
    yaw: f32,
    pitch: f32,
}

impl OrbitCamera {
    pub fn new(target: Vec3, radius: f32) -> Self {
        // yaw inicial desplazado y pitch más bajo para vista 3/4
        Self { target, radius, yaw: 0.9, pitch: 0.25 }
    }

    pub fn handle_input(&mut self, rl: &mut RaylibHandle) {
        let rot_speed = 1.2 * rl.get_frame_time();
        if rl.is_key_down(KeyboardKey::KEY_LEFT) { self.yaw -= rot_speed; }
        if rl.is_key_down(KeyboardKey::KEY_RIGHT) { self.yaw += rot_speed; }
        if rl.is_key_down(KeyboardKey::KEY_UP) { self.pitch += rot_speed; }
        if rl.is_key_down(KeyboardKey::KEY_DOWN) { self.pitch -= rot_speed; }
        self.pitch = self.pitch.clamp(-1.2, 1.2);
        if rl.is_key_down(KeyboardKey::KEY_Q) { self.radius *= 0.97; }
        if rl.is_key_down(KeyboardKey::KEY_E) { self.radius *= 1.03; }
    // Aumentar el mínimo para no penetrar la isla (radio ~4)
    self.radius = self.radius.clamp(5.0, 25.0);
    }

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
