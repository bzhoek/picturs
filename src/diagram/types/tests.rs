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

  #[test]
  fn place_in_top_right() {
    let rect = Rect { left: 10., top: 20.0, right: 100.0, bottom: 200.0 };
    let caption = Rect { left: 0.0, top: 0.0, right: 40.0, bottom: 20.0 };

    let to_edge = Edge::from("ne");

    let rect_ne = to_edge.edge_point(&rect);
    assert_eq!(Point::new(100., 20.), rect_ne);

    let center = caption.center();
    let delta = rect_ne - center;
    let moved = center + delta;
    assert_eq!(Point::new(100., 20.), moved);
    let moved_caption = caption.with_offset(delta);
    assert_eq!(Rect::new(80., 10., 120., 30.), moved_caption);

    let at_edge = Edge::from("ne");
    let caption_ne = at_edge.edge_delta(&caption);
    assert_eq!(Point::new(-20., 10.), caption_ne);

    let final_caption = moved_caption.with_offset(caption_ne);
    assert_eq!(Rect::new(60., 20., 100., 40.), final_caption);
  }

  #[test]
  fn place_in_bottom_left() {
    let rect = Rect { left: 10., top: 20.0, right: 100.0, bottom: 200.0 };
    let caption = Rect { left: 0.0, top: 0.0, right: 40.0, bottom: 20.0 };

    let to_edge = Edge::from("sw");

    let rect_ne = to_edge.edge_point(&rect);
    assert_eq!(Point::new(10., 200.), rect_ne);

    let center = caption.center();
    let delta = rect_ne - center;
    let moved = center + delta;
    assert_eq!(Point::new(10., 200.), moved);
    let moved_caption = caption.with_offset(delta);
    assert_eq!(Rect::new(-10., 190., 30., 210.), moved_caption);

    let at_edge = Edge::from("sw");
    let caption_ne = at_edge.edge_delta(&caption);
    assert_eq!(Point::new(20., -10.), caption_ne);

    let final_caption = moved_caption.with_offset(caption_ne);
    assert_eq!(Rect::new(10., 180., 50., 200.), final_caption);
  }

  #[test]
  fn place_outside_bottom_center() {
    let rect = Rect { left: 10., top: 20.0, right: 100.0, bottom: 200.0 };
    let caption = Rect { left: 0.0, top: 0.0, right: 40.0, bottom: 20.0 };

    let to_edge = Edge::from("s");

    let rect_ne = to_edge.edge_point(&rect);
    assert_eq!(Point::new(55., 200.), rect_ne);

    let center = caption.center();
    let delta = rect_ne - center;
    let moved = center + delta;
    assert_eq!(Point::new(55., 200.), moved);
    let moved_caption = caption.with_offset(delta);
    assert_eq!(Rect::new(35., 190., 75., 210.), moved_caption);

    let at_edge = Edge::from("n");
    let caption_ne = at_edge.edge_delta(&caption);
    assert_eq!(Point::new(0., 10.), caption_ne);

    let final_caption = moved_caption.with_offset(caption_ne);
    assert_eq!(Rect::new(35., 200., 75., 220.), final_caption);
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
