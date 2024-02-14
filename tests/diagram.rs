#[cfg(test)]
mod diagram {
  use std::ops::{Mul, Sub};

  use skia_safe::{Color, Point, Rect, Size, Vector};
  use picturs::create_diagram;

  use picturs::diagram::index::Index;
  use picturs::diagram::types::{Edge, Movement, Node, Paragraph, Radius, Shape, Unit};
  use picturs::diagram::types::Node::{Container, Primitive};
  use picturs::diagram::types::Shape::Rectangle;

  static TQBF: &str = "the quick brown fox jumps over the lazy dog";

  fn rectangle(title: Option<(&str, f32)>) -> Shape {
    let paragraph = title.map(|(title, width)| {
      let size = Size::new(120., 17.);
      Paragraph { text: title, widths: vec!(width), height: size.height, size }
    });
    Rectangle(Color::BLACK, paragraph, Radius::default(), Color::TRANSPARENT, None)
  }

  #[test]
  fn single_box_untitled() {
    let string = r#"box"#;
    let diagram = create_diagram(string);

    assert_eq!(vec![
      Primitive(None,
                Rect::from_xywh(0., 0., 120., 76.),
                Rect::from_xywh(0., 0., 120., 68.),
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
                Rect::from_xywh(0., 0., 120., 76.),
                Rect::from_xywh(0., 0., 120., 68.),
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
                Rect::from_xywh(0., 0., 120., 76.),
                Rect::from_xywh(0., 0., 120., 68.),
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
                Rect::from_xywh(0., 0., 120., 76.),
                Rect::from_xywh(0., 0., 120., 68.),
                Color::BLUE,
                rectangle(None)),
      Primitive(None,
                Rect::from_xywh(0., 76., 120., 76.),
                Rect::from_xywh(0., 76., 120., 68.),
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
                Rect::from_xywh(0., 0., 136., 100.),
                Rect::from_xywh(0., 0., 136., 92.),
                vec![
                  Primitive(None,
                            Rect::from_xywh(8., 8., 120., 76.),
                            Rect::from_xywh(8., 8., 120., 68.),
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
                Rect::from_xywh(0., 0., 136., 113.),
                Rect::from_xywh(0., 0., 136., 105.),
                vec![
                  Primitive(None,
                            Rect::from_xywh(8., 8., 120., 76.),
                            Rect::from_xywh(8., 8., 120., 68.),
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
    let size = Size::new(120., 68.);

    let tqbf = Rectangle(Color::BLACK, Some(Paragraph { text: TQBF, widths: vec!(72., 78., 115., 68.), height: 68., size }), Radius::default(), Color::TRANSPARENT, None);

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
  fn layout_node() {
    let string =
      r#"
      box.left "This goes to the left hand side"
      box.right "While this goes to the right hand side" .nw 2cm right from left.ne
      "#;
    let diagram = create_diagram(string);

    let left = diagram.used_rect("left").unwrap();
    let expected = Rect { left: 0., top: 0., right: 120., bottom: 68. };
    assert_eq!(&expected, left);

    let right = diagram.used_rect("right").unwrap();
    let expected = Rect::from_xywh(196., 0., 120., 68.);
    assert_eq!(&expected, right);
  }

  #[test]
  fn to_edge() {
    let rect = Rect::from_xywh(40., 40., 100., 200.);

    let edge = Edge::from("nw");
    let center = rect.center();
    assert_eq!(Point::new(90., 140.), center);

    let nw = edge.edge_point(&rect);
    assert_eq!(Point::new(40., 40.), nw);

    let edge = Edge::from("se");
    let se = edge.edge_point(&rect);
    assert_eq!(Point::new(140., 240.), se);
  }

  #[test]
  fn to_edge_from_negative() {
    let rect = Rect { left: 0.0, top: -30.0, right: 360.0, bottom: 30.0 };

    let edge = Edge::from("sw");
    let center = rect.center();
    assert_eq!(Point::new(180., 0.), center);

    let nw = edge.edge_point(&rect);
    assert_eq!(Point::new(0., 30.), nw);
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
    assert_eq!(Point::new(-3., 0.), result);
  }

  #[test]
  fn offset_from_rect() {
    let rect = Rect::from_xywh(40., 40., 40., 40.);
    let movements = vec![
      Movement::new(2., Unit::Cm, Edge::from("e")),
      Movement::new(1., Unit::Cm, Edge::from("s")),
    ];
    let result = Index::offset_from_rect(&rect, &Edge::from("nw"), &movements);
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
                                  Rect::from_xywh(0., 0., 120., 76.),
                                  Rect::from_xywh(0., 0., 120., 68.),
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