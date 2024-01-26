#![allow(dead_code)]

use std::collections::HashMap;
use std::error::Error;
use std::f32::consts::PI;
use std::ops::{Add, Sub};

use log::{error, warn};
use pest::iterators::{Pair, Pairs};
use pest::Parser;
use pest_derive::Parser;
use skia_safe::{Color, ISize, PaintStyle, Point, Rect, Vector};

use crate::diagram::Node::{Container, Primitive};
use crate::skia::Canvas;
use crate::types::{Anchor, Distance, Edge, Length};

pub static A5: (i32, i32) = (798, 562);

pub type Radius = Length;

type Result<T> = std::result::Result<T, Box<dyn Error>>;

#[derive(Parser)]
#[grammar = "diagram.pest"]
pub struct DiagramParser;

#[derive(Debug, PartialEq)]
pub enum Node<'a> {
  Container(Option<&'a str>, Radius, Option<&'a str>, Rect, Rect, Vec<Node<'a>>),
  Primitive(Option<&'a str>, Rect, Rect, Color, Shape<'a>),
}

type Displacement = (Anchor, Vec<Distance>, Edge);

#[derive(Debug, PartialEq)]
pub enum Shape<'a> {
  Dot(Edge, Radius),
  Arrow(Option<&'a str>, Edge, Option<Distance>, Edge),
  Line(Option<&'a str>, Edge, Option<Distance>, Edge),
  Rectangle(Color, Option<Paragraph<'a>>, Radius, Color, Option<Displacement>),
  Text(&'a str, Option<Displacement>),
}

#[derive(Debug, PartialEq)]
pub struct Paragraph<'a> {
  pub text: &'a str,
  pub widths: Vec<f32>,
  pub height: f32,
}

#[derive(Debug)]
pub struct Diagram<'a> {
  pub nodes: Vec<Node<'a>>,
  pub index: HashMap<String, Rect>,
  size: ISize,
  offset: Point,
}

impl<'i> Diagram<'i> {
  pub fn offset(size: impl Into<ISize>, offset: impl Into<Point>) -> Self {
    Self {
      nodes: vec![],
      index: HashMap::new(),
      size: size.into(),
      offset: offset.into(),
    }
  }

  pub fn parse_string(&mut self, string: &'i str) -> Pairs<'i, Rule> {
    let top = DiagramParser::parse(Rule::picture, string).unwrap();
    let mut canvas = Canvas::new(self.size);
    canvas.cursor = self.offset;
    let (ast, _bounds) = Self::pairs_to_nodes(top.clone(), vec![], &mut canvas, &self.offset, &mut self.index);
    self.nodes = ast;
    top
  }

  pub fn pairs_to_nodes<'a>(pairs: Pairs<'a, Rule>, mut ast: Vec<Node<'a>>, canvas: &mut Canvas, offset: &Point, index: &mut HashMap<String, Rect>)
    -> (Vec<Node<'a>>, Rect) {
    let mut bounds = Rect::from_xywh(offset.x, offset.y, 0., 0.);

    for pair in pairs.into_iter() {
      match pair.as_rule() {
        Rule::move_to => {
          let distances = Self::rule_to_distances(&pair, Rule::distances);
          distances.iter().for_each(|distances| {
            Self::offset_rect(&mut bounds, distances)
          });
        }
        Rule::container => {
          let id = Self::rule_to_string(&pair, Rule::id);
          let attributes = Self::find_rule(&pair, Rule::attributes).unwrap();
          let (_width, _height, radius) = Self::parse_dimension(&attributes);
          let title = Self::rule_to_string(&attributes, Rule::inner);
          let location = Self::rule_to_location(&attributes, Rule::location);

          let mut used = Rect::from_xywh(bounds.left, bounds.bottom, 0., 0.);
          Self::position_rect(index, &location, &mut used);

          let mut inset = Point::new(used.left, used.bottom);
          inset.offset((BLOCK_PADDING, BLOCK_PADDING));
          let (nodes, inner)
            = Self::pairs_to_nodes(pair.into_inner(), vec![], canvas, &inset, index);
          used.top = inner.top - BLOCK_PADDING;
          used.left = inner.left - BLOCK_PADDING;
          used.bottom = inner.bottom + BLOCK_PADDING;
          used.right = inner.right + 2. * BLOCK_PADDING;

          if let Some(title) = title {
            let text_inset = inner.with_inset((TEXT_PADDING, TEXT_PADDING));
            let (_widths, down) = canvas.paragraph(title, (0., 0.), text_inset.width());
            used.bottom = inner.bottom + down + TEXT_PADDING;
          }

          if let Some(id) = id {
            index.insert(id.into(), used);
          }

          let mut rect = used;
          rect.bottom += BLOCK_PADDING;
          ast.push(Container(id, radius, title, rect, used, nodes));

          bounds.left = rect.left;
          bounds.right = bounds.right.max(rect.right);
          bounds.bottom = rect.bottom;
        }
        Rule::dot => {
          let attributes = Self::find_rule(&pair, Rule::dot_attributes).unwrap();
          let color = Self::rule_to_color(&attributes, Rule::color).unwrap_or(Color::BLUE);
          let radius = Self::rule_to_radius(&attributes);

          let target = Self::location_to_edge(&pair, Rule::target).unwrap();
          let point = Self::point_index(index, &target, &[]).unwrap();
          let rect = Rect::from_xywh(point.x, point.y, 0., 0.);
          ast.push(Primitive(None, rect, rect, color, Shape::Dot(target, radius)));
        }
        Rule::arrow => {
          let id = Self::rule_to_string(&pair, Rule::id);

          let source = Self::location_to_edge(&pair, Rule::source).unwrap();
          let distance = Self::rule_to_distance(&pair, Rule::distance);
          let target = Self::location_to_edge(&pair, Rule::target).unwrap();

          let start = Self::point_index(index, &source, &[]);
          let end = Self::point_index(index, &target, &[]);

          if let Some((start, end)) = start.zip(end) {
            let rect = Rect { left: start.x, top: start.y, right: end.x, bottom: end.y };
            ast.push(Primitive(id, rect, rect, Color::BLACK, Shape::Arrow(id, source, distance, target)));
          }
        }
        Rule::line => {
          let id = Self::rule_to_string(&pair, Rule::id);

          let source = Self::location_to_edge(&pair, Rule::source).unwrap();
          let distance = Self::rule_to_distance(&pair, Rule::distance);
          let target = Self::location_to_edge(&pair, Rule::target).unwrap();

          let start = Self::point_index(index, &source, &[]);
          let end = Self::point_index(index, &target, &[]);

          if let Some((start, end)) = start.zip(end) {
            let rect = Rect { left: start.x, top: start.y, right: end.x, bottom: end.y };
            ast.push(Primitive(id, rect, rect, Color::BLACK, Shape::Line(id, source, distance, target)));
          }
        }
        Rule::rectangle => {
          let id = Self::rule_to_string(&pair, Rule::id);
          let attributes = Self::find_rule(&pair, Rule::attributes).unwrap();
          let (width, height, radius) = Self::parse_dimension(&attributes);

          let stroke = Self::rule_to_color(&attributes, Rule::color).unwrap_or(Color::BLUE);
          let fill = Self::rule_to_color(&attributes, Rule::fill).unwrap_or(Color::TRANSPARENT);
          let text_color = Self::rule_to_color(&attributes, Rule::text_color).unwrap_or(Color::BLACK);
          let title = Self::rule_to_string(&attributes, Rule::inner);
          let location = Self::rule_to_location(&attributes, Rule::location);

          let mut para_height = None;
          let paragraph = title.map(|title| {
            let (widths, height) = canvas.paragraph(title, (0., 0.), width - 2. * TEXT_PADDING);
            para_height = Some(height);
            Paragraph { text: title, widths, height }
          });

          let height = height.unwrap_or_else(|| {
            para_height.unwrap_or(40.)
          });

          let mut used = Rect::from_xywh(bounds.left, bounds.bottom, width, height.max(40.));
          used.bottom += BLOCK_PADDING;
          Self::position_rect(index, &location, &mut used);

          if let Some(id) = id {
            index.insert(id.into(), used);
          }

          let mut rect = used;
          rect.bottom += BLOCK_PADDING;
          ast.push(Primitive(id, rect, used, stroke, Shape::Rectangle(text_color, paragraph, radius, fill, location)));

          bounds.top = bounds.top.min(rect.top);
          bounds.left = rect.left;
          bounds.right = bounds.right.max(rect.right);
          bounds.bottom = rect.bottom
        }
        Rule::text => {
          let id = Self::rule_to_string(&pair, Rule::id);
          let title = Self::rule_to_string(&pair, Rule::inner).unwrap();
          let attributes = Self::find_rule(&pair, Rule::text_attributes).unwrap();
          let (width, _height, _radius) = Self::parse_dimension(&attributes);
          let location = Self::rule_to_location(&pair, Rule::location);
          let (_widths, height) = canvas.paragraph(title, (0., 0.), width - 2. * TEXT_PADDING);

          let mut used = Rect::from_xywh(bounds.left, bounds.bottom, width, height);
          used.bottom += BLOCK_PADDING;

          Self::position_rect(index, &location, &mut used);

          if let Some(id) = id {
            index.insert(id.into(), used);
          }

          let rect = used;
          ast.push(Primitive(id, rect, used, Color::BLACK, Shape::Text(title, location)));

          bounds.top = bounds.top.min(rect.top);
          bounds.left = rect.left;
          bounds.right = bounds.right.max(rect.right);
          bounds.bottom = rect.bottom
        }
        _ => {
          warn!("unmatched {:?}", pair);
          // let inset = Point::new(bounds.left, bounds.bottom);
          // (ast, bounds) = Self::pairs_to_nodes(pair.into_inner(), ast, canvas, &inset);
        }
      }
    }
    (ast, bounds)
  }

  fn parse_dimension(attributes: &Pair<Rule>) -> (f32, Option<f32>, Radius) {
    let width = Self::rule_to_length(attributes, Rule::width).map(|length| length.pixels()).unwrap_or(120.);
    let height = Self::rule_to_length(attributes, Rule::height).map(|length| length.pixels());
    let radius = Self::rule_to_radius(attributes);

    (width, height, radius)
  }

  fn position_rect(index: &mut HashMap<String, Rect>, location: &Option<(Anchor, Vec<Distance>, Edge)>, used: &mut Rect) {
    if let Some((anchor, distances, edge)) = &location {
      if let Some(rect) = Self::offset_index(index, edge, distances) {
        *used = Rect::from_xywh(rect.left, rect.top, used.width(), used.height());
        let offset = anchor.topleft_offset(used);
        used.offset(offset);
      }
    }
  }

  pub fn used_rect(&self, id: &str) -> Option<&Rect> {
    self.find_node(id).map(|node| {
      match node {
        Primitive(_, _, used, _, _) => used,
        _ => panic!("not a primitive")
      }
    })
  }

  fn find_node<'a>(&'a self, id: &str) -> Option<&Node<'a>> {
    Self::find_nodes(&self.nodes, id)
  }

  fn find_nodes<'a>(nodes: &'a [Node], node_id: &str) -> Option<&'a Node<'a>> {
    nodes.iter().find(|node| {
      match node {
        Primitive(Some(id), _, _, _, _) => {
          id == &node_id
        }
        Container(id, _, _, _, _, nodes) => {
          if let Some(id) = id {
            if id == &node_id {
              return true;
            }
          }
          Self::find_nodes(nodes, node_id).is_some()
        }
        _ => false
      }
    })
  }

  pub fn point_from(&self, edge: &Edge, distances: &[Distance]) -> Option<Point> {
    Self::point_index(&self.index, edge, distances)
  }

  fn point_index(index: &HashMap<String, Rect>, edge: &Edge, distances: &[Distance]) -> Option<Point> {
    index.get(&edge.id).map(|rect| {
      Self::point_from_rect(rect, &edge.anchor, distances)
    }).or_else(|| {
      error!("{} edge id not found", edge.id);
      None
    })
  }

  pub fn point_from_rect(rect: &Rect, anchor: &Anchor, distances: &[Distance]) -> Point {
    let point = anchor.to_edge(rect);
    for distance in distances.iter() {
      let _ = point.add(distance.offset());
    }
    point
  }

  pub fn offset_from(&self, edge: &Edge, distances: &[Distance]) -> Option<Rect> {
    Self::offset_index(&self.index, edge, distances)
  }

  fn offset_index(index: &HashMap<String, Rect>, edge: &Edge, distances: &[Distance]) -> Option<Rect> {
    index.get(&edge.id).map(|rect| {
      Self::offset_from_rect(rect, &edge.anchor, distances)
    })
  }

  pub fn offset_from_rect(rect: &Rect, anchor: &Anchor, distances: &[Distance]) -> Rect {
    let point = anchor.to_edge(rect);
    let mut rect = Rect::from_xywh(point.x, point.y, rect.width(), rect.height());
    Self::offset_rect(&mut rect, distances);
    rect
  }

  fn offset_rect(rect: &mut Rect, distances: &[Distance]) {
    for distance in distances.iter() {
      rect.offset(distance.offset());
    }
  }

  pub fn node_mut(&mut self, id: &str, distances: Vec<Distance>) {
    if let Primitive(_, _, ref mut rect, _, _) = Diagram::find_nodes_mut(&mut self.nodes, id).unwrap() {
      for distance in distances.iter() {
        rect.offset(distance.offset());
      }
    }
  }

  fn find_nodes_mut<'a: 'i>(nodes: &'i mut [Node<'a>], node_id: &str) -> Option<&'i mut Node<'a>> {
    for node in nodes.iter_mut() {
      match node {
        Primitive(Some(id), _, _, _, _) => {
          if id == &node_id {
            return Some(node);
          }
        }
        Container(_, _, _, _, _, nodes) => {
          if let Some(node) = Self::find_nodes_mut(nodes, node_id) {
            return Some(node);
          }
        }
        _ => {}
      }
    }
    None
  }

  pub fn render_to_file(&self, filepath: &str) {
    let mut canvas = Canvas::new(self.size);
    canvas.cursor = self.offset;
    self.render_to_canvas(&mut canvas, &self.nodes);
    canvas.write_png(filepath);
  }

  fn render_to_canvas(&self, canvas: &mut Canvas, nodes: &[Node]) {
    for node in nodes.iter() {
      match node {
        Container(_id, radius, title, _rect, used, nodes) => {
          self.render_to_canvas(canvas, nodes);

          if let Some(title) = title {
            canvas.paint.set_style(PaintStyle::Fill);
            canvas.paint.set_color(Color::BLACK);
            let inset = used.with_inset((TEXT_PADDING, TEXT_PADDING));
            let origin = (inset.left, inset.bottom - 16.);
            canvas.paragraph(title, origin, inset.width());
          }

          canvas.paint.set_style(PaintStyle::Stroke);
          canvas.paint.set_color(Color::RED);
          canvas.rectangle(used, radius.pixels());
        }
        Primitive(_id, _, used, color, shape) => {
          self.render_shape(canvas, used, color, shape);
        }
      }
    }
  }

  fn render_shape(&self, canvas: &mut Canvas, used: &Rect, color: &Color, shape: &Shape) {
    match shape {
      Shape::Dot(edge, radius) => {
        let point = Self::point_from_rect(used, &edge.anchor, &[]);
        canvas.paint.set_style(PaintStyle::Fill);
        canvas.paint.set_color(*color);
        canvas.circle(&point, radius.pixels());
      }
      Shape::Arrow(_, from, distance, to) => {
        canvas.move_to(used.left, used.top);
        let mut point = Point::new(used.left, used.top);
        if let Some(distance) = distance {
          point = point.add(distance.offset());

          if distance.is_horizontal() {
            canvas.line_to(point.x, point.y);
            canvas.line_to(point.x, used.bottom);
          } else {
            canvas.line_to(point.x, point.y);
            canvas.line_to(used.right, point.y);
          }
        } else {
          let p1 = if from.anchor.is_vertical() && to.anchor.is_horizontal() {
            Point::new(used.left, used.bottom)
          } else if from.anchor.is_horizontal() && to.anchor.is_vertical() {
            Point::new(used.right, used.top)
          } else {
            Point::new(used.left, used.top)
          };

          let p2 = Point::new(used.right, used.bottom);
          canvas.line_to(p1.x, p1.y);
          canvas.line_to(p2.x, p2.y);
          canvas.stroke();

          let direction = p2.sub(p1);
          Self::draw_arrow_head(canvas, p2, direction);
        }
      }
      Shape::Line(_, _, distance, _) => {
        canvas.move_to(used.left, used.top);
        let mut point = Point::new(used.left, used.top);
        if let Some(distance) = distance {
          point = point.add(distance.offset());

          if distance.is_horizontal() {
            canvas.line_to(point.x, point.y);
            canvas.line_to(point.x, used.bottom);
          } else {
            canvas.line_to(point.x, point.y);
            canvas.line_to(used.right, point.y);
          }
        }

        canvas.line_to(used.right, used.bottom);
        canvas.stroke();
      }
      Shape::Rectangle(text_color, paragraph, radius, fill, _) => {
        canvas.paint.set_style(PaintStyle::Stroke);
        canvas.paint.set_color(*color);
        canvas.rectangle(used, radius.pixels());

        canvas.paint.set_style(PaintStyle::Fill);
        canvas.paint.set_color(*fill);
        canvas.rectangle(used, radius.pixels());

        if let Some(paragraph) = paragraph {
          canvas.paint.set_style(PaintStyle::Fill);
          canvas.paint.set_color(*text_color);
          let mut rect = *used;
          if paragraph.widths.len() == 1 {
            rect.top += (used.height() - paragraph.height) / 2. - Canvas::get_font_descent();
            rect.left += (used.width() - paragraph.widths.first().unwrap()) / 2.;
          } else {
            rect = rect.with_inset((TEXT_PADDING, TEXT_PADDING));
          }
          Self::render_paragraph(canvas, &rect, &paragraph.text);
        }
      }
      Shape::Text(title, _) => {
        Self::render_paragraph(canvas, used, title);
      }
    }
  }

  fn draw_arrow_head(canvas: &mut Canvas, p2: Point, direction: Point) {
    let angle = direction.y.atan2(direction.x);
    let arrow = 25. * PI / 180.;
    let size = 15.;
    canvas.move_to(
      p2.x - size * (angle - arrow).cos(),
      p2.y - size * (angle - arrow).sin());
    canvas.line_to(p2.x, p2.y);
    canvas.line_to(
      p2.x - size * (angle + arrow).cos(),
      p2.y - size * (angle + arrow).sin());
    canvas.fill();
  }

  fn render_paragraph(canvas: &mut Canvas, rect: &Rect, title: &&str) {
    let origin = (rect.left, rect.top);
    canvas.paragraph(title, origin, rect.width());
  }

  fn rule_to_location(pair: &Pair<Rule>, rule: Rule) -> Option<(Anchor, Vec<Distance>, Edge)> {
    Self::dig_rule(pair, rule)
      .map(|p| {
        let mut anchor: Option<Anchor> = None;
        let mut directions: Vec<Distance> = vec![];
        let mut edge: Option<Edge> = None;

        for rule in p.into_inner() {
          match rule.as_rule() {
            Rule::anchor => { anchor = Some(Anchor::new(rule.as_str())); }
            Rule::distance => {
              let distance = Self::pair_to_distance(rule);
              directions.push(distance);
            }
            Rule::edge => { edge = Some(Self::pair_to_edge(rule)); }
            _ => {}
          }
        };
        (anchor.unwrap(), directions, edge.unwrap())
      })
  }

  fn rule_to_distance(pair: &Pair<Rule>, rule: Rule) -> Option<Distance> {
    Self::find_rule(pair, rule).map(Self::pair_to_distance)
  }

  fn rule_to_distances(pair: &Pair<Rule>, rule: Rule) -> Option<Vec<Distance>> {
    Self::find_rule(pair, rule)
      .map(|p| {
        let mut directions: Vec<Distance> = vec![];

        for rule in p.into_inner() {
          match rule.as_rule() {
            Rule::distance => {
              let distance = Self::pair_to_distance(rule);
              directions.push(distance);
            }
            rule => error!("Unexpected rule {:?}", rule)
          }
        };
        directions
      })
  }

  fn pair_to_radius(pair: Pair<Rule>) -> Radius {
    let length = Self::find_rule(&pair, Rule::length)
      .and_then(|p| p.as_str().parse::<usize>().ok())
      .unwrap();
    let unit = Self::rule_to_string(&pair, Rule::unit)
      .unwrap();
    Radius::new(length as f32, unit.into())
  }

  fn pair_to_distance(pair: Pair<Rule>) -> Distance {
    let length = Self::find_rule(&pair, Rule::length)
      .and_then(|p| p.as_str().parse::<usize>().ok())
      .unwrap();
    let unit = Self::rule_to_string(&pair, Rule::unit)
      .unwrap();
    let direction = match Self::rule_to_string(&pair, Rule::direction).unwrap() {
      "left" => Vector::new(-1., 0.),
      "right" => Vector::new(1., 0.),
      "up" => Vector::new(0., -1.),
      _ => Vector::new(0., 1.),
    };
    Distance::new(length as f32, unit.into(), direction)
  }

  fn location_to_edge(pair: &Pair<Rule>, rule: Rule) -> Option<Edge> {
    Self::find_rule(pair, rule)
      .and_then(|pair| Self::find_rule(&pair, Rule::edge))
      .map(Self::pair_to_edge)
  }

  fn rule_to_edge(pair: &Pair<Rule>, rule: Rule) -> Option<Edge> {
    Self::find_rule(pair, rule).map(Self::pair_to_edge)
  }

  fn pair_to_edge(pair: Pair<Rule>) -> Edge {
    let id = Self::rule_to_string(&pair, Rule::id).unwrap();
    let anchor = Self::rule_to_string(&pair, Rule::anchor).unwrap();
    Edge::new(id, anchor)
  }

  fn rule_to_radius(pair: &Pair<Rule>) -> Radius {
    Self::dig_rule(pair, Rule::radius)
      .map(Self::pair_to_radius)
      .unwrap_or(Radius::default())
  }

  fn rule_to_length(pair: &Pair<Rule>, rule: Rule) -> Option<Length> {
    Self::dig_rule(pair, rule).map(Self::pair_to_length)
  }

  fn pair_to_length(pair: Pair<Rule>) -> Length {
    let length = Self::find_rule(&pair, Rule::length)
      .and_then(|p| p.as_str().parse::<usize>().ok())
      .unwrap();
    let unit = Self::rule_to_string(&pair, Rule::unit)
      .unwrap();
    Length::new(length as f32, unit.into())
  }


  fn rule_to_color(pair: &Pair<Rule>, rule: Rule) -> Option<Color> {
    Self::dig_rule(pair, rule)
      .and_then(|pair| Self::find_rule(&pair, Rule::id))
      .map(|p| p.as_str())
      .map(|color| match color {
        "white" => Color::WHITE,
        "lgray" => Color::LIGHT_GRAY,
        "dgray" => Color::DARK_GRAY,
        "black" => Color::BLACK,
        "yellow" => Color::YELLOW,
        "red" => Color::RED,
        "green" => Color::GREEN,
        "blue" => Color::BLUE,
        "gray" => Color::GRAY,
        "cyan" => Color::CYAN,
        "magenta" => Color::MAGENTA,
        _ => panic!("unknown color {}", color)
      })
  }

  fn rule_to_string<'a>(pair: &Pair<'a, Rule>, rule: Rule) -> Option<&'a str> {
    Self::dig_rule(pair, rule)
      .map(|p| p.as_str())
  }

  fn find_rule<'a>(pair: &Pair<'a, Rule>, rule: Rule) -> Option<Pair<'a, Rule>> {
    pair.clone().into_inner()
      .find(|p| p.as_rule() == rule)
  }

  fn dig_rule<'a>(pair: &Pair<'a, Rule>, rule: Rule) -> Option<Pair<'a, Rule>> {
    for pair in pair.clone().into_inner() {
      if pair.as_rule() == rule {
        return Some(pair);
      }
      if let Some(pair) = Self::dig_rule(&pair, rule) {
        return Some(pair);
      }
    }
    None
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

