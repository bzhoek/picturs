use std::ops::Sub;

use skia_safe::Point;

use picturs::trig::{x_from_degrees, y_from_degrees};

#[cfg(test)]
mod tests {
  use skia_safe::{Rect, scalar};

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
    assert_eq!(x, 0.44444445);
    let x = x_from_degrees(45.);
    assert_eq!(x, 1.);
    let x = x_from_degrees(90.);
    assert_eq!(x, 1.);
    let x = x_from_degrees(135.);
    assert_eq!(x, 1.);
    let x = x_from_degrees(180.);
    assert_eq!(x, 0.);
    let x = x_from_degrees(200.);
    assert_eq!(x, -0.44444445);
    let x = x_from_degrees(225.);
    assert_eq!(x, -1.);
    let x = x_from_degrees(315.);
    assert_eq!(x, -1.);
    let x = x_from_degrees(360.);
    assert_eq!(x, 0.0);
  }

  #[test]
  fn y_from_angle() {
    let y = y_from_degrees(45.);
    assert_eq!(y, 1.);
    let y = y_from_degrees(65.);
    assert_eq!(y, 0.5555556);
    let y = y_from_degrees(90.);
    assert_eq!(y, 0.);
    let y = y_from_degrees(135.);
    assert_eq!(y, -1.);
    let y = y_from_degrees(180.);
    assert_eq!(y, -1.);
    let y = y_from_degrees(225.);
    assert_eq!(y, -1.);
    let y = y_from_degrees(245.);
    assert_eq!(y, -0.5555556);
    let y = y_from_degrees(315.);
    assert_eq!(y, 1.);
    let y = y_from_degrees(360.);
    assert_eq!(y, 1.0);
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

  fn angle_between(p1: &Point, p2: &Point) -> f32 {
    let direction = p2.sub(*p1);
    direction.y.atan2(direction.x)
  }

  // https://stackoverflow.com/questions/1073336/circle-line-segment-collision-detection-algorithm
  #[test]
  #[allow(non_snake_case)]
  fn circle_intersect() {
    let rect = Rect::from_xywh(0., 0., 20., 24.);
    let top = Rect::from_xywh(rect.left, rect.top, rect.width(), rect.height() / 3.);

    let E = rect.center();
    let L = Point::new(top.left, top.bottom());

    let C = Point::new(top.center_x(), top.bottom);
    let r = top.width() / 2.;

    let d = L.sub(E);
    let f = E.sub(C);

    let a = d.dot(d);
    let b = 2. * f.dot(d);
    let c = f.dot(f) - r * r;
    let discriminant = b * b - 4. * a * c;
    assert_eq!(discriminant, 40000.);
    let discriminant = discriminant.sqrt();
    let t1 = (-b - discriminant) / (2. * a);
    let t2 = (-b + discriminant) / (2. * a);
    assert_eq!(t1, -0.7241379);
    assert_eq!(t2, 1.0);
  }

  #[test]
  #[allow(non_snake_case)]
  fn cylinder_intersect() {
    let rect = Rect::from_xywh(0., 0., 20., 24.);
    let top = Rect::from_xywh(rect.left, rect.top, rect.width(), rect.height() / 6.);
    let bottom = Rect::from_xywh(rect.left, rect.bottom - top.height(), rect.width(), top.height());
    assert_eq!(top.height(), 4.);
    assert_eq!(bottom.top, 20.);

    let center = rect.center();

    let tc = Point::new(top.center_x(), top.bottom);
    let direction = tc.sub(center);
    let angle = direction.y.atan2(direction.x);
    assert_eq!(angle.to_degrees().round(), -90.);

    let angle = angle_between(&center, &tc);
    assert_eq!(angle.to_degrees().round(), -90.);

    let tl = Point::new(top.left, top.bottom);
    let rm = Point::new(rect.right, rect.center_y());
    let angle = angle_between(&center, &rm);
    assert_eq!(angle.round(), 0.);

    let direction = tl.sub(center);
    let angle = direction.y.atan2(direction.x);
    assert_eq!(angle.to_degrees().round(), -141.);

    let tr = Point::new(top.right, top.bottom);
    let direction = tr.sub(center);
    let angle = direction.y.atan2(direction.x);
    assert_eq!(angle.to_degrees().round(), -39.);

    // https://stackoverflow.com/questions/30006155/calculate-intersect-point-between-arc-and-line
  }

  fn lerp(a: scalar, b: scalar, t: f32) -> scalar {
    a + (b - a) * t
  }
}