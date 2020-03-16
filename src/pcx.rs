#[inline]
fn read_uint16_le(buf: &[u8]) -> u16 {
  ((buf[1] as u16) << 8) + buf[0] as u16
}

fn read_pixels<'a, 'b>(buf: &'a [u8], pixels: &'b mut Vec<u8>) -> (&'a[u8], &'b[u8]) {
  let mut i = 0;
  let mut pos = 0;

  while i < pixels.len() {
    let mut val = buf[pos]; pos += 1;
    let mut len = 1;

    if val > 192 {
      len = val - 192;
      val = buf[pos]; pos += 1;
    }

    while len > 0 {
      pixels[i] = val;
      i += 1;
      len -= 1;
    }
  }

  (&buf[pos..], &pixels[..])
}

fn read_palette(buf: &[u8]) -> Result<&[u8], &'static str> {
  if buf[0] != 0x0C {
    return Err("PCX extended palette marker 0x0C not found.");
  }

  Ok(&buf[1..769])
}

fn get_dimensions(buf: &[u8]) -> (usize, usize) {
  let x0 = read_uint16_le(&buf[4..6]) as usize;
  let y0 = read_uint16_le(&buf[6..8]) as usize;
  let x1 = read_uint16_le(&buf[8..10]) as usize;
  let y1 = read_uint16_le(&buf[10..12]) as usize;

  return (x1 - x0 + 1, y1 - y0 + 1);
}

pub fn pcx_read<'a>(buf: &'a[u8], out: &mut [u8], mask: Option<&[u8]>) -> &'a[u8] {
  let (width, height) = get_dimensions(&buf);
  let buf_length = width * height;

  let alpha = match mask {
    None => vec![0xFFu8; buf_length],
    Some(mask_buf) => {
      let mut mask_out_buf = vec![0xFFu8; buf_length];
      read_pixels(&mask_buf[0x80..], &mut mask_out_buf);

      mask_out_buf
    }
  };

  let mut pixels = vec![0; buf_length];
  let (rest, _) = read_pixels(&buf[0x80..], &mut pixels);
  let palette = read_palette(&rest).expect("read_palette failed.");

  for i in 0..pixels.len() {
    out[4 * i + 0] = palette[0 + 3 * pixels[i] as usize];
    out[4 * i + 1] = palette[1 + 3 * pixels[i] as usize];
    out[4 * i + 2] = palette[2 + 3 * pixels[i] as usize];
    out[4 * i + 3] = alpha[i];
  }

  return rest;
}

pub fn pcx_texture_array(buf: &[u8], out: &mut [u8], index_table: &[usize], mask_index_table: Option<&[usize]>) {
  let (width, height) = get_dimensions(&buf);
  let len = width * height * 4;
  for (i, idx) in index_table.iter().enumerate() {
    pcx_read(&buf[*idx..], &mut out[(i * len)..], mask_index_table.and_then(|mit| Some(&buf[mit[i]..])));
  }
}

pub fn pcx_read_palette_array<'a>(buf: &'a[u8], index: &[usize]) -> Vec<&'a[u8]> {
  let mut out: Vec<&'a[u8]> = vec![buf; index.len()];

  for (i, pos) in index.iter().enumerate() {
    let length = if i < index.len() - 1 {
      index[i + 1] - index[i]
    } else {
      buf.len() - index[i]
    };

    out[i] = read_palette(&buf[*pos + length - 769..]).expect("read_palette failed");
  }

  return out;
}

// pub fn pcx_read_palette(buf: &[u8], ) {
//   let mut palette: [RGBColor; 256] = [RGBColor::default(); 256];
//   read_palette(rest, &mut palette).expect("read_palette failed.");

// }

#[cfg(test)]
mod tests {
  use std::fs::File;
  use std::io::BufReader;
  use std::io::Read;
  
  // Note this useful idiom: importing names from outer (for mod tests) scope.
  use super::*;

  #[test]
  fn test_read_pcx_header() {
    let file = File::open("tests/tran_desertbrown.pcx").expect("File not found!");

    let mut buf_reader = BufReader::new(file);
    let mut buffer = Vec::new();

    buf_reader.read_to_end(&mut buffer).expect("read_to_end failed.");
    let (width, height) = get_dimensions(&buffer);

    assert_eq!(width, 256);
    assert_eq!(height, 256);
  }

  #[test]
  fn test_pcx_read() {
    let file = File::open("tests/fire.pcx").expect("File not found!");
    let mut buf_reader = BufReader::new(file);
    let mut buffer = Vec::new();

    buf_reader.read_to_end(&mut buffer).expect("read_to_end failed.");

    let mut out = [0u8; 256 * 256 * 4];
    pcx_read(&buffer, &mut out, None);
  }

  #[test]
  fn test_pcx_read_palette_array() {
    let file = File::open("tests/fire.pcx").expect("File not found!");
    let mut buf_reader = BufReader::new(file);
    let mut buffer = Vec::new();

    buf_reader.read_to_end(&mut buffer).expect("read_to_end failed.");

    pcx_read_palette_array(&buffer[..], &[0usize; 1]);
  }
}
