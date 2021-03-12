// use rayon::prelude::*;

fn elevation_at(i: usize, w: usize, h: usize, elv: &[u8]) -> [f32; 4] {
  let row = i / w;
  let col = i % w;

  if row == 0 || row == h - 1 || col == 0 || col == w - 1 {
    return [0.0, 0.0, 0.0, 0.0];
  }

  return [
    (elv[i] as f32) / 16.0,
    (elv[i + w + (i / w % 2)] as f32) / 16.0,
    (elv[i + w + (i / w % 2) - 1] as f32) / 16.0,
    (elv[i + 1] as f32) / 16.0,
  ];
}

pub fn triangulate_map(map: &mut Vec<f32>, width: usize, height: usize, elevation: &[u8]) {
  map.chunks_mut(12).enumerate().for_each(|(i, r)| {
    let x = i % width;
    let y = i / width;

    let off = (y % 2) as f32;
    let elv = elevation_at(i, width, height, elevation);
    let fx = 2.0 * x as f32;
    let fy = 2.0 * y as f32;

    r[0] = fx + 0.0 + off;
    r[1] = fy + 0.0 - elv[0];
    r[2] = fx + 1.0 + off;
    r[3] = fy + 2.0 - elv[1];
    r[4] = fx - 1.0 + off;
    r[5] = fy + 2.0 - elv[2];

    r[6] = fx + 0.0 + off;
    r[7] = fy + 0.0 - elv[0];
    r[8] = fx + 2.0 + off;
    r[9] = fy + 0.0 - elv[3];
    r[10] = fx + 1.0 + off;
    r[11] = fy + 2.0 - elv[1];
  })
}
