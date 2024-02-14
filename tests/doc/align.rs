#[cfg(test)]
mod align {
  use picturs::test::assert_diagram;

  use crate::create_diagram;

  #[test]
  fn doc_above_below() -> anyhow::Result<()> {
    let string = r#"
      right
      set line ln 3cm
      arrow "above" above
      flow 1cm
      line "center"
      flow 1cm
      arrow "below" below
      flow 1cm
      // arrow ln 2cm "left" left
      "#;
    let diagram = create_diagram(string);
    assert_diagram(diagram, "doc/above_below")?;
    Ok(())
  }

  #[test]
  fn doc_left_right() -> anyhow::Result<()> {
    let string = r#"
      down
      box "" wd 1in ht 0
      arrow "left" left
      flow 1cm
      line "center"
      flow 1cm
      arrow "right" right
      "#;
    let diagram = create_diagram(string);
    assert_diagram(diagram, "doc/left_right")?;
    Ok(())
  }

  #[test]
  fn doc_dot_caption() -> anyhow::Result<()> {
    let string = r#"
      dot color red rad 4pt
      "#;
    let diagram = create_diagram(string);
    assert_diagram(diagram, "doc/dot_caption")?;
    Ok(())
  }

}