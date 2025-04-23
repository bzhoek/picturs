#[cfg(test)]
mod edge {
  use skia_safe::{Point, Rect};
  use crate::diagram::types::Edge;
  use crate::diagram::types::EdgeDirection::{Horizontal, Vertical};

  #[test]
  fn degrees() {
    let edge = Edge::from(90.);
    assert_eq!(edge.direction, Horizontal);

    let edge = Edge::from(360.);
    assert_eq!(edge.direction, Vertical);

    let edge = Edge::from(0.);
    assert_eq!(edge.direction, Vertical);
  }

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
}

#[cfg(test)]
mod endings {
  use crate::diagram::types::{Ending, Endings};

  #[test]
  fn ending() {
    assert_eq!(Ending::from("<"), Ending::Arrow);
  }

  #[test]
  fn endings() {
    assert_eq!(Endings::from("<->"), Endings { start: Ending::Arrow, end: Ending::Arrow });
    assert_eq!(Endings::from("<-"), Endings { start: Ending::Arrow, end: Ending::None });
    assert_eq!(Endings::from("->"), Endings { start: Ending::None, end: Ending::Arrow });
  }
}
