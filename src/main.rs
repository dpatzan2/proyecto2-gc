mod camera;
mod color;
mod light;
mod material;
mod ray_intersect;
mod cube;
mod plane;
mod texture;
mod voxel_world;
mod island;
mod skybox;
mod framebuffer;

use camera::OrbitCamera;
use color::Color;
use light::PointLight;
use material::{Material, MaterialKind};
use ray_intersect::{Ray, SceneObject};

use texture::{sample_grass_block_surface, sample_trunk, sample_leaves, sample_stone};
use voxel_world::VoxelWorld;
use island::{build_island, IslandParams};


use skybox::Skybox;
use framebuffer::RLFramebuffer;

const WIDTH: i32 = 800;
const HEIGHT: i32 = 600;
const RENDER_SCALE: f32 = 1.0; 
const MAX_DEPTH: i32 = 4;

// --- Trazador recursivo con reflexión y refracción simples ---
fn trace(ray: Ray, world: &VoxelWorld, depth: i32, sun_dir: color::Vec3, sky: &Skybox) -> Color {
    if depth <= 0 { return Color::black(); }
    let mut closest: Option<ray_intersect::HitInfo> = None;
    if let Some(h) = world.intersect(&ray) { closest = Some(h); }
    if let Some(hit) = closest {
        const EPS: f32 = 4e-4;
        let light_dir = -sun_dir;
        let shadow_origin = hit.position + hit.normal * (EPS * 6.0) + light_dir * (EPS * 4.0);
        let in_shadow = if hit.material.kind == MaterialKind::Water {
            world.occluded_ignore_water(shadow_origin, light_dir, 200.0)
        } else {
            world.occluded(shadow_origin, light_dir, 200.0)
        };
        let view_dir = (-ray.dir).normalized();
        let ambient = 0.05;
        let ndotl = hit.normal.dot(light_dir).max(0.0);
        let sun_intensity = 1.4;
        let (diffuse_f, specular_f) = if !in_shadow && ndotl > 0.0 {
            let reflect_dir = (2.0 * hit.normal * ndotl - light_dir).normalized();
            let spec_angle = reflect_dir.dot(view_dir).max(0.0);
            (ndotl * sun_intensity, spec_angle.powf(hit.material.shininess) * hit.material.specular * sun_intensity)
        } else { (0.0, 0.0) };
        let mut base_col = hit.material.color;
        let mut is_water = false;
        let mut water_normal = hit.normal;
        if hit.material.kind == MaterialKind::Water {
            is_water = true;
       
            let p = hit.position * 3.3;
            let hx = ((p.x.floor() as i32).wrapping_mul(92837111) ^ (p.z.floor() as i32).wrapping_mul(689287499)) as u32;
            let hy = ((p.x.floor() as i32 + 13).wrapping_mul(362437) ^ (p.z.floor() as i32 + 17).wrapping_mul(97531)) as u32;
            let n1 = ((hx ^ (hx>>13)) & 0xffff) as f32 / 65535.0;
            let n2 = ((hy ^ (hy>>11)) & 0xffff) as f32 / 65535.0;
            let ang = n1 * std::f32::consts::TAU;
            let amp = 0.05 + 0.05 * n2;
            let t1 = if hit.normal.y.abs() < 0.9 { color::Vec3::new(0.0,1.0,0.0).cross(hit.normal).normalized() } else { color::Vec3::new(1.0,0.0,0.0) };
            let t2 = hit.normal.cross(t1).normalized();
            let ripple = (t1 * ang.cos() + t2 * ang.sin()) * amp;
            water_normal = (hit.normal + ripple).normalized();
            let up_factor = hit.normal.y.max(0.0);
            base_col = base_col * (0.25 + 0.35 * up_factor) + Color::new(0.04,0.09,0.16) * 0.55;
        }
    if hit.object_id == ray_intersect::ObjectId::Cube && hit.material.kind != MaterialKind::Glass {
            let center_pos = hit.position - hit.normal * 0.5;
            let vx = center_pos.x.round() as i32; let vy = center_pos.y.round() as i32; let vz = center_pos.z.round() as i32;
            match hit.material.kind {
                MaterialKind::Terrain => { let exposed = world.is_top_exposed(vx, vy, vz); base_col = sample_grass_block_surface(hit.normal, hit.u, hit.v, exposed); },
                MaterialKind::Trunk => { base_col = sample_trunk(hit.normal, hit.u, hit.v); },
                MaterialKind::Leaves => { base_col = sample_leaves(hit.u, hit.v); },
                MaterialKind::Stone => { base_col = sample_stone(hit.u, hit.v); },
                _ => {}
            }
        }
        // Reflexión / Refracción
    let mut refl_col = Color::black();
    let mut refr_col = Color::black();
    let n = if is_water { water_normal } else { hit.normal };
        if hit.material.reflectivity > 0.01 {
            let rdir = (ray.dir - n * 2.0 * ray.dir.dot(n)).normalized();
            let r_origin = hit.position + rdir * EPS * 6.0;
            refl_col = trace(Ray { origin: r_origin, dir: rdir }, world, depth - 1, sun_dir, sky);
        }
        if hit.material.transparency > 0.01 {
            let mut n1 = 1.0; let mut n2 = hit.material.ior;
            let mut normal = n;
            let cos_i = -normal.dot(ray.dir).max(-1.0).min(1.0);
            if cos_i < 0.0 { // dentro -> fuera
                normal = -normal; n1 = hit.material.ior; n2 = 1.0;
            }
            let eta = n1 / n2;
            let k = 1.0 - eta * eta * (1.0 - cos_i * cos_i);
            if k >= 0.0 {
                let refr_dir = (ray.dir * eta + normal * (eta * cos_i - k.sqrt())).normalized();
                let r_origin = hit.position + refr_dir * EPS * 4.0;
                if is_water {
             
                    let mut current_origin = r_origin;
                    let mut steps = 0;
                    let max_steps = 16;
                    let mut final_col = Color::black();
                    let mut hit_solid = false;
                    loop {
                        if steps >= max_steps { break; }
                        if let Some(h2) = world.intersect(&Ray{origin: current_origin, dir: refr_dir}) {
                            if h2.material.kind == MaterialKind::Water {
                                current_origin = h2.position + refr_dir * EPS * 8.0; 
                                continue;
                            } else {
                     
                                current_origin = h2.position + refr_dir * EPS * 2.0;
                                final_col = trace(Ray{origin: current_origin, dir: refr_dir}, world, depth - 1, sun_dir, sky);
                                hit_solid = true;
                                break;
                            }
                        } else {
           
                            let tsky = 0.5 * (refr_dir.y + 1.0);
                            final_col = Color::new(0.2,0.3,0.6)*(1.0 - tsky) + Color::new(0.8,0.9,1.0)*tsky;
                            break;
                        }
                    }
          
                    if !hit_solid {
                        let down = color::Vec3::new(0.0, -1.0, 0.0);
                        if let Some(_h3) = world.intersect(&Ray{ origin: current_origin, dir: down }) {
                            final_col = trace(Ray{ origin: current_origin, dir: down }, world, depth - 1, sun_dir, sky);
                        }
                    }
                    let depth_factor = (steps as f32 * 0.16).min(1.0);
                    let absorption = Color::new(0.02,0.04,0.08) * depth_factor * 0.7;
                    refr_col = (final_col * (1.0 - 0.45*depth_factor) + absorption).clamped();
                } else {
                    refr_col = trace(Ray { origin: r_origin, dir: refr_dir }, world, depth - 1, sun_dir, sky);
                }
            }
            if hit.material.reflectivity < 0.01 {
                    let r0 = ((n1 - n2) / (n1 + n2)).powi(2);
                let c = 1.0 - cos_i.abs();
                let fresnel = r0 + (1.0 - r0) * c.powi(5);

                if is_water {
                    let rdir = (ray.dir - n * 2.0 * ray.dir.dot(n)).normalized();
                    let r_origin = hit.position + rdir * EPS * 6.0;
                    let surface_ref = trace(Ray { origin: r_origin, dir: rdir }, world, depth - 1, sun_dir, sky);
                    refr_col = surface_ref * fresnel + refr_col * (1.0 - fresnel);
                    refr_col = surface_ref * fresnel + refr_col * (1.0 - fresnel);
                } else {
                    refl_col = refr_col * fresnel + refl_col * (1.0 - fresnel);
                }
            }
        }
        let base = base_col * ambient;
        let diff_col = base_col * diffuse_f;
        let spec_col = Color::white() * specular_f;
        let mut surf = base + diff_col + spec_col;
        if hit.material.transparency > 0.0 { surf = surf * (1.0 - hit.material.transparency) + refr_col * hit.material.transparency; }
        if hit.material.reflectivity > 0.0 { surf = surf * (1.0 - hit.material.reflectivity) + refl_col * hit.material.reflectivity; }
        surf.clamped()
    } else {

    sky.sample(ray.dir)
    }
}

