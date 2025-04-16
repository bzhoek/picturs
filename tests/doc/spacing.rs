#[cfg(test)]
mod spacing {
  use picturs::assert_diagram;

  #[test]
  fn expand() {
    let string = r#"
      box dotted color red {
        box "expand"
      }
      "#;
    assert_diagram!(string);
  }

  #[test]
  fn shrink() {
    let string = r#"
      box color blue {
        box "shrink" dotted color red
      }
      "#;
    assert_diagram!(string);
  }

}