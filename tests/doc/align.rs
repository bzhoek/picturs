#[cfg(test)]
mod align {
  use picturs::test::assert_diagram;

  use crate::{assert_diagram, create_diagram};

  #[test]
  fn above_below() -> anyhow::Result<()> {
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
    assert_diagram!(diagram);
    Ok(())
  }

  #[test]
  fn left_right() -> anyhow::Result<()> {
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
    assert_diagram!(diagram);
    Ok(())
  }

  #[test]
  fn dot_caption() -> anyhow::Result<()> {
    let string = r#"
      dot color red rad 4pt "N" above
      "#;
    let diagram = create_diagram(string);
    assert_diagram!(diagram);
    Ok(())
  }
}