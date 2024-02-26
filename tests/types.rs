#[cfg(test)]
mod tests {
  use skia_safe::{Point, Rect};

  use picturs::diagram::types::Edge;

  #[test]
  fn to_top_left() {
    let rect = Rect::from_xywh(40., 40., 10., 20.);

    let edge = Edge::from("nw");
    assert_eq!(edge.tuple(), (-0.5, -0.5));
    let nw = edge.topleft_offset(&rect);
    assert_eq!(nw, Point::new(-0., -0.));

    let edge = Edge::from("ne");
    assert_eq!(edge.tuple(), (0.5, -0.5));
    let ne = edge.topleft_offset(&rect);
    assert_eq!(ne, Point::new(-10., -0.));

    let edge = Edge::from("se");
    assert_eq!(edge.tuple(), (0.5, 0.5));
    let se = edge.topleft_offset(&rect);
    assert_eq!(se, Point::new(-10., -20.));

    let edge = Edge::from("c");
    assert_eq!(edge.tuple(), (0., 0.));
    let se = edge.topleft_offset(&rect);
    assert_eq!(se, Point::new(-5., -10.));
  }
}