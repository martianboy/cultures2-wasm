use web_sys::console;
use wasm_bindgen::JsValue;
// use image::dxt::{DXTEncoder, DXTVariant};

use std::cmp;
// use std::fmt;
use std::io::{BufWriter, Write};

struct BmdHeader {
  num_frames: usize,
  num_pixels: usize,
  num_rows: usize,
}

#[derive(Copy, Clone, Debug)]
struct BmdFrameInfo {
  frame_type: u32,
  dx: i32,
  dy: i32,
  width: usize,
  len: usize,
  off: usize,
}

#[derive(Clone, Debug)]
struct BmdRowInfo {
  raw: u32,
  indent: usize,
  offset: usize,
}

#[derive(Clone, Debug)]
pub struct BmdStats {
  pub width: usize,
  pub height: usize,
  pub frames: usize,
  pub encoded_length: usize,
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

  for (ch, f) in buf[12..section_length + 12].chunks(24).zip(frames.iter_mut()) {
    f.frame_type = read_uint32_le(&ch);
    f.dx = read_uint32_le(&ch[4..]) as i32;
    f.dy = read_uint32_le(&ch[8..]) as i32;
    f.width = read_uint32_le(&ch[12..]) as usize;
    f.len = read_uint32_le(&ch[16..]) as usize;
    f.off = read_uint32_le(&ch[20..]) as usize;
  }

  Ok(&buf[section_length + 12..])
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
    rows[i].raw = u;
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
  let mut bmd_stats_vec = vec![BmdStats { frames: 0, width: 0, height: 0, encoded_length: 0 }; count];

  for i in 0..count {
    let (rest, header) = read_bmd_header(remaining_slice);
    let mut frames = vec![BmdFrameInfo { frame_type: 0, dx: 0, dy: 0, width: 0, len: 0, off: 0 }; header.num_frames];
    let mut shadow_frames: Option<Vec<BmdFrameInfo>> = None;
    let rest = read_frames(rest, &mut frames[..]).expect("read_frames failed");
    let rest = skip_section(rest);
    let rest = skip_section(rest);
    remaining_slice = rest;

    if has_shadow[i] > 0 {
      let (rest, header) = read_bmd_header(remaining_slice);
      let mut fv = vec![BmdFrameInfo { frame_type: 0, dx: 0, dy: 0, width: 0, len: 0, off: 0 }; header.num_frames];
      let rest = read_frames(rest, &mut fv[..]).expect("read_frames failed");
      let rest = skip_section(rest);
      let rest = skip_section(rest);
      remaining_slice = rest;

      shadow_frames = Some(fv);
    }

    let mut stat = &mut bmd_stats_vec[i];
    stat.frames += header.num_frames;

    if let Some(s_frames) = shadow_frames {
      for (f, fs) in frames.iter().zip(s_frames.iter()) {
        let x0 = cmp::min(f.dx, fs.dx);
        let y0 = cmp::min(f.dy, fs.dy);
        let x1 = cmp::max(f.width as i32 + f.dx, fs.width as i32 + fs.dx);
        let y1 = cmp::max(f.width as i32 + f.dx, fs.width as i32 + fs.dx);

        stat.width = cmp::max(stat.width, (x1 - x0) as usize);
        stat.height = cmp::max(stat.height, (y1 - y0) as usize);
      }
    } else {
      for f in frames {
        if stat.width < f.width {
          stat.width = f.width;
        }
        if stat.height < f.len {
          stat.height = f.len;
        }
      }
    }

    // stat.width += stat.width % 4;
    // stat.height += stat.height % 4;

    stat.encoded_length = 4 * stat.width * stat.height; // calc_output_size(stat.width as u32, stat.height as u32);
  }

  return bmd_stats_vec;
}

#[inline]
fn write_uint32_le(buf: &mut [u8], val: u32) {
  buf[0] = (val & 0xFF) as u8;
  buf[1] = ((val & 0xFF00) >> 8) as u8;
  buf[2] = ((val & 0xFF0000) >> 16) as u8;
  buf[3] = ((val & 0xFF000000) >> 24) as u8;
}

macro_rules! bmd {
  ($e:expr) => {
    {
      let (rest, header) = read_bmd_header($e);
    
      let mut frames = vec![BmdFrameInfo { frame_type: 0, dx: 0, dy: 0, width: 0, len: 0, off: 0 }; header.num_frames];
      let rest = read_frames(rest, &mut frames[..]).expect("read_frames failed");
      let (rest, pixels) = read_pixels(rest).expect("read_pixels failed.");
      let mut rows = vec![BmdRowInfo { raw: 0, indent: 0, offset: 0 }; header.num_rows];
      let rest = read_rows(rest, &mut rows[..]).expect("read_rows failed.");

      (frames, (pixels, (rows, rest)))
    }
  };
}

