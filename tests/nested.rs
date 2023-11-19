#[cfg(test)]
mod tests {
  use std::fs;

  use pest::iterators::Pair;
  use pest::Parser;

  use picturs::{dump_pic, parse_nodes, PicParser, Rule};
  use picturs::Node::{Container, Primitive};
  use picturs::Shape::{Arc, Arrow, Box};

  fn parse_pic(string: &str) -> Pair<Rule> {
    PicParser::parse(Rule::picture, &*string).unwrap().next().unwrap()
  }

  #[test]
  fn it_dumps_hierarchy() {
    let string = fs::read_to_string("tests/nested.pic").unwrap();
    let pair = parse_pic(&*string);
    dump_pic(1, pair);
  }

  #[test]
  fn it_parses_containers() {
    let string = fs::read_to_string("tests/nested.pic").unwrap();
    let pair = parse_pic(&*string);
    let nodes = parse_nodes(pair, vec![]);

    assert_eq!(nodes, vec![Container(vec![
      Primitive(Arc, vec![]),
      Primitive(Arrow, vec![]),
      Primitive(Box, vec![]),
    ], vec![])]);
  }
}