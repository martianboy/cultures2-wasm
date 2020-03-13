use nom::IResult;
use nom::{do_parse, named, take, tag, many1};
use nom::number::complete::{le_u8, le_u16};

pub struct PcxHeader<'a> {
  pub magic: u8,
  pub version: u8,
  pub encoding_method: u8,
  pub bits_per_pixel: u8,
  pub width: usize,
  pub height: usize,
  pub h_dpi: u16,
  pub v_dpi: u16,
  pub palette: &'a[u8],
  pub reserved: u8,
  pub color_planes: u8,
  pub bytes_per_color_plane: u16,
  pub palette_type: u16,
  pub h_res: u16,
  pub v_res: u16,
  // reserved_block: [u8; 54],
}

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

fn read_pixels<'a, 'b>(buf: &'a [u8], pixels: &'b mut Vec<u8>) -> IResult<&'a[u8], &'b[u8], ()> {
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

  Ok((&buf[pos..], &pixels[..]))
}

pub struct RGBColor {
  pub red: u8,
  pub green: u8,
  pub blue: u8,
}

named!(pcx_palette_rgba<RGBColor>, do_parse!(
  red: le_u8 >>
  green: le_u8 >>
  blue: le_u8 >>
  (RGBColor {
    red,
    green,
    blue,
  })
));

named!(pcx_palette<Vec<RGBColor>>, do_parse!(
  tag!([0x0C]) >>
  colors: many1!(pcx_palette_rgba) >>
  (colors)
));

named!(pcx_header<PcxHeader>, do_parse!(
  magic: le_u8 >>
  version: le_u8 >>
  encoding_method: le_u8 >>
  bits_per_pixel: le_u8 >>
  x0: le_u16 >>
  y0: le_u16 >>
  x1: le_u16 >>
  y1: le_u16 >>
  h_dpi: le_u16 >>
  v_dpi: le_u16 >>
  palette: take!(48) >>
  reserved: le_u8 >>
  color_planes: le_u8 >>
  bytes_per_color_plane: le_u16 >>
  palette_type: le_u16 >>
  h_res: le_u16 >>
  v_res: le_u16 >>
  take!(54) >>
  (PcxHeader {
    magic,
    version,
    encoding_method,
    bits_per_pixel,
    width: (x1 - x0 + 1) as usize,
    height: (y1 - y0 + 1) as usize,
    h_dpi,
    v_dpi,
    palette,
    reserved,
    color_planes,
    bytes_per_color_plane,
    palette_type,
    h_res,
    v_res,
  })
));

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
  let (rest, header) = pcx_header(buf).expect("pcx_header failed");
  let buf_length = header.width * header.height;

  let alpha = match mask {
    None => vec![0xFFu8; buf_length],
    Some(mask_buf) => {
      let mut mask_out_buf = vec![0xFFu8; buf_length];
      read_pixels(&mask_buf[0x80..], &mut mask_out_buf).expect("read_pixels failed for mask buffer.");

      mask_out_buf
    }
  };

  let mut pixels = vec![0; buf_length];
  let (rest, _) = read_pixels(&rest, &mut pixels).expect("read_pixels failed.");
  let (rest, palette) = pcx_palette(rest).expect("pcx_palette failed.");

  for i in 0..pixels.len() {
    out[4 * i + 0] = palette[pixels[i] as usize].red;
    out[4 * i + 1] = palette[pixels[i] as usize].green;
    out[4 * i + 2] = palette[pixels[i] as usize].blue;
    out[4 * i + 3] = alpha[i];
  }

  return rest;
}

pub fn pcx_texture_array(buf: &[u8], out: &mut [u8], index_table: &[usize], mask_index_table: Option<&[usize]>) {
  let (_, header) = pcx_header(buf).expect("pcx_header failed");
  let len = header.width * header.height * 4;
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
    let (_, header) = pcx_header(&buffer[..]).expect("pcx_header failed");

    assert_eq!(header.magic, 0x0A);
    assert_eq!(header.version, 0x05);
    assert_eq!(header.encoding_method, 0x01);
    assert_eq!(header.bits_per_pixel, 0x08);
    assert_eq!(header.width, 256);
    assert_eq!(header.height, 256);
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
