use web_sys::console;

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


#[inline]
fn write_uint32_le(buf: &mut [u8], val: u32) {
  buf[0] = (val & 0xFF) as u8;
  buf[1] = ((val & 0xFF00) >> 8) as u8;
  buf[2] = ((val & 0xFF0000) >> 16) as u8;
  buf[3] = ((val & 0xFF000000) >> 24) as u8;
}

#[wasm_bindgen]
pub fn create_bmd_texture_array(bmd_buf: &[u8], palette_buf: &[u8], bmd_index: &[usize], bmd_frame_instance_count: &[usize], has_shadow: &[u8], palette_index: &[usize], frame_palette_index: &[usize]) -> Box<[u8]> {
  let _timer = timer::Timer::new("create_bmd_texture_array");

  let palettes = pcx::pcx_read_palette_array(palette_buf, palette_index);
  let bmd_stats = bmd::bmd_stats(bmd_buf, has_shadow, bmd_index.len());
  let total_buf_length = 4 * 4 + bmd_stats.iter().zip(bmd_frame_instance_count).fold(0, |r, (s, c)| r + c * (2 * 4 + s.encoded_length));

  let mut images = vec![0u8; total_buf_length];
  let mut out_ptr = 0usize;
  let mut frame_ptr = 0;

  for i in 0..bmd_index.len() {
    let s = &bmd_stats[i];

    // Write header
    write_uint32_le(&mut images[out_ptr..], bmd_frame_instance_count[i] as u32); out_ptr += 4;
    write_uint32_le(&mut images[out_ptr..], s.width as u32); out_ptr += 4;
    write_uint32_le(&mut images[out_ptr..], s.height as u32); out_ptr += 4;
    write_uint32_le(&mut images[out_ptr..], (bmd_frame_instance_count[i] * s.encoded_length) as u32); out_ptr += 4;

    // Write texture 2d image
    let frame_instance_count = frame_palette_index[i];

    let mut it = frame_palette_index[bmd_index.len() + frame_ptr..bmd_index.len() + frame_ptr + frame_instance_count * 2].chunks(2).map(|c| (&c[0], &c[1]));
    frame_ptr += frame_instance_count * 2;

    bmd::read_bmd(s.width, s.height, bmd_frame_instance_count[i], has_shadow[i] > 0, &bmd_buf[bmd_index[i]..], &mut images[out_ptr..], &mut it, &palettes, false);
    out_ptr += bmd_frame_instance_count[i] * s.encoded_length;
  }

  return images.into_boxed_slice();
}
