#[cfg(test)]
mod tests {
  use std::fs;

  use pest::Parser;

  use picturs::{dump_rules, Node, parse_nodes, PicParser, Rule, shapes, ShapeType};
  use picturs::Node::{Attribute, Container, Primitive, String};
  use picturs::ShapeType::{Arc, Arrow, Box};

  #[test]
  fn it_dumps_hierarchy() {
    let string = fs::read_to_string("tests/nested.pic").unwrap();
    let pair = PicParser::parse(Rule::picture, &*string).unwrap().next().unwrap();
    dump_rules(1, pair);
  }

  #[test]
  fn it_parses_containers() {
    let string = fs::read_to_string("tests/nested.pic").unwrap();
    let pair = PicParser::parse(Rule::picture, &*string).unwrap().next().unwrap();
    let nodes = parse_nodes(pair, vec![]);

    assert_eq!(nodes, vec![Container(vec![
      Primitive(Arc, vec![]),
      Primitive(Arrow, vec![]),
      Primitive(Box, vec![]),
    ], vec![])]);
  }
}