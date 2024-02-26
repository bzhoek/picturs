use std::ops::{Add, Sub};
use skia_safe::{Point, Rect, scalar};
use picturs::trig::angle_at;

#[derive(Debug, PartialEq)]
struct Edge {
  from: Point,
  to: Point,
}

impl Edge {
  fn new(from: impl Into<Point>, to: impl Into<Point>) -> Self {
    Edge { from: from.into(), to: to.into() }
  }

  fn angle(&self) -> f32 {
    self.angle_to(&self.to)
  }

  fn angle_to(&self, to: &Point) -> f32 {
    let direction = to.sub(self.from);
    direction.y.atan2(direction.x)
  }

  /// https://en.wikipedia.org/wiki/Interpolation
  fn interpolate(&self, t: scalar) -> Point {
    let x = Edge::lerp(self.from.x, self.to.x, t);
    let y = Edge::lerp(self.from.y, self.to.y, t);
    Point::new(x, y)
  }

  fn intersect_factor(a: impl Into<Point>, b: impl Into<Point>, c: impl Into<Point>, d: impl Into<Point>) -> Option<scalar> {
    let a = a.into();
    let b = b.into();
    let c = c.into();
    let d = d.into();

    let alpha = (d.x - c.x) * (c.y - a.y) - (d.y - c.y) * (c.x - a.x);
    let beta = (b.x - a.x) * (c.y - a.y) - (b.y - a.y) * (c.x - a.x);
    let bottom = (d.x - c.x) * (b.y - a.y) - (d.y - c.y) * (b.x - a.x);

    let alpha = alpha / bottom;
    let beta = beta / bottom;

    if alpha > 0. && alpha < 1. && beta > 0. && beta < 1. {
      return Some(alpha);
    }
    None
  }

  fn lerp(a: scalar, b: scalar, t: f32) -> scalar {
    a + (b - a) * t
  }

  fn intersects(&self, with: &Edge) -> Option<Point> {
    Self::intersect_factor(self.from, self.to, with.from, with.to)
      .map(|alpha| {
        let x0 = Self::lerp(self.from.x, self.to.x, alpha);
        let y0 = Self::lerp(self.from.y, self.to.y, alpha);
        Point::new(x0, y0)
      })
  }
}

struct Shape {
  edges: Vec<Edge>,
  bounds: Rect,
}

impl Shape {
  fn cylinder(x: f32, y: f32, width: f32, height: f32) -> Self {
    let bounds = Rect::from_xywh(x, y, width, height);
    let half = height / 6.;
    let tl = Point::new(x, y + half);
    let tr = Point::new(x + width, y + half);
    let br = Point::new(x + width, y + height - half);
    let bl = Point::new(x, y + height - half);
    let edges = vec![
      Edge { from: tl, to: bl },
      Edge { from: tr, to: br },
    ];
    Shape { edges, bounds }
  }

  fn rectangle(x: f32, y: f32, width: f32, height: f32) -> Self {
    let bounds = Rect::from_xywh(x, y, width, height);
    let tl = Point::new(x, y);
    let tr = Point::new(x + width, y);
    let br = Point::new(x + width, y + height);
    let bl = Point::new(x, y + height);
    let edges = vec![
      Edge { from: tl, to: tr },
      Edge { from: tr, to: br },
      Edge { from: br, to: bl },
      Edge { from: bl, to: tl },
    ];
    Shape { edges, bounds }
  }

  fn triangle(x: f32, y: f32, width: f32, height: f32) -> Self {
    let bounds = Rect::from_xywh(x, y, width, height);
    let tc = Point::new(x + width / 2., y);
    let br = Point::new(x + width, y + height);
    let bl = Point::new(x, y + height);
    let edges = vec![
      Edge { from: tc, to: br },
      Edge { from: br, to: bl },
      Edge { from: bl, to: tc },
    ];
    Shape { edges, bounds }
  }

