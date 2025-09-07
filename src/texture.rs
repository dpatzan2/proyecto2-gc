// Sistema mínimo de "texturas" procedurales para evitar incluir assets con copyright.
// Objetivo: simular (sin copiar) la apariencia de un bloque de césped estilo voxel.
// Se generan patrones con ruido hash simple para top (césped), side (transición) y bottom (tierra).

use crate::color::Color;

// Hash determinista 2D -> [0,1]
fn hash2(x: i32, y: i32) -> f32 {
	let mut h = x.wrapping_mul(374761393) ^ y.wrapping_mul(668265263);
	h = (h ^ (h >> 13)).wrapping_mul(1274126177);
	((h ^ (h >> 16)) & 0xffff) as f32 / 65535.0
}

fn noise(u: f32, v: f32, scale: i32) -> f32 {
	let x = (u * scale as f32).floor() as i32;
	let y = (v * scale as f32).floor() as i32;
	hash2(x, y)
}

// Césped (top): verde con ligeras variaciones de luminosidad y tono.
pub fn sample_grass_top(u: f32, v: f32) -> Color {
	let n = noise(u, v, 32);
	// Paleta base (verde vivo)
	let base = Color::new(0.20, 0.55, 0.15);
	let tint = Color::new(0.10 * n, 0.25 * n, 0.05 * n);
	(base + tint).clamped()
}

// Tierra (bottom): marrón con ruido.
pub fn sample_dirt(u: f32, v: f32) -> Color {
	let n = noise(u, v, 24);
	let base = Color::new(0.38, 0.25, 0.10);
	let tint = Color::new(0.10 * n, 0.07 * n, 0.04 * n);
	(base - Color::new(0.05 * (1.0 - n), 0.03 * (1.0 - n), 0.02 * (1.0 - n)) + tint).clamped()
}

// Lado: transición vertical césped -> tierra (blend según v).
pub fn sample_grass_side(u: f32, v: f32) -> Color {
	// v: 0 abajo (tierra), 1 arriba (césped). Suavizar con curva.
	let t = v.clamp(0.0, 1.0);
	let smooth = t * t * (3.0 - 2.0 * t); // smoothstep
	let grass = sample_grass_top(u, v * 1.2); // un poco más de ruido
	let dirt = sample_dirt(u * 1.3, v * 0.8);
	(dirt * (1.0 - smooth) + grass * smooth).clamped()
}

// Facade simple para escoger textura de un "bloque de césped" según cara.
// face_normal: componente dominante de la normal (se asume normal axis-aligned)
// normal ya normalizada; usamos signo para distinguir top/bottom.
pub fn sample_grass_block(normal: crate::color::Vec3, u: f32, v: f32) -> Color {
	// Determinar cara: eje de mayor valor absoluto.
	let ax = normal.x.abs();
	let ay = normal.y.abs();
	let az = normal.z.abs();
	if ay >= ax && ay >= az { // top o bottom
		if normal.y > 0.0 { sample_grass_top(u, v) } else { sample_dirt(u, v) }
	} else if ax >= ay && ax >= az { // lados X
		sample_grass_side(u, v)
	} else { // lados Z
		sample_grass_side(u, v)
	}
}

// Variante: si la cara superior no está expuesta (hay otro voxel encima) forzar dirt para replicar estratos.
pub fn sample_grass_block_surface(normal: crate::color::Vec3, u: f32, v: f32, is_top_exposed: bool) -> Color {
	// Si no está expuesto: todo tierra (como bloque enterrado en Minecraft)
	if !is_top_exposed { return sample_dirt(u, v); }
	// Caso expuesto: top césped, bottom tierra, lados transición
	let ax = normal.x.abs();
	let ay = normal.y.abs();
	let az = normal.z.abs();
	if ay >= ax && ay >= az { // top/bottom
		if normal.y > 0.0 { sample_grass_top(u, v) } else { sample_dirt(u, v) }
	} else if ax >= ay && ax >= az { sample_grass_side(u, v) } else { sample_grass_side(u, v) }
}

// Tronco (abeto): veta vertical. Usamos u como ángulo/x según cara, v para vetas.
pub fn sample_trunk(normal: crate::color::Vec3, u: f32, v: f32) -> Color {
	// Determinar si estamos en top/bottom del tronco para mostrar corte circular simplificado
	let ax = normal.x.abs(); let ay = normal.y.abs(); let az = normal.z.abs();
	if ay >= ax && ay >= az { // corte
		// Anillos: dist al centro en UV (u,v en [0,1])
		let du = (u - 0.5).abs();
		let dv = (v - 0.5).abs();
		let r = (du*du + dv*dv).sqrt();
		let rings = ((r * 16.0).sin()*0.5+0.5).powf(0.8);
		let base = Color::new(0.55, 0.40, 0.18);
		let dark = Color::new(0.32, 0.20, 0.08);
		return (base * rings + dark * (1.0 - rings)).clamped();
	}
	// Lado: veta vertical con bandas perlin-like simples
	let stripes = ((u * 32.0).sin()*0.5+0.5).powf(1.4);
	let noise_v = ((v * 8.0 + (u*4.0).sin()*0.5).sin()*0.5+0.5);
	let base = Color::new(0.45, 0.29, 0.12);
	let highlight = Color::new(0.62, 0.43, 0.18);
	(base * (0.6 + 0.4*stripes) + highlight * (0.25*noise_v)).clamped()
}

// Hojas abeto: verde más oscuro con moteado translúcido simple.
pub fn sample_leaves(u: f32, v: f32) -> Color {
	let n = noise(u*1.7 + 3.1, v*1.9 + 7.7, 40);
	let base = Color::new(0.06, 0.30, 0.08);
	let tint = Color::new(0.04 * n, 0.12 * n, 0.05 * n);
	(base + tint).clamped()
}

// (En el futuro se podría generalizar a un enum TextureKind con múltiples materiales.)
pub struct Texture; // placeholder para compatibilidad si se expande.

