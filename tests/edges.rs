mod common;

#[cfg(test)]
mod tests {
  use assert_matches::assert_matches;

  use picturs::diagram::types::Node::Primitive;
  use picturs::test::assert_diagram;

  use crate::common::create_diagram;

  #[test]
  fn visual_edge_bottom_left() -> anyhow::Result<()> {
    let string = r#"
      box "A" wd 1in
      box "B" wd 2in
      "#;
    let diagram = create_diagram(string);
    assert_matches!(diagram.nodes[0], Primitive(_, rect, ..) if rect.x() == 0.);
    assert_matches!(diagram.nodes[1], Primitive(_, rect, ..) if rect.x() == 0.);
    assert_diagram(diagram, "target/visual_edge_bottom_left")?;
    Ok(())
  }

  #[test]
  fn visual_edge_down_center() -> anyhow::Result<()> {
    let string = r#"
      down
      box "A" wd 1in
      box "B" wd 2in
      "#;
    let diagram = create_diagram(string);
    assert_matches!(diagram.nodes[0], Primitive(_, rect, ..) if rect.x() < 0.);
    assert_matches!(diagram.nodes[1], Primitive(_, rect, ..) if rect.x() < 0.);
    assert_diagram(diagram, "target/visual_edge_down_center")?;
    Ok(())
  }
}