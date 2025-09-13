
use crate::color::Color;

// Hash determinista 2D -> [0,1] - mejorado para mejor distribución
fn hash2(x: i32, y: i32) -> f32 {
    let mut h = x.wrapping_mul(374761393) ^ y.wrapping_mul(668265263);
    h = (h ^ (h >> 13)).wrapping_mul(1274126177);
    h = h ^ (h >> 16);
    ((h & 0xffff) as f32) / 65535.0
}

// Función para crear patrones de píxeles discretos (estilo Minecraft)
fn pixelated_noise(u: f32, v: f32, scale: i32) -> f32 {
    let x = (u * scale as f32).floor() as i32;
    let y = (v * scale as f32).floor() as i32;
    hash2(x, y)
}

// Función para mezclar colores de manera más realista
fn blend_colors(base: Color, overlay: Color, factor: f32) -> Color {
    Color::new(
        base.r * (1.0 - factor) + overlay.r * factor,
        base.g * (1.0 - factor) + overlay.g * factor,
        base.b * (1.0 - factor) + overlay.b * factor,
    )
}

// Césped (top): Verde vibrante con patrón de píxeles como Minecraft
pub fn sample_grass_top(u: f32, v: f32) -> Color {
    
    let pixel_u = (u * 16.0).floor() / 16.0;
    let pixel_v = (v * 16.0).floor() / 16.0;
    
    let noise1 = pixelated_noise(pixel_u, pixel_v, 16);
    let noise2 = pixelated_noise(pixel_u * 2.0, pixel_v * 2.0, 32);
    

    let base_green = Color::new(0.486, 0.733, 0.216); 
    let dark_green = Color::new(0.357, 0.565, 0.153);   
    let light_green = Color::new(0.565, 0.824, 0.259);  
 
    let mut result = base_green;
    if noise1 > 0.6 {
        result = blend_colors(result, dark_green, 0.4);
    } else if noise1 < 0.3 {
        result = blend_colors(result, light_green, 0.3);
    }
    
   
    if noise2 > 0.7 {
        result = blend_colors(result, dark_green, 0.2);
    }
    
    result.clamped()
}

// Tierra (bottom): Marrón con textura granular como Minecraft
pub fn sample_dirt(u: f32, v: f32) -> Color {
    let pixel_u = (u * 16.0).floor() / 16.0;
    let pixel_v = (v * 16.0).floor() / 16.0;
    
    let noise1 = pixelated_noise(pixel_u, pixel_v, 16);
    let noise2 = pixelated_noise(pixel_u * 1.5, pixel_v * 1.5, 24);
    
   
    let base_brown = Color::new(0.545, 0.396, 0.282);
    let dark_brown = Color::new(0.463, 0.318, 0.208);
    let light_brown = Color::new(0.627, 0.475, 0.357);
    let reddish = Color::new(0.592, 0.384, 0.255);

    let mut result = base_brown;
    

    if noise1 > 0.65 {
        result = blend_colors(result, dark_brown, 0.5);
    } else if noise1 < 0.25 {
        result = blend_colors(result, light_brown, 0.4);
    } else if noise1 > 0.45 && noise1 < 0.55 {
        result = blend_colors(result, reddish, 0.3);
    }
    

    if noise2 > 0.8 {
        result = blend_colors(result, dark_brown, 0.15);
    }
    
    result.clamped()
}


pub fn sample_grass_side(u: f32, v: f32) -> Color {
    let pixel_u = (u * 16.0).floor() / 16.0;
    let pixel_v = (v * 16.0).floor() / 16.0;
    
  
    let grass_height = 0.8;
    
    if v > grass_height {

        let noise = pixelated_noise(pixel_u, pixel_v, 16);
        let base_green = Color::new(0.357, 0.565, 0.153);
        let dark_green = Color::new(0.278, 0.463, 0.118);
        
        if noise > 0.5 {
            blend_colors(base_green, dark_green, 0.3)
        } else {
            base_green
        }
    } else {
      
        sample_dirt(pixel_u, pixel_v * 1.2)
    }
}

// Función principal para bloques de césped
pub fn sample_grass_block(normal: crate::color::Vec3, u: f32, v: f32) -> Color {
    let ax = normal.x.abs();
    let ay = normal.y.abs();
    let az = normal.z.abs();
    
    if ay >= ax && ay >= az { 
        if normal.y > 0.0 { 
            sample_grass_top(u, v) 
        } else { 
            sample_dirt(u, v) 
        }
    } else { // caras laterales
        sample_grass_side(u, v)
    }
}

