mod common;

#[cfg(test)]
mod tests {
  use picturs::test::assert_diagram;

  use crate::common::create_diagram;

  #[test]
  fn visual_units() -> anyhow::Result<()> {
    let string = r#"
      box "box" wd 1in ht 0.5in
      "#;
    let diagram = create_diagram(string);
    assert_diagram(diagram, "target/visual_units")?;
    Ok(())
  }

}