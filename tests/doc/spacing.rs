#[cfg(test)]
mod spacing {
  use picturs::assert_diagram;
  use picturs::diagram::create_diagram;

  #[test]
  fn expand() {
    let string = r#"
      box dotted color red {
        box "expand"
      }
      "#;
    let diagram = create_diagram(string);
    assert_diagram!(diagram);
  }

  #[test]
  fn shrink() {
    let string = r#"
      box color blue {
        box "shrink" dotted color red
      }
      "#;
    let diagram = create_diagram(string);
    assert_diagram!(diagram);
  }

}