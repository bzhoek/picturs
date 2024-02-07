use std::ops::Add;

use log::{debug, warn};
use pest::iterators::{Pair, Pairs};
use pest::Parser;
use pest_derive::Parser;
use skia_safe::{Color, ISize, Point, Rect};

use crate::diagram::conversion::Conversion;
use crate::diagram::index::{Index, ShapeName};
use crate::diagram::renderer::Renderer;
use crate::diagram::rules::Rules;
use crate::diagram::types::{BLOCK_PADDING, Config, Movement, Edge, Flow, Node, ObjectEdge, Paragraph, Shape, ShapeConfig, Unit};
use crate::diagram::types::Node::{Container, Primitive};
use crate::skia::Canvas;

#[derive(Parser)]
#[grammar = "diagram.pest"]
pub struct DiagramParser;

#[derive(Debug)]
pub struct Diagram<'a> {
  pub nodes: Vec<Node<'a>>,
  size: ISize,
  inset: Point,
  bounds: Rect,
}

impl<'i> Diagram<'i> {
  pub fn inset(size: impl Into<ISize>, inset: impl Into<Point>) -> Self {
    Self {
      nodes: vec![],
      size: size.into(),
      inset: inset.into(),
      bounds: Default::default(),
    }
  }

  pub fn parse_string(&mut self, string: &'i str) -> Pairs<'i, Rule> {
    let top = DiagramParser::parse(Rule::picture, string).unwrap();
    let mut canvas = Canvas::new(self.size);
    let flow = Flow::new("left");
    let config = Config::new(flow, 120., 60.);
    let mut index = Index::default();
    let (ast, bounds) = Self::nodes_from(top.clone(), vec![], &mut canvas, &Point::default(), config, &mut index);
    self.nodes = ast;
    self.bounds = bounds;
    top
  }

  pub fn nodes_from<'a>(pairs: Pairs<'a, Rule>, mut ast: Vec<Node<'a>>, canvas: &mut Canvas, offset: &Point, mut config: Config, index: &mut Index)
                        -> (Vec<Node<'a>>, Rect) {
    let mut bounds = Rect::from_xywh(offset.x, offset.y, 0., 0.);
    let mut cursor = Point::new(offset.x, offset.y);

    for pair in pairs.into_iter() {
      let result = Self::node_from(pair, &mut config, index, &mut cursor, canvas);

      if let Some((rect, node)) = result {
        ast.push(node);
        Self::update_bounds(&mut bounds, rect);
        let point = config.flow.end.edge_point(&rect);
        cursor = point
      }
    }
    (ast, bounds)
  }

  fn node_from<'a>(pair: Pair<'a, Rule>, config: &mut Config, index: &mut Index, cursor: &mut Point, canvas: &mut Canvas) -> Option<(Rect, Node<'a>)> {
    let result = match pair.as_rule() {
      Rule::unit_config => {
        config.unit = Unit::from(pair.into_inner().as_str());
        None
      }
      Rule::box_config => {
        Self::config_shape(&mut config.rectangle, pair, &config.unit);
        None
      }
      Rule::circle_config => {
        Self::config_shape(&mut config.circle, pair, &config.unit);
        None
      }
      Rule::flow => {
        config.flow = Flow::new(pair.as_str());
        None
      }
      Rule::container => Self::container_from(&pair, config, index, cursor, canvas),
      Rule::rectangle => Self::rectangle_from(&pair, config, index, cursor, canvas),
      Rule::file => Self::file_from(&pair, config, index, cursor, canvas),
      Rule::ellipse => Self::ellipse_from(&pair, config, index, cursor, canvas),
      Rule::cylinder => Self::cylinder_from(&pair, config, index, cursor, canvas),
      Rule::oval => Self::oval_from(&pair, config, index, cursor, canvas),
      Rule::dot => Self::dot_from(&pair, config, index),
      Rule::arrow => Self::arrow_from(pair, index, cursor, config),
      Rule::line => Self::line_from(pair, index, cursor, config),
      Rule::move_to => Self::move_from(&pair, cursor, &config.unit),
      Rule::flow_to => Self::flow_from(pair, cursor, config),
      Rule::circle => Self::circle_from(&pair, config, index, cursor, canvas),
      Rule::text => Self::text_from(&pair, config, index, cursor, canvas),
      _ => {
        debug!("Unmatched {:?}", pair);
        None
      }
    };
    result
  }

