struct BmdHeader {
  num_frames: usize,
  num_pixels: usize,
  num_rows: usize,
}

struct BmdFrameInfo {
  frame_type: u32,
  width: usize,
  len: usize,
  off: usize,
}

struct BmdRowInfo {
  indent: usize,
  offset: usize,
}

impl BmdRowInfo {
  fn from_u32(u: u32) -> Self {
    BmdRowInfo {
      indent: (u >> 22) as usize,
      offset: (u & ((1 << 22) - 1)) as usize
    }
  }
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

fn read_rows<'a>(buf: &'a[u8], rows: &mut [BmdRowInfo]) -> Result<&'a[u8], &'static str> {
  if buf[0] != 0xE9 || buf[1] != 0x03 {
    return Err("read_frames: starting point is incorrect.");
  }

  let section_length = read_uint32_le(&buf[0x08..]) as usize;
  let mut rest: &[u8] = &buf[12..];

  for i in 0..section_length / 4 {
    let u = read_uint32_le(&rest);
    rows[i].indent = (u >> 22) as usize;
    rows[i].indent = (u & ((1 << 22) - 1)) as usize;
    rest = &rest[4..];
  }

  Ok(&rest)
}

fn read_pixels<'a>(buf: &'a[u8]) -> Result<(&'a[u8], &'a[u8]), &'static str> {
  if buf[0] != 0xE9 || buf[1] != 0x03 {
    return Err("read_pixels: starting point is incorrect.");
  }

  let section_length = read_uint32_le(&buf[0x08..]) as usize;

  Ok((&buf[section_length + 12..], &buf[12..section_length]))
}

