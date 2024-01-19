use std::ops::{Add, Mul};

use skia_safe::{Point, Rect, Vector};

#[derive(Debug)]
#[derive(PartialEq)]
pub struct Anchor {
  pub x: f32,
  pub y: f32,
}

impl Anchor {
  pub fn new(string: &str) -> Self {
    let dot_removed = string.trim_start_matches('.');
    match dot_removed.to_lowercase().as_str() {
      "n" => Self { x: 0., y: -0.5 },
      "ne" => Self { x: 0.5, y: -0.5 },
      "e" => Self { x: 0.5, y: 0. },
      "se" => Self { x: 0.5, y: 0.5 },
      "s" => Self { x: 0., y: 0.5 },
      "sw" => Self { x: -0.5, y: 0.5 },
      "w" => Self { x: -0.5, y: 0. },
      "nw" => Self { x: -0.5, y: -0.5 },
      _ => Self { x: 0., y: 0. }
    }
  }

  pub fn to_tuple(&self) -> (f32, f32) {
    (self.x, self.y)
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
}

#[derive(Debug)]
#[derive(PartialEq)]
pub enum Unit {
  Cm,
  Pc,
  Pt,
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

#[derive(Debug)]
#[derive(PartialEq)]
pub struct Length {
  length: f32,
  unit: Unit,
}

impl Default for Length {
  fn default() -> Self {
    Self {
      length: 0.,
      unit: Unit::Cm,
    }
  }
}

impl Length {
  pub fn new(length: f32, unit: Unit) -> Self {
    Self { length, unit }
  }

  pub(crate) fn pixels(&self) -> f32 {
    match self.unit {
      Unit::Cm => self.length * 38.,
      Unit::Pc => self.length * 16.,
      Unit::Pt => self.length * 1.3333,
    }
  }
}

#[derive(Debug)]
#[derive(PartialEq)]
pub struct Distance {
  length: f32,
  unit: Unit,
  pub direction: Vector,
}

impl Default for Distance {
  fn default() -> Self {
    Self {
      length: 0.,
      unit: Unit::Cm,
      direction: Vector::new(0., 0.),
    }
  }
}

impl Distance {
  pub fn new(length: f32, unit: Unit, direction: Vector) -> Self {
    Self { length, unit, direction }
  }

  pub(crate) fn pixels(&self) -> f32 {
    match self.unit {
      Unit::Cm => self.length * 38.,
      Unit::Pc => self.length * 16.,
      Unit::Pt => self.length * 1.3333,
    }
  }

  pub(crate) fn offset(&self) -> Point {
    self.direction.mul(self.pixels())
  }
}

#[derive(Debug)]
#[derive(PartialEq)]
pub struct Edge {
  pub(crate) id: String,
  pub(crate) anchor: Anchor,
}

impl Edge {
  pub fn new(id: &str, edge: &str) -> Self {
    let anchor = Anchor::new(edge);
    Self { id: id.to_string(), anchor }
  }
}
