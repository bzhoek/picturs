#[cfg(test)]
mod edges {
  use picturs::assert_diagram;

  #[test]
  fn edge_top_right() {
    let string = r#"
      top
      box.a "A" wd 1in
      text "Align top"
      box.b "B" wd 2in
      arrow from a.ne end b.nw
      dot at a.ne color red rad 4pt
      dot at b.nw color green rad 4pt
      "#;
    assert_diagram!(string);
  }

  #[test]
  fn edge_bottom_left() {
    let string = r#"
      continue down-left
      box.a "A" wd 1in
      text "Default align left"
      box.b "B" wd 2in
      arrow from a.sw end b.nw
      "#;
    assert_diagram!(string);
  }

  #[test]
  fn edge_down_center() {
    let string = r#"
      down
      box.a "A" wd 1in
      flow 2cm
      box.b "B" wd 2in
      flow 2cm
      box.c "C" wd 3in
      arrow from a.s end b.n "Caption left" left
      line from b.s end c.n "Caption right" right
      "#;
    assert_diagram!(string);
  }
}