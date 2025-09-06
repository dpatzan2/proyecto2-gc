mod camera;
mod color;
mod framebuffer;
mod light;
mod material;
mod ray_intersect;
mod cube; // implementación del cubo (antes sphere.rs)
mod plane; // plano infinito simple
mod texture; // solo stub, no se usan texturas

use camera::OrbitCamera;
use color::Color;
use framebuffer::FrameBuffer;
use light::PointLight;
use material::Material;
use ray_intersect::{Ray, SceneObject};
use cube::Cube; // cubo axis-aligned
use plane::Plane;

use raylib::prelude as rl;
use raylib::prelude::RaylibDraw;
use rayon::prelude::*;

const WIDTH: i32 = 800; // resolución de ventana
const HEIGHT: i32 = 600;
const RENDER_SCALE: f32 = 0.6; // render interno (0.5-1.0). Menor = más rápido

fn main() {
    // Inicializar ventana
    let (mut rl, thread) = raylib::init()
        .size(WIDTH, HEIGHT)
        .title("Ray Tracing - Cubo Simple")
        .build();
    rl.set_target_fps(60);

    // Escena
    // Color del cubo más vivo (naranja brillante)
    let material = Material {
        color: Color::new(0.95, 0.35, 0.05),
        specular: 0.9,
        shininess: 64.0,
    };
    let cube = Cube::new(color::Vec3::new(0.0, 1.2, 0.0), 1.0, material); // cubo elevado (bottom ahora a y=0.7)

    let floor_material = Material {
        color: Color::white(), // piso blanco plano
        specular: 0.05,
        shininess: 4.0,
    };
    let plane = Plane::new(0.0, floor_material);

    let light = PointLight {
        position: color::Vec3::new(2.5, 3.0, 2.0),
        intensity: 1.4,
        color: Color::white(),
    };

    let mut camera = OrbitCamera::new(color::Vec3::new(0.0, 0.0, 0.0), 4.0);
    let internal_w = (WIDTH as f32 * RENDER_SCALE) as usize;
    let internal_h = (HEIGHT as f32 * RENDER_SCALE) as usize;
    let mut fb = FrameBuffer::new(internal_w, internal_h);

    while !rl.window_should_close() {
        camera.handle_input(&mut rl);

        // PAR render (CPU) sobre buffer interno escalado
        let aspect = internal_w as f32 / internal_h as f32;
        let slice: &mut [Color] = fb.data_mut();
        slice.par_iter_mut().enumerate().for_each(|(idx, pix)| {
            let x = idx % internal_w;
            let y = idx / internal_w;
            let u = x as f32 / (internal_w - 1) as f32;
            let v = y as f32 / (internal_h - 1) as f32;
            let ray = camera.generate_ray(u, v, aspect);
            // Intersección más cercana
            let mut closest: Option<ray_intersect::HitInfo> = None;
            if let Some(h) = cube.intersect(&ray) { closest = Some(h); }
            if let Some(hp) = plane.intersect(&ray) {
                if closest.as_ref().map(|c| hp.t < c.t).unwrap_or(true) { closest = Some(hp); }
            }
            let mut color = Color::black();
            if let Some(hit) = closest {
                const EPS: f32 = 1e-4;
                let to_light = light.position - hit.position;
                let light_distance = to_light.length();
                let light_dir = to_light * (1.0 / light_distance);
                let shadow_ray = Ray { origin: hit.position + hit.normal * EPS * 4.0, dir: light_dir };
                let mut in_shadow = false;
                if let Some(hs) = cube.intersect(&shadow_ray) { if hs.t > EPS && hs.t < light_distance { in_shadow = true; } }
                if !in_shadow { if let Some(hsp) = plane.intersect(&shadow_ray) { if hsp.t > EPS && hsp.t < light_distance { in_shadow = true; } } }
                let view_dir = (-ray.dir).normalized();
                let ambient = 0.12;
                let ndotl = hit.normal.dot(light_dir).max(0.0);
                let (diffuse, specular) = if !in_shadow && ndotl > 0.0 {
                    let reflect_dir = (2.0 * hit.normal * ndotl - light_dir).normalized();
                    let spec_angle = reflect_dir.dot(view_dir).max(0.0);
                    (ndotl, spec_angle.powf(hit.material.shininess) * hit.material.specular)
                } else { (0.0, 0.0) };
                let att = light.intensity / (1.0 + light_distance * light_distance);
                let base = hit.material.color * ambient;
                let diff_col = hit.material.color * diffuse * att;
                let spec_col = Color::white() * specular * att;
                color = (base + diff_col + spec_col).clamped();
            } else {
                let t = 0.5 * (ray.dir.y + 1.0);
                color = Color::new(0.2, 0.3, 0.6) * (1.0 - t) + Color::new(0.8, 0.9, 1.0) * t;
            }
            *pix = color;
        });

    // Nota: se podría usar una textura persistente y API update (investigar más adelante).

        let mut d = rl.begin_drawing(&thread);
        d.clear_background(rl::Color::BLACK);
        let scale_x = WIDTH as f32 / internal_w as f32;
        let scale_y = HEIGHT as f32 / internal_h as f32;
        for (idx, c) in fb.iter().enumerate() {
            let x = idx % internal_w;
            let y = idx / internal_w;
            let rx = (x as f32 * scale_x) as i32;
            let ry = (y as f32 * scale_y) as i32;
            let rw = (scale_x.ceil()) as i32;
            let rh = (scale_y.ceil()) as i32;
            d.draw_rectangle(rx, ry, rw, rh, c.to_raylib());
        }
        d.draw_text("Orbit: Flechas, Zoom: Q/E", 10, 10, 14, rl::Color::WHITE);
    }
}
