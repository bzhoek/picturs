use std::fs;

use pest::Parser;

use picturs::{parse_shape, PicParser, Rule, Shape};

fn main() {
  let string = fs::read_to_string("pikchr.pic").unwrap();
  let pairs = PicParser::parse(Rule::picture, &*string).unwrap();

  for pair in pairs {
    for inner in pair.into_inner() {
      let shape = parse_shape(inner, Shape::default());
      println!("{:?}", shape);
    }
  }
}

