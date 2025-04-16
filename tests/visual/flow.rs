#[cfg(test)]
mod flow {
  use picturs::assert_diagram;
  use picturs::diagram::create_diagram;

  #[test]
  fn flow_default() {
    let string = r#"
      right
      line
      box "box"
      arrow
      "#;
    let diagram = create_diagram(string);
    assert_diagram!(diagram);
  }
}
