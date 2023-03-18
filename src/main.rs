use std::error::Error;
use std::fs;

use pest::iterators::Pair;
use pest::Parser;
use pest_derive::Parser;

#[derive(Parser)]
#[grammar = "pic.pest"] // relative to project `src`
struct PicParser;

type Result<T> = std::result::Result<T, Box<dyn Error>>;

#[derive(Debug)]
pub enum ShapeType {
  Arrow,
  Box,
}

#[derive(Debug)]
struct Shape {
  class: Option<ShapeType>,
  text: Vec<String>,
}

fn object_definition(pair: Pair<Rule>, mut shape: Shape) -> Shape {
  for pair in pair.into_inner() {
    match pair.as_rule() {
      Rule::object_class => {
        shape.class = Option::from(match pair.as_str() {
          "arrow" => ShapeType::Arrow,
          "box" => ShapeType::Box,
          _ => unreachable!()
        })
      }
      Rule::attribute => {
        shape = object_definition(pair, shape);
      }
      Rule::string => {
        shape.text.push(pair.into_inner().as_str().to_string());
      }
      _ => { println!("{:?}", pair) }
    }
  }
  shape
}

fn main() {
  let string = fs::read_to_string("pikchr.pic").unwrap();
  let pairs = PicParser::parse(Rule::picture, &*string).unwrap();

  for pair in pairs {
    for inner in pair.into_inner() {
      let mut shape = Shape { class: None, text: Vec::new() };
      let shape = object_definition(inner, shape);
      println!("{:?}", shape);
    }
  }
}


#[cfg(test)]
mod tests {
  #[test]
  fn it_works() {
    let result = 2 + 2;
    assert_eq!(result, 4);
  }
}