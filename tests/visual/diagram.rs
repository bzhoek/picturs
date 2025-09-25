#[cfg(test)]
mod diagram {
  use picturs::assert_diagram;

  #[test]
  fn double_containers() {
    let string = r#"
      continue down-left
      group.now "Now" stroke black {
        box.step3 rad 4pt "What do we need to start doing now"
      }
      group.future rd 4pt "March" stroke black {
        box.step1 "Imagine it is four months into the future"
        box.step2 "What would you like to write about the past period"
        box.note "IMPORTANT: write in past tense"
      }
      line route from now.e 1cm right end future.e
      "#;
    assert_diagram!(string);
  }

  #[test]
  fn effort_to_impact() {
    let string = r#"
      continue down-left
      box.step1 "Effort"
      box.step2 "Output"  .w 2cm right 1cm up from step1.n
      box.step3 "Outcome" .n 2cm right 1cm down from step2.e
      box.step4 "Impact"  .e 2cm left 1cm down from step3.s
      arrow route from step1.n end step2.w
      arrow route from step2.e end step3.n
      arrow route from step3.s end step4.e
      "#;
    assert_diagram!(string);
  }

  #[test]
  fn move_flow() {
    let string = r#"
      continue down-left
      box "Top"
      move 1cm right 1cm down
      box "Middle"
      flow 1cm
      box "Bottom"
      "#;
    assert_diagram!(string);
  }

  #[test]
  fn text_shape() {
    let string = r#"
      continue down-left
      group stroke black {
        text "Now"
        box rad 4pt "What do we need to start doing now"
      }
      "#;
    assert_diagram!(string);
  }

  #[test]
  fn remember_the_future() {
    let string = r#"
      continue down-left
      group.now "Now" stroke black {
        box.step3 "What do we need to start doing now"
      }
      group.future "March" .nw 8cm right from now.ne stroke black {
        box.step1 "Imagine it is four months into the future"
        box.step2 "What would you like to write about the past period"
        box.note "IMPORTANT: write in past tense"
      }
      line route from now.n 1cm up end future.n
      line route from future.s 1cm down end now.s
      "#;
    assert_diagram!(string);
  }

  #[test]
  fn whole_ast() {
    let string = r#"
      continue down-left
      group.now "Now" stroke black {
        box.step3 "What do we need to start doing now"
      }
      group.future "March" .nw 1cm right from now.ne stroke black {
        box.step1 "Imagine it is four months into the future"
        box.step2 "What would you like to write about the past period"
        box.note "IMPORTANT: write in past tense"
      }
      "#;
    assert_diagram!(string);
  }

  #[test]
  fn side_by_side() {
    let string = r#"
      continue down-left
      box.left "This goes to the left hand side"
      box.right "While this goes to the right hand side" .nw 2cm right 1cm down from left.ne
      "#;
    assert_diagram!(string);
  }

  #[test]
  fn width_and_height() {
    let string = r#"
      continue down-left
      box wd 4cm ht 4cm "This goes to the left hand side"
      "#;
    assert_diagram!(string);
  }

  #[test]
  fn right_center_left() {
    let string = r#"
      continue down-left
      box.left "This goes to the left hand side" color green fill white
      box.right "While this goes to the right hand side" color magenta fill gray text white .w 2cm right from left.ne
      "#;
    assert_diagram!(string);
  }

  #[test]
  fn top_down_line() {
    let string = r#"
      continue down-left
      box.top    "Top"
      box.bottom "Bottom" .n 2cm down from top.s
      arrow from top.s end bottom.n
      dot at top.s color red rad 4pt
      dot at top.n color green rad 4pt
      "#;
    assert_diagram!(string);
  }
}
