use std::ops::{Add, Mul};

use skia_safe::{Color, Font, Point, Rect, Size, Vector};

use crate::diagram::conversion::{HEIGHT, WIDTH};
use crate::diagram::types::EdgeDirection::{Horizontal, Vertical};
use crate::trig::{x_from_degrees, y_from_degrees};

pub const BLOCK_PADDING: f32 = 8.;

#[derive(Debug, PartialEq)]
pub enum Node<'a> {
  Container(Option<&'a str>, Radius, Option<&'a str>, Rect, Rect, Vec<Node<'a>>),
  Primitive(Option<&'a str>, Rect, Rect, Color, Shape<'a>),
}

type EdgeMovement = (Edge, Vec<Movement>, ObjectEdge);

#[derive(Debug, PartialEq)]
pub enum Shape<'a> {
  Move(),
  Dot(Point, Radius, Option<Caption<'a>>),
  Arrow(ObjectEdge, Option<Movement>, ObjectEdge, Option<Caption<'a>>),
  Line(Point, Option<Movement>, Point, Option<Caption<'a>>),
  Rectangle(Color, Option<Paragraph<'a>>, Radius, Color, Option<EdgeMovement>),
  Circle(Color, Option<Paragraph<'a>>, Color, Option<EdgeMovement>),
  Ellipse(Color, Option<Paragraph<'a>>, Color, Option<EdgeMovement>),
  Cylinder(Color, Option<Paragraph<'a>>, Color, Option<EdgeMovement>),
  Oval(Color, Option<Paragraph<'a>>, Color, Option<EdgeMovement>),
  Font(Font),
  Text(Paragraph<'a>, Option<EdgeMovement>),
  File(Color, Option<Paragraph<'a>>, Length, Color, Option<(Edge, Vec<Movement>, ObjectEdge)>),
}

#[derive(Debug, PartialEq)]
pub struct Paragraph<'a> {
  pub text: &'a str,
  pub widths: Vec<f32>,
  pub height: f32,
  pub size: Size,
}

#[derive(Debug, PartialEq)]
pub struct Caption<'a> {
  pub text: &'a str,
  pub edge: Edge,
  pub size: Size,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Config {
  pub(crate) flow: Flow,
  pub(crate) unit: Unit,
  pub(crate) width: f32,
  pub(crate) height: f32,
  pub(crate) line: Length,
  pub(crate) dot: Length,
  pub(crate) circle: ShapeConfig,
  pub(crate) ellipse: ShapeConfig,
  pub(crate) oval: ShapeConfig,
  pub(crate) rectangle: ShapeConfig,
  pub(crate) file: ShapeConfig,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ShapeConfig {
  pub(crate) padding: f32,
  pub(crate) width: f32,
  pub(crate) height: f32,
}

impl Default for ShapeConfig {
  fn default() -> Self {
    Self { padding: BLOCK_PADDING, width: WIDTH.trunc(), height: HEIGHT.trunc() }
  }
}

impl Config {
  pub fn new(flow: Flow, width: f32, height: f32) -> Self {
    Self {
      flow,
      width,
      height,
      dot: Radius::new(4., Unit::Px),
      line: Length::new(2., Unit::Cm),
      unit: Unit::default(),
      circle: ShapeConfig::default(),
      ellipse: ShapeConfig::default(),
      oval: ShapeConfig::default(),
      rectangle: ShapeConfig::default(),
      file: ShapeConfig {
        padding: 0.0,
        width: width * 0.666,
        height: width,
      },
    }
  }
}

#[derive(Debug, Default, PartialEq)]
pub enum Direction {
  #[default]
  S,
  EN,
  E,
  SW,
}

impl From<&str> for Direction {
  fn from(item: &str) -> Self {
    match item {
      "top" => Direction::EN,
      "right" => Direction::E,
      "down" => Direction::S,
      "left" => Direction::SW,
      _ => panic!("Unknown direction {}", item)
    }
  }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Flow {
  pub(crate) start: Edge,
  pub(crate) end: Edge,
}

impl Flow {
  pub fn new(dir: impl Into<Direction>) -> Self {
    match dir.into() {
      Direction::EN => Flow::start("en"),
      Direction::E => Flow::start("e"),
      Direction::S => Flow::start("s"),
      _ => Flow::start("sw"),
    }
  }

  pub fn start(start: impl Into<Edge>) -> Self {
    let end = start.into();
    Self { start: end.flip(), end }
  }
}

#[derive(Copy, Clone, Debug, Default, PartialEq)]
pub enum EdgeDirection {
  #[default]
  Horizontal,
  Vertical,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Edge {
  pub direction: EdgeDirection,
  pub x: f32,
  pub y: f32,
}

// first character determines the direction (horizontal or vertical)
impl From<&str> for Edge {
  fn from(item: &str) -> Self {
    let dot_removed = item.trim_start_matches('.');
    match dot_removed.to_lowercase().as_str() {
      "n" | "up" | "above" => Self { direction: Vertical, x: 0., y: -0.5 },
      "ne" => Self { direction: Vertical, x: 0.5, y: -0.5 },
      "en" => Self { direction: Horizontal, x: 0.5, y: -0.5 },
      "e" | "right" => Self { direction: Horizontal, x: 0.5, y: 0. },
      "se" => Self { direction: Vertical, x: 0.5, y: 0.5 },
      "s" | "down" | "below" => Self { direction: Vertical, x: 0., y: 0.5 },
      "sw" => Self { direction: Vertical, x: -0.5, y: 0.5 },
      "w" | "left" => Self { direction: Horizontal, x: -0.5, y: 0. },
      "nw" => Self { direction: Vertical, x: -0.5, y: -0.5 },
      "wn" => Self { direction: Horizontal, x: -0.5, y: -0.5 },
      _ => Self { direction: Horizontal, x: 0., y: 0. }
    }
  }
}

impl From<f32> for Edge {
  fn from(degrees: f32) -> Self {
    let x = x_from_degrees(degrees) / 2.;
    let y = y_from_degrees(degrees) / -2.; // TODO: check with tests/types.rs:80
    let direction = if x == 1. {
      Horizontal
    } else {
      Vertical
    };
    Self { direction, x, y }
  }
}


impl Edge {

  pub fn flip(&self) -> Self {
    match self.direction {
      Horizontal => Self { direction: Horizontal, x: self.x * -1., y: self.y },
      Vertical => Self { direction: Vertical, x: self.x, y: self.y * -1. }
    }
  }

  pub fn mirror(&self) -> Self {
    match self.direction {
      Horizontal => Self { direction: Horizontal, x: self.x * -1., y: self.y * -1. },
      Vertical => Self { direction: Vertical, x: self.x * -1., y: self.y * -1. },
    }
  }
  pub fn tuple(&self) -> (f32, f32) {
    (self.x, self.y)
  }

  pub fn vector(&self) -> Vector {
    Vector::new(self.x, self.y).mul(2.0)
  }

  pub fn edge_point(&self, rect: &Rect) -> Point {
    let mut point = rect.center();
    point.offset((self.x * rect.width(), self.y * rect.height()));
    point
  }

  /// Returns the offset to adjust top-left corner with
  pub fn topleft_offset(&self, rect: &Rect) -> Point {
    let point = Point::new(self.x, self.y);
    let point = point.add(Vector::new(0.5, 0.5));
    Point::new(-point.x * rect.width(), -point.y * rect.height())
  }

  // adjust the top-left corner of the rectangle so that is at the edge
  pub fn offset(&self, rect: &mut Rect) {
    let offset = self.topleft_offset(rect);
    rect.offset(offset);
  }

  pub fn horizontal(&self) -> bool {
    self.y == 0.
  }

  pub fn vertical(&self) -> bool {
    self.x == 0.
  }
}

#[derive(Clone, Debug, Default, PartialEq)]
pub enum Unit {
  Pt,
  Pc,
  Cm,
  In,
  #[default]
  Px,
}

impl From<&str> for Unit {
  fn from(item: &str) -> Self {
    match item {
      "cm" => Unit::Cm,
      "in" => Unit::In,
      "pc" => Unit::Pc,
      "pt" => Unit::Pt,
      "px" => Unit::Px,
      _ => panic!("unknown unit {}", item)
    }
  }
}

pub type Radius = Length;

#[derive(Clone, Debug, Default, PartialEq)]
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
      Unit::In => self.length * 118.,
      Unit::Pc => self.length * 16.,
      Unit::Pt => self.length * 1.3333,
      _ => self.length * 1.,
    }
  }
}

#[derive(Debug, Default, PartialEq)]
pub struct Movement {
  pub(crate) length: Length,
  pub edge: Edge,
}

impl Movement {
  pub fn new(length: f32, unit: Unit, edge: Edge) -> Self {
    let length = Length::new(length, unit);
    Self { length, edge }
  }

  pub fn offset(&self) -> Point {
    self.edge.vector().mul(self.length.pixels())
  }

  pub fn is_horizontal(&self) -> bool {
    self.edge.direction == Horizontal
  }

  pub fn is_vertical(&self) -> bool {
    self.edge.direction == Vertical
  }
}

#[derive(Debug, PartialEq)]
pub struct ObjectEdge {
  pub(crate) id: String,
  pub(crate) edge: Edge,
}

impl ObjectEdge {
  pub fn new(id: &str, edge: impl Into<Edge>) -> Self {
    Self { id: id.into(), edge: edge.into() }
  }
}
