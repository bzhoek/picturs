pub fn factors_for(degrees: f32) -> (f32, f32) {
  let x = x_from_degrees(degrees);
  let y = y_from_degrees(degrees);
  (x, y)
}

pub fn x_from_degrees(degrees: f32) -> f32 {
  match degrees as i32 {
    1..=45 => degrees / 45.,
    46..=134 => 1.,
    135..=225 => (180. - degrees) / 45.,
    226..=314 => -1.,
    315..=360 => (degrees - 360.) / 45.,
    _ => 0.
  }
}

pub fn y_from_degrees(degrees: f32) -> f32 {
  match degrees as i32 {
    1..=45 => 1.,
    46..=134 => (90. - degrees) / 45.,
    135..=225 => -1.,
    226..=314 => (degrees - 270.) / 45.,
    315..=360 => 1.,
    _ => 0.
  }
}