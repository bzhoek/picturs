#[cfg(test)]
mod edges {
  use picturs::test::assert_diagram;

  use crate::{assert_diagram, create_diagram};

  #[test]
  fn all_edges() -> anyhow::Result<()> {
    let string = r#"
      box.a wd 1in ht 1in
      dot at a.n color red rad 4pt "N" above
      dot at a.s color red rad 4pt "S" below
      dot at a.e color red rad 4pt "E" right
      dot at a.w color red rad 4pt "W" left
      "#;
    let diagram = create_diagram(string);
    assert_diagram!(diagram);
    Ok(())
  }

}