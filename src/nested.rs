#![allow(dead_code)]

use std::error::Error;

use pest::iterators::{Pair, Pairs};
use pest::Parser;
use pest_derive::Parser;
use skia_safe::{Color, PaintStyle, Point, Rect};

use crate::Distance;
use crate::nested::Node::{Container, Primitive};
use crate::skia::Canvas;

type Result<T> = std::result::Result<T, Box<dyn Error>>;

#[derive(Parser)]
#[grammar = "nested.pest"]
pub struct NestedParser;

#[derive(PartialEq)]
#[derive(Debug)]
pub enum Node<'a> {
  Primitive(Option<&'a str>, Rect, Rect, Shape<'a>),
  Container(Option<&'a str>, Rect, Rect, Vec<Node<'a>>),
}

#[derive(Debug)]
#[derive(PartialEq)]
pub enum Shape<'a> {
  Line(Option<&'a str>, &'a str, &'a str),
  Rectangle(Option<&'a str>, Option<(&'a str, Vec<Distance>, (&'a str, &'a str))>),
}

pub struct Compass {
  pub x: f32,
  pub y: f32,
}

impl Compass {
  pub fn convert(string: &str) -> Self {
    match string.to_lowercase().as_str() {
      "n" => Self { x: 0., y: -0.5 },
      "ne" => Self { x: 0.5, y: -0.5 },
      "e" => Self { x: 0.5, y: 0. },
      "se" => Self { x: 0.5, y: 0.5 },
      "s" => Self { x: 0., y: 0.5 },
      "sw" => Self { x: -0.5, y: 0.5 },
      "w" => Self { x: -0.5, y: 0. },
      "nw" => Self { x: -0.5, y: -0.5 },
      _ => Self { x: 0., y: 0. }
    }
  }
}

#[derive(Default)]
pub struct Diagram<'a> {
  pub nodes: Vec<Node<'a>>,
  offset: Point,
}

impl<'i> Diagram<'i> {
  pub fn offset(offset: impl Into<Point>) -> Self {
    Self {
      nodes: vec![],
      offset: offset.into(),
    }
  }

