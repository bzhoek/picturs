#[cfg(test)]
mod units {
  use picturs::test::assert_diagram;

  use crate::create_diagram;

  #[test]
  fn visual_units() -> anyhow::Result<()> {
    let string = r#"
      down
      set unit cm
      box "A" wd 1in ht 0.5in
      box "B" 10 right
      "#;
    let diagram = create_diagram(string);
    assert_diagram(diagram, "target/visual_units")?;
    Ok(())
  }

  #[test]
  fn visual_text_fit() -> anyhow::Result<()> {
    let string = r#"
      set font "Courier"
      right
      text "ssh" fit
      text " -L " fit
      text.local "1025" fit
      text ":localhost:" fit
      text.remote "25" fit
      text.host " home.hoek.com" fit
      set font "Helvetica"
      line from local.n 1cm up "local port" above
      line from remote.s 1cm down "remote port" below
      line from host.n 1cm up "on host" above
      "#;
    let diagram = create_diagram(string);
    assert_diagram(diagram, "target/visual_text_fit")?;
    Ok(())
  }

}