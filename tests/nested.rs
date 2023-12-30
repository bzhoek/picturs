#[cfg(test)]
mod tests {
  use std::fs;
  use std::path::Path;
  use std::process::Command;

  use anyhow::Result;
  use skia_safe::{Point, Rect};

  use picturs::nested::{Compass, Diagram, Shape};
  use picturs::nested::Node::{Container, Primitive};
  use picturs::nested::Shape::Rectangle;

  static TQBF: &str = "the quick brown fox jumps over the lazy dog";

  fn zero_rect() -> Rect {
    Rect::from_xywh(0., 0., 0., 0.)
  }

  fn parse_string(string: &str) -> Diagram {
    let mut diagram = Diagram::default();
    diagram.parse_string(string);
    diagram
  }

  #[test]
  fn single_box_untitled() {
    let string = r#"box"#;
    let diagram = parse_string(string);

    assert_eq!(vec![
      Primitive(None,
                Rect::from_xywh(0., 0., 120., 56.),
                Rect::from_xywh(0., 0., 120., 48.),
                Rectangle(None, None)),
    ], diagram.nodes);
  }

  #[test]
  fn single_box_id() {
    let string = r#"box.first "title""#;
    let diagram = parse_string(string);

    assert_eq!(vec![
      Primitive(Some("first"),
                Rect::from_xywh(0., 0., 120., 56.),
                Rect::from_xywh(0., 0., 120., 48.),
                Rectangle(Some("title"), None)),
    ], diagram.nodes);
  }

  #[test]
  fn single_box_with_title() {
    let string = r#"box "title""#;
    let diagram = parse_string(string);

    assert_eq!(vec![
      Primitive(None,
                Rect::from_xywh(0., 0., 120., 56.),
                Rect::from_xywh(0., 0., 120., 48.),
                Rectangle(Some("title"), None)),
    ], diagram.nodes);
  }

  #[test]
  fn double_box() {
    let string = "box
                         box";
    let diagram = parse_string(string);

    assert_eq!(vec![
      Primitive(None,
                Rect::from_xywh(0., 0., 120., 56.),
                Rect::from_xywh(0., 0., 120., 48.),
                Rectangle(None, None)),
      Primitive(None,
                Rect::from_xywh(0., 56., 120., 56.),
                Rect::from_xywh(0., 56., 120., 48.),
                Rectangle(None, None)),
    ], diagram.nodes);
  }

  #[test]
  fn nested_box_id() {
    let string = "box.parent { box }";
    let diagram = parse_string(string);

    assert_eq!(vec![
      Container(None,
                Rect::from_xywh(0., 0., 144., 80.),
                Rect::from_xywh(0., 0., 144., 72.),
                vec![
                  Primitive(None,
                            Rect::from_xywh(8., 8., 120., 56.),
                            Rect::from_xywh(8., 8., 120., 48.),
                            Rectangle(None, None))
                ])
    ], diagram.nodes);
  }

  #[test]
  fn nested_box_with_title() {
    let string = r#"box "parent" { box "child" }"#;
    let diagram = parse_string(string);

    assert_eq!(vec![
      Container(Some("parent"),
                Rect::from_xywh(0., 0., 144., 93.),
                Rect::from_xywh(0., 0., 144., 85.),
                vec![
                  Primitive(None,
                            Rect::from_xywh(8., 8., 120., 56.),
                            Rect::from_xywh(8., 8., 120., 48.),
                            Rectangle(Some("child"), None))
                ])
    ], diagram.nodes);
  }

  #[test]
  fn line_from_a_to_b() {
    let string = r#"line from now.n to future.n"#;
    let diagram = parse_string(string);

    assert_eq!(vec![
      Primitive(None,
                zero_rect(),
                zero_rect(),
                Shape::Line(None, "now.n", "future.n")),
    ], diagram.nodes);
  }

