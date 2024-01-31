use std::collections::HashMap;
use std::ops::Add;
use log::error;
use skia_safe::{Point, Rect};
use crate::diagram::types::{Displacement, Edge, ObjectEdge};

#[derive(Debug)]
pub enum ShapeName {
  Rectangle,
  Container,
  Circle,
  Text,
}

#[derive(Debug, Default)]
pub struct Index {
  ids: HashMap<String, Rect>,
  shapes: Vec<(ShapeName, Rect)>,
}

impl Index {
  pub fn insert(&mut self, name: ShapeName, id: Option<&str>, rect: Rect) {
    if let Some(id) = id {
      self.ids.insert(id.into(), rect);
    }
    self.shapes.push((name, rect));
  }

  pub fn position_rect(&self, location: &Option<(Edge, Vec<Displacement>, ObjectEdge)>, used: &mut Rect) {
    if let Some((edge, distances, object)) = &location {
      if let Some(rect) = self.offset_index(object, distances) {
        *used = Rect::from_xywh(rect.left, rect.top, used.width(), used.height());
        let offset = edge.topleft_offset(used);
        used.offset(offset);
      }
    }
  }

  fn offset_index(&self, edge: &ObjectEdge, distances: &[Displacement]) -> Option<Rect> {
    self.ids.get(&edge.id).map(|rect| {
      Self::offset_from_rect(rect, &edge.edge, distances)
    })
  }

  pub fn offset_from_rect(rect: &Rect, edge: &Edge, distances: &[Displacement]) -> Rect {
    let point = edge.edge_point(rect);
    let mut rect = Rect::from_xywh(point.x, point.y, rect.width(), rect.height());
    Self::offset_rect(&mut rect, distances);
    rect
  }

  pub fn offset_rect(rect: &mut Rect, distances: &[Displacement]) {
    for distance in distances.iter() {
      rect.offset(distance.offset());
    }
  }

  pub fn point_index(&self, edge: &ObjectEdge, distances: &[Displacement]) -> Option<Point> {
    self.ids.get(&edge.id).map(|rect| {
      Self::point_from_rect(rect, &edge.edge, distances)
    }).or_else(|| {
      error!("{} edge id not found", edge.id);
      None
    })
  }

  pub fn point_from_rect(rect: &Rect, edge: &Edge, distances: &[Displacement]) -> Point {
    let point = edge.edge_point(rect);
    for distance in distances.iter() {
      let _ = point.add(distance.offset());
    }
    point
  }
}