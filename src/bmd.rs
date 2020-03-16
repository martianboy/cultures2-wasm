use web_sys::console;
use wasm_bindgen::JsValue;

use std::cmp;
use std::fmt;

struct BmdHeader {
  num_frames: usize,
  num_pixels: usize,
  num_rows: usize,
}

#[derive(Copy, Clone)]
struct BmdFrameInfo {
  frame_type: u32,
  width: usize,
  len: usize,
  off: usize,
}

#[derive(Clone)]
struct BmdRowInfo {
  indent: usize,
  offset: usize,
}

#[derive(Clone)]
pub struct BmdStats {
  pub width: usize,
  pub height: usize,
  pub frames: usize,
}


#[inline]
fn read_uint32_le(buf: &[u8]) -> u32 {
  ((buf[3] as u32) << 24) + ((buf[2] as u32) << 16) + ((buf[1] as u32) << 8) + buf[0] as u32
}

fn read_bmd_header(buf: &[u8]) -> (&[u8], BmdHeader) {
  let header = BmdHeader {
    num_frames: read_uint32_le(&buf[12..16]) as usize,
    num_pixels: read_uint32_le(&buf[16..20]) as usize,
    num_rows: read_uint32_le(&buf[20..24]) as usize,
  };

  (&buf[0x24..], header)
}

fn read_frames<'a>(buf: &'a[u8], frames: &mut [BmdFrameInfo]) -> Result<&'a[u8], &'static str> {
  if buf[0] != 0xE9 || buf[1] != 0x03 {
    return Err("read_frames: starting point is incorrect.");
  }

  let section_length = read_uint32_le(&buf[0x08..]) as usize;
  let mut rest: &[u8] = &buf[12..];

  for i in 0..section_length / 24 {
    frames[i].frame_type = read_uint32_le(&rest);
    frames[i].width = read_uint32_le(&rest[12..]) as usize;
    frames[i].len = read_uint32_le(&rest[16..]) as usize;
    frames[i].off = read_uint32_le(&rest[20..]) as usize;

    rest = &rest[24..];
  }

  Ok(&rest)
}

fn skip_section(buf: &[u8]) -> &[u8] {
  let section_length = read_uint32_le(&buf[0x08..]) as usize;
  &buf[12 + section_length..]
}

fn read_rows<'a>(buf: &'a[u8], rows: &mut [BmdRowInfo]) -> Result<&'a[u8], &'static str> {
  if buf[0] != 0xE9 || buf[1] != 0x03 {
    return Err("read_frames: starting point is incorrect.");
  }

  let section_length = read_uint32_le(&buf[0x08..]) as usize;
  let mut rest: &[u8] = &buf[12..];

  for i in 0..(section_length / 4) {
    let u = read_uint32_le(&rest);
    rows[i].indent = (u >> 22) as usize;
    rows[i].offset = (u & ((1 << 22) - 1)) as usize;
    rest = &rest[4..];
  }

  Ok(&rest)
}

fn read_pixels<'a>(buf: &'a[u8]) -> Result<(&'a[u8], &'a[u8]), &'static str> {
  if buf[0] != 0xE9 || buf[1] != 0x03 {
    return Err("read_pixels: starting point is incorrect.");
  }

  let section_length = read_uint32_le(&buf[0x08..]) as usize;

  Ok((&buf[section_length + 12..], &buf[12..12 + section_length]))
}

pub fn bmd_stats(buf: &[u8], has_shadow: &[u8], count: usize) -> Vec<BmdStats> {
  let mut remaining_slice = buf;
  let mut bmd_stats_vec = vec![BmdStats { frames: 0, width: 0, height: 0 }; count];

  for i in 0..count {
    let (rest, header) = read_bmd_header(remaining_slice);
    let mut frames = vec![BmdFrameInfo { frame_type: 0, width: 0, len: 0, off: 0 }; header.num_frames];
    let mut shadow_frames: Option<Vec<BmdFrameInfo>> = None;
    let rest = read_frames(rest, &mut frames[..]).expect("read_frames failed");
    let rest = skip_section(rest);
    let rest = skip_section(rest);
    remaining_slice = rest;

    if has_shadow[i] > 0 {
      let (rest, header) = read_bmd_header(remaining_slice);
      let mut fv = vec![BmdFrameInfo { frame_type: 0, width: 0, len: 0, off: 0 }; header.num_frames];
      let rest = read_frames(rest, &mut fv[..]).expect("read_frames failed");
      let rest = skip_section(rest);
      let rest = skip_section(rest);
      remaining_slice = rest;

      shadow_frames = Some(fv);
    }

    let mut stat = &mut bmd_stats_vec[i];
    stat.frames += header.num_frames;

    for f in frames {
      if stat.width < f.width {
        stat.width = f.width;
      }
      if stat.height < f.len {
        stat.height = f.len;
      }
    }

    if let Some(s_frames) = shadow_frames {
      for f in s_frames {
        if stat.width < f.width {
          stat.width = f.width;
        }
        if stat.height < f.len {
          stat.height = f.len;
        }
      }
    }
  }

  return bmd_stats_vec;
}

