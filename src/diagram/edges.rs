use std::ops::{Add, Sub};
use skia_safe::{Point, Rect, scalar};
use crate::trig::angle_at;

#[derive(Debug, PartialEq)]
pub struct Edge {
  pub from: Point,
  pub to: Point,
}

impl Edge {
  pub fn new(from: impl Into<Point>, to: impl Into<Point>) -> Self {
    Edge {
      from: from.into(),
      to: to.into(),
    }
  }

  pub fn angle(&self) -> f32 {
    self.angle_to(&self.to)
  }

  pub fn angle_to(&self, to: &Point) -> f32 {
    let direction = to.sub(self.from);
    direction.y.atan2(direction.x)
  }

  /// https://en.wikipedia.org/wiki/Interpolation
  pub fn interpolate(&self, t: scalar) -> Point {
    let x = Edge::lerp(self.from.x, self.to.x, t);
    let y = Edge::lerp(self.from.y, self.to.y, t);
    Point::new(x, y)
  }

  fn intersect_factor(
    a: impl Into<Point>,
    b: impl Into<Point>,
    c: impl Into<Point>,
    d: impl Into<Point>,
  ) -> Option<scalar> {
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

  pub fn intersects(&self, with: &Edge) -> Option<Point> {
    Self::intersect_factor(self.from, self.to, with.from, with.to).map(|alpha| {
      let x0 = Self::lerp(self.from.x, self.to.x, alpha);
      let y0 = Self::lerp(self.from.y, self.to.y, alpha);
      Point::new(x0, y0)
    })
  }
}

pub struct EdgeFinder {
  pub edges: Vec<Edge>,
  pub bounds: Rect,
}

impl EdgeFinder {
  pub fn cylinder(x: f32, y: f32, width: f32, height: f32) -> Self {
    let bounds = Rect::from_xywh(x, y, width, height);
    let half = height / 6.;
    let tl = Point::new(x, y + half);
    let tr = Point::new(x + width, y + half);
    let br = Point::new(x + width, y + height - half);
    let bl = Point::new(x, y + height - half);
    let edges = vec![Edge { from: tl, to: bl }, Edge { from: tr, to: br }];
    EdgeFinder { edges, bounds }
  }

  pub fn rectangle(x: f32, y: f32, width: f32, height: f32) -> Self {
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
    EdgeFinder { edges, bounds }
  }

  pub fn triangle(x: f32, y: f32, width: f32, height: f32) -> Self {
    let bounds = Rect::from_xywh(x, y, width, height);
    let tc = Point::new(x + width / 2., y);
    let br = Point::new(x + width, y + height);
    let bl = Point::new(x, y + height);
    let edges = vec![
      Edge { from: tc, to: br },
      Edge { from: br, to: bl },
      Edge { from: bl, to: tc },
    ];
    EdgeFinder { edges, bounds }
  }

  pub fn diamond(x: f32, y: f32, width: f32, height: f32) -> Self {
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
    EdgeFinder { edges, bounds }
  }

  pub fn file(x: f32, y: f32, width: f32, height: f32, radius: f32) -> Self {
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
    EdgeFinder { edges, bounds }
  }

  pub fn intersect(&self, degrees: f32) -> Option<Point> {
    let line = self.center_to_edge(degrees);
    self.edges.iter().find_map(|edge| line.intersects(edge))
  }

  pub fn center_to_edge(&self, degrees: f32) -> Edge {
    let from = self.bounds.center();
    let length = self.bounds.width().max(self.bounds.height());
    let offset: Point = angle_at(degrees, length).into();
    Edge::new(from, from.add(offset))
  }

  pub fn intersect_line(&self, line: &Edge) -> Option<Point> {
    self.edges.iter().find_map(|edge| line.intersects(edge))
  }

  #[allow(non_snake_case)]
  pub fn intersect_ellipse(line: &Edge, ellipse: &Rect) -> Option<(scalar, scalar)> {
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

}