  pub fn parse_string(&mut self, string: &'i str) -> Pairs<'i, Rule> {
    let top = NestedParser::parse(Rule::picture, string).unwrap();
    let mut canvas = Canvas::new(400, 800);
    canvas.cursor = self.offset;
    let mut inside = Rect::from_xywh(0., 0., 400., 800.);
    inside.offset(self.offset);
    let (ast, _bounds) = self.pairs_to_nodes(top.clone(), vec![], &mut canvas, &self.offset);
    self.nodes = ast;
    top
  }

  pub fn pairs_to_nodes<'a>(&self, pairs: Pairs<'a, Rule>, mut ast: Vec<Node<'a>>, canvas: &mut Canvas, offset: &Point)
                            -> (Vec<Node<'a>>, Rect) {
    let mut bounds = Rect::from_xywh(offset.x, offset.y, 0., 0.);

    for pair in pairs.into_iter() {
      match pair.as_rule() {
        Rule::container => {
          let title = rule_to_string(&pair, Rule::inner);
          let mut used = Rect::from_xywh(bounds.left, bounds.bottom, 0., 0.);
          let mut inset = Point::new(used.left, used.bottom);
          inset.offset((BLOCK_PADDING, BLOCK_PADDING));
          let (nodes, inner)
            = self.pairs_to_nodes(pair.into_inner(), vec![], canvas, &inset);
          used.bottom = inner.bottom + BLOCK_PADDING;
          used.right = inner.right + 2. * BLOCK_PADDING;

          if let Some(title) = title {
            let text_inset = inner.with_inset((TEXT_PADDING, TEXT_PADDING));
            let down = canvas.paragraph(title, (0., 0.), text_inset.width());
            used.bottom = inner.bottom + down + TEXT_PADDING;
          }

          let mut rect = used;
          rect.bottom += BLOCK_PADDING;
          ast.push(Container(title, rect, used, nodes));

          bounds.right = bounds.right.max(rect.right);
          bounds.bottom = rect.bottom;
        }
        Rule::line => {
          let id = rule_to_string(&pair, Rule::id);
          let source = rule_to_string(&pair, Rule::source).unwrap();
          let target = rule_to_string(&pair, Rule::target).unwrap();
          let rect = Rect::from_xywh(bounds.left, bounds.bottom, 0., 0.);
          ast.push(Primitive(id, rect, rect, Shape::Line(id, source, target))
          );
        }
        Rule::rectangle => {
          let title = rule_to_string(&pair, Rule::inner);
          let id = rule_to_string(&pair, Rule::id);
          let location = rule_to_location(&pair, Rule::location);
          let height = match title {
            Some(title) => canvas.paragraph(title, (0., 0.), 120. - 2. * TEXT_PADDING),
            None => 40.,
          };

          let mut used = Rect::from_xywh(bounds.left, bounds.bottom, 120., height.max(40.));
          used.bottom += BLOCK_PADDING;
          let mut rect = used;
          rect.bottom += BLOCK_PADDING;
          ast.push(Primitive(id, rect, used, Shape::Rectangle(title, location)));

          bounds.right = bounds.right.max(rect.right);
          bounds.bottom = rect.bottom
        }
        _ => {
          println!("unmatched {:?}", pair);
          let inset = Point::new(bounds.left, bounds.bottom);
          (ast, bounds) = self.pairs_to_nodes(pair.into_inner(), ast, canvas, &inset);
        }
      }
    }
    (ast, bounds)
  }

  pub fn find_node<'a>(&'a self, id: &str) -> Option<&Node<'a>> {
    self.search_nodes(&self.nodes, id)
  }

  fn search_nodes<'a>(&'a self, nodes: &'a [Node], node_id: &str) -> Option<&Node<'a>> {
    for node in nodes.iter() {
      match node {
        Primitive(Some(id), _, _, _) => {
          if id == &node_id {
            return Some(node);
          }
        }
        Container(_, _, _, nodes) => {
          if let Some(node) = self.search_nodes(nodes, node_id) {
            return Some(node);
          };
        }
        _ => {}
      }
    }
    None
  }

  pub fn render(&self, width: i32, height: i32, filepath: &str) {
    let mut canvas = Canvas::new(width, height);
    canvas.cursor = self.offset;
    self.render_nodes(&self.nodes, &mut canvas);
    canvas.write_png(filepath);
  }

  fn render_nodes(&self, nodes: &[Node], canvas: &mut Canvas) {
    for node in nodes.iter() {
      match node {
        Container(title, _rect, used, nodes) => {
          self.render_nodes(nodes, canvas);

          if let Some(title) = title {
            canvas.paint.set_style(PaintStyle::Fill);
            canvas.paint.set_color(Color::BLACK);
            let inset = used.with_inset((TEXT_PADDING, TEXT_PADDING));
            let origin = (inset.left, inset.bottom - 16.);
            canvas.paragraph(title, origin, inset.width());
          }

          canvas.paint.set_style(PaintStyle::Stroke);
          canvas.paint.set_color(Color::RED);
          canvas.rectangle(used);
        }
        Primitive(_id, rect, used, shape) => {
          self.render_shape(shape, rect, used, canvas);
        }
      }
    }
  }

  pub fn layout_node(&self, node: &Node) -> Option<&Rect> {
    match node {
      Primitive(_id, _rect, used, shape) => {
        Some((used, shape))
      }
      _ => None
    }.and_then(|(_used, shape)| {
      match shape {
        Shape::Rectangle(_title, Some(location)) => {
          let (_mycompass, _distance, (other, _compass)) = location;
          self.used_rect(other)
        }
        _ => None,
      }
    })
  }

  pub fn used_rect(&self, id: &str) -> Option<&Rect> {
    self.find_node(id).map(|node| {
      match node {
        Primitive(_, _, used, _) => used,
        _ => panic!("not a primitive")
      }
    })
  }

  fn render_shape(&self, shape: &Shape, rect: &Rect, used: &Rect, canvas: &mut Canvas) -> Rect {
    let mut moved = *used;

    let mut adjust = |other: &Rect, distance: &Distance| {
      moved.top = other.top;
      moved.bottom = moved.top + rect.height();
      moved.left = other.right + distance.pixels();
      moved.right = moved.left + rect.width()
    };

    match shape {
      Shape::Line(_, _, _) => {}
      Shape::Rectangle(title, location) => {
        if let Some(location) = location {
          let (_compass, distance, edge) = location;
          if let Some(node) = self.find_node(edge.0) {
            match node {
              Primitive(_, other, _, _) => adjust(other, distance.first().unwrap()),
              _ => {}
            };
          };
        }

        canvas.paint.set_style(PaintStyle::Stroke);
        canvas.paint.set_color(Color::BLUE);
        canvas.rectangle(&moved);

        if let Some(title) = title {
          canvas.paint.set_style(PaintStyle::Fill);
          canvas.paint.set_color(Color::BLACK);
          let inset = moved.with_inset((TEXT_PADDING, TEXT_PADDING));
          let origin = (inset.left, moved.top);
          canvas.paragraph(title, origin, inset.width());
        }
      }
    }
    moved
  }
}

