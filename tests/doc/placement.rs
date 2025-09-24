#[cfg(test)]
mod placement {
  use picturs::assert_diagram;

  #[test]
  fn size() {
    let string = r#"
      canvas wd 0.75in ht 0.5in
      grid
      "#;
    assert_diagram!(string, None);
  }

  #[test]
  fn grid_center() {
    let string = r#"
      grid
      box "Hello"
      "#;
    assert_diagram!(string, None);
  }

}