use std::collections::HashMap;
use std::ops::Add;

use log::error;
use skia_safe::{Point, Rect};

use crate::diagram::attributes::{Attributes, OpenAttributes};
use crate::diagram::types::{Displacement, Edge, Movement, ObjectEdge};

#[derive(Debug, Clone, PartialEq)]
pub enum ShapeName {
  Box,
  Dot,
  Container,
  Circle,
  Ellipse,
  Cylinder,
  Text,
  Oval,
  File,
  Arrow,
  Line,
  Path,
}

impl ShapeName {
  pub fn some(name: &str) -> Option<Self> {
    match name {
      "box" => Some(ShapeName::Box),
      "container" => Some(ShapeName::Container),
      "ellipse" => Some(ShapeName::Ellipse),
      "cylinder" => Some(ShapeName::Cylinder),
      "circle" => Some(ShapeName::Circle),
      "text" => Some(ShapeName::Text),
      "oval" => Some(ShapeName::Oval),
      _ => None
    }
  }
}

impl From<&str> for ShapeName {
  fn from(name: &str) -> Self {
    Self::some(name).unwrap_or_else(|| panic!("unknown shape {}", name))
  }
}

#[derive(Debug, Default)]
pub struct Index<'i> {
  ids: HashMap<String, Rect>,
  shapes: Vec<(ShapeName, Rect)>,
  open: Vec<(ShapeName, Attributes<'i>)>,
  closed: Vec<(ShapeName, Attributes<'i>)>,
}

impl<'a> Index<'a> {

  pub fn add(&mut self, name: ShapeName, attrs: Attributes<'a>, rect: Rect) {
    let id = self.insert_type(&name, attrs);
    self.insert_shape(name, id, rect);
  }

  fn insert_type(&mut self, name: &ShapeName, attrs: Attributes<'a>) -> Option<&'a str> {
    match attrs {
      Attributes::Closed { id, .. } => {
        self.closed.push((name.clone(), attrs));
        id
      }
      Attributes::Open { id, .. } => {
        self.open.push((name.clone(), attrs));
        id
      }
    }
  }

  pub fn insert_shape(&mut self, name: ShapeName, id: Option<&str>, rect: Rect) {
    if let Some(id) = id {
      self.ids.insert(id.into(), rect);
    }
    self.shapes.push((name, rect));
  }

  pub(crate) fn last_open(&self, shape: ShapeName) -> Option<&(ShapeName, Attributes<'_>)> {
    Self::last_shape(shape, &self.open)
  }

  pub(crate) fn last_closed(&self, shape: ShapeName) -> Option<&(ShapeName, Attributes<'_>)> {
    Self::last_shape(shape, &self.closed)
  }

  fn last_shape<'b>(shape: ShapeName, vec: &'b Vec<(ShapeName, Attributes<'b>)>) -> Option<&'b (ShapeName, Attributes<'b>)> {
    vec.iter().filter(|(name, _)| {
      shape == *name
    }).next_back()
  }

  /// modify a rectangle by any edge and displacements
  pub fn position_rect(&self, location: &Option<(Edge, Vec<Displacement>, ObjectEdge)>, used: &mut Rect) {
    if let Some((edge, movements, object)) = &location {
      if let Some(rect) = self.offset_index(object, movements) {
        *used = Rect::from_xywh(rect.left, rect.top, used.width(), used.height());
        edge.offset(used);
      }
    }
  }

  fn offset_index(&self, object: &ObjectEdge, movements: &[Displacement]) -> Option<Rect> {
    match &*object.id {
      "#last" => self.shapes.last().map(|(_shape, rect)| rect),
      id if ShapeName::some(id).is_some() => self.last(ShapeName::some(id).unwrap()).map(|(_shape, rect)| rect),
      id => self.ids.get(id)
    }.map(|rect| {
      Self::offset_from_rect(rect, &object.edge, movements)
    })
  }

  fn last(&self, shape: ShapeName) -> Option<&(ShapeName, Rect)> {
    self.shapes.iter().filter(|(name, _)| {
      shape == *name
    }).next_back()
  }

  pub fn offset_from_rect(rect: &Rect, edge: &Edge, movements: &[Displacement]) -> Rect {
    let point = edge.edge_point(rect);
    let mut rect = Rect::from_xywh(point.x, point.y, rect.width(), rect.height());
    Self::offset_rect(&mut rect, movements);
    rect
  }

  pub fn offset_rect(rect: &mut Rect, movements: &[Displacement]) {
    for movement in movements.iter() {
      rect.offset(movement.offset());
    }
  }

  pub fn point_index(&self, edge: Option<&ObjectEdge>, movements: &[Displacement]) -> Option<Point> {
    edge.and_then(|edge| {
      self.ids.get(&edge.id).map(|rect| {
        Self::point_from_rect(rect, &edge.edge, movements)
      }).or_else(|| {
        error!("{} edge id not found", edge.id);
        None
      })
    })
  }

  pub fn points_from(&self, start: &Point, source: &Option<ObjectEdge>, movement: &Option<Displacement>, target: &Option<ObjectEdge>, route: bool) -> Vec<Point> {
    let mut movements = vec!();
    let mut points = vec!();
    if let Some(object) = source {
      movements.push(Movement::ObjectStart { object: object.clone() })
    } else {
      points.push(*start);
    }
    if let Some(movement) = movement {
      movements.push(Movement::Relative { displacement: movement.clone() })
    }
    if let Some(object) = target {
      movements.push(Movement::ObjectEnd { object: object.clone() })
    }
    self.add_movements_as_points(start, &movements, route, &mut points);
    points
  }

  /// add points from movements to a vector
  pub fn add_movements_as_points(&self, start: &Point, movements: &[Movement], route: bool, points: &mut Vec<Point>) {
    let mut last = *start;
    for movement in movements.iter() {
      match movement {
        Movement::ObjectEnd { object } => {
          let terminal = self.point_from(object).unwrap_or_else(|| panic!("Index to have {:?}", object));
          if route {
            if let Some(point) = Self::straighten_point(last, terminal, object.edge.vertical()) {
              points.push(point);
            }
          }
          last = terminal;
        }
        Movement::ObjectStart { object } => {
          last = self.point_from(object).unwrap_or_else(|| panic!("Index to have {:?}", object));
        }
        Movement::Relative { displacement: movement } => {
          last = last.add(movement.offset());
        }
      }
      points.push(last);
    }
  }

  fn straighten_point(last: Point, current: Point, vertical: bool) -> Option<Point> {
    if vertical && current.x != last.x {
      Point::new(current.x, last.y).into()
    } else if current.y != last.y {
      Point::new(last.x, current.y).into()
    } else {
      None
    }
  }

  /// return points from movements relative to a start point
  pub fn points_from_movements(&self, start: &Point, movements: &[Movement]) -> Vec<Point> {
    let mut point = *start;
    let points = movements.iter().map(|movement| {
      match movement {
        Movement::Relative { displacement: movement } => {
          point = point.add(movement.offset());
          point
        }
        Movement::ObjectStart { object } => {
          point = self.point_from(object).unwrap_or_else(|| panic!("Index to have {:?}", object));
          point
        }
        Movement::ObjectEnd { object } => {
          point = self.point_from(object).unwrap_or_else(|| panic!("Index to have {:?}", object));
          point
        }
      }
    }).collect::<Vec<_>>();
    vec![*start].into_iter().chain(points).collect()
  }

  pub fn point_from(&self, edge: &ObjectEdge) -> Option<Point> {
    let rect = self.ids.get(&edge.id);
    rect.map(|rect| {
      Self::point_from_rect(rect, &edge.edge, &[])
    })
  }

  pub fn point_from_rect(rect: &Rect, edge: &Edge, displacements: &[Displacement]) -> Point {
    let point = edge.edge_point(rect);
    for displacement in displacements.iter() {
      let _ = point.add(displacement.offset());
    }
    point
  }

  pub(crate) fn copy_open_attributes(&self, attrs: &mut OpenAttributes, shape: ShapeName) {
    if !attrs.same {
      return;
    }
    if let Some((_, Attributes::Open { endings, .. })) = self.last_open(shape) {
      attrs.endings = endings.clone();
    }
  }
}