mod utils;
mod tessellate;

use wasm_bindgen::prelude::*;

// When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
// allocator.
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

#[wasm_bindgen]
pub fn triangulate(w: usize, h: usize, elevation: &[u8]) -> Box<[f32]> {
  let mut tris = vec![0.0; w * h * 2 * 2 * 3];
  tessellate::triangulate_map(&mut tris, w, h, elevation);

  return tris.into_boxed_slice();
}
