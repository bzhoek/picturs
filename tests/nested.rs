#[cfg(test)]
mod tests {
  use std::fs;

  use pest::Parser;

  use picturs::{dump_pic, parse_nodes, PicParser, Rule};
  use picturs::Node::{Container, Primitive};
  use picturs::Shape::{Arc, Arrow, Box};

  #[test]
  fn it_dumps_hierarchy() {
    let string = fs::read_to_string("tests/nested.pic").unwrap();
    let pair = PicParser::parse(Rule::picture, &*string).unwrap().next().unwrap();
    dump_pic(1, pair);
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