  fn diamond(x: f32, y: f32, width: f32, height: f32) -> Self {
    let bounds = Rect::from_xywh(x, y, width, height);
    let tc = Point::new(x + width / 2., y);
    let mr = Point::new(x + width, y + height / 2.);
    let bc = Point::new(x + width / 2., y + height);
    let ml = Point::new(x, y + height / 2.);
    let edges = vec![
      Edge { from: tc, to: mr },
      Edge { from: mr, to: bc },
      Edge { from: bc, to: ml },
      Edge { from: ml, to: tc },
    ];
    Shape { edges, bounds }
  }

  fn file(x: f32, y: f32, width: f32, height: f32, radius: f32) -> Self {
    let bounds = Rect::from_xywh(x, y, width, height);
    let tl = Point::new(x, y);
    let tr1 = Point::new(x + width - radius, y);
    let tr2 = Point::new(x + width, y + radius);
    let br = Point::new(x + width, y + height);
    let bl = Point::new(x, y + height);
    let edges = vec![
      Edge { from: tl, to: tr1 },
      Edge { from: tr1, to: tr2 },
      Edge { from: tr2, to: br },
      Edge { from: br, to: bl },
      Edge { from: bl, to: tl },
    ];
    Shape { edges, bounds }
  }

  fn intersect(&self, degrees: f32) -> Option<Point> {
    let from = self.bounds.center();
    let length = self.bounds.width().max(self.bounds.height());
    let offset: Point = angle_at(degrees, length).into();
    let line = Edge::new(from, from.add(offset));
    self.edges.iter().find_map(|edge| line.intersects(edge))
  }
}

#[cfg(test)]
mod tests {
  use std::ops::{Add, Sub};
  use skia_safe::{Color, PaintStyle};
  use picturs::skia::Canvas;
  use picturs::test::assert_canvas;
  use picturs::trig::angle_at;
  use super::*;

  fn angle_from(from: impl Into<Point>, angle: f32, length: f32) -> Edge {
    let from = from.into();
    let offset: Point = angle_at(angle, length).into();
    Edge::new(from, from.add(offset))
  }

  #[test]
  fn angle() {
    let angle = angle_from((10., 10.), 0., 10.);
    assert_eq!(Edge::new((10, 10), (10, 0)), angle);
  }

  #[test]
  fn intersect() {
    let edge = Edge::new((-10, -10), (5, -10));
    let angle = angle_from((0., 0.), 25., 40.);
    let intersect = angle.intersects(&edge);
    assert_eq!(round(intersect), Some(Point::new(5., -10.)));
  }

  #[test]
  fn rectangle() {
    let shape = Shape::rectangle(0., 0., 20., 20.);
    assert_eq!(4, shape.edges.len());
    assert_eq!(Edge::new((0, 0), (20, 0)), shape.edges[0]);
  }

  #[test]
  fn rectangle_intersects() {
    let shape = Shape::rectangle(0., 0., 20., 20.);

    let angle = angle_from(shape.bounds.center(), 0., 11.);
    let intersect = angle.intersects(&shape.edges[0]);
    assert_eq!(Some(Point::new(10., -0.)), round(intersect));

    let intersect = angle.intersects(&shape.edges[1]);
    assert_eq!(None, intersect);

    let angle = angle_from(shape.bounds.center(), 90., 11.);
    let intersect = angle.intersects(&shape.edges[1]);
    assert_eq!(Some(Point::new(20., 10.)), round(intersect));
  }

  fn round(point: Option<Point>) -> Option<Point> {
    point.map(|p| Point::new(p.x.round(), p.y.round()))
  }

  #[test]
  fn triangle() {
    let shape = Shape::triangle(0., 0., 20., 20.);
    assert_eq!(3, shape.edges.len());
    assert_eq!(Edge::new((10, 0), (20, 20)), shape.edges[0]);
  }

  #[test]
  fn triangle_intersects() {
    let shape = Shape::triangle(0., 0., 20., 20.);

    let angle = angle_from(shape.bounds.center(), 45., 10.);
    let intersect = angle.intersects(&shape.edges[0]);
    assert_eq!(Some(Point::new(13., 7.)), round(intersect));
  }

  #[test]
  fn triangle_intersect() {
    let shape = Shape::triangle(0., 0., 20., 20.);
    let intersect = shape.intersect(45.);
    assert_eq!(Some(Point::new(13., 7.)), round(intersect));
  }

