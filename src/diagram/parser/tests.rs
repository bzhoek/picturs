use std::ops::Mul;

use skia_safe::{Point, Rect, Vector};

use crate::diagram::conversion::Conversion;
use crate::diagram::create_diagram;
use crate::diagram::index::Index;
use crate::diagram::parser::{Diagram, Rule};
use crate::diagram::types::{Config, Displacement, Edge, Node, Unit};

// static TQBF: &str = "the quick brown fox jumps over the lazy dog";

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

fn box_used() -> Rect {
  Rect::from_xywh(0.5, 0.5, 88., 67.)
}

#[test]
fn should_parse_font() {
  let string = r#"set font "Menlo" 15pt"#;
  let diagram = create_diagram(string);
  let node = diagram.nodes.first().unwrap();
  match node {
    Node::Font(font) => {
      assert_eq!("Menlo", font.typeface().unwrap().family_name());
      assert_eq!(15., font.size());
    }
    _ => panic!("Expected Font")
  }
}

#[ignore]
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


