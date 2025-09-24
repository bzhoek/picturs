#[cfg(test)]
mod align {
  use picturs::assert_diagram;

  #[test]
  fn above_below() {
    let string = r#"
      right
      set line ln 3cm
      arrow "above"
      flow 1cm
      line "center" center
      flow 1cm
      arrow "below" below
      flow 1cm
      // arrow ln 2cm "left" left
      "#;
    assert_diagram!(string);
  }

  #[test]
  fn left_right() {
    let string = r#"
      down
      box "" wd 1in ht 0
      arrow "left" left
      flow 1cm
      line "center" center
      flow 1cm
      arrow "right" right
      "#;
    assert_diagram!(string);
  }

  #[test]
  fn dot_caption() {
    let string = r#"
      dot color red rad 4pt "N" above
      "#;
    assert_diagram!(string);
  }
}