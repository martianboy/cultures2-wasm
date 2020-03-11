fn tri_a_coords(i: usize, w: usize) -> [usize; 3] {
  let row = i / w;
  return [
      i as usize,
      (i + w + (row % 2)) as usize,
      (i + w + (row % 2) - 1) as usize,
  ];
}

fn tri_b_coords(i: usize, w: usize) -> [usize; 3] {
  let row = i / w;
  return [
      i as usize,
      (i + 1) as usize,
      (i + w + (row % 2)) as usize,
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
    (elv[coords[0]] as f32) / 25.0,
    (elv[coords[1]] as f32) / 25.0,
    (elv[coords[2]] as f32) / 25.0,
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
    (elv[coords[0]] as f32) / 25.0,
    (elv[coords[1]] as f32) / 25.0,
    (elv[coords[2]] as f32) / 25.0,
  ];
}

fn tri_a(map: &mut Vec<f32>, i: usize, width: usize, height: usize, elevation: &[u8]) {
  let y = i / (2 * width);
  let x = (i % (2 * width)) / 2;
  let ei = y * width + x;
  let off = (y % 2) as f32;
  let elv = elevation_at_tri_a(ei, width, height, &elevation);
  let fx = x as f32;
  let fy = y as f32;

  map[6 * i + 0] = (2.0 * fx + 0.0 + off) / 2.0;
  map[6 * i + 1] = (fy - elv[0] - 1.0) / 2.0;
  map[6 * i + 2] = (2.0 * fx + 1.0 + off) / 2.0;
  map[6 * i + 3] = (fy - elv[1]) / 2.0;
  map[6 * i + 4] = (2.0 * fx - 1.0 + off) / 2.0;
  map[6 * i + 5] = (fy - elv[2]) / 2.0;
}

fn tri_b(map: &mut Vec<f32>, i: usize, width: usize, height: usize, elevation: &[u8]) {
  let y = i / (2 * width);
  let x = (i % (2 * width)) / 2;
  let ei = y * width + x;
  let off = (y % 2) as f32;
  let elv = elevation_at_tri_b(ei, width, height, elevation);
  let fx = x as f32;
  let fy = y as f32;

  map[6 * i + 0] = (2.0 * fx + 0.0 + off) / 2.0;
  map[6 * i + 1] = (fy - elv[0] - 1.0) / 2.0;
  map[6 * i + 2] = (2.0 * fx + 2.0 + off) / 2.0;
  map[6 * i + 3] = (fy - elv[1] - 1.0) / 2.0;
  map[6 * i + 4] = (2.0 * fx + 1.0 + off) / 2.0;
  map[6 * i + 5] = (fy - elv[2]) / 2.0;
}

pub fn triangulate_map(map: &mut Vec<f32>, width: usize, height: usize, elevation: &[u8]) {
  let len = width * height * 2;
  let mut a_or_b: bool = false;

  for i in 0..len {
    if a_or_b {
      tri_b(map, i, width, height, elevation);
    } else {
      tri_a(map, i, width, height, elevation);
    }
    a_or_b = !a_or_b
  }
}