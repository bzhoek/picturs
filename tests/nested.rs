#[cfg(test)]
mod tests {
  use std::fs;
  use std::ops::Mul;
  use std::path::Path;
  use std::process::Command;

  use anyhow::Result;
  use skia_safe::{Point, Rect, Vector};
  use picturs::{Distance, Edge};

  use picturs::nested::{Compass, Diagram, Node, Shape};
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

    let left = diagram.used_rect("left").unwrap();
    let expected = Rect { left: 32., top: 32., right: 152., bottom: 91. };
    assert_eq!(&expected, left);

    let right = diagram.used_rect("right").unwrap();
    let expected = Rect::from_xywh(228., 32., 120., 59.);
    assert_eq!(&expected, right);
  }

  #[test]
  fn side_by_side() -> Result<()> {
    let string =
      r#"
      box.left "This goes to the left hand side"
      box.right "While this goes to the right hand side" @nw 2cm right 1cm down from left.ne
      "#;
    let mut diagram = Diagram::offset((32., 32.));
    diagram.parse_string(string);
    assert_eq!(2, diagram.nodes.len());

    let rect = diagram.used_rect("left").unwrap();
    let expected = Rect { left: 32., top: 32., right: 152., bottom: 91. };
    assert_eq!(&expected, rect);

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

    let compass = Compass::new("nw");
    let center = rect.center();
    assert_eq!(Point::new(90., 140.), center);

    let nw = compass.to_edge(&rect);
    assert_eq!(Point::new(40., 40.), nw);

    let compass = Compass::new("se");
    let se = compass.to_edge(&rect);
    assert_eq!(Point::new(140., 240.), se);
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
    let _point = Point::new(32., 32.);
    let offset = Vector::new(-1., 0.);
    let result = offset.mul(3.);
    assert_eq!(Point::new(-3., 0.), result);
  }

  #[test]
  fn node_mut() {
    let string =
      r#"
      box.left "This goes to the left hand side"
      box.right "While this goes to the right hand side" @nw 2cm right from left.ne
      "#;
    let mut diagram = Diagram::offset((32., 32.));
    diagram.parse_string(string);

    let rect = diagram.used_rect("right").unwrap();
    let expected = Rect::from_xywh(228., 32., 120., 59.);
    assert_eq!(&expected, rect);

    let distances = vec![
      Distance::new(2., "cm".to_string(), Vector::new(1., 0.)),
      Distance::new(1., "cm".to_string(), Vector::new(0., 1.)),
    ];

    let left = diagram.used_rect("left").unwrap();
    let expected = Rect::from_xywh(32., 32., 120., 59.);
    assert_eq!(&expected, left);

    let edge = Edge::new("left", "ne"); // 32 + 120 + (2 * 38) = 228
    let shifted = diagram.offset_from(&edge, &distances).unwrap();
    let expected = Rect::from_xywh(228., 70., 120., 59.);
    assert_eq!(expected, shifted);

    diagram.node_mut("right", distances);
    let rect = diagram.used_rect("right").unwrap();
    let expected = Rect::from_xywh(304., 70., 120., 59.);
    assert_eq!(&expected, rect);
  }

  #[test]
  fn offset_from_rect() {
    let rect = Rect::from_xywh(40., 40., 40., 40.);
    let distances = vec![
      Distance::new(2., "cm".to_string(), Vector::new(1., 0.)),
      Distance::new(1., "cm".to_string(), Vector::new(0., 1.)),
    ];
    let result = Diagram::offset_from_rect(&rect, &Compass::new("nw"), &distances);
    let expected = Rect { left: 116.0, top: 78.0, right: 156.0, bottom: 118.0 };
    assert_eq!(expected, result);
  }

  #[derive(Debug)]
  struct Primitives<'a> {
    primitives: Vec<Node<'a>>,
  }

  #[test]
  fn test_primitives_mut() {
    let mut primitive = Primitive(None,
                                  Rect::from_xywh(0., 0., 120., 56.),
                                  Rect::from_xywh(0., 0., 120., 48.),
                                  Rectangle(None, None));
    dbg!(&primitive);
    match primitive {
      Primitive(_, ref mut rect, _, _) => {
        rect.bottom += 8.;
      }
      _ => {}
    }
    dbg!(&primitive);

    let mut primitives = vec![primitive];
    let rect = find_rect(&mut primitives);
    if let Some(rect) = rect {
      rect.bottom += 16.
    }
    dbg!(&primitives);

    let mut primitives = Primitives { primitives };
    let rect = primitives.find_primitive();
    if let Some(rect) = rect {
      rect.bottom += 16.
    }
    dbg!(&primitives);
  }

  impl Primitives<'_> {
    fn find_primitive(&mut self) -> Option<&mut Rect> {
      let first = self.primitives.first_mut();
      let rect = match first.unwrap() {
        Primitive(_, ref mut rect, _, _) => {
          rect.bottom += 8.;
          Some(rect)
        }
        _ => None
      };
      rect
    }
  }

  fn find_rect<'a>(nodes: &'a mut Vec<Node>) -> Option<&'a mut Rect> {
    for node in nodes.iter_mut() {
      match node {
        Primitive(_, ref mut rect, _, _) => {
          rect.bottom += 8.;
          return Some(rect);
        }
        Container(_, _, _, nodes) => {
          find_rect(nodes);
        }
      }
    }
    None
  }
}