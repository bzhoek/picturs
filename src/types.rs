use std::ops::{Add, Mul};

use skia_safe::{Point, Rect, Vector};

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Edge {
  pub x: f32,
  pub y: f32,
}

impl Edge {
  pub fn new(string: &str) -> Self {
    let dot_removed = string.trim_start_matches('.');
    match dot_removed.to_lowercase().as_str() {
      "n" | "up" => Self { x: 0., y: -0.5 },
      "ne" => Self { x: 0.5, y: -0.5 },
      "e" | "right" => Self { x: 0.5, y: 0. },
      "se" => Self { x: 0.5, y: 0.5 },
      "s" | "down" => Self { x: 0., y: 0.5 },
      "sw" => Self { x: -0.5, y: 0.5 },
      "w" | "left" => Self { x: -0.5, y: 0. },
      "nw" => Self { x: -0.5, y: -0.5 },
      _ => Self { x: 0., y: 0. }
    }
  }

  pub fn flip(&self) -> Self {
    match self {
      v if v.y == 0. && v.x != 0. => Self { x: -self.x, y: self.y },
      v if v.y != 0. && v.x == 0. => Self { x: self.x, y: -self.y },
      _ => Self { x: self.x, y: -self.y }
    }
  }

  pub fn tuple(&self) -> (f32, f32) {
    (self.x, self.y)
  }

  pub fn vector(&self) -> Vector {
    Vector::new(self.x, self.y).mul(2.0)
  }

  pub fn to_edge(&self, rect: &Rect) -> Point {
    let mut point = rect.center();
    point.offset((self.x * rect.width(), self.y * rect.height()));
    point
  }

  pub fn topleft_offset(&self, rect: &Rect) -> Point {
    let point = Point::new(self.x, self.y);
    let point = point.add(Vector::new(0.5, 0.5));
    Point::new(rect.width() * -point.x, rect.height() * -point.y)
  }

  pub fn horizontal(&self) -> bool {
    self.y == 0.
  }

  pub fn vertical(&self) -> bool {
    self.x == 0.
  }
}

#[derive(Debug, Default, PartialEq)]
pub enum Unit {
  #[default]
  Pt,
  Pc,
  Cm,
}

impl From<&str> for Unit {
  fn from(item: &str) -> Self {
    match item {
      "cm" => Unit::Cm,
      "pc" => Unit::Pc,
      "pt" => Unit::Pt,
      _ => panic!("unknown unit {}", item)
    }
  }
}

#[derive(Debug, Default, PartialEq)]
pub struct Length {
  length: f32,
  unit: Unit,
}

impl Length {
  pub fn new(length: f32, unit: Unit) -> Self {
    Self { length, unit }
  }

  pub fn pixels(&self) -> f32 {
    match self.unit {
      Unit::Cm => self.length * 38.,
      Unit::Pc => self.length * 16.,
      Unit::Pt => self.length * 1.3333,
    }
  }
}

#[derive(Debug, Default, PartialEq)]
pub struct Displacement {
  length: Length,
  direction: Vector,
}

impl Displacement {
  pub fn new(length: f32, unit: Unit, direction: Vector) -> Self {
    let length = Length::new(length, unit);
    Self { length, direction }
  }

  pub fn offset(&self) -> Point {
    self.direction.mul(self.length.pixels())
  }

  pub fn is_horizontal(&self) -> bool {
    self.direction.x != 0.
  }

  pub fn is_vertical(&self) -> bool {
    self.direction.y != 0.
  }
}

#[derive(Debug, PartialEq)]
pub struct ObjectEdge {
  pub(crate) id: String,
  pub(crate) edge: Edge,
}

impl ObjectEdge {
  pub fn new(id: &str, edge: &str) -> Self {
    Self { id: id.into(), edge: Edge::new(edge) }
  }
}
