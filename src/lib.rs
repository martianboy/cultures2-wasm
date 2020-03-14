mod utils;
mod tessellate;
mod pcx;
mod bmd;
mod timer;

use wasm_bindgen::prelude::*;

// #[cfg(feature = "wee_alloc")]
// #[global_allocator]
// static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

#[wasm_bindgen]
pub fn triangulate(w: usize, h: usize, elevation: &[u8]) -> Box<[f32]> {
  let _timer = timer::Timer::new("triangulate");

  let mut tris = vec![0.0; w * h * 2 * 2 * 3];
  tessellate::triangulate_map(&mut tris, w, h, elevation);

  return tris.into_boxed_slice();
}

#[wasm_bindgen]
pub fn create_2d_texture_masked(w: usize, h: usize, buf: &[u8], index: &[usize], mask_index: &[usize]) -> Box<[u8]> {
  let _timer = timer::Timer::new("create_2d_texture_masked");

  let mut out = vec![0u8; w * h * index.len() * 4];
  pcx::pcx_texture_array(&buf, &mut out[..], &index, Some(&mask_index));

  return out.into_boxed_slice();
}

#[wasm_bindgen]
pub fn create_2d_texture(w: usize, h: usize, buf: &[u8], index: &[usize]) -> Box<[u8]> {
  let _timer = timer::Timer::new("create_2d_texture");

  let mut out = vec![0u8; w * h * index.len() * 4];
  pcx::pcx_texture_array(&buf, &mut out[..], &index, None);

  return out.into_boxed_slice();
}