const BLOCK_PADDING: f32 = 8.;
const TEXT_PADDING: f32 = 4.;

fn rule_to_location<'a>(pair: &Pair<'a, Rule>, rule: Rule)
                        -> Option<(&'a str, Vec<Distance>, (&'a str, &'a str))> {
  find_rule(pair, rule)
    .map(|p| {
      let mut compass: Option<&str> = None;
      let mut directions: Vec<Distance> = vec![];
      let mut edge: Option<(&str, &str)> = None;

      for rule in p.into_inner() {
        match rule.as_rule() {
          Rule::compass => { compass = Some(rule.as_str()); }
          Rule::distance => {
            let distance = pair_to_distance(rule);
            directions.push(distance);
          }
          Rule::edge => { edge = Some(pair_to_edge(rule)); }
          _ => {}
        }
      };
      (compass.unwrap(), directions, edge.unwrap())
    })
}

fn rule_to_distance(pair: &Pair<Rule>, rule: Rule) -> Option<Distance> {
  find_rule(pair, rule).map(pair_to_distance)
}

fn pair_to_distance(pair: Pair<Rule>) -> Distance {
  let length = find_rule(&pair, Rule::length)
    .and_then(|p| p.as_str().parse::<usize>().ok())
    .unwrap();
  let unit = rule_to_string(&pair, Rule::unit)
    .unwrap();
  let direction = rule_to_string(&pair, Rule::direction)
    .unwrap();
  Distance::new(length as f32, unit.to_owned(), direction.to_owned())
}

fn rule_to_edge<'a>(pair: &Pair<'a, Rule>, rule: Rule) -> Option<(&'a str, &'a str)> {
  find_rule(pair, rule).map(pair_to_edge)
}

fn pair_to_edge(pair: Pair<Rule>) -> (&str, &str) {
  (
    rule_to_string(&pair, Rule::id).unwrap(),
    rule_to_string(&pair, Rule::compass).unwrap(),
  )
}

fn rule_to_string<'a>(pair: &Pair<'a, Rule>, rule: Rule) -> Option<&'a str> {
  find_rule(pair, rule)
    .map(|p| p.as_str())
}

fn find_rule<'a>(pair: &Pair<'a, Rule>, rule: Rule) -> Option<Pair<'a, Rule>> {
  pair.clone().into_inner()
    .find(|p| p.as_rule() == rule)
}

#[allow(dead_code)]
pub fn dump_nested(level: usize, pairs: Pairs<Rule>) {
  for pair in pairs.into_iter() {
    println!("{:level$} {:?}", level, pair);
    dump_nested(level + 1, pair.into_inner());
  }
}

