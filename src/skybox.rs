use crate::color::{Color, Vec3};


pub struct Skybox;

impl Skybox {
    pub fn new() -> Self { Skybox }
    pub fn sample(&self, dir: Vec3) -> Color {
        let d = dir.normalized();
        let t = (d.y + 1.0) * 0.5;
        let horizon = Color::new(0.78, 0.83, 0.92);
        let ground = Color::black();
        let zenith = Color::new(0.22, 0.46, 0.93);
        if t < 0.40 {
            let k = t / 0.40; ground * (1.0 - k) + horizon * k
        } else if t < 0.55 {

            horizon
        } else if t < 0.85 {
            let k = (t - 0.55) / 0.30; horizon * (1.0 - k) + zenith * k
        } else {
            zenith
        }
    }
}
