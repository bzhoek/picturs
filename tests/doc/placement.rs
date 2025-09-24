#[cfg(test)]
mod placement {
  use picturs::assert_diagram;

  #[test]
  fn grid_center() {
    let string = r#"
      box "Hello"
      grid
      "#;
    assert_diagram!(string, None);
  }

}