  fn container_from<'a>(pair: &Pair<'a, Rule>, config: &Config, index: &mut Index, cursor: &Point, canvas: &mut Canvas) -> Option<(Rect, Node<'a>)> {
    let id = Conversion::identified(pair);
    let attributes = Rules::get_rule(pair, Rule::attributes);
    let radius = Conversion::radius(&attributes, &config.unit).unwrap_or_default();
    let padding = Conversion::padding(&attributes, &config.unit).unwrap_or(config.rectangle.padding);
    let title = Conversion::string_dig(&attributes, Rule::inner);
    let location = Conversion::location_from(&attributes, &config.flow.end, &config.unit);

    let mut used = Rect::from_xywh(cursor.x, cursor.y, 0., 0.);
    index.position_rect(&location, &mut used);

    let mut inset = Point::new(used.left, used.bottom);
    inset.offset((padding, padding));

    let (nodes, inner) = {
      let mut config = config.clone();
      Conversion::flow(&attributes).into_iter().for_each(|flow| {
        config.flow = flow;
      });
      Self::nodes_from(pair.clone().into_inner(), vec![], canvas, &inset, config, index)
    };

    used = inner.with_outset((padding, padding));

    if let Some(title) = title {
      let text_inset = inner.with_inset((TEXT_PADDING, TEXT_PADDING));
      let (_widths, down) = canvas.paragraph(title, (0., 0.), text_inset.width());
      used.bottom = inner.bottom + down + TEXT_PADDING;
    }

    index.insert(ShapeName::Container, id, used);

    let mut rect = used;
    rect.bottom += padding;
    Some((rect, Container(id, radius, title, rect, used, nodes)))
  }

  fn oval_from<'a>(pair: &Pair<'a, Rule>, config: &Config, index: &mut Index, cursor: &Point, canvas: &mut Canvas) -> Option<(Rect, Node<'a>)> {
    let id = Conversion::identified(pair);
    let attributes = Rules::get_rule(pair, Rule::attributes);
    let width = Conversion::width(&attributes, &config.unit).unwrap_or(config.oval.width);
    let height = Conversion::height(&attributes, &config.unit).unwrap_or(config.oval.height);

    let (stroke, fill, text_color) = Conversion::colors_from(&attributes);
    let title = Conversion::string_dig(&attributes, Rule::inner);
    let location = Conversion::location_from(&attributes, &config.flow.end, &config.unit);

    let paragraph = Self::paragraph_height(title, width, canvas);
    let height = paragraph.as_ref().map(|paragraph| height.max(paragraph.height)).unwrap_or(height);

    let mut used = Rect::from_xywh(cursor.x, cursor.y, width, height);

    Self::position_rect_on_edge(&config.flow.start, &location, &mut used);
    index.position_rect(&location, &mut used);

    index.insert(ShapeName::Oval, id, used);

    let oval = Primitive(id, used, used, stroke, Shape::Oval(text_color, paragraph, fill, location));
    Some((used, oval))
  }

  fn rectangle_from<'a>(pair: &Pair<'a, Rule>, config: &Config, index: &mut Index, cursor: &Point, canvas: &mut Canvas) -> Option<(Rect, Node<'a>)> {
    let id = Conversion::identified(pair);
    let attributes = Rules::find_rule(pair, Rule::attributes).unwrap();

    let radius = Conversion::radius(&attributes, &config.unit).unwrap_or_default();
    let width = Conversion::width(&attributes, &config.unit).unwrap_or(config.rectangle.width);
    let height = Conversion::height(&attributes, &config.unit).unwrap_or(config.rectangle.height);
    let padding = Conversion::padding(&attributes, &config.unit).unwrap_or(config.rectangle.padding);
    let (stroke, fill, text_color) = Conversion::colors_from(&attributes);
    let title = Conversion::string_dig(&attributes, Rule::inner);
    let location = Conversion::location_from(&attributes, &config.flow.end, &config.unit);

    let paragraph = Self::paragraph_height(title, width, canvas);
    let height = paragraph.as_ref().map(|paragraph| height.max(paragraph.height)).unwrap_or(height);

    let mut used = Rect::from_xywh(cursor.x, cursor.y, width, height);
    used.bottom += padding;
    Self::adjust_topleft(&config.flow, &mut used);
    index.position_rect(&location, &mut used);

    index.insert(ShapeName::Rectangle, id, used);

    let mut rect = used;
    if config.flow.end.x <= 0. {
      rect.bottom += padding;
    }

    let rectangle = Primitive(id, rect, used, stroke, Shape::Rectangle(text_color, paragraph, radius, fill, location));
    Some((rect, rectangle))
  }

  fn file_from<'a>(pair: &Pair<'a, Rule>, config: &Config, index: &mut Index, cursor: &Point, canvas: &mut Canvas) -> Option<(Rect, Node<'a>)> {
    let id = Conversion::identified(pair);
    let attributes = Rules::find_rule(pair, Rule::attributes).unwrap();

    let radius = Conversion::radius(&attributes, &config.unit).unwrap_or_default();
    let width = Conversion::width(&attributes, &config.unit).unwrap_or(config.file.width);
    let height = Conversion::height(&attributes, &config.unit).unwrap_or(config.file.height);
    let padding = Conversion::padding(&attributes, &config.unit).unwrap_or(config.file.padding);
    let (stroke, fill, text_color) = Conversion::colors_from(&attributes);
    let title = Conversion::string_dig(&attributes, Rule::inner);
    let location = Conversion::location_from(&attributes, &config.flow.end, &config.unit);

    let paragraph = Self::paragraph_height(title, width, canvas);
    let height = paragraph.as_ref().map(|paragraph| height.max(paragraph.height)).unwrap_or(height);

    let mut used = Rect::from_xywh(cursor.x, cursor.y, width, height);
    used.bottom += padding;
    Self::adjust_topleft(&config.flow, &mut used);
    index.position_rect(&location, &mut used);

    index.insert(ShapeName::File, id, used);

    let mut rect = used;
    if config.flow.end.x <= 0. {
      rect.bottom += padding;
    }

    let file = Primitive(id, rect, used, stroke, Shape::File(text_color, paragraph, radius, fill, location));
    Some((rect, file))
  }

  fn ellipse_from<'a>(pair: &Pair<'a, Rule>, config: &Config, index: &mut Index, cursor: &Point, canvas: &mut Canvas) -> Option<(Rect, Node<'a>)> {
    let id = Conversion::identified(pair);
    let attributes = Rules::get_rule(pair, Rule::attributes);
    let width = Conversion::width(&attributes, &config.unit).unwrap_or(config.ellipse.width);
    let height = Conversion::height(&attributes, &config.unit).unwrap_or(config.ellipse.height);

    let (stroke, fill, text_color) = Conversion::colors_from(&attributes);
    let title = Conversion::string_dig(&attributes, Rule::inner);
    let location = Conversion::location_from(&attributes, &config.flow.end, &config.unit);

    let paragraph = Self::paragraph_height(title, width, canvas);
    let height = paragraph.as_ref().map(|paragraph| height.max(paragraph.height)).unwrap_or(height);

    let mut used = Rect::from_xywh(cursor.x, cursor.y, width, height);

    Self::adjust_topleft(&config.flow, &mut used);
    index.position_rect(&location, &mut used);

    index.insert(ShapeName::Ellipse, id, used);

    let ellipse = Primitive(id, used, used, stroke, Shape::Ellipse(text_color, paragraph, fill, location));
    Some((used, ellipse))
  }

  fn cylinder_from<'a>(pair: &Pair<'a, Rule>, config: &Config, index: &mut Index, cursor: &Point, canvas: &mut Canvas) -> Option<(Rect, Node<'a>)> {
    let id = Conversion::identified(pair);
    let attributes = Rules::get_rule(pair, Rule::attributes);
    let width = Conversion::width(&attributes, &config.unit).unwrap_or(config.ellipse.width);
    let height = Conversion::height(&attributes, &config.unit).unwrap_or(config.ellipse.height);

    let (stroke, fill, text_color) = Conversion::colors_from(&attributes);
    let title = Conversion::string_dig(&attributes, Rule::inner);
    let location = Conversion::location_from(&attributes, &config.flow.end, &config.unit);

    let paragraph = Self::paragraph_height(title, width, canvas);
    let height = paragraph.as_ref().map(|paragraph| height.max(paragraph.height)).unwrap_or(height);

    let mut used = Rect::from_xywh(cursor.x, cursor.y, width, height);

    Self::adjust_topleft(&config.flow, &mut used);
    index.position_rect(&location, &mut used);

    index.insert(ShapeName::Circle, id, used);

    let cylinder = Primitive(id, used, used, stroke, Shape::Cylinder(text_color, paragraph, fill, location));
    Some((used, cylinder))
  }

  fn config_shape(config: &mut ShapeConfig, pair: Pair<Rule>, unit: &Unit) {
    pair.into_inner().for_each(|pair| {
      match pair.as_rule() {
        Rule::padding => {
          let length = Conversion::length_from(pair, unit);
          config.padding = length.pixels();
        }
        Rule::height => {
          let length = Conversion::length_from(pair, unit);
          config.height = length.pixels();
        }
        Rule::width => {
          let length = Conversion::length_from(pair, unit);
          config.width = length.pixels();
        }
        _ => {
          warn!("Ignored {:?}", pair);
        }
      }
    });
  }

  fn arrow_from<'a>(pair: Pair<'a, Rule>, index: &mut Index, cursor: &Point, config: &Config) -> Option<(Rect, Node<'a>)> {
    let id = Conversion::identified(&pair);
    let caption = Conversion::caption(&pair);
    let length = Conversion::length(&pair, &config.unit).unwrap_or(config.line.pixels());

    let (source, movement, target) = Self::source_movement_target_from_pair(&pair, &config.unit);
    let start = index.point_index(&source, &[]).unwrap_or(*cursor);
    let end = index.point_index(&target, &[])
      .unwrap_or(Self::displace_from_start(start, &movement, &config.flow, length));
    let (rect, used) = Self::rect_from_points(start, &movement, end);

    index.insert(ShapeName::Arrow, id, used);
    let node = Primitive(id, rect, rect, Color::BLACK, Shape::Arrow(caption, source, movement, target));
    Some((used, node))
  }

  fn line_from<'a>(pair: Pair<'a, Rule>, index: &mut Index, cursor: &Point, config: &Config) -> Option<(Rect, Node<'a>)> {
    let id = Conversion::identified(&pair);
    let caption = Conversion::caption(&pair);
    let length = Conversion::length(&pair, &config.unit).unwrap_or(config.line.pixels());

    let (start, movement, end) = Self::points_from_pair(index, cursor, config, &pair, length);
    let (rect, used) = Self::rect_from_points(start, &movement, end);

    index.insert(ShapeName::Line, id, used);
    let node = Primitive(id, rect, rect, Color::BLACK, Shape::Line(caption, start, movement, end));
    Some((used, node))
  }

  fn source_movement_target_from_pair(pair: &Pair<Rule>, unit: &Unit) -> (ObjectEdge, Option<Movement>, ObjectEdge) {
    let source = Conversion::location_to_edge(pair, Rule::source)
      .unwrap_or(ObjectEdge::new("source", "e"));
    let movement = Conversion::rule_to_movement(pair, Rule::movement, unit);
    let target = Conversion::location_to_edge(pair, Rule::target)
      .unwrap_or(ObjectEdge::new("source", "w"));
    (source, movement, target)
  }

  fn displace_from_start(start: Point, movement: &Option<Movement>, flow: &Flow, default: f32) -> Point {
    movement.as_ref()
      .map(|movement| start.add(movement.offset()))
      .unwrap_or_else(|| {
        let movement = Movement::new(default, Unit::Px, flow.end.clone());
        start.add(movement.offset())
      })
  }

  fn points_from_pair(index: &mut Index, cursor: &Point, config: &Config, pair: &Pair<Rule>, default: f32) -> (Point, Option<Movement>, Point) {
    let (source, movement, target) = Self::source_movement_target_from_pair(pair, &config.unit);
    let start = index.point_index(&source, &[]).unwrap_or(*cursor);
    let end = index.point_index(&target, &[])
      .unwrap_or(Self::displace_from_start(start, &movement, &config.flow, default));
    (start, movement, end)
  }

  fn rect_from_points(start: Point, movement: &Option<Movement>, end: Point) -> (Rect, Rect) {
    let rect = Rect { left: start.x, top: start.y, right: end.x, bottom: end.y };
    let mut used = rect;
    if let Some(movement) = &movement {
      used.offset(movement.offset());
    }
    (rect, used)
  }

  fn dot_from<'a>(pair: &Pair<'a, Rule>, config: &Config, index: &mut Index) -> Option<(Rect, Node<'a>)> {
    let attributes = Rules::find_rule(pair, Rule::dot_attributes).unwrap();
    let color = Conversion::stroke_color(&attributes).unwrap_or(Color::BLUE);
    let radius = Conversion::radius(&attributes, &config.unit).unwrap_or_default();

    let object = Conversion::object_edge_from_pair(pair).unwrap();
    let point = index.point_index(&object, &[]).unwrap();
    let rect = Rect::from_xywh(point.x, point.y, 0., 0.);

    let dot = Primitive(None, rect, rect, color, Shape::Dot(object, radius));
    Some((rect, dot))
  }

  fn flow_from<'a>(pair: Pair<'a, Rule>, cursor: &Point, config: &mut Config) -> Option<(Rect, Node<'a>)> {
    let length = Conversion::length_from(pair, &config.unit);
    let mut used = Rect::from_xywh(cursor.x, cursor.y, 0., 0.);
    if config.flow.end.horizontal() {
      used.right += length.pixels();
    } else {
      used.bottom += length.pixels();
    }
    let node = Primitive(None, used, used, Color::BLACK, Shape::Move());
    Some((used, node))
  }

  fn move_from<'a>(pair: &Pair<'a, Rule>, cursor: &Point, unit: &Unit) -> Option<(Rect, Node<'a>)> {
    Conversion::movements_from_pair(pair, unit).map(|movements| {
      let mut used = Rect::from_xywh(cursor.x, cursor.y, 0., 0.);
      Index::offset_rect(&mut used, &movements);
      (used, Primitive(None, used, used, Color::BLACK, Shape::Move()))
    })
  }

  fn circle_from<'a>(pair: &Pair<'a, Rule>, config: &Config, index: &mut Index, cursor: &Point, canvas: &mut Canvas) -> Option<(Rect, Node<'a>)> {
    let id = Conversion::identified(pair);
    let attributes = Rules::get_rule(pair, Rule::attributes);
    let width = Conversion::width(&attributes, &config.unit).unwrap_or(config.circle.height);
    let height = Conversion::height(&attributes, &config.unit).unwrap_or(config.circle.height);

    let (stroke, fill, text_color) = Conversion::colors_from(&attributes);
    let title = Conversion::string_dig(&attributes, Rule::inner);
    let location = Conversion::location_from(&attributes, &config.flow.end, &config.unit);

    let paragraph = Self::paragraph_height(title, width, canvas);
    let height = paragraph.as_ref().map(|paragraph| height.max(paragraph.height)).unwrap_or(height);

    let mut used = Rect::from_xywh(cursor.x, cursor.y, width, height);

    Self::adjust_topleft(&config.flow, &mut used);
    index.position_rect(&location, &mut used);

    index.insert(ShapeName::Circle, id, used);

    let circle = Primitive(id, used, used, stroke, Shape::Circle(text_color, paragraph, fill, location));
    Some((used, circle))
  }

  fn paragraph_height<'a>(title: Option<&'a str>, width: f32, canvas: &mut Canvas) -> Option<Paragraph<'a>> {
    title.map(|title| {
      let (widths, height) = canvas.paragraph(title, (0., 0.), width - 2. * TEXT_PADDING);
      Paragraph { text: title, widths, height }
    })
  }

  fn position_rect_on_edge(start: &Edge, location: &Option<(Edge, Vec<Movement>, ObjectEdge)>, used: &mut Rect) {
    let start = match location {
      Some((edge, _, _)) => edge,
      None => start
    };
    let offset = start.topleft_offset(used);
    used.offset(offset);
  }

  fn adjust_topleft(flow: &Flow, used: &mut Rect) {
    let offset = flow.start.topleft_offset(used);
    used.offset(offset);
  }

  fn text_from<'a>(pair: &Pair<'a, Rule>, config: &Config, index: &mut Index, cursor: &Point, canvas: &mut Canvas) -> Option<(Rect, Node<'a>)> {
    let id = Conversion::identified(pair);
    let title = Conversion::string_dig(pair, Rule::inner).unwrap();
    let attributes = Rules::find_rule(pair, Rule::text_attributes).unwrap();
    let width = Conversion::width(&attributes, &config.unit).unwrap_or(config.width);
    let location = Conversion::location_from(pair, &config.flow.end, &config.unit);
    let (_widths, height) = canvas.paragraph(title, (0., 0.), width - 2. * TEXT_PADDING);

    let mut used = Rect::from_xywh(cursor.x, cursor.y, width, height);
    used.bottom += BLOCK_PADDING;

    Self::adjust_topleft(&config.flow, &mut used);
    index.position_rect(&location, &mut used);

    index.insert(ShapeName::Text, id, used);

    let text = Primitive(id, used, used, Color::BLACK, Shape::Text(title, location));
    Some((used, text))
  }

  fn update_bounds(bounds: &mut Rect, rect: Rect) {
    bounds.top = bounds.top.min(rect.top);
    bounds.left = bounds.left.min(rect.left);
    bounds.right = bounds.right.max(rect.right);
    bounds.bottom = bounds.bottom.max(rect.bottom);
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

  pub fn node_mut(&mut self, id: &str, movements: Vec<Movement>) {
    if let Primitive(_, _, ref mut rect, _, _) = Diagram::find_nodes_mut(&mut self.nodes, id).unwrap() {
      for movement in movements.iter() {
        rect.offset(movement.offset());
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
    self.write_to_file(filepath, &mut canvas);
  }

  pub fn shrink_to_file(&mut self, filepath: &str) {
    let size = self.bounds.with_outset(self.inset);
    let size = ISize::new(size.width() as i32, size.height() as i32);
    let mut canvas = Canvas::new(size);
    self.write_to_file(filepath, &mut canvas);
  }

  fn write_to_file(&mut self, filepath: &str, canvas: &mut Canvas) {
    canvas.translate(-self.bounds.left + self.inset.x, -self.bounds.top + self.inset.y);
    Renderer::render_to_canvas(canvas, &self.nodes);
    canvas.write_png(filepath);
  }
}

pub const TEXT_PADDING: f32 = 4.;

#[allow(dead_code)]
pub fn dump_nested(level: usize, pairs: Pairs<Rule>) {
  for pair in pairs.into_iter() {
    println!("{:level$} {:?}", level, pair);
    dump_nested(level + 1, pair.into_inner());
  }
}