use std::ops::Mul;

use skia_safe::{Color, Point, Rect, Size, Vector};

use crate::diagram::conversion::Conversion;
use crate::diagram::create_diagram;
use crate::diagram::index::Index;
use crate::diagram::parser::{Diagram, Rule};
use crate::diagram::types::{CommonAttributes, Config, Displacement, Edge, Paragraph, Radius, Shape, Unit};
use crate::diagram::types::Node::{Container, Primitive};

static TQBF: &str = "the quick brown fox jumps over the lazy dog";

#[test]
fn should_copy_same_attributes_from_line() {
  let mut index = Index::default();
  let config = Config::default();
  let cursor = Point::new(0., 0.);
  let rectangle = Conversion::pair_for(Rule::rectangle, r#"box.pic1 ht 2in wd 1in "Primary Interrupt Controller""#);
  Diagram::box_from(&rectangle, &config, &mut index, &cursor);

  let line = Conversion::pair_for(Rule::line, r#"line from 1/8 pic1.w 1.5in left "Timer" ljust opaque ->"#);
  Diagram::line_from(line, &config, &mut index, &cursor);

  let same = Conversion::pair_for(Rule::line, r#"line from 2/8 pic1.w same "Keyboard""#);
  Diagram::line_from(same, &config, &mut index, &cursor);
}

#[test]
fn should_parse_untitled_box() {
  let string = r#"box"#;
  let diagram = create_diagram(string);

  assert_eq!(
    diagram.nodes,
    vec![Primitive(
      common(None, None),
      rectangle(None)),
    ]);
}

fn common(id: Option<&str>, rect: Option<Rect>) -> CommonAttributes {
  let rect = rect.unwrap_or(box_used());
  CommonAttributes::new(id, rect, Color::BLUE, 1.)
}

fn box_used() -> Rect {
  Rect::from_xywh(0.5, 0.5, 88., 67.)
}

#[test]
fn should_parse_title() {
  let string = r#"box "title""#;
  let diagram = create_diagram(string);

  assert_eq!(
    diagram.nodes,
    vec![Primitive(
      common(None, None),
      rectangle(Some(("title", 31.)))),
    ]);
}

#[test]
fn should_parse_box_id() {
  let string = r#"box.first "title""#;
  let diagram = create_diagram(string);

  assert_eq!(
    diagram.nodes,
    vec![Primitive(
      common(Some("first"), None),
      rectangle(Some(("title", 31.)))),
    ]);
}

#[test]
fn double_box() {
  let string = "box
                         box";
  let diagram = create_diagram(string);

  assert_eq!(
    diagram.nodes,
    vec![Primitive(
      common(None, None),
      rectangle(None)),
         Primitive(
           common(None, Rect::from_xywh(0.5, 75.5, 88., 67.).into()),
           rectangle(None)),
    ]);
}

#[test]
fn nested_box_id() {
  let string = "box.parent { box }";
  let diagram = create_diagram(string);

  assert_eq!(
    diagram.nodes,
    vec![Container(
      Some("parent"), 0., None,
      Rect::from_xywh(0.5, 0.5, 104., 91.),
      vec![Primitive(
        common(None, Rect::from_xywh(8.5, 8.5, 88., 67.).into()),
        rectangle(None))
      ])
    ]);
}

#[test]
fn nested_box_with_title() {
  let string = r#"box "parent" { box "child" }"#;
  let diagram = create_diagram(string);

  assert_eq!(
    diagram.nodes,
    vec![Container(
      None, 0., Some("parent".into()),
      Rect::from_xywh(0.5, 0.5, 104., 104.),
      vec![Primitive(
        common(None, Rect::from_xywh(8.5, 8.5, 88., 67.).into()),
        rectangle(Some(("child", 40.))))
      ])
    ]);
}

#[test]
fn box_with_wrapping_title() {
  let string = format!(r#"box "{}""#, TQBF);
  let diagram = create_diagram(&string);
  let paragraph1_rect = Rect::from_xywh(0.5, 0.5, 88.0, 93.);

  let size = Size::new(88., 85.);
  let tqbf = Shape::Box(
    Color::BLACK,
    Some(Paragraph { text: TQBF.into(), widths: vec!(72., 78., 50., 66., 68.), height: 85., size }),
    Radius::default(), Color::TRANSPARENT, None);

  assert_eq!(
    diagram.nodes,
    vec![Primitive(
      common(None, paragraph1_rect.into()),
      tqbf),
    ]);
}

#[test]
fn layout_node() {
  let string =
    r#"
      box.left "This goes to the left hand side"
      box.right "While this goes to the right hand side" .nw 2cm right from left.ne
      "#;
  let diagram = create_diagram(string);

  let left = diagram.used_rect("left").unwrap();
  let expected = box_used();
  assert_eq!(left, &expected);

  let right = diagram.used_rect("right").unwrap();
  let expected = Rect::from_xywh(164.5, 0.5, 88., 76.);
  assert_eq!(right, &expected);
}

#[test]
fn parse_multiple_directions() {
  let string =
    r#"
      box.left "Left"
      box "Right" .nw 1cm right 2cm down from left.ne
      "#;

  create_diagram(string);

  let _point = Point::new(32., 32.);
  let offset = Vector::new(-1., 0.);
  let result = offset.mul(3.);
  assert_eq!(result, Point::new(-3., 0.));
}

#[test]
fn offset_from_rect() {
  let rect = Rect::from_xywh(40., 40., 40., 40.);
  let movements = vec![
    Displacement::new(2., Unit::Cm, Edge::from("e")),
    Displacement::new(1., Unit::Cm, Edge::from("s")),
  ];
  let result = Index::offset_from_rect(&rect, &Edge::from("nw"), &movements);
  let expected = Rect { left: 116.0, top: 78.0, right: 156.0, bottom: 118.0 };
  assert_eq!(result, expected);
}

fn rectangle(title: Option<(&str, f32)>) -> Shape {
  let paragraph = title.map(|(title, width)| {
    let size = Size::new(88., 17.);
    Paragraph { text: title.into(), widths: vec!(width), height: size.height, size }
  });
  Shape::Box(Color::BLACK, paragraph, Radius::default(), Color::TRANSPARENT, None)
}

