use std::collections::HashMap;
use std::ops::Add;

use log::error;
use skia_safe::{Point, Rect};

use crate::diagram::attributes::Attributes;
use crate::diagram::types::{Displacement, Edge, Movement, ObjectEdge};

#[derive(Debug, PartialEq)]
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
}

impl<'a> Index<'a> {
  pub fn insert(&mut self, name: ShapeName, id: Option<&str>, rect: Rect) {
    if let Some(id) = id {
      self.ids.insert(id.into(), rect);
    }
    self.shapes.push((name, rect));
  }

  pub(crate) fn add_open(&mut self, name: ShapeName, attrs: Attributes<'a>) {
    self.open.push((name, attrs));
  }

  pub(crate) fn last_open(&self, shape: ShapeName) -> Option<&(ShapeName, Attributes)> {
    self.open.iter().filter(|(name, _)| {
      shape == *name
    }).last()
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
    }).last()
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

  pub fn points_from_movements(&self, cursor: &Point, movements: &[Movement]) -> Vec<Point> {
    let mut point = *cursor;
    movements.iter().map(|movement| {
      match movement {
        Movement::Relative { displacement: movement } => {
          point = point.add(movement.offset());
          point
        }
        Movement::Absolute { object } => {
          point = self.point_from(object).unwrap_or_else(|| panic!("Index to have {:?}", object));
          point
        }
      }
    }).collect::<Vec<_>>()
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

}