#[cfg(test)]
mod tests {
  use std::fs;
  use std::ops::{Mul, Sub};
  use std::path::Path;
  use std::process::Command;

  use anyhow::Result;
  use skia_safe::{Color, Point, Rect, Vector};

  use picturs::diagram::{A5, Diagram, Node, Paragraph, Radius};
  use picturs::diagram::Node::{Container, Primitive};
  use picturs::diagram::Shape::Rectangle;
  use picturs::init_logging;
  use picturs::types::{Anchor, Distance, Edge, Unit};

  static TQBF: &str = "the quick brown fox jumps over the lazy dog";

  fn create_diagram(string: &str) -> Diagram {
    let mut diagram = Diagram::offset(A5, (0., 0.));
    diagram.parse_string(string);
    diagram
  }

  fn create_diagram_inset(string: &str) -> Diagram {
    init_logging();
    let mut diagram = Diagram::offset(A5, (32., 32.));
    diagram.parse_string(string);
    diagram
  }

1  fn rectangle(title: Option<(&str, f32)>) -> picturs::diagram::Shape {
    let paragraph = title.map(|(title, width)| {
      Paragraph { text: title, widths: vec!(width), height: 17. }
    });
    Rectangle(Color::BLACK, paragraph, Radius::default(), Color::TRANSPARENT, None)
  }

  #[test]
  fn single_box_untitled() {
    let string = r#"box"#;
    let diagram = create_diagram(string);

    assert_eq!(vec![
      Primitive(None,
        Rect::from_xywh(0., 0., 120., 56.),
        Rect::from_xywh(0., 0., 120., 48.),
        Color::BLUE,
        rectangle(None)),
    ], diagram.nodes);
  }

  #[test]
  fn single_box_id() {
    let string = r#"box.first "title""#;
    let diagram = create_diagram(string);

    assert_eq!(vec![
      Primitive(Some("first"),
        Rect::from_xywh(0., 0., 120., 56.),
        Rect::from_xywh(0., 0., 120., 48.),
        Color::BLUE,
        rectangle(Some(("title", 31.)))),
    ], diagram.nodes);
  }

  #[test]
  fn single_box_with_title() {
    let string = r#"box "title""#;
    let diagram = create_diagram(string);

    assert_eq!(vec![
      Primitive(None,
        Rect::from_xywh(0., 0., 120., 56.),
        Rect::from_xywh(0., 0., 120., 48.),
        Color::BLUE,
        rectangle(Some(("title", 31.)))),
    ], diagram.nodes);
  }

  #[test]
  fn double_box() {
    let string = "box
                         box";
    let diagram = create_diagram(string);

    assert_eq!(vec![
      Primitive(None,
        Rect::from_xywh(0., 0., 120., 56.),
        Rect::from_xywh(0., 0., 120., 48.),
        Color::BLUE,
        rectangle(None)),
      Primitive(None,
        Rect::from_xywh(0., 56., 120., 56.),
        Rect::from_xywh(0., 56., 120., 48.),
        Color::BLUE,
        rectangle(None)),
    ], diagram.nodes);
  }

  #[test]
  fn nested_box_id() {
    let string = "box.parent { box }";
    let diagram = create_diagram(string);

    assert_eq!(vec![
      Container(Some("parent"), Radius::default(), None,
        Rect::from_xywh(0., 0., 144., 80.),
        Rect::from_xywh(0., 0., 144., 72.),
        vec![
          Primitive(None,
            Rect::from_xywh(8., 8., 120., 56.),
            Rect::from_xywh(8., 8., 120., 48.),
            Color::BLUE,
            rectangle(None))
        ])
    ], diagram.nodes);
  }

  #[test]
  fn nested_box_with_title() {
    let string = r#"box "parent" { box "child" }"#;
    let diagram = create_diagram(string);

    assert_eq!(vec![
      Container(None, Radius::default(), Some("parent"),
        Rect::from_xywh(0., 0., 144., 93.),
        Rect::from_xywh(0., 0., 144., 85.),
        vec![
          Primitive(None,
            Rect::from_xywh(8., 8., 120., 56.),
            Rect::from_xywh(8., 8., 120., 48.),
            Color::BLUE,
            rectangle(Some(("child", 40.))))
        ])
    ], diagram.nodes);
  }

