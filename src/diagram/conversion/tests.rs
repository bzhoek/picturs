use crate::diagram::conversion::Conversion;
use crate::diagram::parser::Rule;
use crate::diagram::types::{Edge, ObjectEdge};

mod string {
  use super::*;

  #[test]
  fn newline() {
    let string = "\"ソフトウェア製品生産管理: \\nソフトウェア工学における\"";
    let string = subject(string);
    assert_eq!(string.as_str(), "ソフトウェア製品生産管理: \nソフトウェア工学における");
  }

  fn subject(string: &str) -> String {
    let pair = Conversion::pair_for(Rule::string, string);
    Conversion::string_from(pair)
  }
}

mod thickness {
  use super::*;

  #[test]
  fn thick() {
    let thickness = subject("thick");
    assert_eq!(thickness, 3.);
  }

  fn subject(string: &str) -> f32 {
    let pair = Conversion::pair_for(Rule::thickness, string);
    Conversion::thickness_from(pair)
  }
}

mod endings {
  use crate::diagram::types::{Ending, Endings};

  use super::*;

  #[test]
  fn mixed_endings() {
    let endings = subject("*->");
    assert_eq!(endings, Endings { start: Ending::Dot, end: Ending::Arrow });
  }

  fn subject(string: &str) -> Endings {
    let pair = Conversion::pair_for(Rule::endings, string);
    Conversion::endings_from(pair)
  }
}

mod colors {
  use skia_safe::Color;

  use crate::diagram::conversion::Conversion;
  use crate::diagram::parser::Rule;

  #[test]
  fn named_color() {
    let color = subject("color red");
    assert_eq!(color, Some(Color::RED));
  }

  #[test]
  fn rgb_color() {
    let color = subject("color #645590");
    assert_eq!(color, Some(Color::from(0xFF645590)));
  }

  fn subject(string: &str) -> Option<Color> {
    let pair = Conversion::pair_for(Rule::stroke, string);
    Conversion::color_from(pair)
  }
}

mod object_edge_degrees {
  use super::*;

  #[test]
  fn with_vertical_fraction() {
    let object = subject("a.e");
    assert_eq!(object, ObjectEdge::new("a", Edge::right()));

    let object = subject("a.90");
    assert_eq!(object, ObjectEdge::new("a", Edge::right()));

    let object = subject("a.s");
    assert_eq!(object, ObjectEdge::new("a", Edge::below()));

    let object = subject("a.180");
    assert_eq!(object, ObjectEdge::new("a", Edge::below()));
  }

  fn subject(string: &str) -> ObjectEdge {
    let pair = Conversion::pair_for(Rule::object_edge, string);
    Conversion::object_edge_from_degrees(pair)
  }
}

mod object_edge {
  use super::*;

  #[test]
  fn with_vertical_fraction() {
    let object = subject("a.w");
    assert_eq!(object, ObjectEdge::new("a", Edge::left()));
  }

  fn subject(string: &str) -> ObjectEdge {
    let pair = Conversion::pair_for(Rule::object_edge, string);
    Conversion::object_edge_from(pair)
  }
}

mod fractions {
  use super::*;

  #[test]
  fn with_horizontal_fraction() {
    let object = subject("from 1/4 a.s").unwrap();
    let mut edge = Edge::below();
    edge.x = -0.25;
    assert_eq!(object, ObjectEdge::new("a", edge));
  }

  #[test]
  fn with_vertical_fraction() {
    let object = subject("from 1/4 a.w").unwrap();
    let mut edge = Edge::left();
    edge.y = -0.25;
    assert_eq!(object, ObjectEdge::new("a", edge));
  }

  fn subject(string: &str) -> Option<ObjectEdge> {
    let pair = Conversion::pair_for(Rule::line_attributes, string);
    Conversion::fraction_edge_for(&pair, Rule::source)
  }
}