#[cfg(test)]
mod diagram {
  use picturs::create_diagram;
  use picturs::test::assert_diagram;

  use crate::{assert_diagram};

  #[test]
  fn double_containers() {
    let string =
      r#"box.now "Now" {
        box.step3 rad 4pt "What do we need to start doing now"
      }
      box.future rd 4pt "March" {
        box.step1 "Imagine it is four months into the future"
        box.step2 "What would you like to write about the past period"
        box.note "IMPORTANT: write in past tense"
      }
      line from now.e 1cm right to future.e
      "#;
    let diagram = create_diagram(string);
    assert_diagram!(diagram);
  }

  #[test]
  fn effort_to_impact() {
    let string =
      r#"
      box.step1 "Effort"
      box.step2 "Output"  .w 2cm right 1cm up from step1.n
      box.step3 "Outcome" .n 2cm right 1cm down from step2.e
      box.step4 "Impact"  .e 2cm left 1cm down from step3.s
      arrow from step1.n to step2.w
      arrow from step2.e to step3.n
      arrow from step3.s to step4.e
      "#;
    let diagram = create_diagram(string);
    assert_diagram!(diagram);
  }

  #[test]
  fn move_flow() {
    let string =
      r#"
      box "Top"
      move 1cm right 1cm down
      box "Middle"
      flow 1cm
      box "Bottom"
      "#;
    let diagram = create_diagram(string);
    assert_diagram!(diagram);
  }

  #[test]
  fn text_shape() {
    let string =
      r#"box {
        text "Now"
        box rad 4pt "What do we need to start doing now"
      }
      "#;
    let diagram = create_diagram(string);
    assert_diagram!(diagram);
  }

  #[test]
  fn remember_the_future() {
    let string =
      r#"box.now "Now" {
        box.step3 "What do we need to start doing now"
      }
      box.future "March" .nw 8cm right from now.ne {
        box.step1 "Imagine it is four months into the future"
        box.step2 "What would you like to write about the past period"
        box.note "IMPORTANT: write in past tense"
      }
      line from now.n 1cm up to future.n
      line from future.s 1cm down to now.s
      "#;
    let diagram = create_diagram(string);
    assert_diagram!(diagram);
  }

  #[test]
  fn whole_ast() {
    let string =
      r#"box.now "Now" {
        box.step3 "What do we need to start doing now"
      }
      box.future "March" .nw 1cm right from now.ne {
        box.step1 "Imagine it is four months into the future"
        box.step2 "What would you like to write about the past period"
        box.note "IMPORTANT: write in past tense"
      }
      "#;
    let diagram = create_diagram(string);
    assert_diagram!(diagram);
  }

  #[test]
  fn side_by_side() {
    let string =
      r#"
      box.left "This goes to the left hand side"
      box.right "While this goes to the right hand side" .nw 2cm right 1cm down from left.ne
      "#;
    let diagram = create_diagram(string);
    assert_diagram!(diagram);
  }

  #[test]
  fn width_and_height() {
    let string =
      r#"
      box wd 4cm ht 4cm "This goes to the left hand side"
      "#;
    let diagram = create_diagram(string);
    assert_diagram!(diagram);
  }

  #[test]
  fn right_center_left() {
    let string =
      r#"
      box.left "This goes to the left hand side" color green fill white
      box.right "While this goes to the right hand side" color magenta fill gray text_color white .w 2cm right from left.ne
      "#;
    let diagram = create_diagram(string);
    assert_diagram!(diagram);
  }

  #[test]
  fn top_down_line() {
    let string =
      r#"
      box.top    "Top"
      box.bottom "Bottom" .n 2cm down from top.s
      arrow from top.s to bottom.n
      dot at top.s color red rad 4pt
      dot at top.n color green rad 4pt
      "#;
    let diagram = create_diagram(string);
    assert_diagram!(diagram);
  }

}