pub fn read_bmd<'a>(w: usize, h: usize, instance_count: usize, has_shadow: bool, buf: &[u8], out: &mut [u8], frame_palette_index: &mut impl std::iter::Iterator<Item = (&'a usize, &'a usize)>, palettes: &Vec<&[u8]>, _debug: bool) -> usize {
  // if _debug { console::log_2(&"read_bmd: 1".into(), &JsValue::from(has_shadow)); }
  let (frames, (pixels, (rows, rest))) = bmd!(buf);

  let mut frame_offset_ptr = 0usize;
  let mut out_pointer: usize = instance_count * 8;

  let encoded_frame_length = w * h * 4;

  if has_shadow {
    let (s_frames, (s_pixels, (s_rows, _))) = bmd!(rest);

    for (i, (&fi, &pi)) in frame_palette_index.enumerate() {  // .map(|(&fi, &pi)| { (((&s_frames[fi], &frames[fi]), palettes[pi])) })
      // if _debug { console::log_1(&format!("read_bmd #{}: begin - {} - fi: {} - pi: {}", &frames.len(), i, fi, pi).into()); }

      if fi < frames.len() {

        let f = &frames[fi];
        let p = &palettes[pi];

        if fi >= s_frames.len() {
          write_uint32_le(&mut out[frame_offset_ptr..], f.dx as u32);
          write_uint32_le(&mut out[frame_offset_ptr + 4..], f.dy as u32);

          read_bmd_frame(
            w,
            cmp::max(0, f.dx) as usize,
            cmp::max(0, f.dy) as usize,
            f,
            &rows[f.off..f.off + f.len],
            &pixels[rows[f.off].offset..],
            &mut out[out_pointer..],
            p,
            _debug
          );
        } else {
          let fs = &s_frames[fi];

          write_uint32_le(&mut out[frame_offset_ptr..], cmp::min(f.dx, fs.dx) as u32);
          write_uint32_le(&mut out[frame_offset_ptr + 4..], cmp::min(f.dy, fs.dy) as u32);
    
          // if _debug { console::log_1(&format!("read_bmd #{}", i).into()); }
          read_bmd_frame(
            w,
            cmp::max(0, fs.dx - f.dx) as usize,
            cmp::max(0, fs.dy - f.dy) as usize,
            fs,
            &s_rows[fs.off..fs.off + fs.len],
            &s_pixels[s_rows[fs.off].offset..],
            &mut out[out_pointer..],
            p,
            _debug
          );
          read_bmd_frame(
            w,
            cmp::max(0, f.dx - fs.dx) as usize,
            cmp::max(0, f.dy - fs.dy) as usize,
            f,
            &rows[f.off..f.off + f.len],
            &pixels[rows[f.off].offset..],
            &mut out[out_pointer..],
            p,
            _debug
          );
        }
      }

      // if _debug { console::log_1(&format!("read_bmd #{}: done", i).into()); }

      frame_offset_ptr += 8;
      out_pointer += encoded_frame_length;
    }
  } else {
    for (i, (&fi, &pi)) in frame_palette_index.enumerate() {  // .map(|(&fi, &pi)| { (((&s_frames[fi], &frames[fi]), palettes[pi])) })
      if _debug { console::log_1(&format!("read_bmd #{}: begin - {} - fi: {} - pi: {}", &frames.len(), i, fi, pi).into()); }

      if fi < frames.len() {
        let f = &frames[fi];
        let p = &palettes[pi];

        if _debug { console::log_1(&format!("read_bmd (no shadow) #{}: dx: {}, dy: {}", i, f.dx, f.dy).into()); }

        write_uint32_le(&mut out[frame_offset_ptr..], f.dx as u32);
        write_uint32_le(&mut out[frame_offset_ptr + 4..], f.dy as u32);

        read_bmd_frame(
          w,
          cmp::max(0, f.dx) as usize,
          cmp::max(0, f.dy) as usize,
          f,
          &rows[f.off..f.off + f.len],
          &pixels[rows[f.off].offset..],
          &mut out[out_pointer..],
          p,
          _debug
        );
      }

      frame_offset_ptr += 8;
      out_pointer += encoded_frame_length;
    }
  }

  return out_pointer;
}

