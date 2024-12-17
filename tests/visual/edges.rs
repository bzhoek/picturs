#[cfg(test)]
mod edges {
  use picturs::assert_diagram;

  use picturs::diagram::create_diagram;

  #[test]
  fn edge_top_right() {
    let string = r#"
      top
      box.a "A" wd 1in
      text "Align top"
      box.b "B" wd 2in
      arrow from a.ne to b.nw
      dot at a.ne color red rad 4pt
      dot at b.nw color green rad 4pt
      "#;
    let diagram = create_diagram(string);
    assert_diagram!(diagram);
  }

  #[test]
  fn edge_bottom_left() {
    let string = r#"
      continue down-left
      box.a "A" wd 1in
      text "Default align left"
      box.b "B" wd 2in
      arrow from a.sw to b.nw
      "#;
    let diagram = create_diagram(string);
    assert_diagram!(diagram);
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
      arrow from a.s to b.n "Caption left" left
      line from b.s to c.n "Caption right" right
      "#;
    let diagram = create_diagram(string);
    assert_diagram!(diagram);
  }
}