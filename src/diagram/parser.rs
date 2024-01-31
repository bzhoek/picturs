#![allow(dead_code)]

use std::collections::HashMap;
use std::error::Error;
use std::f32::consts::PI;
use std::ops::{Add, Sub};

use log::{debug, error, warn};
use pest::iterators::{Pair, Pairs};
use pest::Parser;
use pest_derive::Parser;
use skia_safe::{Color, ISize, PaintStyle, Point, Rect};

use crate::diagram::conversion::Conversion;
use crate::diagram::parser::Node::{Container, Primitive};
use crate::diagram::rules::Rules;
use crate::diagram::types::{Config, Displacement, Edge, Flow, Length, ObjectEdge, ShapeConfig};
use crate::skia::Canvas;

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

type EdgeDisplacement = (Edge, Vec<Displacement>, ObjectEdge);

#[derive(Debug, PartialEq)]
pub enum Shape<'a> {
  Move(),
  Dot(ObjectEdge, Radius),
  Arrow(Option<&'a str>, ObjectEdge, Option<Displacement>, ObjectEdge),
  Line(Option<&'a str>, Point, Option<Displacement>, Point),
  Rectangle(Color, Option<Paragraph<'a>>, Radius, Color, Option<EdgeDisplacement>),
  Circle(Color, Option<Paragraph<'a>>, Color, Option<EdgeDisplacement>),
  Text(&'a str, Option<EdgeDisplacement>),
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
  inset: Point,
  bounds: Rect,
}

impl<'i> Diagram<'i> {
  pub fn inset(size: impl Into<ISize>, inset: impl Into<Point>) -> Self {
    Self {
      nodes: vec![],
      index: HashMap::new(),
      size: size.into(),
      inset: inset.into(),
      bounds: Default::default(),
    }
  }

  pub fn parse_string(&mut self, string: &'i str) -> Pairs<'i, Rule> {
    let top = DiagramParser::parse(Rule::picture, string).unwrap();
    let mut canvas = Canvas::new(self.size);
    let flow = Flow::new("sw");
    let config = Config::new(flow, 120., 60.);
    let (ast, bounds) = Self::pairs_to_nodes(top.clone(), vec![], &mut canvas, &Point::default(), config, &mut self.index);
    self.nodes = ast;
    self.bounds = bounds;
    top
  }

  pub fn pairs_to_nodes<'a>(pairs: Pairs<'a, Rule>, mut ast: Vec<Node<'a>>, canvas: &mut Canvas, offset: &Point, mut config: Config, index: &mut HashMap<String, Rect>)
    -> (Vec<Node<'a>>, Rect) {
    let mut bounds = Rect::from_xywh(offset.x, offset.y, 0., 0.);
    let mut cursor = Point::new(offset.x, offset.y);

    for pair in pairs.into_iter() {
      let result = match pair.as_rule() {
        Rule::box_config => {
          Self::config_shape(&mut config.rectangle, pair);
          None
        }
        Rule::circle_config => {
          Self::config_shape(&mut config.circle, pair);
          None
        }
        Rule::flow => {
          config.flow = Flow::new(pair.as_str());
          None
        }
        Rule::container => {
          let id = Conversion::rule_to_string(&pair, Rule::id);
          let attributes = Rules::get_rule(&pair, Rule::attributes);
          let (_height, radius) = Conversion::dimensions_from(&attributes);
          let padding = Conversion::padding(&attributes).unwrap_or(config.rectangle.padding);
          let title = Conversion::rule_to_string(&attributes, Rule::inner);
          let location = Conversion::location_from(&attributes, &config.flow.start);

          let mut used = Rect::from_xywh(cursor.x, cursor.y, 0., 0.);
          Self::position_rect(index, &location, &mut used);

          let mut inset = Point::new(used.left, used.bottom);
          inset.offset((padding, padding));
          let (nodes, inner) = {
            let mut config = config.clone();
            Conversion::flow(&attributes).iter().for_each(|flow| {
              config.flow = *flow;
            });
            Self::pairs_to_nodes(pair.into_inner(), vec![], canvas, &inset, config, index)
          };
          used.top = inner.top - padding;
          used.left = inner.left - padding;
          used.bottom = inner.bottom + padding;
          used.right = inner.right + 2. * padding;

          if let Some(title) = title {
            let text_inset = inner.with_inset((TEXT_PADDING, TEXT_PADDING));
            let (_widths, down) = canvas.paragraph(title, (0., 0.), text_inset.width());
            used.bottom = inner.bottom + down + TEXT_PADDING;
          }

          if let Some(id) = id {
            index.insert(id.into(), used);
          }

          let mut rect = used;
          rect.bottom += padding;
          Some((rect, Container(id, radius, title, rect, used, nodes)))
        }
        Rule::dot => Self::dot_from_pair(index, &pair),
        Rule::arrow => Self::arrow_from_pair(index, &cursor, &config.flow, pair),
        Rule::line => Self::line_from_pair(index, &cursor, &config.flow, pair),
        Rule::move_to => Self::move_from_pair(&pair, cursor),
        Rule::rectangle => Self::rectangle_from_pair(canvas, index, &cursor, &config, &pair),
        Rule::circle => Self::circle_from_pair(canvas, index, &cursor, &config, &pair),
        Rule::text => Self::text_from_pair(canvas, index, &cursor, &config, &pair),
        _ => {
          debug!("Unmatched {:?}", pair);
          None
        }
      };

      if let Some((rect, node)) = result {
        ast.push(node);
        Self::update_bounds(&mut bounds, rect);
        let point = config.flow.end.edge_point(&rect);
        cursor = point
      }
    }
    (ast, bounds)
  }

  fn config_shape(config: &mut ShapeConfig, pair: Pair<Rule>) {
    pair.into_inner().for_each(|pair| {
      match pair.as_rule() {
        Rule::padding => {
          let length = Self::length_from(pair);
          config.padding = length.pixels();
        }
        Rule::height => {
          let length = Self::length_from(pair);
          config.height = length.pixels();
        }
        Rule::width => {
          let length = Self::length_from(pair);
          config.width = length.pixels();
        }
        _ => {
          warn!("Ignored {:?}", pair);
        }
      }
    });
  }

  fn length_from(pair: Pair<Rule>) -> Length {
    let mut width = pair.into_inner();
    let length = Self::next_to_usize(&mut width).unwrap();
    let unit = Self::next_to_string(&mut width).unwrap_or("px");
    Length::new(length as f32, unit.into())
  }

  fn next_to_usize(iter: &mut Pairs<Rule>) -> Option<usize> {
    iter.next().and_then(|p| p.as_str().parse::<usize>().ok())
  }
  fn next_to_string<'a>(iter: &mut Pairs<'a, Rule>) -> Option<&'a str> {
    iter.next().map(|p| p.as_str())
  }


  fn arrow_from_pair<'a>(index: &HashMap<String, Rect>, cursor: &Point, flow: &Flow, pair: Pair<'a, Rule>) -> Option<(Rect, Node<'a>)> {
    let id = Conversion::rule_to_string(&pair, Rule::id);

    let (source, displacement, target) = Self::source_displacement_target_from_pair(&pair);
    let start = Self::point_index(index, &source, &[])
      .unwrap_or(*cursor);
    let end = Self::point_index(index, &target, &[])
      .unwrap_or(Self::displace_from_start(start, &displacement, flow));

    let (rect, used) = Self::rect_from_points(start, &displacement, end);
    let node = Primitive(id, rect, rect, Color::BLACK, Shape::Arrow(id, source, displacement, target));
    Some((used, node))
  }

  fn source_displacement_target_from_pair(pair: &Pair<Rule>) -> (ObjectEdge, Option<Displacement>, ObjectEdge) {
    let source = Conversion::location_to_edge(pair, Rule::source)
      .unwrap_or(ObjectEdge::new("source", "e"));
    let distance = Conversion::rule_to_distance(pair, Rule::displacement);
    let target = Conversion::location_to_edge(pair, Rule::target)
      .unwrap_or(ObjectEdge::new("source", "w"));
    (source, distance, target)
  }

  fn displace_from_start(start: Point, displacement: &Option<Displacement>, flow: &Flow) -> Point {
    displacement.as_ref()
      .map(|displacement| start.add(displacement.offset()))
      .unwrap_or_else(|| {
        let distance = Displacement::new(2., "cm".into(), flow.end.vector());
        start.add(distance.offset())
      })
  }

  fn line_from_pair<'a>(index: &mut HashMap<String, Rect>, cursor: &Point, flow: &Flow, pair: Pair<'a, Rule>) -> Option<(Rect, Node<'a>)> {
    let id = Conversion::rule_to_string(&pair, Rule::id);
    let (start, distance, end) = Self::points_from_pair(index, cursor, flow, &pair);
    let (rect, used) = Self::rect_from_points(start, &distance, end);
    let node = Primitive(id, rect, rect, Color::BLACK, Shape::Line(id, start, distance, end));
    Some((used, node))
  }

  fn points_from_pair(index: &mut HashMap<String, Rect>, cursor: &Point, flow: &Flow, pair: &Pair<Rule>) -> (Point, Option<Displacement>, Point) {
    let (source, displacement, target) = Self::source_displacement_target_from_pair(pair);
    let start = Self::point_index(index, &source, &[])
      .unwrap_or(*cursor);
    let end = Self::point_index(index, &target, &[])
      .unwrap_or(Self::displace_from_start(start, &displacement, flow));
    (start, displacement, end)
  }

  fn rect_from_points(start: Point, distance: &Option<Displacement>, end: Point) -> (Rect, Rect) {
    let rect = Rect { left: start.x, top: start.y, right: end.x, bottom: end.y };
    let mut used = rect;
    if let Some(displacement) = &distance {
      used.offset(displacement.offset());
    }
    (rect, used)
  }

  fn dot_from_pair<'a>(index: &HashMap<String, Rect>, pair: &Pair<'a, Rule>) -> Option<(Rect, Node<'a>)> {
    let attributes = Rules::find_rule(pair, Rule::dot_attributes).unwrap();
    let color = Conversion::rule_to_color(&attributes, Rule::color).unwrap_or(Color::BLUE);
    let radius = Conversion::radius(&attributes).unwrap_or_default();

    let target = Conversion::object_edge_from_pair(pair).unwrap();
    let point = Self::point_index(index, &target, &[]).unwrap();
    let rect = Rect::from_xywh(point.x, point.y, 0., 0.);
    let dot = Primitive(None, rect, rect, color, Shape::Dot(target, radius));
    Some((rect, dot))
  }

  fn move_from_pair<'a>(pair: &Pair<'a, Rule>, cursor: Point) -> Option<(Rect, Node<'a>)> {
    Conversion::displacements_from_pair(pair).map(|displacements| {
      let mut used = Rect::from_xywh(cursor.x, cursor.y, 0., 0.);
      Self::offset_rect(&mut used, &displacements);
      (used, Primitive(None, used, used, Color::BLACK, Shape::Move()))
    })
  }

  fn rectangle_from_pair<'a>(canvas: &mut Canvas, index: &mut HashMap<String, Rect>, cursor: &Point, config: &Config, pair: &Pair<'a, Rule>) -> Option<(Rect, Node<'a>)> {
    let id = Conversion::rule_to_string(pair, Rule::id);
    let attributes = Rules::find_rule(pair, Rule::attributes).unwrap();

    let (height, radius) = Conversion::dimensions_from(&attributes);
    let width = Conversion::width(&attributes).unwrap_or(config.rectangle.width);
    let padding = Conversion::padding(&attributes).unwrap_or(config.rectangle.padding);
    let (stroke, fill, text_color) = Conversion::colors_from(&attributes);
    let title = Conversion::rule_to_string(&attributes, Rule::inner);
    let location = Conversion::location_from(&attributes, &config.flow.start);

    let mut para_height = None;
    let paragraph = title.map(|title| {
      let (widths, height) = canvas.paragraph(title, (0., 0.), width - 2. * TEXT_PADDING);
      para_height = Some(height);
      Paragraph { text: title, widths, height }
    });

    let height = height.unwrap_or_else(|| {
      para_height.unwrap_or(config.rectangle.height)
    });

    let mut used = Rect::from_xywh(cursor.x, cursor.y, width, height.max(config.rectangle.height));
    used.bottom += padding;
    Self::adjust_topleft(&config.flow, &mut used);
    Self::position_rect(index, &location, &mut used);

    if let Some(id) = id {
      index.insert(id.into(), used);
    }

    let mut rect = used;
    if config.flow.end.x <= 0. {
      rect.bottom += padding;
    }

    let rectangle = Primitive(id, rect, used, stroke, Shape::Rectangle(text_color, paragraph, radius, fill, location));
    Some((rect, rectangle))
  }

  fn circle_from_pair<'a>(canvas: &mut Canvas, index: &mut HashMap<String, Rect>, cursor: &Point, config: &Config, pair: &Pair<'a, Rule>) -> Option<(Rect, Node<'a>)> {
    let id = Conversion::rule_to_string(pair, Rule::id);
    let attributes = Rules::get_rule(pair, Rule::attributes);
    let width = Conversion::width(&attributes)
      .unwrap_or(config.circle.height);
    let height = Conversion::width(&attributes);

    let (stroke, fill, text_color) = Conversion::colors_from(&attributes);
    let title = Conversion::rule_to_string(&attributes, Rule::inner);
    let location = Conversion::location_from(&attributes, &config.flow.start);

    let mut para_height = None;
    let paragraph = title.map(|title| {
      let (widths, height) = canvas.paragraph(title, (0., 0.), width - 2. * TEXT_PADDING);
      para_height = Some(height);
      Paragraph { text: title, widths, height }
    });

    let height = height.unwrap_or_else(|| {
      para_height.unwrap_or(config.circle.height)
    });

    let mut used = Rect::from_xywh(cursor.x, cursor.y, width, height.max(config.circle.height));

    Self::adjust_topleft(&config.flow, &mut used);
    Self::position_rect(index, &location, &mut used);

    if let Some(id) = id {
      index.insert(id.into(), used);
    }

    let circle = Primitive(id, used, used, stroke, Shape::Circle(text_color, paragraph, fill, location));
    Some((used, circle))
  }

  fn adjust_topleft(flow: &Flow, used: &mut Rect) {
    let offset = flow.start.topleft_offset(used);
    used.offset(offset);
  }

  fn text_from_pair<'a>(canvas: &mut Canvas, index: &mut HashMap<String, Rect>, cursor: &Point, config: &Config, pair: &Pair<'a, Rule>) -> Option<(Rect, Node<'a>)> {
    let id = Conversion::rule_to_string(pair, Rule::id);
    let title = Conversion::rule_to_string(pair, Rule::inner).unwrap();
    let attributes = Rules::find_rule(pair, Rule::text_attributes).unwrap();
    let width = Conversion::width(&attributes).unwrap_or(config.width);
    let location = Conversion::location_from(pair, &config.flow.start);
    let (_widths, height) = canvas.paragraph(title, (0., 0.), width - 2. * TEXT_PADDING);

    let mut used = Rect::from_xywh(cursor.x, cursor.y, width, height);
    used.bottom += BLOCK_PADDING;

    Self::position_rect(index, &location, &mut used);

    if let Some(id) = id {
      index.insert(id.into(), used);
    }

    let text = Primitive(id, used, used, Color::BLACK, Shape::Text(title, location));
    Some((used, text))
  }

  fn update_bounds(bounds: &mut Rect, rect: Rect) {
    bounds.top = bounds.top.min(rect.top);
    bounds.left = bounds.left.min(rect.left);
    bounds.right = bounds.right.max(rect.right);
    bounds.bottom = bounds.bottom.max(rect.bottom);
  }

  fn position_rect(index: &mut HashMap<String, Rect>, location: &Option<(Edge, Vec<Displacement>, ObjectEdge)>, used: &mut Rect) {
    if let Some((edge, distances, object)) = &location {
      if let Some(rect) = Self::offset_index(index, object, distances) {
        *used = Rect::from_xywh(rect.left, rect.top, used.width(), used.height());
        let offset = edge.topleft_offset(used);
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

  pub fn point_from(&self, edge: &ObjectEdge, distances: &[Displacement]) -> Option<Point> {
    Self::point_index(&self.index, edge, distances)
  }

  fn point_index(index: &HashMap<String, Rect>, edge: &ObjectEdge, distances: &[Displacement]) -> Option<Point> {
    index.get(&edge.id).map(|rect| {
      Self::point_from_rect(rect, &edge.edge, distances)
    }).or_else(|| {
      error!("{} edge id not found", edge.id);
      None
    })
  }

  pub fn point_from_rect(rect: &Rect, edge: &Edge, distances: &[Displacement]) -> Point {
    let point = edge.edge_point(rect);
    for distance in distances.iter() {
      let _ = point.add(distance.offset());
    }
    point
  }

  pub fn offset_from(&self, edge: &ObjectEdge, distances: &[Displacement]) -> Option<Rect> {
    Self::offset_index(&self.index, edge, distances)
  }

  fn offset_index(index: &HashMap<String, Rect>, edge: &ObjectEdge, distances: &[Displacement]) -> Option<Rect> {
    index.get(&edge.id).map(|rect| {
      Self::offset_from_rect(rect, &edge.edge, distances)
    })
  }

  pub fn offset_from_rect(rect: &Rect, edge: &Edge, distances: &[Displacement]) -> Rect {
    let point = edge.edge_point(rect);
    let mut rect = Rect::from_xywh(point.x, point.y, rect.width(), rect.height());
    Self::offset_rect(&mut rect, distances);
    rect
  }

  fn offset_rect(rect: &mut Rect, distances: &[Displacement]) {
    for distance in distances.iter() {
      rect.offset(distance.offset());
    }
  }

  pub fn node_mut(&mut self, id: &str, distances: Vec<Displacement>) {
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

  pub fn render_to_file(&mut self, filepath: &str) {
    let mut canvas = Canvas::new(self.size);
    canvas.translate(-self.bounds.left + self.inset.x, -self.bounds.top + self.inset.y);
    // Self::final_placement(&mut self.nodes);
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
  fn final_placement(nodes: &mut [Node]) {
    for node in nodes.iter_mut() {
      match node {
        Container(_id, _, _, _rect, used, nodes) => {
          used.top += 16.;
          Self::final_placement(nodes);
        }
        Primitive(_id, _, used, _, _) => {
          used.top += 16.;
        }
      }
    }
  }

  fn render_shape(&self, canvas: &mut Canvas, used: &Rect, color: &Color, shape: &Shape) {
    match shape {
      Shape::Dot(edge, radius) => {
        let point = Self::point_from_rect(used, &edge.edge, &[]);
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
          let p1 = if from.edge.vertical() && to.edge.horizontal() {
            Point::new(used.left, used.bottom)
          } else if from.edge.horizontal() && to.edge.vertical() {
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
      Shape::Line(_, _, displacement, _) => {
        canvas.move_to(used.left, used.top);
        let mut point = Point::new(used.left, used.top);
        if let Some(displacement) = displacement {
          point = point.add(displacement.offset());

          if displacement.is_horizontal() {
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
      Shape::Circle(text_color, paragraph, fill, _) => {
        canvas.paint.set_style(PaintStyle::Stroke);
        canvas.paint.set_color(*color);
        canvas.circle(&used.center(), used.width() / 2.);

        canvas.paint.set_style(PaintStyle::Fill);
        canvas.paint.set_color(*fill);
        canvas.circle(&used.center(), used.width() / 2.);

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
      _ => {}
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
}

pub const BLOCK_PADDING: f32 = 8.;
const TEXT_PADDING: f32 = 4.;

#[allow(dead_code)]
pub fn dump_nested(level: usize, pairs: Pairs<Rule>) {
  for pair in pairs.into_iter() {
    println!("{:level$} {:?}", level, pair);
    dump_nested(level + 1, pair.into_inner());
  }
}

