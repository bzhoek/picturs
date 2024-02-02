mod common;

#[cfg(test)]
mod tests {
  use picturs::test::assert_diagram;

  use crate::common::create_diagram;

  #[test]
  fn visual_units() -> anyhow::Result<()> {
    let string = r#"
      down
      box "A" wd 1in ht 0.5in
      box "B" 40 right
      "#;
    let diagram = create_diagram(string);
    assert_diagram(diagram, "target/visual_units")?;
    Ok(())
  }

}