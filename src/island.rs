use crate::{voxel_world::VoxelWorld, material::{Material, MaterialKind}};
use crate::color::Color;



pub struct IslandParams {
    pub top_radius: i32,
    pub top_height: i32,   
    pub plateau_variation: i32, 
    pub depth: i32,           
}

impl Default for IslandParams {
    fn default() -> Self {
        Self { top_radius: 6, top_height: 6, plateau_variation: 2, depth: 7 }
    }
}

fn hash2(x: i32, y: i32) -> f32 {
    let mut h = x.wrapping_mul(374761393) ^ y.wrapping_mul(668265263);
    h = (h ^ (h >> 13)).wrapping_mul(1274126177);
    ((h ^ (h >> 16)) & 0xffff) as f32 / 65535.0
}

pub fn build_island(world: &mut VoxelWorld, surface_mat: Material, trunk: Material, leaves: Material, stone_mat: Material, params: IslandParams) {
    let pr = params.top_radius;
    let h_top = params.top_height;

    // 1. Plataforma superior (plana si variation=0, ligera irregular si >0)
    if params.plateau_variation == 0 {
        for x in -pr..=pr { for z in -pr..=pr {
            if (x*x + z*z) as f32 <= (pr as f32 + 0.05).powi(2) { world.add_voxel(x, h_top, z, surface_mat); }
        }}
    } else {
        for x in -pr..=pr { for z in -pr..=pr {
            let r_norm = (x as f32 * x as f32 + z as f32 * z as f32).sqrt() / pr as f32;
            if r_norm <= 1.04 {
                let n = hash2(x, z);
                let local_bump = (n * params.plateau_variation as f32).floor() as i32; // 0..variation
                world.add_voxel(x, h_top + local_bump, z, surface_mat);
            }
        }}
    }

    // 2. Capas inferiores con estrechamiento
    for layer in 1..=params.depth {
        let y = h_top - layer;
        let shrink = layer as f32 * 0.7;
        let r_layer = (pr as f32 - shrink).max(2.0) as i32;
        for x in -r_layer..=r_layer { for z in -r_layer..=r_layer {
            let rr = x*x + z*z;
            if rr as f32 <= (r_layer as f32 + 0.25).powi(2) {
                let edge_noise = hash2(x + layer, z - layer);
                if rr as f32 > (r_layer as f32 - 1.0).powi(2) && edge_noise > 0.78 { continue; }
    
                let chosen = if layer >= 3 { stone_mat } else { surface_mat };
                world.add_voxel(x, y, z, chosen);
                if layer == params.depth && edge_noise > 0.84 { // pequeñas estalactitas
                    for extra in 1..=2 { world.add_voxel(x, y - extra, z, stone_mat); }
                }
            }
        }}
    }

    // 3. Árbol estilo Minecraft (tronco 1x1, hojas por capas) desplazado del centro
    let tree_x = -2;
    let tree_z = 1;
    let base_y = h_top + if params.plateau_variation == 0 { 0 } else {  (hash2(tree_x, tree_z) * params.plateau_variation as f32).floor() as i32 };
    let trunk_height = 5;
    for ty in 0..trunk_height { world.add_voxel(tree_x, base_y + ty, tree_z, trunk); }
    let leaf_base = base_y + trunk_height - 1;
    let add_layer = |world: &mut VoxelWorld, y: i32, size: i32| {
        let r = size/2;
        for lx in -r..=r { for lz in -r..=r {
            let ax = lx.abs(); let az = lz.abs();
            if size == 5 && ax == 2 && az == 2 { continue; } // recortar esquinas
            world.add_voxel(tree_x + lx, y, tree_z + lz, leaves);
        }}
    };
    add_layer(world, leaf_base, 5);
    add_layer(world, leaf_base + 1, 5);
    add_layer(world, leaf_base + 2, 3);
    world.add_voxel(tree_x, leaf_base + 3, tree_z, leaves);
    
        // 4. Cubo de cofre justo a la par del árbol
        let chest_x = tree_x + 1;e
        let chest_y = base_y + 1;
        let chest_z = tree_z;
        let chest_mat = Material::new_basic(Color::new(1.0, 1.0, 1.0), 0.2, 12.0, MaterialKind::Stone); 
        world.add_voxel(chest_x, chest_y, chest_z, chest_mat); 
}