  #[test]
  fn diamond() {
    let shape = Shape::diamond(0., 0., 20., 20.);
    assert_eq!(4, shape.edges.len());
    assert_eq!(Edge::new((10, 0), (20, 10)), shape.edges[0]);
  }

  #[test]
  fn file() {
    let shape = Shape::file(0., 0., 20., 20., 5.);
    assert_eq!(5, shape.edges.len());
    assert_eq!(shape.edges[0], Edge::new((0, 0), (15, 0)));
    assert_eq!(shape.edges[1], Edge::new((15, 0), (20, 5)));
  }

  #[test]
  fn file_intersect() {
    let shape = Shape::file(-10., -10., 20., 20., 5.);
    assert_eq!(shape.edges[0], Edge::new((-10, -10), (5, -10)));

    let angle = angle_from(shape.bounds.center(), 45., 20.);
    let intersect = angle.intersects(&shape.edges[0]);
    assert_eq!(round(intersect), None);

    let intersect = shape.intersect(45.);
    assert_eq!(round(intersect), Some(Point::new(7., -8.)));
  }

  // http://www.csharphelper.com/howtos/howto_line_ellipse_intersection.html
  // https://github.com/davidfig/intersects/blob/master/ellipse-line.js
  #[test]
  fn arc_intersect() {
    let rect = Rect::from_xywh(8., 8., 40., 48.);
    let shape = Shape::cylinder(8., 8., 40., 48.);
    let top = Rect::from_xywh(rect.left, rect.top, rect.width(), rect.height() / 3.);
    let bottom = Rect::from_xywh(rect.left, rect.bottom - top.height(), rect.width(), top.height());

    let line = Edge::new(rect.center(), (rect.right + 1., rect.center_y()));

    let mut canvas = prepare_cylinder_canvas(&rect, &top, &bottom, &line);

    let result = if in_arc_range(&top, &line) {
      intersect_ellipse(&top, &line)
    } else if in_arc_range(&bottom, &line) {
      intersect_ellipse(&bottom, &line)
    } else {
      let intersect = shape.intersect(275.);
      if let Some(i) = intersect {
        canvas.paint.set_color(Color::GREEN);
        canvas.circle(&i, 1.);
      }
      None
    };

    if let Some((t1, t2)) = result {
      let i1 = line.interpolate(t1);
      let i2 = line.interpolate(t2);

      canvas.paint.set_color(Color::RED);
      canvas.circle(&i1, 1.);
      canvas.circle(&i2, 1.);
    }

    assert_canvas(canvas, "target/ellipse_intersect").unwrap();
  }

  fn prepare_cylinder_canvas(rect: &Rect, top: &Rect, bottom: &Rect, line: &Edge) -> Canvas {
    let mut canvas = Canvas::new((56, 64));

    canvas.paint.set_style(PaintStyle::Stroke);

    canvas.paint.set_color(Color::BLUE);
    canvas.rectangle(&rect, 0.);

    canvas.paint.set_color(Color::WHITE);
    canvas.circle(&line.from, 1.);
    canvas.ellipse(&top);
    canvas.ellipse(&bottom);
    canvas
  }

  #[allow(non_snake_case)]
  fn intersect_ellipse(ellipse: &Rect, line: &Edge) -> Option<(scalar, scalar)> {
    let e = ellipse.center();
    let w = ellipse.width() / 2.;
    let h = ellipse.height() / 2.;
    let p1 = line.from.sub(e);
    let p2 = line.to.sub(e);

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

  fn in_arc_range(rect: &Rect, line: &Edge) -> bool {
    let mid_left = Point::new(rect.left, rect.center_y());
    let mid_right = Point::new(rect.right, rect.center_y());
    let left_angle = line.angle_to(&mid_left);
    let right_angle = line.angle_to(&mid_right);
    let line_angle = line.angle();
    return if line_angle < 0. {
      line_angle < right_angle && line_angle > left_angle
    } else {
      line_angle > right_angle && line_angle < left_angle
    };
  }
}