  #[test]
  fn box_with_wrapping_title() {
    let string = format!(r#"box "{}""#, TQBF);
    let diagram = parse_string(&string);
    let paragraph1_rect = Rect::from_xywh(0., 0., 120., 84.);
    let paragraph2_rect = Rect::from_xywh(0., 0., 120., 76.);

    assert_eq!(vec![
      Primitive(None,
                paragraph1_rect,
                paragraph2_rect,
                Rectangle(Some(TQBF), None)),
    ], diagram.nodes);
  }

  #[test]
  fn parse_extended_example() -> Result<()> {
    let string =
      r#"box.now "Now" {
        box.step3 "What do we need to start doing now"
      }
      box.future "March" {
        box.step1 "Imagine it is four months into the future"
        box.step2 "What would you like to write about the past period"
        box.note "IMPORTANT: write in past tense"
      }
      line from now.n to future.n
      "#;
    let mut diagram = Diagram::offset((32., 32.));
    diagram.parse_string(string);
    dbg!(&diagram.nodes);
    assert_visual(diagram, "target/extended")?;
    Ok(())
  }

  #[test]
  fn layout_node() {
    let string =
      r#"
      box.left "This goes to the left hand side"
      box.right "While this goes to the right hand side" @nw 2cm right from left.ne
      "#;
    let mut diagram = Diagram::offset((32., 32.));
    diagram.parse_string(string);

    let right = diagram.find_node("right").unwrap();
    let used = match right {
      Primitive(_id, _rect, mut used, _shape) => {
        used.bottom += 8.;
        Some(used)
      }
      _ => None
    };

    let expected = Rect { left: 32., top: 99., right: 152., bottom: 166. };
    assert_eq!(Some(&expected), used.as_ref());

    let expected = Rect { left: 32., top: 32., right: 152., bottom: 91. };
    let other = diagram.used_rect("left");
    assert_eq!(Some(&expected), other);

    let other = diagram.layout_node(right);
    assert_eq!(Some(&expected), other);
  }

  #[test]
  fn side_by_side() -> Result<()> {
    let string =
      r#"
      box.left "This goes to the left hand side"
      box.right "While this goes to the right hand side" @nw 2cm right from left.ne
      "#;
    let mut diagram = Diagram::offset((32., 32.));
    diagram.parse_string(string);
    assert_eq!(2, diagram.nodes.len());
    dbg!(&diagram.nodes);
    let left = diagram.find_node("left").unwrap();
    let left_rect = match left {
      Primitive(_, rect, _, _) => { rect }
      Container(_, rect, _, _) => { rect }
    };
    let rect = Rect { left: 32., top: 32., right: 152., bottom: 99. };
    assert_eq!(&rect, left_rect);

    assert_visual(diagram, "target/side_by_side")?;
    Ok(())
  }

  fn assert_visual(diagram: Diagram, prefix: &str) -> Result<()> {
    let ref_file = format!("{}.png", prefix);
    let last_file = format!("{}-last.png", prefix);
    diagram.render(400, 800, &*last_file);
    if !Path::new(&ref_file).exists() {
      fs::rename(last_file, ref_file)?;
    } else {
      let diff_file = format!("{}-diff.png", prefix);
      let output = Command::new("compare")
        .arg("-metric")
        .arg("rmse")
        .arg(&last_file)
        .arg(ref_file)
        .arg(&diff_file)
        .output()?;
      assert!(output.status.success());
      fs::remove_file(last_file)?;
      fs::remove_file(diff_file)?;
    }
    Ok(())
  }

  #[test]
  fn upper_left() {
    let rect = Rect::from_xywh(40., 40., 100., 200.);

    let compass = Compass::convert("nw");
    let mut center = rect.center();
    assert_eq!(Point::new(90., 140.), center);

    center.offset((compass.x * rect.width(), compass.y * rect.height()));
    assert_eq!(Point::new(40., 40.), center);

    let compass = Compass::convert("se");
    let mut center = rect.center();
    center.offset((compass.x * rect.width(), compass.y * rect.height()));
    assert_eq!(Point::new(140., 240.), center);
  }

  #[test]
  fn parse_multiple_directions() {
    let string =
      r#"
      box.left "Left"
      box "Right" @nw 1cm right 2cm down from left.ne
      "#;
    let mut diagram = Diagram::offset((32., 32.));
    let _top = diagram.parse_string(string);
    dbg!(&diagram.nodes);
    // dump_nested(0, top);
  }
}