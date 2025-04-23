#[cfg(test)]
mod tests;

use std::fmt::Display;
use std::ops::{Add, Mul};

use crate::diagram::attributes::{Attributes, EdgeMovement};
use skia_safe::{scalar, Color, Font, FontMgr, FontStyle, Point, Rect, Size, Vector};

use crate::diagram::conversion::{HEIGHT, WIDTH};
use crate::diagram::types::EdgeDirection::{Horizontal, Vertical};
use crate::trig::{x_from_degrees, y_from_degrees};

pub const BLOCK_PADDING: f32 = 8.;

pub type Used = Rect;
pub type Thickness = f32;

pub struct R(pub Rect);

impl Display for R {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "({:>6.1},{:>6.1})-({:>6.1},{:>6.1})", self.0.left, self.0.top, self.0.right, self.0.bottom)
  }
}
#[derive(Debug, PartialEq)]
pub struct CommonAttributes<'a> {
  pub(crate) id: Option<&'a str>,
  pub used: Rect,
  pub(crate) stroke: Color,
  pub(crate) thickness: f32,
}

impl<'a> CommonAttributes<'a> {
  pub(crate) fn new(id: Option<&'a str>, used: Rect, stroke: Color, thickness: f32) -> Self {
    Self {
      id,
      used,
      stroke,
      thickness,
    }
  }
}

