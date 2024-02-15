#[cfg(test)]
mod tests {
  use std::ops::Sub;

  use skia_safe::Point;

  #[test]
  fn angle_from_point() {
    let p1 = Point::new(0., 0.);
    let p2 = Point::new(20., 20.);
    let direction = p2.sub(p1);
    let angle = direction.y.atan2(direction.x);
    assert_eq!(45., angle.to_degrees())
  }

  #[test]
  fn angle_to_point() {
    let point = point_from(0., 20.);
    assert_eq!((20., 0.), point)
  }

  #[test]
  fn south_angle() {
    let point = point_from(90., 20.);
    assert_eq!((-8.742278e-7, 20.), point)
  }

  fn point_from(degrees: f32, length: f32) -> (f32, f32) {
    let radians = degrees.to_radians();
    let sin_cos = radians.sin_cos();
    (sin_cos.1 * length, sin_cos.0 * length)
  }
}