fn main() {
    let skybox = Skybox::new();


    let (mut rl, thread) = raylib::init()
        .size(WIDTH as i32, HEIGHT as i32)
        .title("Raytracer 3D - Proyecto2 (raylib)")
        .build();
    rl.set_target_fps(60);
    let dirt_grass_mat = Material::new_basic(Color::new(0.4, 0.3, 0.2), 0.35, 24.0, MaterialKind::Terrain);
    let stone_mat = Material::new_stone(Color::new(0.5,0.5,0.52));
    let water_mat = Material::new_water(Color::new(0.25,0.4,0.55));
    let trunk_mat = Material::new_basic(Color::new(0.45, 0.28, 0.12), 0.2, 12.0, MaterialKind::Trunk);
    let leaves_mat = Material::new_basic(Color::new(0.18, 0.55, 0.22), 0.08, 8.0, MaterialKind::Leaves);
    let _glass_mat = Material::new_glass(Color::new(0.9, 0.95, 1.0), 1.52, 0.15, 0.9);

   
    let mut world = VoxelWorld::new();
    let params = IslandParams { top_radius: 7, top_height: 6, plateau_variation: 0, depth: 8 };
    let top_height = params.top_height; 
    build_island(&mut world, dirt_grass_mat, trunk_mat, leaves_mat, stone_mat, params);
    world.recompute_exposed();

    let _light = PointLight { position: color::Vec3::new(8.0, 20.0, -10.0), intensity: 3.0, color: Color::white() }; // posición ilustrativa

    let mut camera = OrbitCamera::new(color::Vec3::new(0.0, 4.0, 0.0), 10.0);
    camera.set_orbit(0.9, 0.25, 10.5);

    const POND_CX: i32 = 3;
    const POND_CZ: i32 = -2;
    const POND_R: i32 = 3;          // radio base
    let water_surface_y = top_height - 1;
    let water_deep_y = top_height - 2;

    let hash2 = |x: i32, z: i32| -> f32 {
        let mut h = x.wrapping_mul(374761393) ^ z.wrapping_mul(668265263);
        h = (h ^ (h >> 13)).wrapping_mul(1274126177);
        ((h ^ (h >> 16)) & 0xffff) as f32 / 65535.0
    };

    for x in (POND_CX-POND_R-2)..=(POND_CX+POND_R+2) { for z in (POND_CZ-POND_R-2)..=(POND_CZ+POND_R+2) {
        let dx = x - POND_CX; let dz = z - POND_CZ; let dist = ((dx*dx + dz*dz) as f32).sqrt();
        let jitter = hash2(x, z) * 1.2 - 0.6; // -0.6..+0.6
        let eff_r = POND_R as f32 + jitter;
        if dist <= eff_r { 
            world.remove_voxel(x, top_height, z);
          
            let depth_boost = if dist < (POND_R as f32 * 0.55) && hash2(x+11, z-7) > 0.35 { 1 } else { 0 };
            if depth_boost == 1 { 
                world.remove_voxel(x, water_surface_y, z);
                world.remove_voxel(x, water_deep_y, z);
                world.add_voxel(x, water_deep_y, z, water_mat);
            }
            world.add_voxel(x, water_surface_y, z, water_mat);
        } else if dist <= eff_r + 1.2 { 
            world.remove_voxel(x, water_surface_y, z); 
            if !world.has_voxel(x, top_height, z) { world.add_voxel(x, top_height, z, dirt_grass_mat); }
        }
    }}
 
    for x in (POND_CX-POND_R-1)..=(POND_CX+POND_R+1) { for z in (POND_CZ-POND_R-1)..=(POND_CZ+POND_R+1) {
        if world.has_voxel(x, water_surface_y, z) || world.has_voxel(x, water_deep_y, z) {
         
            let bed_y = if world.has_voxel(x, water_deep_y, z) { water_deep_y - 1 } else { water_surface_y - 1 };
            if !world.has_voxel(x, bed_y, z) { world.add_voxel(x, bed_y, z, dirt_grass_mat); }
        }
    }}
    world.recompute_exposed();
    let internal_w = (WIDTH as f32 * RENDER_SCALE) as u32;
    let internal_h = (HEIGHT as f32 * RENDER_SCALE) as u32;
    let mut fb = RLFramebuffer::new(internal_w, internal_h);

    let mut sun_az: f32 = -0.8;  
    let mut sun_el: f32 = 1.05;  

    let src_w = fb.width();
    let src_h = fb.height();
    while !rl.window_should_close() {

        let rot_speed = 1.0/30.0 * std::f32::consts::PI; 
    use raylib::prelude::KeyboardKey::*;
    if rl.is_key_down(KEY_LEFT) { camera.orbit_delta(-rot_speed, 0.0); }
    if rl.is_key_down(KEY_RIGHT) { camera.orbit_delta(rot_speed, 0.0); }
    if rl.is_key_down(KEY_UP) { camera.orbit_delta(0.0, rot_speed*0.5); }
    if rl.is_key_down(KEY_DOWN) { camera.orbit_delta(0.0, -rot_speed*0.5); }
    if rl.is_key_down(KEY_Q) || rl.is_key_down(KEY_Z) || rl.is_key_down(KEY_MINUS) { camera.zoom_mul(0.98); }
    if rl.is_key_down(KEY_E) || rl.is_key_down(KEY_X) || rl.is_key_down(KEY_EQUAL) { camera.zoom_mul(1.02); }
    if rl.is_key_down(KEY_J) || rl.is_key_down(KEY_A) { sun_az -= 0.03; }
    if rl.is_key_down(KEY_L) || rl.is_key_down(KEY_D) { sun_az += 0.03; }
    if rl.is_key_down(KEY_I) || rl.is_key_down(KEY_W) { sun_el = (sun_el + 0.03).min(1.45); }
    if rl.is_key_down(KEY_K) || rl.is_key_down(KEY_S) { sun_el = (sun_el - 0.03).max(0.10); }
        let ce = sun_el.cos();
        let se = sun_el.sin();
        let sun_dir = color::Vec3::new( sun_az.cos() * ce, -se, sun_az.sin() * ce ).normalized();

        // Recalcular frame
        let aspect = src_w as f32 / src_h as f32;
        // Render secuencial
        for y in 0..src_h {
            for x in 0..src_w {
                let u = x as f32 / (src_w - 1) as f32;
                let v = y as f32 / (src_h - 1) as f32;
                let ray = camera.generate_ray(u, v, aspect);
                let col = trace(ray, &world, MAX_DEPTH, sun_dir, &skybox);
                fb.set_pixel(x as u32, y as u32, col);
            }
        }
        if rl.is_key_pressed(KEY_P) { fb.save("render.png"); }
        fb.present(&mut rl, &thread);
    }
}
