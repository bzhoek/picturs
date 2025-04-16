/// See [picturs.pest#continuation] for the grammar

#[cfg(test)]
mod flow {
  use picturs::assert_diagram_string;

  #[test]
  fn flow_default_is_right() {
    let string = r#"
      line
      box "box"
      arrow
      "#;
    assert_diagram_string!(string);

    let string = r#"
      right
      line
      box "box"
      arrow
      "#;
    assert_diagram_string!(string);
  }

  #[test]
  fn flow_down() {
    let string = r#"
      down
      line
      box "box"
      arrow
      "#;
    assert_diagram_string!(string);
  }
}
