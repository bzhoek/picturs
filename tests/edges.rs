mod common;

#[cfg(test)]
mod tests {
  use assert_matches::assert_matches;

  use picturs::diagram::types::Node::Primitive;
  use picturs::test::assert_diagram;

  use crate::common::create_diagram;

  #[test]
  fn visual_edge_top_right() -> anyhow::Result<()> {
    let string = r#"
      top
      box.a "A" wd 1in
      text "Align top"
      box.b "B" wd 2in
      arrow from a.ne to b.nw
      dot a.ne color red rad 4pt
      dot b.nw color green rad 4pt
      "#;
    let diagram = create_diagram(string);
    assert_matches!(diagram.nodes[0], Primitive(_, rect, ..) if rect.x() == 0. && rect.y() == 0.);
    assert_matches!(diagram.nodes[2], Primitive(_, rect, ..) if rect.y() == 0.);
    assert_diagram(diagram, "target/visual_edge_top_right")?;
    Ok(())
  }

  #[test]
  fn visual_edge_bottom_left() -> anyhow::Result<()> {
    let string = r#"
      box.a "A" wd 1in
      text "Default align left"
      box.b "B" wd 2in
      arrow from a.sw to b.nw
      "#;
    let diagram = create_diagram(string);
    assert_matches!(diagram.nodes[0], Primitive(_, rect, ..) if rect.x() == 0.);
    assert_matches!(diagram.nodes[2], Primitive(_, rect, ..) if rect.x() == 0.);
    assert_diagram(diagram, "target/visual_edge_bottom_left")?;
    Ok(())
  }

  #[test]
  fn visual_edge_down_center() -> anyhow::Result<()> {
    let string = r#"
      down
      box.a "A" wd 1in
      text "Align center"
      box.b "B" wd 2in
      arrow from a.s to b.n
      "#;
    let diagram = create_diagram(string);
    assert_matches!(diagram.nodes[0], Primitive(_, rect, ..) if rect.x() < 0.);
    assert_matches!(diagram.nodes[2], Primitive(_, rect, ..) if rect.x() < 0.);
    assert_diagram(diagram, "target/visual_edge_down_center")?;
    Ok(())
  }
}