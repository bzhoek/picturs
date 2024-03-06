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

mod endings {
  use crate::diagram::types::{Ending, Endings};

  #[test]
  fn ending() {
    let subject = "<->";
    let start = &subject[0..2];
    let end = &subject[1..];
    assert_eq!(start, "<-");
    assert_eq!(end, "->");
    assert_eq!(Ending::from(start), Ending::Arrow);
  }

  #[test]
  fn endings() {
    assert_eq!(Endings::from("<->"), Endings { start: Ending::Arrow, end: Ending::Arrow });
    assert_eq!(Endings::from("<-"), Endings { start: Ending::Arrow, end: Ending::None });
    assert_eq!(Endings::from("->"), Endings { start: Ending::None, end: Ending::Arrow });
  }
}
