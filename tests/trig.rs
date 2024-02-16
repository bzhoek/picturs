#[cfg(test)]
mod tests {
  use std::ops::Sub;

  use skia_safe::{Point, Rect};
  use picturs::trig::{point_from, x_from_degrees, y_from_degrees};

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
    let point = circle_edge(0., 20.);
    assert_eq!((2.3849762e-7, -20.), point)
  }

  #[test]
  fn circle_south() {
    let point = circle_edge(180., 20.);
    assert_eq!((2.7814185e-6, 20.), point)
  }

  fn circle_edge(degrees: f32, length: f32) -> (f32, f32) {
    let north = degrees + 270.;
    let radians = north.to_radians();
    let sin_cos = radians.sin_cos();
    (sin_cos.1 * length, sin_cos.0 * length)
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


  #[test]
  fn rect_nnw() {
    let rect = Rect::from_xywh(0., 0., 20., 10.);
    let degrees = 20.;
    let point = point_from(degrees, rect);
    assert_eq!((4.4444447, 5.), point)
  }
}