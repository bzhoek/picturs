#[cfg(test)]
mod edges {
  use picturs::create_diagram;
  use picturs::test::assert_diagram;

  use crate::{assert_diagram};

  #[test]
  fn all_edges() -> anyhow::Result<()> {
    let string = r#"
      box.a wd 1in ht 1in
      dot at a.n color red rad 4pt "N" above
      dot at a.s same "S" below
      dot at a.e same "E" right
      dot at a.w same "W" left
      dot at a.nw color blue rad 4pt "NW" nw
      dot at a.ne same "NE" ne
      dot at a.sw same "SW" sw
      dot at a.se same "SE" se
      "#;
    let diagram = create_diagram(string);
    assert_diagram!(diagram);
    Ok(())
  }
}