  #[test]
  fn box_with_wrapping_title() {
    let string = format!(r#"box "{}""#, TQBF);
    let diagram = create_diagram(&string);
    let paragraph1_rect = Rect::from_xywh(0., 0., 120., 84.);
    let paragraph2_rect = Rect::from_xywh(0., 0., 120., 76.);

    let tqbf = Rectangle(Color::BLACK, Some(Paragraph { text: TQBF, widths: vec!(72., 78., 115., 68.), height: 68. }), Radius::default(), Color::TRANSPARENT, None);

    assert_eq!(vec![
      Primitive(None,
        paragraph1_rect,
        paragraph2_rect,
        Color::BLUE,
        tqbf),
    ], diagram.nodes);
  }

  #[test]
  fn arrow() {
    let p1 = Point::new(0., 0.);
    let p2 = Point::new(20., 20.);
    let direction = p2.sub(p1);
    let angle = direction.y.atan2(direction.x);
    assert_eq!(45., angle.to_degrees())
  }

  #[test]
  fn visual_double_containers() -> Result<()> {
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
    let diagram = create_diagram_inset(string);

    assert_visual(diagram, "target/visual_double_containers")?;
    Ok(())
  }

  #[test]
  fn visual_effort_to_impact() -> Result<()> {
    let string =
      r#"
      move 4cm down 4cm right
      box.step1 "Effort"
      box.step2 "Output"  .w 2cm right 1cm up from step1.n
      box.step3 "Outcome" .n 2cm right 1cm down from step2.e
      box.step4 "Impact"  .e 2cm left 1cm down from step3.s
      arrow from step1.n to step2.w
      arrow from step2.e to step3.n
      arrow from step3.s to step4.e
      "#;
    let diagram = create_diagram_inset(string);

    assert_visual(diagram, "target/visual_effort_to_impact")?;
    Ok(())
  }

  #[test]
  fn visual_text_shape() -> Result<()> {
    let string =
      r#"box {
        text "Now"
        box rad 4pt "What do we need to start doing now"
      }
      "#;
    let diagram = create_diagram_inset(string);

    assert_visual(diagram, "target/visual_text_shape")?;
    Ok(())
  }

  #[test]
  fn visual_remember_the_future() -> Result<()> {
    let string =
      r#"box.now "Now" {
        box.step3 "What do we need to start doing now"
      }
      box.future "March" .nw 8cm right from now.ne {
        box.step1 "Imagine it is four months into the future"
        box.step2 "What would you like to write about the past period"
        box.note "IMPORTANT: write in past tense"
      }
      line from now.n 1pc up to future.n
      line from future.s 2pc down to now.s
      "#;
    let diagram = create_diagram_inset(string);