pub fn read_bmd(w: usize, h: usize, has_shadow: bool, buf: &[u8], out: &mut [u8], frame_palette_index: &[usize], palettes: &Vec<&[u8]>, _debug: bool) {
  // if _debug { console::log_2(&"read_bmd: 1".into(), &JsValue::from(has_shadow)); }

  let (rest, header) = read_bmd_header(buf);
  let mut frames = vec![BmdFrameInfo { frame_type: 0, width: 0, len: 0, off: 0 }; header.num_frames];
  let rest = read_frames(rest, &mut frames[..]).expect("read_frames failed");
  let (rest, pixels) = read_pixels(rest).expect("read_pixels failed.");
  let mut rows = vec![BmdRowInfo { indent: 0, offset: 0 }; header.num_rows];
  let rest = read_rows(rest, &mut rows[..]).expect("read_rows failed.");

  let mut row_pointer: usize = 0;

  if has_shadow {
    let (rest, s_header) = read_bmd_header(rest);
    let mut s_frames = vec![BmdFrameInfo { frame_type: 0, width: 0, len: 0, off: 0 }; s_header.num_frames];
    let rest = read_frames(rest, &mut s_frames[..]).expect("read_frames failed");
    let (rest, s_pixels) = read_pixels(rest).expect("read_pixels failed.");
    let mut s_rows = vec![BmdRowInfo { indent: 0, offset: 0 }; s_header.num_rows];
    read_rows(rest, &mut s_rows[..]).expect("read_rows failed.");

    // if _debug { console::log_1(&"read_bmd: 3".into()); }

    for (i, f) in s_frames.iter().enumerate() {
      // if _debug { console::log_1(&format!("f.width = {}, f.len = {}", frames[i].width, frames[i].len).into()); }
      // if _debug { console::log_1(&format!("frames[{}].width = {}, frames[{}].len = {}", i, f.width, i, f.len).into()); }
      // if _debug { console::log_1(&format!("{} - {} = {}", frames[i].width, f.width, frames[i].width - f.width).into()); }

      let dw = frames[i].width - cmp::min(frames[i].width, f.width);
      let dh = frames[i].len - cmp::min(frames[i].len, f.len);

      // if _debug { console::log_1(&format!("dw = {}, dh = {}", dw, dh).into()); }

      let padding_w = dw + (w - frames[i].width) / 2;
      let padding_h = dh + h - frames[i].len;

      // if _debug { console::log_3(&"read_bmd: frame:".into(), &JsValue::from(i as u32), &JsValue::from(f.len as u32)); }
      read_bmd_frame(w, padding_w, padding_h, f, &s_rows[row_pointer..row_pointer + f.len], &s_pixels[s_rows[f.off].offset..], &mut out[i * w * h * 4..], palettes[frame_palette_index[i]], _debug);
      
      row_pointer += f.len;
    }

    // if _debug { console::log_1(&"read_bmd: 4".into()); }
  }

  row_pointer = 0;
  for (i, f) in frames.iter().enumerate() {
    let padding_w = (w - f.width) / 2;
    let padding_h = h - f.len;

    read_bmd_frame(w, padding_w, padding_h, f, &rows[f.off..f.off + f.len], &pixels[rows[f.off].offset..], &mut out[i * w * h * 4..], palettes[frame_palette_index[i]], _debug);
    row_pointer += f.len;
  }
}

