mod camera;
mod color;
mod framebuffer;
mod light;
mod material;
mod ray_intersect;
mod cube; // implementación del cubo
mod plane; // plano infinito simple
mod texture; // solo stub, no se usan texturas
mod voxel_world;
mod island;

use camera::OrbitCamera;
use color::Color;
use framebuffer::FrameBuffer;
use light::PointLight;
use material::{Material, MaterialKind};
use ray_intersect::{Ray, SceneObject};
use cube::Cube; // (quedó para construcción preliminar si se desea)
use texture::{sample_grass_block_surface, sample_trunk, sample_leaves};
use voxel_world::VoxelWorld;
use island::{build_island, IslandParams};
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

    // ---------- Escena: Isla flotante estilo SkyWars (procedural simple) ----------
    // Material base (solo se usa specular/shininess porque el color vendrá de textura procedural).
    let dirt_grass_mat = Material { color: Color::new(0.4, 0.3, 0.2), specular: 0.35, shininess: 24.0, kind: MaterialKind::Terrain };
    let trunk_mat = Material { color: Color::new(0.45, 0.28, 0.12), specular: 0.2, shininess: 12.0, kind: MaterialKind::Trunk };
    let leaves_mat = Material { color: Color::new(0.18, 0.55, 0.22), specular: 0.08, shininess: 8.0, kind: MaterialKind::Leaves };

    // Generar agregados de cubos en forma de elipse con variación de altura y colgantes.
    let mut world = VoxelWorld::new();
    let params = IslandParams { top_radius: 7, top_height: 6, plateau_variation: 0, depth: 8 };
    build_island(&mut world, dirt_grass_mat, trunk_mat, leaves_mat, params);
    world.recompute_exposed();

    // Eliminamos el plano: la isla está flotando
    // let plane = Plane::new(0.0, floor_material);

    // Luz tipo "sol" pre-mediodía: alta y algo desplazada al norte (negative Z) y este (positive X)
    // Luz tipo sol direccional (usaremos la posición solo para mostrar, pero el shading será direccional)
    let light = PointLight { position: color::Vec3::new(8.0, 20.0, -10.0), intensity: 3.0, color: Color::white() }; // posición ilustrativa

    let mut camera = OrbitCamera::new(color::Vec3::new(0.0, 4.0, 0.0), 10.0);
    let internal_w = (WIDTH as f32 * RENDER_SCALE) as usize;
    let internal_h = (HEIGHT as f32 * RENDER_SCALE) as usize;
    let mut fb = FrameBuffer::new(internal_w, internal_h);

    // Parámetros de la luz direccional (sol) controlables
    let mut sun_az: f32 = -0.8;  // azimut en radianes (rotación en plano XZ)
    let mut sun_el: f32 = 1.05;  // elevación (0=horizonte, ~1.57=zenit)

    while !rl.window_should_close() {
    camera.handle_input(&mut rl);

    // Controles de luz: J/L azimut, I/K elevación
    if rl.is_key_down(rl::KeyboardKey::KEY_J) { sun_az -= 0.02; }
    if rl.is_key_down(rl::KeyboardKey::KEY_L) { sun_az += 0.02; }
    if rl.is_key_down(rl::KeyboardKey::KEY_I) { sun_el = (sun_el + 0.02).min(1.45); }
    if rl.is_key_down(rl::KeyboardKey::KEY_K) { sun_el = (sun_el - 0.02).max(0.10); }
    // Calcular dirección del sol (vector desde el sol hacia la escena)
    let ce = sun_el.cos();
    let se = sun_el.sin();
    let sun_dir = color::Vec3::new( sun_az.cos() * ce, -se, sun_az.sin() * ce ).normalized();

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
            if let Some(h) = world.intersect(&ray) { closest = Some(h); }
            let mut color = Color::black();
            if let Some(hit) = closest {
                const EPS: f32 = 4e-4;
                // Dirección del sol precomputada
                let light_dir = -sun_dir; // desde punto hacia el sol
                // Ray de sombra infinito (limitamos distancia máxima razonable)
                let shadow_origin = hit.position + hit.normal * (EPS * 6.0) + light_dir * (EPS * 4.0);
                let mut in_shadow = world.occluded(shadow_origin, light_dir, 200.0);
                let view_dir = (-ray.dir).normalized();
                let ambient = 0.05; // menor para que destaque la iluminación directa
                let ndotl = hit.normal.dot(light_dir).max(0.0);
                let sun_intensity = 1.4; // se puede subir para más contraste
                let (diffuse, specular) = if !in_shadow && ndotl > 0.0 {
                    let reflect_dir = (2.0 * hit.normal * ndotl - light_dir).normalized();
                    let spec_angle = reflect_dir.dot(view_dir).max(0.0);
                    (ndotl * sun_intensity, spec_angle.powf(hit.material.shininess) * hit.material.specular * sun_intensity)
                } else { (0.0, 0.0) };
                let att = 1.0; // sin atenuación para luz direccional
                // Base color (material o textura procedural si es el cubo)
                let mut base_col = hit.material.color;
                if hit.object_id == ray_intersect::ObjectId::Cube {
                    let center_pos = hit.position - hit.normal * 0.5;
                    let vx = center_pos.x.round() as i32;
                    let vy = center_pos.y.round() as i32;
                    let vz = center_pos.z.round() as i32;
                    match hit.material.kind {
                        MaterialKind::Terrain => {
                            let exposed = world.is_top_exposed(vx, vy, vz);
                            base_col = sample_grass_block_surface(hit.normal, hit.u, hit.v, exposed);
                        },
                        MaterialKind::Trunk => {
                            base_col = sample_trunk(hit.normal, hit.u, hit.v);
                        },
                        MaterialKind::Leaves => {
                            base_col = sample_leaves(hit.u, hit.v);
                        }
                    }
                }
                let base = base_col * ambient;
                let diff_col = base_col * diffuse * att;
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
    let info = format!("Orbit Flechas | Zoom Q/E | Luz J/L azimut I/K elev | az:{:.2} el:{:.2}", sun_az, sun_el);
    d.draw_text(&info, 10, 10, 14, rl::Color::WHITE);
    }
}
