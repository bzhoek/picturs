use std::ops::Sub;

use skia_safe::Point;

use picturs::trig::{x_from_degrees, y_from_degrees};

#[cfg(test)]
mod tests {
  use skia_safe::{Color, PaintStyle, Rect, scalar};

  use picturs::skia::Canvas;
  use picturs::test::assert_canvas;
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

  fn angle_between(p1: Point, p2: Point) -> f32 {
    let direction = p2.sub(p1);
    direction.y.atan2(direction.x)
  }


  #[allow(non_snake_case)]
  fn intersect_ellipse(ellipse: Rect, from: Point, to: Point) -> Option<(scalar, scalar)> {
    let e = ellipse.center();
    let w = ellipse.width() / 2.;
    let h = ellipse.height() / 2.;
    let p1 = from.sub(e);
    let p2 = to.sub(e);

    let d = p2.sub(p1);
    let A = d.x * d.x / w / w + d.y * d.y / h / h;
    let B = 2. * p1.x * (d.x) / w / w + 2. * p1.y * (d.y) / h / h;
    let C = p1.x * p1.x / w / w + p1.y * p1.y / h / h - 1.;
    let D = B * B - 4. * A * C;
    if D == 0. {} else if D > 0. {
      let t1 = (-B - D.sqrt()) / (2. * A);
      let t2 = (-B + D.sqrt()) / (2. * A);
      return Some((t1, t2));
    }
    None
  }

  // https://github.com/davidfig/intersects/blob/master/ellipse-line.js
  // http://www.csharphelper.com/howtos/howto_line_ellipse_intersection.html
  #[test]
  fn arc_intersect_js() {
    let rect = Rect::from_xywh(8., 8., 40., 48.);
    let top = Rect::from_xywh(rect.left, rect.top, rect.width(), rect.height() / 3.);
    let bottom = Rect::from_xywh(rect.left, rect.bottom - top.height(), rect.width(), top.height());
    let p1 = rect.center();
    let p2 = Point::new(rect.right, rect.top);
    let result = intersect_ellipse(top, p1, p2);
    // assert_eq!(result, Some((0.67752552, 0.922474503)));
    if let Some((t1, t2)) = result {
      let x1 = lerp(p1.x, p2.x, t1);
      let y1 = lerp(p1.y, p2.y, t1);

      let x2 = lerp(p1.x, p2.x, t2);
      let y2 = lerp(p1.y, p2.y, t2);
      // assert_eq!(x1, 23.5505104);
      // assert_eq!(y1, 17.7393875);

      let mut canvas = Canvas::new((56, 64));

      canvas.paint.set_style(PaintStyle::Stroke);

      canvas.paint.set_color(Color::BLUE);
      canvas.rectangle(&rect, 0.);

      canvas.paint.set_color(Color::BLACK);
      canvas.circle(&p1, 1.);
      canvas.ellipse(&top);
      canvas.ellipse(&bottom);

      canvas.paint.set_color(Color::RED);
      canvas.circle(&Point::new(x1, y1), 1.);
      canvas.circle(&Point::new(x2, y2), 1.);

      assert_canvas(canvas, "target/ellipse_intersect").unwrap();
    }
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

    let angle = angle_between(center, tc);
    assert_eq!(angle.to_degrees().round(), -90.);

    let tl = Point::new(top.left, top.bottom);
    let rm = Point::new(rect.right, rect.center_y());
    let angle = angle_between(center, rm);
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

  /// https://en.wikipedia.org/wiki/Interpolation
  fn lerp(a: scalar, b: scalar, t: f32) -> scalar {
    a + (b - a) * t
  }
}