fn read_bmd_frame(w: usize, p_w: usize, p_h: usize, fi: &BmdFrameInfo, rows: &[BmdRowInfo], pixels: &[u8], out: &mut [u8], palette: &[u8], _debug: bool) {
  let mut out_pos;
  let mut pixels_ptr = 0;

  println!("#### {}", rows.len());

  for (i, r) in rows.iter().enumerate() {
    // if _debug { console::log_2(&"read_bmd_frame: row:".into(), &JsValue::from(i as u32)); }
    // if _debug { console::log_1(&format!("r.indent = {}, r.offset = {}", r.indent, r.offset).into()); }

    if pixels_ptr >= pixels.len() { return };

    out_pos = 4 * ((i + p_h) * w + r.indent + p_w);
    // if _debug { console::log_1(&format!("{} = 4 * (({} + {}) * {} + {} + {})", out_pos, i, p_h, w, r.indent, p_w).into()); }

    let mut pixel_block_length: usize = pixels[pixels_ptr] as usize; pixels_ptr += 1;

    while pixel_block_length != 0 {
      if pixel_block_length < 0x80 {
        // if _debug { console::log_1(&format!("out_pos = {}, out.len() = {}", out_pos, out.len()).into()); }
        // if _debug { console::log_1(&format!("writing {} pixels", pixel_block_length).into()); }

        for j in 0..pixel_block_length {
          // if _debug { console::log_1(&format!("pixels #{}", j).into()); }

          if fi.frame_type == 2 {     // Shadow frame
            out[out_pos + 0] = 0;
            out[out_pos + 1] = 0;
            out[out_pos + 2] = 0;
            out[out_pos + 3] = 0x80;
          } else if fi.frame_type == 1 {    // Normal frame
            let color_index = pixels[pixels_ptr] as usize; pixels_ptr += 1;
            out[out_pos + 0] = palette[3 * color_index + 0];
            out[out_pos + 1] = palette[3 * color_index + 1];
            out[out_pos + 2] = palette[3 * color_index + 2];
            out[out_pos + 3] = 0xFF;
          } else {
            // console::log_2(&"read_bmd: frame type unknown:".into(), &JsValue::from(fi.frame_type as u32));
          }
          out_pos += 4;
        }
      } else {
        out_pos += 4 * (pixel_block_length - 0x80);
      }

      pixel_block_length = pixels[pixels_ptr] as usize; pixels_ptr += 1;
    }
  }
}

#[cfg(test)]
mod tests {
  use std::fs::File;
  use std::io::BufReader;
  use std::io::Read;
  use crate::pcx::pcx_read_palette_array;

  use image;
  
  // Note this useful idiom: importing names from outer (for mod tests) scope.
  use super::*;

  #[test]
  fn test_bmd_stats() {
    let file = File::open("tests/ls_trees.bmd").expect("File not found!");

    let mut buf_reader = BufReader::new(file);
    let mut buffer = Vec::new();
    buf_reader.read_to_end(&mut buffer).expect("read_to_end failed.");

    let stats = bmd_stats(&buffer[..], &[0u8; 1][..], 1);

    println!("{} : {} : {}", stats[0].frames, stats[0].width, stats[0].height);
  }

  #[test]
  fn test_read_bmd_frame() {
    let file = File::open("tests/ls_gates.bmd").expect("File not found!");
    let palette_file = File::open("tests/tree01.pcx").expect("Palette file not found!");

    let mut buf_reader = BufReader::new(file);
    let mut buffer = Vec::new();
    buf_reader.read_to_end(&mut buffer).expect("read_to_end failed.");

    let stats = bmd_stats(&buffer[..], &[0u8; 1][..], 1);
    println!("{}:{}:{}", stats[0].frames, stats[0].width, stats[0].height);

    let (rest, header) = read_bmd_header(&buffer[..]);
    let mut frames = vec![BmdFrameInfo { frame_type: 0, width: 0, len: 0, off: 0 }; header.num_frames];
    let rest = read_frames(rest, &mut frames[..]).expect("read_frames failed");
    let (rest, pixels) = read_pixels(rest).expect("read_pixels failed.");
    let mut rows = vec![BmdRowInfo { indent: 0, offset: 0 }; header.num_rows];
    read_rows(rest, &mut rows[..]).expect("read_rows failed.");

    let mut palette_reader = BufReader::new(palette_file);
    let mut palette_buf = Vec::new();
    palette_reader.read_to_end(&mut palette_buf).expect("read_to_end failed.");

    let palette_array = pcx_read_palette_array(&palette_buf, &[0usize]);

    let mut img = vec![0u8; stats[0].width * stats[0].height * 4];
    let fi = &frames[0]; // frames.iter().find(|&x| x.width == stats[0].width).expect("Hey!");
    println!("### {}", pixels.len());
    read_bmd_frame(stats[0].width, (stats[0].width - fi.width) / 2, stats[0].height - fi.len, fi, &rows[fi.off..fi.off + fi.len], &pixels[rows[fi.off].offset..], &mut img[..], &palette_array[0], true);

    image::save_buffer("tests/ls_trees.png", &img[..], stats[0].width as u32, stats[0].height as u32, image::ColorType::Rgba8).unwrap();
  }
}