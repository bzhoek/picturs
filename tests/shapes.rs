use std::ops::Add;
use skia_safe::{Point, Rect};
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

  fn intersects(&self, with: &Edge) -> Option<Point> {
    let x1 = self.from.x;
    let y1 = self.from.y;
    let x2 = self.to.x;
    let y2 = self.to.y;
    let x3 = with.from.x;
    let y3 = with.from.y;
    let x4 = with.to.x;
    let y4 = with.to.y;
    let x = ((x1 * y2 - y1 * x2) * (x3 - x4) - (x1 - x2) * (x3 * y4 - y3 * x4))
      / ((x1 - x2) * (y3 - y4) - (y1 - y2) * (x3 - x4));
    let y = ((x1 * y2 - y1 * x2) * (y3 - y4) - (y1 - y2) * (x3 * y4 - y3 * x4))
      / ((x1 - x2) * (y3 - y4) - (y1 - y2) * (x3 - x4));
    if x.is_nan() || y.is_nan() {
      None
    } else {
      Some(Point::new(x, y))
    }
  }
}

struct Shape {
  edges: Vec<Edge>,
  bounds: Rect,
}

impl Shape {
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
  use std::ops::Add;
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
  fn rectangle() {
    let shape = Shape::rectangle(0., 0., 20., 20.);
    assert_eq!(4, shape.edges.len());
    assert_eq!(Edge::new((0, 0), (20, 0)), shape.edges[0]);
  }

  #[test]
  fn rectangle_intersects() {
    let shape = Shape::rectangle(0., 0., 20., 20.);

    let angle = angle_from(shape.bounds.center(), 0., 10.);
    let intersect = angle.intersects(&shape.edges[0]);
    assert_eq!(Some(Point::new(10., -0.)), round(intersect));

    let intersect = angle.intersects(&shape.edges[1]);
    assert_eq!(None, intersect);

    let angle = angle_from(shape.bounds.center(), 90., 10.);
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
}