    assert_visual(diagram, "target/visual_remember_the_future")?;
    Ok(())
  }

  #[test]
  fn visual_whole_ast() -> Result<()> {
    let string =
      r#"box.now "Now" {
        box.step3 "What do we need to start doing now"
      }
      box.future "March" {
        box.step1 "Imagine it is four months into the future" .nw 1cm right from now.ne
        box.step2 "What would you like to write about the past period"
        box.note "IMPORTANT: write in past tense"
      }
      "#;
    let diagram = create_diagram_inset(string);
    assert_visual(diagram, "target/visual_whole_ast")?;
    Ok(())
  }

  #[test]
  fn layout_node() {
    let string =
      r#"
      box.left "This goes to the left hand side"
      box.right "While this goes to the right hand side" .nw 2cm right from left.ne
      "#;
    let diagram = create_diagram_inset(string);

    let left = diagram.used_rect("left").unwrap();
    let expected = Rect { left: 32., top: 32., right: 152., bottom: 91. };
    assert_eq!(&expected, left);

    let right = diagram.used_rect("right").unwrap();
    let expected = Rect::from_xywh(228., 32., 120., 59.);
    assert_eq!(&expected, right);
  }

  #[test]
  fn visual_side_by_side() -> Result<()> {
    let string =
      r#"
      box.left "This goes to the left hand side"
      box.right "While this goes to the right hand side" .nw 2cm right 1cm down from left.ne
      "#;
    let diagram = create_diagram_inset(string);
    assert_eq!(2, diagram.nodes.len());

    let rect = diagram.used_rect("left").unwrap();
    let expected = Rect { left: 32., top: 32., right: 152., bottom: 91. };
    assert_eq!(&expected, rect);

    assert_visual(diagram, "target/visual_side_by_side")?;
    Ok(())
  }

  #[test]
  fn visual_width_and_height() -> Result<()> {
    let string =
      r#"
      box wd 4cm ht 4cm "This goes to the left hand side"
      "#;
    let diagram = create_diagram_inset(string);
    assert_visual(diagram, "target/visual_width_and_height")?;
    Ok(())
  }

  #[test]
  fn visual_right_center_left() -> Result<()> {
    let string =
      r#"
      box.left "This goes to the left hand side" color green fill white
      box.right "While this goes to the right hand side" color magenta fill gray text white .w 2cm right from left.ne
      "#;
    let diagram = create_diagram_inset(string);
    assert_eq!(2, diagram.nodes.len());

    let rect = diagram.used_rect("left").unwrap();
    let expected = Rect { left: 32., top: 32., right: 152., bottom: 91. };
    assert_eq!(&expected, rect);

    assert_visual(diagram, "target/visual_right_center_left")?;
    Ok(())
  }

  #[test]
  fn visual_top_down_line() -> Result<()> {
    let string =
      r#"
      box.top    "Top"
      box.bottom "Bottom" .n 2cm down from top.s
      arrow from top.s to bottom.n
      "#;
    let diagram = create_diagram_inset(string);
    assert_visual(diagram, "target/visual_top_down_line")?;
    Ok(())
  }

  fn assert_visual(diagram: Diagram, prefix: &str) -> Result<()> {
    let ref_file = format!("{}.png", prefix);
    let last_file = format!("{}-last.png", prefix);
    let diff_file = format!("{}-diff.png", prefix);

    diagram.render_to_file(&*last_file);

    if !Path::new(&ref_file).exists() {
      fs::rename(last_file, ref_file)?;
      if Path::new(&diff_file).exists() {
        fs::remove_file(diff_file)?;
      }
    } else {
      let output = Command::new("/usr/local/bin/compare")
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
  fn to_edge() {
    let rect = Rect::from_xywh(40., 40., 100., 200.);

    let anchor = Anchor::new("nw");
    let center = rect.center();
    assert_eq!(Point::new(90., 140.), center);

    let nw = anchor.to_edge(&rect);
    assert_eq!(Point::new(40., 40.), nw);

    let anchor = Anchor::new("se");
    let se = anchor.to_edge(&rect);
    assert_eq!(Point::new(140., 240.), se);
  }

  #[test]
  fn to_top_left() {
    let rect = Rect::from_xywh(40., 40., 10., 20.);
    /*
    het verschil moet van topleft worden afgetrokken
     */
    let anchor = Anchor::new("nw");
    let factors = anchor.to_tuple();
    assert_eq!((-0.5, -0.5), factors);
    let nw = anchor.topleft_offset(&rect);
    assert_eq!(Point::new(-0., -0.), nw);

    let anchor = Anchor::new("ne");
    let factors = anchor.to_tuple();
    assert_eq!((0.5, -0.5), factors);
    let ne = anchor.topleft_offset(&rect);
    assert_eq!(Point::new(-10., -0.), ne);

    let anchor = Anchor::new("se");
    let factors = anchor.to_tuple();
    assert_eq!((0.5, 0.5), factors);
    let se = anchor.topleft_offset(&rect);
    assert_eq!(Point::new(-10., -20.), se);
  }

  #[test]
  fn parse_multiple_directions() {
    let string =
      r#"
      box.left "Left"
      box "Right" .nw 1cm right 2cm down from left.ne
      "#;

    create_diagram_inset(string);

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
      box.right "While this goes to the right hand side" .nw 2cm right from left.ne
      "#;
    let mut diagram = Diagram::offset(A5, (32., 32.));
    diagram.parse_string(string);

    let rect = diagram.used_rect("right").unwrap();
    let expected = Rect::from_xywh(228., 32., 120., 59.);
    assert_eq!(&expected, rect);

    let distances = vec![
      Distance::new(2., Unit::Cm, Vector::new(1., 0.)),
      Distance::new(1., Unit::Cm, Vector::new(0., 1.)),
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
      Distance::new(2., Unit::Cm, Vector::new(1., 0.)),
      Distance::new(1., Unit::Cm, Vector::new(0., 1.)),
    ];
    let result = Diagram::offset_from_rect(&rect, &Anchor::new("nw"), &distances);
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
      Color::BLACK,
      rectangle(None));

    match primitive {
      Primitive(_, ref mut rect, _, _, _) => {
        rect.bottom += 8.;
      }
      _ => {}
    }

    let mut primitives = vec![primitive];
    let rect = find_rect(&mut primitives);
    if let Some(rect) = rect {
      rect.bottom += 16.
    }

    let mut primitives = Primitives { primitives };
    let rect = primitives.find_primitive();
    if let Some(rect) = rect {
      rect.bottom += 16.
    }
  }

  impl Primitives<'_> {
    fn find_primitive(&mut self) -> Option<&mut Rect> {
      let first = self.primitives.first_mut();
      let rect = match first.unwrap() {
        Primitive(_, ref mut rect, _, _, _) => {
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
        Primitive(_, ref mut rect, _, _, _) => {
          rect.bottom += 8.;
          return Some(rect);
        }
        Container(_, _, _, _, _, nodes) => {
          find_rect(nodes);
        }
      }
    }
    None
  }
}