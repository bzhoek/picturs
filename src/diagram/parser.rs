#[cfg(test)]
mod tests;

use std::ops::Add;

use log::{debug, info, warn};
use pest::iterators::{Pair, Pairs};
use pest_derive::Parser;
use skia_safe::{Color, Font, FontMgr, FontStyle, ISize, Point, Rect, Size};

use crate::diagram::conversion::Conversion;
use crate::diagram::index::{Index, ShapeName};
use crate::diagram::renderer::Renderer;
use crate::diagram::rules::Rules;
use crate::diagram::types::{Arrows, BLOCK_PADDING, Caption, Config, Edge, Flow, Length, Movement, Node, ObjectEdge, Paragraph, Shape, ShapeConfig, Unit};
use crate::diagram::types::Node::{Container, Primitive};
use crate::skia::Canvas;

#[derive(Debug)]
struct ClosedAttributes<'a> {
  id: Option<&'a str>,
  attributes: Pair<'a, Rule>,
  width: f32,
  height: f32,
  padding: f32,
  radius: Length,
  title: Option<&'a str>,
  location: Option<(Edge, Vec<Movement>, ObjectEdge)>,
  stroke: Color,
  fill: Color,
  text: Color,
}

#[derive(Debug)]
struct OpenAttributes<'a> {
  id: Option<&'a str>,
  attributes: Pair<'a, Rule>,
  same: bool,
  caption: Option<Caption<'a>>,
  length: f32,
  arrows: Arrows,
  source: Option<ObjectEdge>,
  target: Option<ObjectEdge>,
  movement: Option<Movement>,
}

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
    let top = Conversion::pairs_for(Rule::picture, string);
    let flow = Flow::new("left");
    let config = Config::new(flow, 120., 60.);
    let mut index = Index::default();
    let (ast, bounds) = Self::nodes_from(top.clone(), vec![], &Point::new(0.5, 0.5), config, &mut index);
    self.nodes = ast;
    self.bounds = bounds;
    top
  }

  pub fn nodes_from<'a>(pairs: Pairs<'a, Rule>, mut ast: Vec<Node<'a>>, offset: &Point, mut config: Config, index: &mut Index)
                        -> (Vec<Node<'a>>, Rect) {
    let mut bounds = Rect::from_xywh(offset.x, offset.y, 0., 0.);
    let mut cursor = Point::new(offset.x, offset.y);

    for pair in pairs.into_iter() {
      let result = Self::node_from(pair, &mut config, index, &mut cursor);

      if let Some((rect, node)) = result {
        ast.push(node);
        Self::update_bounds(&mut bounds, rect);
        let point = config.flow.end.edge_point(&rect);
        cursor = point
      }
    }
    (ast, bounds)
  }

  fn node_from<'a>(pair: Pair<'a, Rule>, config: &mut Config, index: &mut Index, cursor: &mut Point) -> Option<(Rect, Node<'a>)> {
    let result = match pair.as_rule() {
      Rule::container => Self::container_from(&pair, config, index, cursor),
      Rule::circle => Self::circle_from(&pair, config, index, cursor),
      Rule::cylinder => Self::cylinder_from(&pair, config, index, cursor),
      Rule::ellipse => Self::ellipse_from(&pair, config, index, cursor),
      Rule::file => Self::file_from(&pair, config, index, cursor),
      Rule::oval => Self::oval_from(&pair, config, index, cursor),
      Rule::rectangle => Self::box_from(&pair, config, index, cursor),
      Rule::arrow => Self::arrow_from(pair, config, index, cursor),
      Rule::line => Self::line_from(pair, config, index, cursor),
      Rule::text => Self::text_from(&pair, config, index, cursor),
      Rule::dot => Self::dot_from(&pair, config, index, cursor),
      Rule::flow_to => Self::flow_from(pair, cursor, config),
      Rule::move_to => Self::move_from(&pair, cursor, &config.unit),
      Rule::font_config => {
        let name = Conversion::string_dig(&pair, Rule::inner).unwrap();
        let typeface = FontMgr::default().match_family_style(name, FontStyle::default()).unwrap();
        config.font = Font::from_typeface(typeface, 17.0);
        let rect = Rect::from_xywh(cursor.x, cursor.y, 0., 0.);
        let node = Primitive(None, rect, rect, Color::BLACK, Shape::Font(config.font.clone()));
        Some((rect, node))
      }
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
      Rule::line_config => {
        let length = Conversion::length_dig(&pair, Rule::length, &config.unit).unwrap();
        config.line = length;
        None
      }
      Rule::flow => {
        config.flow = Flow::new(pair.as_str());
        None
      }
      _ => {
        debug!("Unmatched {:?}", pair);
        None
      }
    };
    result
  }

  fn closed_attributes<'a>(pair: &Pair<'a, Rule>, config: &Config, shape: &ShapeConfig) -> ClosedAttributes<'a> {
    let attributes = Rules::get_rule(pair, Rule::attributes);
    let (stroke, fill, text) = Conversion::colors_from(&attributes);

    ClosedAttributes {
      id: Conversion::identified(pair),
      width: Conversion::width(&attributes, &config.unit).unwrap_or(shape.width),
      height: Conversion::height(&attributes, &config.unit).unwrap_or(shape.height),
      padding: Conversion::padding(&attributes, &config.unit).unwrap_or(shape.padding),
      radius: Conversion::radius(&attributes, &config.unit).unwrap_or_default(),
      title: Conversion::string_dig(&attributes, Rule::inner),
      location: Conversion::location_from(&attributes, &Edge::from("c"), &config.unit),
      attributes,
      stroke,
      fill,
      text,
    }
  }

  fn container_from<'a>(pair: &Pair<'a, Rule>, config: &Config, index: &mut Index, cursor: &Point) -> Option<(Rect, Node<'a>)> {
    let attrs = Self::closed_attributes(pair, config, &config.rectangle);

    let mut used = Rect::from_xywh(cursor.x, cursor.y, 0., 0.);
    index.position_rect(&attrs.location, &mut used);

    let mut inset = Point::new(used.left, used.bottom);
    inset.offset((attrs.padding, attrs.padding));

    let (nodes, inner) = {
      let mut config = config.clone();
      Conversion::flow(&attrs.attributes).into_iter().for_each(|flow| {
        config.flow = flow;
      });
      Self::nodes_from(pair.clone().into_inner(), vec![], &inset, config, index)
    };

    used = inner.with_outset((attrs.padding, attrs.padding));

    if let Some(title) = attrs.title {
      let text_inset = inner.with_inset((TEXT_PADDING, TEXT_PADDING));
      let (_widths, down) = config.measure_strings(title, text_inset.width());
      used.bottom = inner.bottom + down + TEXT_PADDING;
    }

    index.insert(ShapeName::Container, attrs.id, used);

    let mut rect = used;
    rect.bottom += attrs.padding;
    Some((rect, Container(attrs.id, attrs.radius, attrs.title, rect, used, nodes)))
  }

  fn circle_from<'a>(pair: &Pair<'a, Rule>, config: &Config, index: &mut Index, cursor: &Point) -> Option<(Rect, Node<'a>)> {
    let attrs = Self::closed_attributes(pair, config, &config.circle);

    let paragraph = Self::paragraph_height(attrs.title, attrs.width, config);
    let height = paragraph.as_ref().map(|paragraph| attrs.height.max(paragraph.height)).unwrap_or(attrs.height);

    let mut used = Rect::from_xywh(cursor.x, cursor.y, height, height);

    Self::adjust_topleft(&config.flow, &mut used);
    index.position_rect(&attrs.location, &mut used);

    index.insert(ShapeName::Circle, attrs.id, used);

    let circle = Primitive(
      attrs.id, used, used, attrs.stroke,
      Shape::Circle(attrs.text, paragraph, attrs.fill, attrs.location));
    Some((used, circle))
  }

  fn cylinder_from<'a>(pair: &Pair<'a, Rule>, config: &Config, index: &mut Index, cursor: &Point) -> Option<(Rect, Node<'a>)> {
    let attrs = Self::closed_attributes(pair, config, &config.cylinder);

    let paragraph = Self::paragraph_height(attrs.title, attrs.width, config);
    let height = paragraph.as_ref().map(|paragraph| attrs.height.max(paragraph.height)).unwrap_or(attrs.height);

    let mut used = Rect::from_xywh(cursor.x, cursor.y, attrs.width, height);

    Self::adjust_topleft(&config.flow, &mut used);
    index.position_rect(&attrs.location, &mut used);

    index.insert(ShapeName::Circle, attrs.id, used);

    let cylinder = Primitive(
      attrs.id, used, used, attrs.stroke,
      Shape::Cylinder(attrs.text, paragraph, attrs.fill, attrs.location));
    Some((used, cylinder))
  }

  fn ellipse_from<'a>(pair: &Pair<'a, Rule>, config: &Config, index: &mut Index, cursor: &Point) -> Option<(Rect, Node<'a>)> {
    let attrs = Self::closed_attributes(pair, config, &config.ellipse);

    let paragraph = Self::paragraph_height(attrs.title, attrs.width, config);
    let height = paragraph.as_ref().map(|paragraph| attrs.height.max(paragraph.height)).unwrap_or(attrs.height);

    let mut used = Rect::from_xywh(cursor.x, cursor.y, attrs.width, height);

    Self::adjust_topleft(&config.flow, &mut used);
    index.position_rect(&attrs.location, &mut used);

    index.insert(ShapeName::Ellipse, attrs.id, used);

    let ellipse = Primitive(
      attrs.id, used, used, attrs.stroke,
      Shape::Ellipse(attrs.text, paragraph, attrs.fill, attrs.location));
    Some((used, ellipse))
  }

  fn file_from<'a>(pair: &Pair<'a, Rule>, config: &Config, index: &mut Index, cursor: &Point) -> Option<(Rect, Node<'a>)> {
    let attrs = Self::closed_attributes(pair, config, &config.file);

    let paragraph = Self::paragraph_height(attrs.title, attrs.width, config);
    let height = paragraph.as_ref().map(|paragraph| attrs.height.max(paragraph.height)).unwrap_or(attrs.height);

    let mut used = Rect::from_xywh(cursor.x, cursor.y, attrs.width, height);
    used.bottom += attrs.padding;
    Self::adjust_topleft(&config.flow, &mut used);
    index.position_rect(&attrs.location, &mut used);

    index.insert(ShapeName::File, attrs.id, used);

    let mut rect = used;
    if config.flow.end.x <= 0. {
      rect.bottom += attrs.padding;
    }

    let file = Primitive(
      attrs.id, rect, used, attrs.stroke,
      Shape::File(attrs.text, paragraph, attrs.radius, attrs.fill, attrs.location));
    Some((rect, file))
  }

  fn oval_from<'a>(pair: &Pair<'a, Rule>, config: &Config, index: &mut Index, cursor: &Point) -> Option<(Rect, Node<'a>)> {
    let attrs = Self::closed_attributes(pair, config, &config.oval);
    let paragraph = Self::paragraph_height(attrs.title, attrs.width, config);
    let mut used = Rect::from_xywh(cursor.x, cursor.y, attrs.width, attrs.height);

    Self::position_rect_on_edge(&config.flow.start, &attrs.location, &mut used);
    index.position_rect(&attrs.location, &mut used);

    index.insert(ShapeName::Oval, attrs.id, used);

    let node = Primitive(
      attrs.id, used, used, attrs.stroke,
      Shape::Oval(attrs.text, paragraph, attrs.fill, attrs.location));
    Some((used, node))
  }

  fn box_from<'a>(pair: &Pair<'a, Rule>, config: &Config, index: &mut Index, cursor: &Point) -> Option<(Rect, Node<'a>)> {
    let attrs = Self::closed_attributes(pair, config, &config.rectangle);

    let paragraph = Self::paragraph_height(attrs.title, attrs.width, config);
    let height = paragraph.as_ref().map(|paragraph| attrs.height.max(paragraph.height)).unwrap_or(attrs.height);

    let mut used = Rect::from_xywh(cursor.x, cursor.y, attrs.width, height);
    used.bottom += attrs.padding;
    Self::adjust_topleft(&config.flow, &mut used);
    index.position_rect(&attrs.location, &mut used);

    index.insert(ShapeName::Box, attrs.id, used);

    let mut rect = used;
    if config.flow.end.x <= 0. {
      rect.bottom += attrs.padding;
    }

    let rectangle = Primitive(
      attrs.id, rect, used, attrs.stroke,
      Shape::Box(attrs.text, paragraph, attrs.radius, attrs.fill, attrs.location));
    Some((rect, rectangle))
  }

  fn open_attributes<'a>(pair: &Pair<'a, Rule>, config: &Config) -> OpenAttributes<'a> {
    let attributes = Rules::get_rule(pair, Rule::line_attributes);

    OpenAttributes {
      id: Conversion::identified(pair),
      caption: Conversion::caption(&attributes, config),
      length: Conversion::length(&attributes, &config.unit).unwrap_or(config.line.pixels()),
      arrows: Conversion::arrows(&attributes),
      source: Conversion::location_to_edge(&attributes, Rule::source),
      target: Conversion::location_to_edge(&attributes, Rule::target),
      movement: Conversion::rule_to_movement(&attributes, Rule::movement, &config.unit),
      same: Rules::find_rule(pair, Rule::same).is_some(),
      attributes,
    }
  }

  fn arrow_from<'a>(pair: Pair<'a, Rule>, config: &Config, index: &mut Index, cursor: &Point) -> Option<(Rect, Node<'a>)> {
    let attrs = Self::open_attributes(&pair, config);

    let (source, movement, target) = Self::source_movement_target_from_pair(&attrs.attributes, &config.unit);
    let start = index.point_index(attrs.source.as_ref(), &[]).unwrap_or(*cursor);
    let end = index.point_index(attrs.target.as_ref(), &[])
      .unwrap_or(Self::displace_from_start(start, &attrs.movement, &config.flow, attrs.length));
    let (rect, used) = Self::rect_from_points(start, &movement, end);

    index.insert(ShapeName::Arrow, attrs.id, used);

    let node = Primitive(
      attrs.id, rect, rect, Color::BLACK,
      Shape::Arrow(source, movement, target, attrs.caption));
    Some((used, node))
  }

  fn line_from<'a>(pair: Pair<'a, Rule>, config: &Config, index: &mut Index, cursor: &Point) -> Option<(Rect, Node<'a>)> {
    let attrs = Self::open_attributes(&pair, config);

    let start = index.point_index(attrs.source.as_ref(), &[]).unwrap_or(*cursor);
    let end = index.point_index(attrs.target.as_ref(), &[])
      .unwrap_or(Self::displace_from_start(start, &attrs.movement, &config.flow, attrs.length));

    let (rect, used) = Self::rect_from_points(start, &attrs.movement, end);

    if let Some((caption, movement)) = attrs.caption.as_ref().zip(attrs.movement.as_ref()) {
      if caption.inner.vertical() && movement.edge.vertical() {
        info!("VERTICAL! {:?}", caption.size);
      }
    }

    index.insert(ShapeName::Line, attrs.id, used);
    let node = Primitive(
      attrs.id, rect, rect, Color::BLACK,
      Shape::Line(start, attrs.movement, end, attrs.caption, attrs.arrows));
    Some((used, node))
  }

  fn text_from<'a>(pair: &Pair<'a, Rule>, config: &Config, index: &mut Index, cursor: &Point) -> Option<(Rect, Node<'a>)> {
    let id = Conversion::identified(pair);
    let title = Conversion::string_dig(pair, Rule::inner).unwrap();
    let attributes = Rules::find_rule(pair, Rule::text_attributes).unwrap();
    let location = Conversion::location_from(pair, &config.flow.end, &config.unit);

    let fit = Rules::dig_rule(&attributes, Rule::fit);
    let paragraph = match fit {
      Some(_) => {
        let size = config.measure_string(title);
        Paragraph { text: title, widths: vec![size.width], height: size.height, size }
      }
      None => {
        let width = Conversion::width(&attributes, &config.unit).unwrap_or(config.width);
        let (widths, height) = config.measure_strings(title, width - 2. * TEXT_PADDING);
        let size = Size::new(width, height);
        Paragraph { text: title, widths, height, size }
      }
    };

    let mut used = Rect::from_xywh(cursor.x, cursor.y, paragraph.size.width, paragraph.size.height);
    used.bottom += BLOCK_PADDING;

    Self::adjust_topleft(&config.flow, &mut used);
    index.position_rect(&location, &mut used);

    index.insert(ShapeName::Text, id, used);

    let text = Primitive(
      id, used, used, Color::BLACK,
      Shape::Text(paragraph, location));
    Some((used, text))
  }

  fn dot_from<'a>(pair: &Pair<'a, Rule>, config: &Config, index: &mut Index, cursor: &Point) -> Option<(Rect, Node<'a>)> {
    let caption = Conversion::caption(pair, config);
    let attributes = Rules::find_rule(pair, Rule::dot_attributes).unwrap();
    let _same = Rules::dig_rule(&attributes, Rule::same);
    let color = Conversion::stroke_color(&attributes).unwrap_or(Color::BLUE);
    let radius = Conversion::radius(&attributes, &config.unit).unwrap_or(config.dot.clone());

    let mut point = match Conversion::object_edge_from_pair(pair) {
      Some(_) => {
        let object = Conversion::object_edge_from_pair(pair).unwrap();
        index.point_index(Some(&object), &[]).unwrap()
      }
      None => *cursor
    };

    let inner = pair.clone().into_inner().next().unwrap();
    if inner.as_rule() == Rule::object_edge {
      let (id, degrees) = Conversion::object_edge_in_degrees_from(inner);
      if let Some(degrees) = degrees {
        let edge = Edge::from(degrees as f32);
        let object = ObjectEdge::new(id, edge);
        point = index.point_index(Some(&object), &[]).unwrap()
      }
    };

    let mut bounds = Rect::from_xywh(point.x, point.y, 0., 0.);
    if let Some(caption) = &caption {
      let rect = Renderer::dot_offset_of(&point, &radius, caption);
      Self::update_bounds(&mut bounds, rect);
    }

    let node = Primitive(
      None, bounds, bounds, color,
      Shape::Dot(point, radius, caption));
    Some((bounds, node))
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

  fn rect_from_points(start: Point, movement: &Option<Movement>, end: Point) -> (Rect, Rect) {
    let rect = Rect { left: start.x, top: start.y, right: end.x, bottom: end.y };
    let mut used = rect;
    if let Some(movement) = &movement {
      used.offset(movement.offset());
    }
    (rect, used)
  }

  fn paragraph_height<'a>(title: Option<&'a str>, width: f32, config: &Config) -> Option<Paragraph<'a>> {
    title.map(|title| {
      let (widths, height) = config.measure_strings(title, width - 2. * TEXT_PADDING);
      let size = Size::new(width, height);
      Paragraph { text: title, widths, height, size }
    })
  }

  fn position_rect_on_edge(start: &Edge, location: &Option<(Edge, Vec<Movement>, ObjectEdge)>, used: &mut Rect) {
    let start = match location {
      Some((edge, _, _)) => edge,
      None => start
    };
    start.offset(used);
  }

  fn adjust_topleft(flow: &Flow, used: &mut Rect) {
    flow.start.offset(used);
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