fn read_bmd_frame(w: usize, p_w: usize, p_h: usize, fi: &BmdFrameInfo, rows: &[BmdRowInfo], pixels: &[u8], out: &mut [u8], palette: &[u8], _debug: bool) {
  let mut out_pos;
  let mut pixels_ptr = 0;

  // println!("#### {}", rows.len());

  for (i, r) in rows.iter().enumerate() {
    // if _debug { console::log_2(&"read_bmd_frame: row:".into(), &JsValue::from(i as u32)); }
    // if _debug { console::log_1(&format!("r.indent = {}, r.offset = {}", r.indent, r.offset).into()); }

    // println!("{:?}", r);

    if pixels_ptr >= pixels.len() { return; }
    if r.raw as i32 == -1 { continue; }

    out_pos = 4 * ((i + p_h) * w + r.indent + p_w);
    // if _debug { console::log_1(&format!("{} = 4 * (({} + {}) * {} + {} + {})", out_pos, i, p_h, w, r.indent, p_w).into()); }

    let mut pixel_block_length: usize = pixels[pixels_ptr] as usize; pixels_ptr += 1;

    while pixel_block_length != 0 {
      if pixel_block_length < 0x80 {
        // if _debug { console::log_1(&format!("out_pos = {}, out.len() = {}", out_pos, out.len()).into()); }
        // if _debug { console::log_1(&format!("writing {} pixels", pixel_block_length).into()); }

        for _ in 0..pixel_block_length {
          // if _debug { console::log_1(&format!("pixels #{}", j).into()); }

          if fi.frame_type == 2 {     // Shadow frame
            out[out_pos + 0] = 0;
            out[out_pos + 1] = 0;
            out[out_pos + 2] = 0;
            out[out_pos + 3] = 0x50;
          } else if fi.frame_type == 1 {    // Normal frame
            let color_index = pixels[pixels_ptr] as usize; pixels_ptr += 1;
            out[out_pos + 0] = palette[3 * color_index + 0];
            out[out_pos + 1] = palette[3 * color_index + 1];
            out[out_pos + 2] = palette[3 * color_index + 2];
            out[out_pos + 3] = 0xFF;
          } else if fi.frame_type == 4 {    // Extended frame
            let color_index = pixels[pixels_ptr] as usize; pixels_ptr += 1;
            let pixel_level = pixels[pixels_ptr]; pixels_ptr += 1;

            out[out_pos + 0] = palette[3 * color_index + 0];
            out[out_pos + 1] = palette[3 * color_index + 1];
            out[out_pos + 2] = palette[3 * color_index + 2];
            out[out_pos + 3] = pixel_level; // if pixel_level == 255 { 0xFF } else { 0x00 };
          } else {
            // console::log_2(&"read_bmd: frame type unknown:".into(), &JsValue::from(fi.frame_type as u32));
          }
          out_pos += 4;
        }
      } else {
        out_pos += 4 * 1 * (pixel_block_length - 0x80);
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
  use crate::pcx::{pcx_read_palette_array, read_palette};

  use image;
  use image::png::{PngDecoder, PNGReader};
  use image::ImageDecoder;

  // Note this useful idiom: importing names from outer (for mod tests) scope.
  use super::*;

  // #[test]
  // fn test_dxt1() {
  //   let file = File::open("tests/cat.png").expect("File not found!");
  //   let mut buf_reader = BufReader::new(file);
  //   let img = PngDecoder::new(&mut buf_reader).expect("PngDecoder failed!");
  //   let (w, h) = (352, 352);

  //   let mut buf = vec![0u8; 352 * 352 * 4];
  //   img.read_image(&mut buf).expect("read_image failed.");
  //   // buf_reader.read_to_end(&mut img).expect("read_to_end failed.");

  //   println!("{}x{} -> {} bytes", w, h, calc_output_size(w, h));

  //   let mut enc_buf = vec![0u8; calc_output_size(w, h)];
  //   let mut writer = BufWriter::new(&mut enc_buf);
  //   let encoder = DXTEncoder::new(&mut writer);
  //   encoder.encode(&buf, 352, 352, DXTVariant::DXT1).expect("DXT1 encoder failed.");
  // }

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
  fn test_empty_rows() {
    let file = File::open("../cultures-fun/data/engine2d/bin/bobs/ls_goods_s.bmd").expect("File not found!");

    let mut buf_reader = BufReader::new(file);
    let mut buffer = Vec::new();
    buf_reader.read_to_end(&mut buffer).expect("read_to_end failed.");

    let (frames, (pixels, (rows, rest))) = bmd!(&buffer);
    println!("Empty rows: {}", rows.iter().filter(|r| r.raw as i32 == -1).count());
  }

  #[test]
  fn test_zero_frames() {
    let file = File::open("../cultures-fun/data/engine2d/bin/bobs/ls_ground.bmd").expect("File not found!");

    let mut buf_reader = BufReader::new(file);
    let mut buffer = Vec::new();
    buf_reader.read_to_end(&mut buffer).expect("read_to_end failed.");

    let (rest, header) = read_bmd_header(&buffer);
    let mut frames = vec![BmdFrameInfo { frame_type: 0, dx: 0, dy: 0, width: 0, len: 0, off: 0 }; header.num_frames];
    read_frames(rest, &mut frames[..]).expect("read_frames failed");

    println!("Zero frames: {}", frames.iter().filter(|f| f.frame_type == 0).count());
  }

  #[test]
  fn test_extract_bmd_frame() {
    let file = File::open("../cultures-fun/data/engine2d/bin/bobs/ls_ground.bmd").expect("File not found!");

    let mut buf_reader = BufReader::new(file);
    let mut buffer = Vec::new();
    buf_reader.read_to_end(&mut buffer).expect("read_to_end failed.");

    let (frames, (pixels, (rows, _))) = bmd!(&buffer[..]);

    let file = File::open("../cultures-fun/data/engine2d/bin/palettes/landscapes/rock03.pcx").expect("Palette file not found!");
    let mut buf_reader = BufReader::new(file);
    let mut buffer = Vec::new();
    buf_reader.read_to_end(&mut buffer).expect("read_to_end failed.");

    let pal = read_palette(&buffer[buffer.len() - 769..]).expect("read palette failed.");

    const w: usize = 200;
    const h: usize = 200;

    for &i in [0, 17].iter() {
      let mut img = [0u8; w * h * 4];
      let fi = &frames[i];
      println!("Frame type: {}", fi.frame_type);
      read_bmd_frame(w, 0, 0, fi, &rows[fi.off..fi.off + fi.len], &pixels[rows[fi.off].offset..], &mut img[..], &pal, false);

      let file = File::create(format!("tests/dump_{}.frm", i)).expect("File could not be created!");
      let mut writer = std::io::BufWriter::new(file);
      let mut pxl = &pixels[rows[fi.off].offset..rows[fi.off + fi.len].offset];
      writer.write_all(&mut pxl).expect("Write dump failed.");

      image::save_buffer(format!("tests/ls_trees_{}.png", i), &img[..], w as u32, h as u32, image::ColorType::Rgba8).unwrap();
    }
  }

  // #[test]
  // fn test_read_bmd_frame() {
  //   let file = File::open("tests/ls_gates.bmd").expect("File not found!");
  //   let palette_file = File::open("tests/tree01.pcx").expect("Palette file not found!");

  //   let mut buf_reader = BufReader::new(file);
  //   let mut buffer = Vec::new();
  //   buf_reader.read_to_end(&mut buffer).expect("read_to_end failed.");

  //   let stats = bmd_stats(&buffer[..], &[0u8; 1][..], 1);
  //   println!("{}:{}:{}", stats[0].frames, stats[0].width, stats[0].height);

  //   let (rest, header) = read_bmd_header(&buffer[..]);
  //   let mut frames = vec![BmdFrameInfo { frame_type: 0, width: 0, len: 0, off: 0 }; header.num_frames];
  //   let rest = read_frames(rest, &mut frames[..]).expect("read_frames failed");
  //   let (rest, pixels) = read_pixels(rest).expect("read_pixels failed.");
  //   let mut rows = vec![BmdRowInfo { indent: 0, offset: 0 }; header.num_rows];
  //   read_rows(rest, &mut rows[..]).expect("read_rows failed.");

  //   let mut palette_reader = BufReader::new(palette_file);
  //   let mut palette_buf = Vec::new();
  //   palette_reader.read_to_end(&mut palette_buf).expect("read_to_end failed.");

  //   let palette_array = pcx_read_palette_array(&palette_buf, &[0usize]);

  //   let mut img = vec![0u8; stats[0].width * stats[0].height * 3];
  //   let fi = &frames[0]; // frames.iter().find(|&x| x.width == stats[0].width).expect("Hey!");
  //   println!("### {}", pixels.len());
  //   read_bmd_frame(stats[0].width, (stats[0].width - fi.width) / 2, stats[0].height - fi.len, fi, &rows[fi.off..fi.off + fi.len], &pixels[rows[fi.off].offset..], &mut img[..], &palette_array[0], true);

  //   let mut encoded_buf = vec![0u8; calc_output_size(stats[0].width as u32, stats[0].height as u32)];
  //   let encoder = DXTEncoder::new(BufWriter::new(&mut encoded_buf));
  //   encoder.encode(&img[..], stats[0].width as u32, stats[0].height as u32, DXTVariant::DXT1).expect("Hey!");

  //   image::save_buffer("tests/ls_trees.png", &img[..], stats[0].width as u32, stats[0].height as u32, image::ColorType::Rgba8).unwrap();
  // }
}