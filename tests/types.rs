mod common;

#[cfg(test)]
mod tests {
  use skia_safe::{Point, Rect};

  use picturs::diagram::types::Edge;

  #[test]
  fn to_top_left() {
    let rect = Rect::from_xywh(40., 40., 10., 20.);

    let edge = Edge::from("nw");
    assert_eq!((-0.5, -0.5), edge.tuple());
    let nw = edge.topleft_offset(&rect);
    assert_eq!(Point::new(-0., -0.), nw);

    let edge = Edge::from("ne");
    assert_eq!((0.5, -0.5), edge.tuple());
    let ne = edge.topleft_offset(&rect);
    assert_eq!(Point::new(-10., -0.), ne);

    let edge = Edge::from("se");
    assert_eq!((0.5, 0.5), edge.tuple());
    let se = edge.topleft_offset(&rect);
    assert_eq!(Point::new(-10., -20.), se);

    let edge = Edge::from("c");
    assert_eq!((0., 0.), edge.tuple());
    let se = edge.topleft_offset(&rect);
    assert_eq!(Point::new(-5., -10.), se);
  }
}