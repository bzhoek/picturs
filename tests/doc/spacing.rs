#[cfg(test)]
mod spacing {
  use picturs::assert_diagram;
  use picturs::diagram::create_diagram;

  #[test]
  fn spacing_expand() {
    let string = r#"
      box dotted {
        box "bounds"
      }
      "#;
    let diagram = create_diagram(string);
    assert_diagram!(diagram);
  }

  #[test]
  fn spacing_shrink() {
    let string = r#"
      box {
        box "bounds" dotted
      }
      "#;
    let diagram = create_diagram(string);
    assert_diagram!(diagram);
  }

}