#[allow(clippy::large_enum_variant)]
#[derive(Debug, PartialEq)]
pub enum Node<'a> {
  Container(Attributes<'a>, Used, Vec<Node<'a>>),
  Primitive(CommonAttributes<'a>, Shape),
  Closed(Attributes<'a>, Used, Option<Paragraph>, Shape),
  Open(Attributes<'a>, Used, Shape),
  Font(Font),
  Move(Rect),
}

pub type Radius = f32;

#[derive(Debug, PartialEq)]
pub enum Shape {
  Circle,
  Ellipse,
  Oval,
  Rectangle,
  Cylinder,
  File,
  Text(Paragraph, Option<EdgeMovement>),

  Arrow(ObjectEdge, Option<Displacement>, ObjectEdge, Option<Caption>),
  Line(Vec<Point>, Option<Caption>, Endings),
  Sline(Vec<Point>, Option<Caption>, Endings),
  Path(Vec<Point>, Option<Caption>),

  Dot(Point, Radius, Option<Caption>),
}

#[derive(Debug, PartialEq)]
pub struct Paragraph {
  pub text: String,
  pub widths: Vec<f32>,
  pub height: f32,
  pub size: Size,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Caption {
  pub text: String,
  pub inner: Edge,
  pub outer: Edge,
  pub size: Size,
  pub opaque: bool,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub enum Ending {
  #[default]
  None,
  Arrow,
  Dot,
}

impl From<&str> for Ending {
  fn from(item: &str) -> Self {
    match item {
      "<" | ">" => Ending::Arrow,
      "*" => Ending::Dot,
      _ => Ending::None,
    }
  }
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Endings {
  pub start: Ending,
  pub end: Ending,
}

impl From<&str> for Endings {
  fn from(str: &str) -> Self {
    let mut chars = str.chars();
    let start = match chars.next().unwrap() {
      '<' => Ending::Arrow,
      '*' => Ending::Dot,
      _ => Ending::None,
    };
    let end = match chars.last().unwrap() {
      '>' => Ending::Arrow,
      '*' => Ending::Dot,
      _ => Ending::None,
    };
    Self {
      start,
      end,
    }
  }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Config {
  pub(crate) container: ShapeConfig,
  pub(crate) group: ShapeConfig,
  pub(crate) continuation: Continuation,
  pub(crate) unit: Unit,
  pub(crate) line: Length,
  pub(crate) circle: ShapeConfig,
  pub(crate) dot: Length,
  pub(crate) ellipse: ShapeConfig,
  pub(crate) oval: ShapeConfig,
  pub(crate) rectangle: ShapeConfig,
  pub(crate) text: ShapeConfig,
  pub(crate) file: ShapeConfig,
  pub(crate) cylinder: ShapeConfig,
  pub(crate) font: Font,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ShapeConfig {
  // all in pixels
  pub(crate) padding: f32,
  pub(crate) width: f32,
  pub(crate) height: f32,
  pub(crate) radius: f32,
  pub(crate) space: f32,
  pub(crate) stroke: Color,
}

impl Default for ShapeConfig {
  fn default() -> Self {
    Self {
      padding: 0.,
      width: WIDTH.trunc(),
      height: HEIGHT.trunc(),
      radius: 0.,
      space: 0.,
      stroke: Color::BLUE,
    }
  }
}

impl ShapeConfig {
  pub fn stroke(color: Color) -> Self {
    Self {
      padding: 0.,
      width: WIDTH.trunc(),
      height: HEIGHT.trunc(),
      radius: 0.,
      space: 0.,
      stroke: color,
    }
  }
}

impl Default for Config {
  fn default() -> Self {
    Self::new(Continuation::new("right"))
  }
}

impl Config {
  pub fn new(flow: Continuation) -> Self {
    let typeface = FontMgr::default().match_family_style("Helvetica", FontStyle::default()).unwrap();
    let font = Font::from_typeface(typeface, 17.0);
    Self {
      container: ShapeConfig::stroke(Color::RED),
      group: ShapeConfig::stroke(Color::TRANSPARENT),
      continuation: flow,
      dot: Length::new(4., Unit::Px),
      circle: ShapeConfig::default(),
      cylinder: ShapeConfig::default(),
      ellipse: ShapeConfig::default(),
      line: Length::new(2., Unit::Cm),
      oval: ShapeConfig::default(),
      rectangle: ShapeConfig::default(),
      text: ShapeConfig::default(),
      unit: Unit::default(),
      file: ShapeConfig {
        padding: 8.0,
        width: HEIGHT.trunc(),
        height: WIDTH.trunc(),
        radius: 8.0,
        space: 0.0,
        stroke: Color::BLUE,
      },
      font,
    }
  }

  pub fn measure_string(&self, str: &str) -> Size {
    let (width_with_whitespace, _bounds) = self.font.measure_str(str, None);
    let (font_height, _font_metrics) = self.font.metrics();
    // _bounds.size()
    Size::new(width_with_whitespace.ceil(), font_height.ceil())
  }

  pub fn measure_strings(&self, text: &str, width: f32) -> (Vec<scalar>, scalar) {
    let (font_height, _font_metrics) = self.font.metrics();
    let advance = font_height / 4.;

    let (mut x, mut y) = (0.0, font_height);
    let mut widths: Vec<scalar> = vec![];

    for word in text.split_whitespace() {
      let (word_width, _word_rect) = self.font.measure_str(word, None);
      if x + word_width > width {
        y += font_height;
        widths.push(x.ceil());
        x = 0.;
      }
      x += word_width + advance;
    }

    widths.push(x.ceil());
    (widths, y)
  }
}

/// A continuation is a pair of edges that are connected
#[derive(Clone, Debug, PartialEq)]
pub struct Continuation {
  pub(crate) direction: EdgeDirection,
  pub(crate) start: Edge,
  pub(crate) end: Edge,
}

impl Continuation {
  pub fn new(name: &str) -> Self {
    match name {
      "right-top" | "top" => Continuation::start("en", Horizontal),
      "right" => Continuation::start("e", Horizontal),
      "down" => Continuation::start("s", Vertical),
      "down-left" | "left" => Continuation::start("sw", Vertical),
      _ => panic!("Unknown direction {}", name),
    }
  }

  pub fn start(start: impl Into<Edge>, direction: EdgeDirection) -> Self {
    let end = start.into();
    Self {
      direction,
      start: end.flip(),
      end,
    }
  }
}

#[derive(Copy, Clone, Debug, Default, PartialEq)]
pub enum EdgeDirection {
  #[default]
  Horizontal,
  Vertical,
}

// Transform factors from center to edge
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
      "n" | "up" | "above" => Self::above(),
      "ne" => Self {
        direction: Vertical,
        x: 0.5,
        y: -0.5,
      },
      "en" | "right-top" => Self {
        direction: Horizontal,
        x: 0.5,
        y: -0.5,
      },
      "e" | "right" => Self::right(),
      "se" => Self {
        direction: Vertical,
        x: 0.5,
        y: 0.5,
      },
      "s" | "down" | "below" => Self::below(),
      "sw" => Self {
        direction: Vertical,
        x: -0.5,
        y: 0.5,
      },
      "w" | "left" => Self::left(),
      "nw" => Self {
        direction: Vertical,
        x: -0.5,
        y: -0.5,
      },
      "wn" => Self {
        direction: Horizontal,
        x: -0.5,
        y: -0.5,
      },
      _ => Self::center(),
    }
  }
}

impl From<f32> for Edge {
  fn from(degrees: f32) -> Self {
    let x = x_from_degrees(degrees) / 2.;
    let y = y_from_degrees(degrees) / -2.; // TODO: check with tests/types.rs:80
    let direction = if y.abs() == 0. {
      Horizontal
    } else {
      Vertical
    };
    Self {
      direction,
      x,
      y,
    }
  }
}

impl Edge {
  pub fn above() -> Self {
    Self {
      direction: Vertical,
      x: 0.,
      y: -0.5,
    }
  }

  pub fn below() -> Self {
    Self {
      direction: Vertical,
      x: 0.,
      y: 0.5,
    }
  }

  pub fn left() -> Self {
    Self {
      direction: Horizontal,
      x: -0.5,
      y: 0.,
    }
  }

  pub fn right() -> Self {
    Self {
      direction: Horizontal,
      x: 0.5,
      y: 0.,
    }
  }

  pub fn center() -> Self {
    Self {
      direction: Horizontal,
      x: 0.,
      y: 0.,
    }
  }

  pub fn flip(&self) -> Self {
    match self.direction {
      Horizontal => Self {
        direction: Horizontal,
        x: self.x * -1.,
        y: self.y,
      },
      Vertical => Self {
        direction: Vertical,
        x: self.x,
        y: self.y * -1.,
      },
    }
  }

  pub fn mirror(&self) -> Self {
    match self.direction {
      Horizontal => Self {
        direction: Horizontal,
        x: self.x * -1.,
        y: self.y * -1.,
      },
      Vertical => Self {
        direction: Vertical,
        x: self.x * -1.,
        y: self.y * -1.,
      },
    }
  }
  pub fn tuple(&self) -> (f32, f32) {
    (self.x, self.y)
  }

  pub fn vector(&self) -> Vector {
    Vector::new(self.x, self.y).mul(2.0)
  }

  /// Return the absolute edge point of the `rect`
  pub fn edge_point(&self, rect: &Rect) -> Point {
    let mut point = rect.center();
    point.offset((self.x * rect.width(), self.y * rect.height()));
    point
  }

  /// Returns the delta from the center of `rect`
  pub fn edge_delta(&self, rect: &Rect) -> Point {
    let delta = Point::new(-self.x * rect.width(), -self.y * rect.height());
    delta
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

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub enum Unit {
  Pt,
  Pc,
  Cm,
  In,
  #[default]
  Px,
  Unit,
}

impl From<&str> for Unit {
  fn from(item: &str) -> Self {
    match item {
      "cm" => Unit::Cm,
      "in" => Unit::In,
      "pc" => Unit::Pc,
      "pt" => Unit::Pt,
      "px" => Unit::Px,
      "u" => Unit::Unit,
      _ => panic!("unknown unit {}", item),
    }
  }
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Length {
  length: f32,
  unit: Unit,
}

impl Length {
  pub fn new(length: f32, unit: Unit) -> Self {
    Self {
      length,
      unit,
    }
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

  pub fn points(&self) -> f32 {
    match self.unit {
      Unit::Cm => self.length / 28.3465,
      Unit::In => self.length / 72.,
      Unit::Pc => self.length / 12.,
      Unit::Px => self.length / 1.3333,
      _ => self.length * 1.,
    }
  }
}

#[derive(Clone, Debug, PartialEq)]
pub enum Movement {
  Relative {
    displacement: Displacement,
  },
  ObjectStart {
    object: ObjectEdge,
  },
  ObjectEnd {
    object: ObjectEdge,
  },
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Displacement {
  pub(crate) length: Length,
  pub edge: Edge,
}

impl Displacement {
  pub fn new(length: f32, unit: Unit, edge: Edge) -> Self {
    let length = Length::new(length, unit);
    Self {
      length,
      edge,
    }
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

#[derive(Clone, Debug, PartialEq)]
pub struct ObjectEdge {
  pub(crate) id: String,
  pub(crate) edge: Edge,
}

impl ObjectEdge {
  pub fn new(id: &str, edge: impl Into<Edge>) -> Self {
    Self {
      id: id.into(),
      edge: edge.into(),
    }
  }
}
