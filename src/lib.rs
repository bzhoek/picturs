use std::error::Error;
use pest::iterators::{Pair, Pairs};
use pest_derive::Parser;

#[allow(dead_code)]
type Result<T> = std::result::Result<T, Box<dyn Error>>;

#[derive(Parser)]
#[grammar = "pic.pest"] // relative to project `src`
pub struct PicParser;

#[derive(Debug)]
pub enum ShapeType {
  Arrow,
  Box,
  Unset,
}

#[derive(Debug)]
pub struct Shape {
  class: ShapeType,
  text: Vec<String>,
  path: Vec<String>,
  pub fit: bool,
  pub same: bool, // same styling for this /**/`class`
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

pub fn parse_shape(pair: Pair<Rule>, mut shape: Shape) -> Shape {
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
        shape = parse_shape(pair, shape);
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

pub fn shapes(pairs: Pairs<Rule>) -> Vec<Shape> {
  pairs.map(|pair|
    pair.into_inner()
  ).flat_map(|inners|
    inners.map(|inner| parse_shape(inner, Shape::default()))
  ).collect::<Vec<_>>()
}