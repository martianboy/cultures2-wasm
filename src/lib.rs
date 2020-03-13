mod utils;
mod tessellate;
mod pcx;

use wasm_bindgen::prelude::*;

#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

#[wasm_bindgen]
pub fn triangulate(w: usize, h: usize, elevation: &[u8]) -> Box<[f32]> {
  let mut tris = vec![0.0; w * h * 2 * 2 * 3];
  tessellate::triangulate_map(&mut tris, w, h, elevation);

  return tris.into_boxed_slice();
}

// #[wasm_bindgen]
// pub fn create_2d_texture(w: usize, h: usize, count: usize, index: &[usize], buf: &[u8], mask_index: Option<Box<[usize]>>, mask_buf: Option<Box<[u8]>>) {
  
// }