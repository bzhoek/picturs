use std::ops::Sub;

use skia_safe::Point;

use picturs::trig::{x_from_degrees, y_from_degrees};

#[cfg(test)]
mod tests {
  use skia_safe::scalar;
  use picturs::trig::angle_at;
  use super::*;

  #[test]
  fn angle_from_point() {
    let p1 = Point::new(0., 0.);
    let p2 = Point::new(20., 20.);
    let direction = p2.sub(p1);
    let angle = direction.y.atan2(direction.x);
    assert_eq!(45., angle.to_degrees())
  }

  #[test]
  fn circle_north() {
    let point = angle_at(0., 20.);
    assert_eq!((2.3849762e-7, -20.), point)
  }

  #[test]
  fn circle_south() {
    let point = angle_at(180., 20.);
    assert_eq!((2.7814185e-6, 20.), point)
  }

  #[test]
  fn x_from_angle() {
    let x = x_from_degrees(20.);
    assert_eq!(0.44444445, x);
    let x = x_from_degrees(45.);
    assert_eq!(1., x);
    let x = x_from_degrees(90.);
    assert_eq!(1., x);
    let x = x_from_degrees(135.);
    assert_eq!(1., x);
    let x = x_from_degrees(180.);
    assert_eq!(0., x);
    let x = x_from_degrees(200.);
    assert_eq!(-0.44444445, x);
    let x = x_from_degrees(225.);
    assert_eq!(-1., x);
    let x = x_from_degrees(315.);
    assert_eq!(-1., x);
    let x = x_from_degrees(360.);
    assert_eq!(0.0, x);
  }

  #[test]
  fn y_from_angle() {
    let y = y_from_degrees(45.);
    assert_eq!(1., y);
    let y = y_from_degrees(65.);
    assert_eq!(0.5555556, y);
    let y = y_from_degrees(90.);
    assert_eq!(0., y);
    let y = y_from_degrees(135.);
    assert_eq!(-1., y);
    let y = y_from_degrees(180.);
    assert_eq!(-1., y);
    let y = y_from_degrees(225.);
    assert_eq!(-1., y);
    let y = y_from_degrees(245.);
    assert_eq!(-0.5555556, y);
    let y = y_from_degrees(315.);
    assert_eq!(1., y);
    let y = y_from_degrees(360.);
    assert_eq!(1.0, y);
  }

  // https://www.youtube.com/watch?v=bvlIYX9cgls
  #[test]
  fn line_intersection() {
    let a = Point::new(1., 0.);
    let b = Point::new(3., 3.);
    let ab = b.sub(a);
    assert_eq!(ab, (2., 3.).into());

    let c = Point::new(1., 3.);
    let d = Point::new(3., 1.);
    let cd = d.sub(c);
    assert_eq!(cd, (2., -2.).into());

    let a_ = (d.x - c.x) * (c.y - a.y) - (d.y - c.y) * (c.x - a.x);
    let b_ = (d.x - c.x) * (b.y - a.y) - (d.y - c.y) * (b.x - a.x);
    let c_ = (b.x - a.x) * (c.y - a.y) - (b.y - a.y) * (c.x - a.x);
    assert_eq!(b_, 10.);

    let x0 = lerp(a.x, b.x, a_ / b_);
    assert_eq!(x0, 2.2);
    let x0 = lerp(c.x, d.x, c_ / b_);
    assert_eq!(x0, 2.2);
    let y0 = lerp(c.y, d.y, c_ / b_);
    assert_eq!(y0, 1.8);
  }

  /// https://en.wikipedia.org/wiki/Interpolation
  fn lerp(a: scalar, b: scalar, t: f32) -> scalar {
    a + (b - a) * t
  }
}