use std::error::Error;
use std::fs;

use pest::iterators::{Pair, Pairs};
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
  Unset,
}

#[derive(Debug)]
struct Shape {
  class: ShapeType,
  text: Vec<String>,
  path: Vec<String>,
  fit: bool,
  same: bool, // same styling for this `class`
}

impl Default for Shape {
  fn default() -> Self {
    Shape {
      class: ShapeType::Unset,
      text: Vec::new(),
      path: Vec::new(),
      fit: false,
      same: false,
    }
  }
}

fn object_definition(pair: Pair<Rule>, mut shape: Shape) -> Shape {
  for pair in pair.into_inner() {
    match pair.as_rule() {
      Rule::object_class => {
        shape.class = match pair.as_str() {
          "arrow" => ShapeType::Arrow,
          "box" => ShapeType::Box,
          _ => unreachable!()
        }
      }
      Rule::attribute => {
        println!("{:?}", pair);
        shape = object_definition(pair, shape);
      }
      Rule::same_attribute => {
        shape.same = true;
      }
      Rule::size_attribute => {
        shape.fit = pair.as_str().eq("fit");
      }
      Rule::path_attribute => {
        shape.path.push(pair.into_inner().as_str().to_string());
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
      let shape = object_definition(inner, Shape::default());
      println!("{:?}", shape);
    }
  }
}

fn shapes(pairs: Pairs<Rule>) -> Vec<Shape> {
  pairs.map(|pair|
    pair.into_inner()
  ).flat_map(|inners|
    inners.map(|inner| object_definition(inner, Shape::default()))
  ).collect::<Vec<_>>()
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn it_has_five_shapes() {
    let string = fs::read_to_string("pikchr.pic").unwrap();
    let pairs = PicParser::parse(Rule::picture, &*string).unwrap();
    let result = shapes(pairs);
    assert_eq!(result.len(), 5);
    let first = result.first().unwrap();
    assert_eq!(first.fit, false);
    assert_eq!(first.same, false);
    let last = result.last().unwrap();
    assert_eq!(last.fit, true);
    assert_eq!(last.same, true);
    println!("{:?}", result);
  }
}