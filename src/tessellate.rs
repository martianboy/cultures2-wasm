#[inline]
fn tri_a_coords(i: usize, w: usize) -> [usize; 3] {
  return [
    i as usize,
    (i + w + (i / w % 2)) as usize,
    (i + w + (i / w % 2) - 1) as usize,
  ];
}

#[inline]
fn tri_b_coords(i: usize, w: usize) -> [usize; 3] {
  return [
    i as usize,
    (i + 1) as usize,
    (i + w + (i / w % 2)) as usize,
  ];
}

fn elevation_at_tri_a(i: usize, width: usize, height: usize, elv: &[u8]) -> [f32; 3] {
  let row = i / width;
  let col = i % width;

  if row == 0 || row == height - 1 || col == 0 || col == width - 1 {
    return [0.0, 0.0, 0.0];
  }

  let coords = tri_a_coords(i, width);
  return [
    (elv[coords[0]] as f32) / 16.0,
    (elv[coords[1]] as f32) / 16.0,
    (elv[coords[2]] as f32) / 16.0,
  ];
}

fn elevation_at_tri_b(i: usize, width: usize, height: usize, elv: &[u8]) -> [f32; 3] {
  let row = i / width;
  let col = i % width;
  if row == 0 || row == height - 1 || col == 0 || col == width - 1 {
    return [0.0, 0.0, 0.0];
  }
  let coords = tri_b_coords(i, width);
  return [
    (elv[coords[0]] as f32) / 16.0,
    (elv[coords[1]] as f32) / 16.0,
    (elv[coords[2]] as f32) / 16.0,
  ];
}

pub fn triangulate_map(map: &mut Vec<f32>, width: usize, height: usize, elevation: &[u8]) {
  let len = width * height;

  for i in 0..len {
    let x = i % width;
    let y = i / width;

    let off = (y % 2) as f32;
    let elv_a = elevation_at_tri_a(i, width, height, elevation);
    let elv_b = elevation_at_tri_b(i, width, height, elevation);
    let fx = 2.0 * x as f32;
    let fy = 2.0 * y as f32;

    map[12 * i + 0] = fx + 0.0 + off;
    map[12 * i + 1] = fy + 0.0 - elv_a[0];
    map[12 * i + 2] = fx + 1.0 + off;
    map[12 * i + 3] = fy + 2.0 - elv_a[1];
    map[12 * i + 4] = fx - 1.0 + off;
    map[12 * i + 5] = fy + 2.0 - elv_a[2];

    map[12 * i + 6] = fx + 0.0 + off;
    map[12 * i + 7] = fy + 0.0 - elv_b[0];
    map[12 * i + 8] = fx + 2.0 + off;
    map[12 * i + 9] = fy + 0.0 - elv_b[1];
    map[12 * i + 10] = fx + 1.0 + off;
    map[12 * i + 11] = fy + 2.0 - elv_b[2];
  }
}
