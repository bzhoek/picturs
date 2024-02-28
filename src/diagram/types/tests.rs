use skia_safe::{Point, Rect};
use crate::diagram::types::Edge;

#[test]
fn to_edge() {
  let rect = Rect::from_xywh(40., 40., 100., 200.);

  let edge = Edge::from("nw");
  let center = rect.center();
  assert_eq!(center, Point::new(90., 140.));

  let nw = edge.edge_point(&rect);
  assert_eq!(nw, Point::new(40., 40.));

  let edge = Edge::from("se");
  let se = edge.edge_point(&rect);
  assert_eq!(se, Point::new(140., 240.));
}

#[test]
fn to_edge_from_negative() {
  let rect = Rect { left: 0.0, top: -30.0, right: 360.0, bottom: 30.0 };

  let edge = Edge::from("sw");
  let center = rect.center();
  assert_eq!(center, Point::new(180., 0.));

  let nw = edge.edge_point(&rect);
  assert_eq!(nw, Point::new(0., 30.));
}

