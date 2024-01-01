#![allow(dead_code)]

use std::error::Error;
use std::ops::Add;

use pest::iterators::{Pair, Pairs};
use pest::Parser;
use pest_derive::Parser;
use skia_safe::{Color, PaintStyle, Point, Rect, Vector};

use crate::{Distance, Edge};
use crate::diagram::Node::{Container, Primitive};
use crate::skia::Canvas;

type Result<T> = std::result::Result<T, Box<dyn Error>>;

#[derive(Parser)]
#[grammar = "diagram.pest"]
pub struct NestedParser;

#[derive(PartialEq)]
#[derive(Debug)]
pub enum Node<'a> {
  Container(Option<&'a str>, Option<&'a str>, Rect, Rect, Vec<Node<'a>>),
  Primitive(Option<&'a str>, Rect, Rect, Shape<'a>),
}

type Displacement = (Anchor, Vec<Distance>, Edge);

#[derive(Debug)]
#[derive(PartialEq)]
pub enum Shape<'a> {
  Line(Option<&'a str>, &'a str, &'a str),
  Rectangle(Option<&'a str>),
}

#[derive(Debug)]
#[derive(PartialEq)]
pub struct Anchor {
  pub x: f32,
  pub y: f32,
}

impl Anchor {
  pub fn new(string: &str) -> Self {
    let dot_removed = string.trim_start_matches('.');
    match dot_removed.to_lowercase().as_str() {
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

  pub fn to_tuple(&self) -> (f32, f32) {
    (self.x, self.y)
  }

  pub fn to_edge(&self, rect: &Rect) -> Point {
    let mut point = rect.center();
    point.offset((self.x * rect.width(), self.y * rect.height()));
    point
  }

  pub fn topleft_offset(&self, rect: &Rect) -> Point {
    let point = Point::new(self.x, self.y);
    let point = point.add(Vector::new(0.5, 0.5));
    Point::new(rect.width() * -point.x, rect.height() * -point.y)
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
    let (ast, _bounds) = Diagram::pairs_to_nodes(top.clone(), vec![], &mut canvas, &self.offset);
    self.nodes = ast;
    top
  }

  pub fn pairs_to_nodes<'a>(pairs: Pairs<'a, Rule>, mut ast: Vec<Node<'a>>, canvas: &mut Canvas, offset: &Point)
                            -> (Vec<Node<'a>>, Rect) {
    let mut bounds = Rect::from_xywh(offset.x, offset.y, 0., 0.);

    for pair in pairs.into_iter() {
      match pair.as_rule() {
        Rule::container => {
          let id = Diagram::rule_to_string(&pair, Rule::id);
          let title = Diagram::rule_to_string(&pair, Rule::inner);
          let location = Diagram::rule_to_location(&pair, Rule::location);

          let mut used = Rect::from_xywh(bounds.left, bounds.bottom, 0., 0.);

          if let Some((anchor, distances, edge)) = &location {
            if let Some(rect) = Diagram::offset_in(&ast, edge, distances) {
              used = Rect::from_xywh(rect.left, rect.top, used.width(), used.height());
              let offset = anchor.topleft_offset(&used);
              used.offset(offset);
            }
          }

          let mut inset = Point::new(used.left, used.bottom);
          inset.offset((BLOCK_PADDING, BLOCK_PADDING));
          let (nodes, inner)
            = Diagram::pairs_to_nodes(pair.into_inner(), vec![], canvas, &inset);
          used.bottom = inner.bottom + BLOCK_PADDING;
          used.right = inner.right + 2. * BLOCK_PADDING;

          if let Some(title) = title {
            let text_inset = inner.with_inset((TEXT_PADDING, TEXT_PADDING));
            let down = canvas.paragraph(title, (0., 0.), text_inset.width());
            used.bottom = inner.bottom + down + TEXT_PADDING;
          }

          let mut rect = used;
          rect.bottom += BLOCK_PADDING;
          ast.push(Container(id, title, rect, used, nodes));

          bounds.right = bounds.right.max(rect.right);
          bounds.bottom = rect.bottom;
        }
        Rule::line => {
          let id = Diagram::rule_to_string(&pair, Rule::id);
          let source = Diagram::rule_to_string(&pair, Rule::source).unwrap();
          let target = Diagram::rule_to_string(&pair, Rule::target).unwrap();
          let rect = Rect::from_xywh(bounds.left, bounds.bottom, 0., 0.);
          ast.push(Primitive(id, rect, rect, Shape::Line(id, source, target))
          );
        }
        Rule::rectangle => {
          let title = Diagram::rule_to_string(&pair, Rule::inner);
          let id = Diagram::rule_to_string(&pair, Rule::id);
          let location = Diagram::rule_to_location(&pair, Rule::location);
          let height = match title {
            Some(title) => canvas.paragraph(title, (0., 0.), 120. - 2. * TEXT_PADDING),
            None => 40.,
          };

          let mut used = Rect::from_xywh(bounds.left, bounds.bottom, 120., height.max(40.));
          used.bottom += BLOCK_PADDING;

          if let Some((anchor, distances, edge)) = &location {
            if let Some(rect) = Diagram::offset_in(&ast, edge, distances) {
              used = Rect::from_xywh(rect.left, rect.top, used.width(), used.height());
              let offset = anchor.topleft_offset(&used);
              used.offset(offset);
            }
          }

          let mut rect = used;
          rect.bottom += BLOCK_PADDING;
          ast.push(Primitive(id, rect, used, Shape::Rectangle(title)));

          bounds.right = bounds.right.max(rect.right);
          bounds.bottom = rect.bottom
        }
        _ => {
          println!("unmatched {:?}", pair);
          // let inset = Point::new(bounds.left, bounds.bottom);
          // (ast, bounds) = Diagram::pairs_to_nodes(pair.into_inner(), ast, canvas, &inset);
        }
      }
    }
    (ast, bounds)
  }

  fn find_node<'a>(&'a self, id: &str) -> Option<&Node<'a>> {
    Diagram::find_nodes(&self.nodes, id)
  }

  pub fn used_rect(&self, id: &str) -> Option<&Rect> {
    self.find_node(id).map(|node| {
      match node {
        Primitive(_, _, used, _) => used,
        _ => panic!("not a primitive")
      }
    })
  }

  fn find_nodes<'a>(nodes: &'a [Node], node_id: &str) -> Option<&'a Node<'a>> {
    nodes.iter().find(|node| {
      match node {
        Primitive(Some(id), _, _, _) => {
          id == &node_id
        }
        Container(id, _, _, _, nodes) => {
          if let Some(id) = id {
            if id == &node_id {
              return true;
            }
          }
          Diagram::find_nodes(nodes, node_id).is_some()
        }
        _ => false
      }
    })
  }

  pub fn offset_from(&self, edge: &Edge, distances: &[Distance]) -> Option<Rect> {
    Diagram::offset_in(&self.nodes, edge, distances)
  }

  pub fn offset_in(nodes: &[Node], edge: &Edge, distances: &[Distance]) -> Option<Rect> {
    Diagram::find_nodes(nodes, &edge.id).map(|node| {
      match node {
        Primitive(_, _, used, _) => Diagram::offset_from_rect(used, &edge.anchor, distances),
        Container(_, _, _, used, _) => Diagram::offset_from_rect(used, &edge.anchor, distances),
      }
    })
  }

  pub fn offset_from_rect(rect: &Rect, anchor: &Anchor, distances: &[Distance]) -> Rect {
    let point = anchor.to_edge(rect);
    let mut rect = Rect::from_xywh(point.x, point.y, rect.width(), rect.height());
    for distance in distances.iter() {
      rect.offset(distance.offset());
    }
    rect
  }

  pub fn node_mut(&mut self, id: &str, distances: Vec<Distance>) {
    match Diagram::find_nodes_mut(&mut self.nodes, id).unwrap() {
      Primitive(_, _, ref mut rect, _) => {
        for distance in distances.iter() {
          rect.offset(distance.offset());
        }
      }
      _ => {}
    }
  }

  fn find_nodes_mut<'a: 'i>(nodes: &'i mut [Node<'a>], node_id: &str) -> Option<&'i mut Node<'a>> {
    for node in nodes.iter_mut() {
      match node {
        Primitive(Some(id), _, _, _) => {
          if id == &node_id {
            return Some(node);
          }
        }
        Container(_, _, _, _, nodes) => {
          if let Some(node) = Diagram::find_nodes_mut(nodes, node_id) {
            return Some(node);
          }
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
        Container(id, title, _rect, used, nodes) => {
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
        Primitive(_id, _, used, shape) => {
          self.render_shape(shape, used, canvas);
        }
      }
    }
  }

  fn render_shape(&self, shape: &Shape, used: &Rect, canvas: &mut Canvas) {
    match shape {
      Shape::Line(_, _, _) => {}
      Shape::Rectangle(title) => {
        canvas.paint.set_style(PaintStyle::Stroke);
        canvas.paint.set_color(Color::BLUE);
        canvas.rectangle(used);

        if let Some(title) = title {
          canvas.paint.set_style(PaintStyle::Fill);
          canvas.paint.set_color(Color::BLACK);
          let inset = used.with_inset((TEXT_PADDING, TEXT_PADDING));
          let origin = (inset.left, used.top);
          canvas.paragraph(title, origin, inset.width());
        }
      }
    }
  }

  fn rule_to_location(pair: &Pair<Rule>, rule: Rule) -> Option<(Anchor, Vec<Distance>, Edge)> {
    Diagram::find_rule(pair, rule)
      .map(|p| {
        let mut anchor: Option<Anchor> = None;
        let mut directions: Vec<Distance> = vec![];
        let mut edge: Option<Edge> = None;

        for rule in p.into_inner() {
          match rule.as_rule() {
            Rule::anchor => { anchor = Some(Anchor::new(rule.as_str())); }
            Rule::distance => {
              let distance = Diagram::pair_to_distance(rule);
              directions.push(distance);
            }
            Rule::edge => { edge = Some(Diagram::pair_to_edge(rule)); }
            _ => {}
          }
        };
        (anchor.unwrap(), directions, edge.unwrap())
      })
  }

  fn rule_to_distance(pair: &Pair<Rule>, rule: Rule) -> Option<Distance> {
    Diagram::find_rule(pair, rule).map(Diagram::pair_to_distance)
  }

  fn pair_to_distance(pair: Pair<Rule>) -> Distance {
    let length = Diagram::find_rule(&pair, Rule::length)
      .and_then(|p| p.as_str().parse::<usize>().ok())
      .unwrap();
    let unit = Diagram::rule_to_string(&pair, Rule::unit)
      .unwrap();
    let direction = match Diagram::rule_to_string(&pair, Rule::direction).unwrap() {
      "left" => Vector::new(-1., 0.),
      "right" => Vector::new(1., 0.),
      "up" => Vector::new(0., -1.),
      _ => Vector::new(0., 1.),
    };
    Distance::new(length as f32, unit.into(), direction)
  }

  fn rule_to_edge(pair: &Pair<Rule>, rule: Rule) -> Option<Edge> {
    Diagram::find_rule(pair, rule).map(Diagram::pair_to_edge)
  }

  fn pair_to_edge(pair: Pair<Rule>) -> Edge {
    let id = Diagram::rule_to_string(&pair, Rule::id).unwrap();
    let anchor = Diagram::rule_to_string(&pair, Rule::anchor).unwrap();
    Edge::new(id, anchor)
  }

  fn rule_to_string<'a>(pair: &Pair<'a, Rule>, rule: Rule) -> Option<&'a str> {
    Diagram::find_rule(pair, rule)
      .map(|p| p.as_str())
  }

  fn find_rule<'a>(pair: &Pair<'a, Rule>, rule: Rule) -> Option<Pair<'a, Rule>> {
    pair.clone().into_inner()
      .find(|p| p.as_rule() == rule)
  }
}

const BLOCK_PADDING: f32 = 8.;
const TEXT_PADDING: f32 = 4.;

#[allow(dead_code)]
pub fn dump_nested(level: usize, pairs: Pairs<Rule>) {
  for pair in pairs.into_iter() {
    println!("{:level$} {:?}", level, pair);
    dump_nested(level + 1, pair.into_inner());
  }
}

