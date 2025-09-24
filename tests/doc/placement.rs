#[cfg(test)]
mod placement {
  use picturs::assert_diagram;

  #[test]
  fn sized() {
    let string = r#"
      canvas 0.75x0.5in
      grid
      "#;
    assert_diagram!(string, None);
  }

  #[test]
  fn size_width_height() {
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

  #[test]
  fn captions_multiple() {
    let string = r#"
      grid
      line "Hello"
      box "Big," "World!"
      arrow "Bye"
      "#;
    assert_diagram!(string, None);
  }
}