// Versión con exposición al aire
pub fn sample_grass_block_surface(normal: crate::color::Vec3, u: f32, v: f32, is_top_exposed: bool) -> Color {
    if !is_top_exposed { 
        return sample_dirt(u, v); 
    }
    sample_grass_block(normal, u, v)
}


pub fn sample_trunk(normal: crate::color::Vec3, u: f32, v: f32) -> Color {
    let ax = normal.x.abs();
    let ay = normal.y.abs(); 
    let az = normal.z.abs();
    
    if ay >= ax && ay >= az {
        let pixel_u = (u * 16.0).floor() / 16.0;
        let pixel_v = (v * 16.0).floor() / 16.0;
        
  
        let center_u = 0.5;
        let center_v = 0.5;
        let dist = ((u - center_u) * (u - center_u) + (v - center_v) * (v - center_v)).sqrt();
        
        let noise = pixelated_noise(pixel_u, pixel_v, 16);
        
     
        let light_wood = Color::new(0.686, 0.573, 0.427);  
        let medium_wood = Color::new(0.588, 0.478, 0.337); 
        let dark_wood = Color::new(0.478, 0.384, 0.267);   
        

        let ring_factor = (dist * 8.0).sin() * 0.5 + 0.5;
        let mut result = blend_colors(medium_wood, light_wood, ring_factor * 0.3);

        if noise > 0.7 {
            result = blend_colors(result, dark_wood, 0.3);
        } else if noise < 0.3 {
            result = blend_colors(result, light_wood, 0.2);
        }
        
        result.clamped()
    } else {
        let pixel_u = (u * 16.0).floor() / 16.0;
        let pixel_v = (v * 16.0).floor() / 16.0;
        
        let noise1 = pixelated_noise(pixel_u, pixel_v, 16);
        let noise2 = pixelated_noise(pixel_u * 0.5, pixel_v * 2.0, 12);
        

        let base_bark = Color::new(0.427, 0.337, 0.227);
        let dark_bark = Color::new(0.318, 0.247, 0.157);
        let light_bark = Color::new(0.537, 0.427, 0.298);

        let mut result = base_bark;
        
  
        if noise2 > 0.6 {
            result = blend_colors(result, dark_bark, 0.4);
        }
     
        if noise1 > 0.75 {
            result = blend_colors(result, dark_bark, 0.3);
        } else if noise1 < 0.25 {
            result = blend_colors(result, light_bark, 0.2);
        }
        
        result.clamped()
    }
}

// Hojas de roble - verde más natural con transparencia simulada
pub fn sample_leaves(u: f32, v: f32) -> Color {
    let pixel_u = (u * 16.0).floor() / 16.0;
    let pixel_v = (v * 16.0).floor() / 16.0;
    
    let noise1 = pixelated_noise(pixel_u, pixel_v, 16);
    let noise2 = pixelated_noise(pixel_u * 1.3, pixel_v * 1.7, 20);
    

    let base_leaf = Color::new(0.298, 0.627, 0.216);    
    let dark_leaf = Color::new(0.208, 0.478, 0.145);    
    let light_leaf = Color::new(0.388, 0.714, 0.278);   
    let yellow_tint = Color::new(0.506, 0.667, 0.247);  
    
    let mut result = base_leaf;
    

    if noise1 > 0.7 {
        result = blend_colors(result, dark_leaf, 0.4);
    } else if noise1 < 0.2 {
        result = blend_colors(result, light_leaf, 0.3);
    } else if noise1 > 0.4 && noise1 < 0.6 {
        result = blend_colors(result, yellow_tint, 0.2);
    }
    

    if noise2 > 0.8 {
        result = blend_colors(result, dark_leaf, 0.2);
    }
    
    result.clamped()
}

// Stone 
pub fn sample_stone(u: f32, v: f32) -> Color {
    let pixel_u = (u * 16.0).floor() / 16.0;
    let pixel_v = (v * 16.0).floor() / 16.0;
    
    let noise1 = pixelated_noise(pixel_u, pixel_v, 16);
    let noise2 = pixelated_noise(pixel_u * 1.5, pixel_v * 1.5, 24);
    

    let base_stone = Color::new(0.502, 0.502, 0.502);    
    let dark_stone = Color::new(0.392, 0.392, 0.392);    
    let light_stone = Color::new(0.612, 0.612, 0.612);   
    
    let mut result = base_stone;
    
    if noise1 > 0.65 {
        result = blend_colors(result, dark_stone, 0.4);
    } else if noise1 < 0.3 {
        result = blend_colors(result, light_stone, 0.3);
    }
    
    if noise2 > 0.8 {
        result = blend_colors(result, dark_stone, 0.15);
    }
    
    result.clamped()
}


pub struct Texture;