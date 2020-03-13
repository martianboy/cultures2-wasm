// pub fn run_length<'a, E: ParseError<&'a [u8]>>(buf: &'a[u8]) -> IResult<&'a[u8], &'a[u8], E> {
//   if buf.len() < 1 {
//     Err(Err::Error(make_error(buf, ErrorKind::Eof)))
//   } else if buf[0] > 192 {
//     Ok((&buf[1..], &vec![buf[1]; buf[0] as usize][..]))
//   } else {
//     Ok((&buf[1..], &[buf[0]]))
//   }
// }

// fn read_pixels<'a, E: ParseError<&'a [u8]>>(buf: &[u8], width: usize, height: usize) -> IResult<&'a[u8], &'a[u8], E> {
//   fold_many1!(call!(run_length), &[], |mut acc: &[u8], v| {
//     acc += v;
//     acc
//   })
// }

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

#[derive(Copy, Clone, Debug)]
pub struct RGBColor {
  pub red: u8,
  pub green: u8,
  pub blue: u8,
}

impl Default for RGBColor {
  #[inline]
  fn default() -> RGBColor {
    RGBColor {
      red: 0,
      green: 0,
      blue: 0
    }
  }
}

// named!(pcx_palette_rgba<RGBColor>, do_parse!(
//   red: le_u8 >>
//   green: le_u8 >>
//   blue: le_u8 >>
//   (RGBColor {
//     red,
//     green,
//     blue,
//   })
// ));

// named!(pcx_palette<Vec<RGBColor>>, do_parse!(
//   tag!([0x0C]) >>
//   colors: many1!(pcx_palette_rgba) >>
//   (colors)
// ));

fn pcx_palette(buf: &[u8], palette: &mut [RGBColor; 256]) -> Result<(), &'static str> {
  if buf[0] != 0x0C {
    return Err("PCX extended palette marker 0x0C not found.");
  }

  for i in 0..256 {
    palette[i].red = buf[1 + 3 * i];
    palette[i].green = buf[2 + 3 * i];
    palette[i].blue = buf[3 + 3 * i];
  }

  Ok(())
}

fn read_uint16_le(buf: &[u8]) -> u16 {
  ((buf[1] as u16) << 8) + buf[0] as u16
}

fn pcx_read_dims(buf: &[u8]) -> (usize, usize) {
  let x0 = read_uint16_le(&buf[4..6]) as usize;
  let y0 = read_uint16_le(&buf[6..8]) as usize;
  let x1 = read_uint16_le(&buf[8..10]) as usize;
  let y1 = read_uint16_le(&buf[10..12]) as usize;

  return (x1 - x0 + 1, y1 - y0 + 1);
}

pub fn pcx_read<'a>(buf: &'a[u8], out: &mut [u8], mask: Option<&[u8]>) -> &'a[u8] {
  let (width, height) = pcx_read_dims(&buf);
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
  let mut palette: [RGBColor; 256] = [RGBColor::default(); 256];
  pcx_palette(rest, &mut palette).expect("pcx_palette failed.");

  for i in 0..pixels.len() {
    out[4 * i + 0] = palette[pixels[i] as usize].red;
    out[4 * i + 1] = palette[pixels[i] as usize].green;
    out[4 * i + 2] = palette[pixels[i] as usize].blue;
    out[4 * i + 3] = alpha[i];
  }

  return rest;
}

pub fn pcx_texture_array(buf: &[u8], out: &mut [u8], index_table: &[usize], mask_index_table: Option<&[usize]>) {
  let (width, height) = pcx_read_dims(&buf);
  let len = width * height * 4;
  for (i, idx) in index_table.iter().enumerate() {
    pcx_read(&buf[*idx..], &mut out[(i * len)..], mask_index_table.and_then(|mit| Some(&buf[mit[i]..])));
  }
}

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
    let (width, height) = pcx_read_dims(&buffer);

    assert_eq!(width, 256);
    assert_eq!(height, 256);
  }

  #[test]
  fn test_pcx_read() {
    let file = File::open("tests/tran_desertbrown.pcx").expect("File not found!");
    let mut buf_reader = BufReader::new(file);
    let mut buffer = Vec::new();

    buf_reader.read_to_end(&mut buffer).expect("read_to_end failed.");
    let mut out = [0u8; 256 * 256 * 4];
    pcx_read(&buffer, &mut out, None);
  }
}
