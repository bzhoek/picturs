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
      dot at a.nw color red rad 4pt "NW" nw
      dot at a.ne color red rad 4pt "NE" ne
      dot at a.sw color red rad 4pt "SW" sw
      dot at a.se color red rad 4pt "SE" se
      "#;
    let diagram = create_diagram(string);
    assert_diagram!(diagram);
    Ok(())
  }

}