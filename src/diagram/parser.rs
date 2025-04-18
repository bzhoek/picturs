use std::ops::Add;
use std::path::Path;
use log::{debug, info, warn};
use pest::iterators::{Pair, Pairs};
use pest_derive::Parser;
use skia_safe::{Color, ISize, Point, Rect, Size, Vector};

use crate::diagram::attributes::Attributes;
use crate::diagram::conversion::Conversion;
use crate::diagram::index::{Index, ShapeName};
use crate::diagram::renderer::Renderer;
use crate::diagram::rules::Rules;
use crate::diagram::types::{BLOCK_PADDING, CommonAttributes, Config, Displacement, Edge, EdgeDirection, Continuation, Movement, Node, ObjectEdge, Paragraph, Shape, ShapeConfig, Unit};
use crate::diagram::types::Node::{Closed, Container, Open, Primitive};
use crate::skia::Canvas;

#[cfg(test)]
mod tests;

#[derive(Parser)]
#[grammar = "picturs.pest"]
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
    let config = Config::default();
    let mut index = Index::default();

    let cursor = Point::new(0.5, 0.5);
    let node = Node::Font(config.font.clone());
    let _ast = vec![node];
    let (ast, bounds) = Self::nodes_from(top.clone(), vec![], &cursor, config, &mut index);
    self.nodes = ast;
    self.bounds = bounds;
    top
  }

  pub fn nodes_from<'a>(pairs: Pairs<'a, Rule>, mut ast: Vec<Node<'a>>, offset: &Point, mut config: Config, index: &mut Index<'a>)
                        -> (Vec<Node<'a>>, Rect) {
    let mut bounds = Rect::from_xywh(offset.x, offset.y, 0., 0.);
    let mut cursor = Point::new(offset.x, offset.y);

    for pair in pairs.into_iter() {
      let result = Self::node_from(pair, &mut config, index, &mut cursor);

      if let Some((rect, node)) = result {
        ast.push(node);
        Self::bounds_from_rect(&mut bounds, rect);
        let point = config.continuation.end.edge_point(&rect);
        cursor = point
      }
    }
    (ast, bounds)
  }

  fn transform_nodes(nodes: &mut Vec<Node>, offset: impl Into<Vector>) {
    let offset: Point = offset.into();
    for node in nodes {
      match node {
        Container(_, ref mut used, ref mut nodes) => {
          used.offset(offset);
          println!("Container {:?} delta {:?}", used, offset);
          Self::transform_nodes(nodes, offset);
        }
        Primitive(ref mut attrs, _) => {
          attrs.used.offset(offset);
        }
        Closed(_, ref mut used, _, _) => {
          used.offset(offset);
          println!("Closed {:?}", used);
        }
        Open(_, ref mut used, _) => {
          used.offset(offset);
          println!("Closed {:?}", used);
        }
        Node::Move(_) => {}
        Node::Font(_) => {}
      }
    }
  }

  fn node_from<'a>(pair: Pair<'a, Rule>, config: &mut Config, index: &mut Index<'a>, cursor: &mut Point) -> Option<(Rect, Node<'a>)> {
    let result = match pair.as_rule() {
      Rule::container => Self::container_from(&pair, config, index, cursor, &config.container),
      Rule::group => Self::container_from(&pair, config, index, cursor, &config.group),
      Rule::circle => Self::circle_from(&pair, config, index, cursor),
      Rule::cylinder => Self::cylinder_from(&pair, config, index, cursor),
      Rule::ellipse => Self::ellipse_from(&pair, config, index, cursor),
      Rule::file => Self::file_from(&pair, config, index, cursor),
      Rule::oval => Self::oval_from(&pair, config, index, cursor),
      Rule::rectangle => Self::box_from(&pair, config, index, cursor),
      Rule::arrow => Self::arrow_from(pair, config, index, cursor),
      Rule::line => Self::line_from(pair, config, index, cursor),
      Rule::sline => Self::sline_from(pair, config, index, cursor),
      Rule::path => Self::path_from(pair, config, index, cursor),
      Rule::text => Self::text_from(&pair, config, index, cursor),
      Rule::dot => Self::dot_from(&pair, config, index, cursor),
      Rule::flow_to => Self::flow_from(pair, cursor, config),
      Rule::move_to => Self::move_from(&pair, cursor, &config.unit),
      Rule::font_config => {
        config.font = Conversion::font_from(pair, &config.unit);
        let rect = Rect::from_xywh(cursor.x, cursor.y, 0., 0.);
        let node = Node::Font(config.font.clone());
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
        let length = Conversion::length_in(&pair, Rule::length, &config.unit).unwrap();
        config.line = length;
        None
      }
      Rule::continuation => {
        config.continuation = Continuation::new(pair.as_str());
        None
      }
      Rule::continue_from => {
        let direction = Conversion::str_for(&pair, Rule::continue_direction).unwrap();
        config.continuation = Continuation::new(direction);
        None
      }
      _ => None
    };
    result
  }

  fn container_from<'a>(pair: &Pair<'a, Rule>, config: &Config, index: &mut Index<'a>, cursor: &Point, shape: &ShapeConfig) -> Option<(Rect, Node<'a>)> {
    let (mut attrs, attributes) = Attributes::closed_attributes(pair, config, shape);
    Self::copy_same_attributes(index, &mut attrs, ShapeName::Container);

    if let Attributes::Closed {
      id,
      title,
      padding,
      location,
      ..
    } = &attrs {
      let mut used = Rect::from_xywh(cursor.x, cursor.y, 0., 0.);
      index.position_rect(location, &mut used);

      let mut inset = Point::new(used.left, used.bottom);
      inset.offset((*padding, *padding));

      let original = inset;

      let (mut nodes, bounds) = {
        let mut config = config.clone();
        Conversion::continuation_in(&attributes).into_iter().for_each(|continuation| {
          config.continuation = continuation;
        });
        Self::nodes_from(pair.clone().into_inner(), vec![], &inset, config, index)
      };

      let moved = original - Point::new(bounds.left, bounds.top);
      used = bounds.with_outset((*padding, *padding));

      if let Some(title) = title {
        let text_inset = bounds.with_inset((TEXT_PADDING, TEXT_PADDING));
        let (_widths, down) = config.measure_strings(title, text_inset.width());
        used.bottom = bounds.bottom + down + TEXT_PADDING;
      }

      let mut shifted = used.with_offset(moved);
      Self::adjust_topleft(&config.continuation, &mut shifted);
      let offset = Point::new(shifted.left, shifted.top) - Point::new(used.left, used.top);
      Self::transform_nodes(&mut nodes, offset);

      index.insert(ShapeName::Container, *id, shifted);

      let mut padded = shifted;
      padded.bottom += padding;
      return Some((padded, Container(attrs, shifted, nodes)));
    }
    None
  }

  fn circle_from<'a>(pair: &Pair<'a, Rule>, config: &Config, index: &mut Index, cursor: &Point) -> Option<(Rect, Node<'a>)> {
    let (mut attrs, _) = Attributes::closed_attributes(pair, config, &config.circle);
    Self::copy_same_attributes(index, &mut attrs, ShapeName::Circle);

    if let Attributes::Closed {
      id, title,
      width, height,
      location,
      ..
    } = &attrs {
      let (paragraph, size) = Self::paragraph_sized(title.as_deref(), width, height, config, &config.circle);
      let mut used = Rect::from_xywh(cursor.x, cursor.y, size.height, size.height);

      Self::adjust_topleft(&config.continuation, &mut used);
      index.position_rect(location, &mut used);
      index.insert(ShapeName::Circle, *id, used);

      let circle = Closed(attrs, used, paragraph, Shape::Circle);
      return Some((used, circle));
    }
    None
  }

  fn cylinder_from<'a>(pair: &Pair<'a, Rule>, config: &Config, index: &mut Index, cursor: &Point) -> Option<(Rect, Node<'a>)> {
    let (mut attrs, _) = Attributes::closed_attributes(pair, config, &config.cylinder);
    Self::copy_same_attributes(index, &mut attrs, ShapeName::Cylinder);

    if let Attributes::Closed {
      id, title,
      width, height,
      location,
      ..
    } = &attrs {
      let (paragraph, size) = Self::paragraph_sized(title.as_deref(), width, height, config, &config.cylinder);
      let mut used = Rect::from_point_and_size(*cursor, size);

      Self::adjust_topleft(&config.continuation, &mut used);
      index.position_rect(location, &mut used);

      index.insert(ShapeName::Cylinder, *id, used);

      let cylinder = Closed(attrs, used, paragraph, Shape::Cylinder);
      return Some((used, cylinder));
    }
    None
  }

  fn ellipse_from<'a>(pair: &Pair<'a, Rule>, config: &Config, index: &mut Index, cursor: &Point) -> Option<(Rect, Node<'a>)> {
    let (mut attrs, _) = Attributes::closed_attributes(pair, config, &config.ellipse);
    Self::copy_same_attributes(index, &mut attrs, ShapeName::Ellipse);

    if let Attributes::Closed {
      id, title,
      width, height,
      location,
      ..
    } = &attrs {
      let (paragraph, size) = Self::paragraph_sized(title.as_deref(), width, height, config, &config.ellipse);
      let mut used = Rect::from_point_and_size(*cursor, size);

      Self::adjust_topleft(&config.continuation, &mut used);
      index.position_rect(location, &mut used);
      index.insert(ShapeName::Ellipse, *id, used);

      let ellipse = Closed(attrs, used, paragraph, Shape::Ellipse);
      return Some((used, ellipse));
    }
    None
  }

  fn file_from<'a>(pair: &Pair<'a, Rule>, config: &Config, index: &mut Index, cursor: &Point) -> Option<(Rect, Node<'a>)> {
    let (mut attrs, _) = Attributes::closed_attributes(pair, config, &config.file);
    Self::copy_same_attributes(index, &mut attrs, ShapeName::File);

    if let Attributes::Closed {
      id, title,
      width, height,
      location,
      ..
    } = &attrs {
      let (paragraph, size) = Self::paragraph_sized(title.as_deref(), width, height, config, &config.file);
      let mut used = Rect::from_point_and_size(*cursor, size);

      Self::adjust_topleft(&config.continuation, &mut used);
      index.position_rect(location, &mut used);

      index.insert(ShapeName::File, *id, used);

      let file = Closed(attrs, used, paragraph, Shape::File);
      return Some((used, file));
    }
    None
  }

  fn oval_from<'a>(pair: &Pair<'a, Rule>, config: &Config, index: &mut Index, cursor: &Point) -> Option<(Rect, Node<'a>)> {
    let (mut attrs, _) = Attributes::closed_attributes(pair, config, &config.oval);
    Self::copy_same_attributes(index, &mut attrs, ShapeName::Oval);

    if let Attributes::Closed {
      id, title,
      width, height,
      location,
      ..
    } = &attrs {
      let (paragraph, size) = Self::paragraph_sized(title.as_deref(), width, height, config, &config.oval);
      let mut used = Rect::from_point_and_size(*cursor, size);

      Self::position_rect_on_edge(&config.continuation.start, location, &mut used);
      index.position_rect(location, &mut used);
      index.insert(ShapeName::Oval, *id, used);

      let oval = Closed(attrs, used, paragraph, Shape::Oval);
      return Some((used, oval));
    }
    None
  }

  fn create_rect(width: Option<f32>, height: Option<f32>, config: &ShapeConfig) -> Rect {
    let width = width.unwrap_or(config.width);
    let height = height.unwrap_or(config.height);
    Rect::from_xywh(0., 0., width, height)
  }

  fn box_from<'a>(pair: &Pair<'a, Rule>, config: &Config, index: &mut Index<'a>, cursor: &Point) -> Option<(Rect, Node<'a>)> {
    let (mut attrs, _) = Attributes::closed_attributes(pair, config, &config.rectangle);
    Self::copy_same_attributes(index, &mut attrs, ShapeName::Box);

    if let Attributes::Closed {
      id,
      title,
      width,
      height,
      padding,
      space,
      location,
      ..
    } = &attrs {
      let rect = Self::create_rect(*width, *height, &config.rectangle);
      let rect = Self::adjust_rect(&rect, config.continuation.direction, -*space);

      let (paragraph, size) = Self::paragraph_sized_(title.as_deref(), rect.size(), config);
      let mut inner = Rect::from_point_and_size(*cursor, size);
      inner.bottom += padding; // for text

      Self::adjust_topleft(&config.continuation, &mut inner);
      index.position_rect(location, &mut inner);

      let outer = Self::adjust_rect(&inner, config.continuation.direction, *space);
      index.insert(ShapeName::Box, *id, outer);
      index.add_open(ShapeName::Box, attrs.clone());

      // TODO explain the outer bounds concept with regards to the continuation cursor
      let bounds = Self::adjust_rect(&outer, config.continuation.direction, *padding);
      let rectangle = Closed(attrs, inner, paragraph, Shape::Rectangle);
      return Some((bounds, rectangle));
    }
    None
  }

  fn adjust_rect(rect: &Rect, direction: EdgeDirection, change: f32) -> Rect {
    let mut rect = *rect;
    match direction {
      EdgeDirection::Horizontal => rect.right += change,
      EdgeDirection::Vertical => rect.bottom += change
    }
    rect
  }

  fn arrow_from<'a>(pair: Pair<'a, Rule>, config: &Config, index: &mut Index<'a>, cursor: &Point) -> Option<(Rect, Node<'a>)> {
    let (mut attrs, attributes) = Attributes::open_attributes(&pair, config, Rule::line_attributes);
    Self::copy_same_attributes(index, &mut attrs, ShapeName::Arrow);

    if let Attributes::Open {
      id, source, target, length, ref caption, ..
    } = &attrs {
      let (source_edge, movement, target_edge) = Self::source_movement_target_from_pair(&attributes, &config.unit);
      let start = index.point_index(source.as_ref(), &[]).unwrap_or(*cursor);
      let end = index.point_index(target.as_ref(), &[])
        .unwrap_or(Self::displace_from_start(start, &movement, &config.continuation, *length));
      let (rect, used) = Self::rect_from_points(start, &movement, end);

      index.insert(ShapeName::Arrow, *id, used);
      index.add_open(ShapeName::Arrow, attrs.clone());

      let shape = Shape::Arrow(source_edge, movement, target_edge, caption.clone());
      let node = Open(attrs, rect, shape);
      return Some((used, node));
    }
    None
  }

  fn line_from<'a>(pair: Pair<'a, Rule>, config: &Config, index: &mut Index<'a>, cursor: &Point) -> Option<(Rect, Node<'a>)> {
    let (mut attrs, _) = Attributes::open_attributes(&pair, config, Rule::line_attributes);
    Self::copy_same_attributes(index, &mut attrs, ShapeName::Line);

    match &attrs {
      Attributes::Closed { .. } => panic!("Wrong type"),
      Attributes::Open {
        id,
        source,
        target,
        movement,
        caption,
        length,
        ref endings,
        ..
      } => {
        let start = index.point_index(source.as_ref(), &[]).unwrap_or(*cursor);
        let end = index.point_index(target.as_ref(), &[])
          .unwrap_or(Self::displace_from_start(start, movement, &config.continuation, *length));

        let (rect, used) = Self::rect_from_points(start, movement, end);

        if let Some((caption, movement)) = caption.as_ref().zip(movement.as_ref()) {
          if caption.inner.vertical() && movement.edge.vertical() {
            info!("VERTICAL! {:?}", caption.size);
          }
        }

        index.insert(ShapeName::Line, *id, used);
        index.add_open(ShapeName::Line, attrs.clone());

        let shape = Shape::Line(start, movement.clone(), end, caption.clone(), endings.clone());
        let node = Open(attrs, rect, shape);
        Some((used, node))
      }
    }
  }

  fn sline_from<'a>(pair: Pair<'a, Rule>, config: &Config, index: &mut Index<'a>, cursor: &Point) -> Option<(Rect, Node<'a>)> {
    let (mut attrs, _) = Attributes::open_attributes(&pair, config, Rule::line_attributes);
    Self::copy_same_attributes(index, &mut attrs, ShapeName::Line);

    match &attrs {
      Attributes::Closed { .. } => panic!("Wrong type"),
      Attributes::Open {
        id,
        source,
        target,
        movement,
        caption,
        length,
        ref endings,
        stroke,
        ..
      } => {
        let start = index.point_index(source.as_ref(), &[]).unwrap_or(*cursor);
        let end = index.point_index(target.as_ref(), &[])
          .unwrap_or(Self::displace_from_start(start, movement, &config.continuation, *length));

        let mut rect = Rect::from_point_and_size(start, (0, 0));
        Self::bounds_from_point(&mut rect, &end);
        debug!("sline_from {:?} {:?}", pair.as_str(), stroke);

        index.insert(ShapeName::Line, *id, rect);
        index.add_open(ShapeName::Line, attrs.clone());

        let shape = Shape::Sline(vec!(start, end), caption.clone(), endings.clone());
        let node = Open(attrs, rect, shape);
        Some((rect, node))
      }
    }
  }

  pub(crate) fn path_from<'a>(pair: Pair<'a, Rule>, config: &Config, index: &mut Index<'a>, cursor: &Point) -> Option<(Rect, Node<'a>)> {
    let (attrs, _) = Attributes::open_attributes(&pair, config, Rule::line_attributes);

    if let Attributes::Open {
      id,
      caption,
      ..
    } = &attrs {
      let mut movements = vec![];
      let pairs = Rules::get_rule(&pair, Rule::line_attributes).into_inner();

      pairs.for_each(|pair| {
        match pair.as_rule() {
          Rule::rel_movement | Rule::abs_movement => {
            let movement1 = Conversion::movement_from(pair, &config.unit);
            movements.push(movement1);
          }
          _ => {}
        }
      });

      let points = Self::points_from_movements(cursor, &movements, index);
      let used = Self::bounds_from_points(cursor, &points);
      index.insert(ShapeName::Path, *id, used);

      let shape = Shape::Path(*cursor, points, caption.clone());
      let node = Open(attrs, used, shape);
      return Some((used, node));
    };
    None
  }

  fn points_from_movements(cursor: &Point, movements: &[Movement], index: &mut Index) -> Vec<Point> {
    let mut point = *cursor;
    movements.iter().map(|movement| {
      match movement {
        Movement::Relative { displacement: movement } => {
          point = point.add(movement.offset());
          point
        }
        Movement::Absolute { object } => {
          point = index.point_from(object).unwrap();
          point
        }
      }
    }).collect::<Vec<_>>()
  }

  fn bounds_from_points(cursor: &Point, points: &[Point]) -> Rect {
    let mut used = Rect::from_point_and_size(*cursor, (0, 0));
    for point in points.iter() {
      Self::bounds_from_point(&mut used, point);
    }
    used
  }

  fn copy_same_attributes(index: &mut Index, attrs: &mut Attributes, shape: ShapeName) {
    match attrs {
      Attributes::Closed {
        same,
        width,
        height,
        ..
      } => {
        if !*same {
          return;
        }
        if let Some((_shape, Attributes::Closed {
          width: last_width,
          height: last_height,
          ..
        })) = index.last_open(shape) {
          if width.is_none() {
            *width = *last_width;
          }
          if height.is_none() {
            *height = *last_height;
          }
        }
      }
      Attributes::Open {
        same,
        endings,
        movement,
        caption,
        ..
      } => {
        if !*same {
          return;
        }
        if let Some((_shape, Attributes::Open {
          endings: last_endings,
          movement: last_movement,
          caption: last_caption,
          ..
        })) = index.last_open(shape) {
          *endings = last_endings.clone();
          if movement.is_none() {
            movement.clone_from(last_movement);
          }
          if let Some(caption) = &mut *caption {
            if let Some(last) = last_caption.as_ref() {
              caption.inner = last.inner.clone();
              caption.outer = last.outer.clone();
              caption.opaque = last.opaque;
            }
          }
        }
      }
    }
  }

  fn text_from<'a>(pair: &Pair<'a, Rule>, config: &Config, index: &mut Index, cursor: &Point) -> Option<(Rect, Node<'a>)> {
    let id = Conversion::identified_in(pair);
    let title = Conversion::string_in(pair, Rule::inner).unwrap();
    let attributes = Rules::find_rule(pair, Rule::text_attributes).unwrap();
    let location = Conversion::location_for(pair, &config.continuation.end, &config.unit);

    let fit = Rules::dig_rule(&attributes, Rule::fit);
    let paragraph = match fit {
      Some(_) => {
        let size = config.measure_string(title);
        Paragraph { text: title.into(), widths: vec![size.width], height: size.height, size }
      }
      None => {
        let width = Conversion::width_into(&attributes, &config.unit).unwrap_or(config.text.width);
        let (widths, height) = config.measure_strings(title, width - 2. * TEXT_PADDING);
        let size = Size::new(width, height);
        Paragraph { text: title.into(), widths, height, size }
      }
    };

    let mut used = Rect::from_xywh(cursor.x, cursor.y, paragraph.size.width, paragraph.size.height);
    used.bottom += BLOCK_PADDING;

    Self::adjust_topleft(&config.continuation, &mut used);
    index.position_rect(&location, &mut used);

    index.insert(ShapeName::Text, id, used);

    let common = CommonAttributes::new(id, used, Color::BLACK, 1.);
    let shape = Shape::Text(paragraph, location);
    let text = Primitive(common, shape);
    Some((used, text))
  }

  fn dot_from<'a>(pair: &Pair<'a, Rule>, config: &Config, index: &mut Index, cursor: &Point) -> Option<(Rect, Node<'a>)> {
    let (mut attrs, attributes) = Attributes::open_attributes(pair, config, Rule::dot_attributes);
    Self::copy_same_attributes(index, &mut attrs, ShapeName::Dot);

    match &attrs {
      Attributes::Closed { .. } => panic!("Wrong type"),
      Attributes::Open {
        id,
        caption,
        source,
        ..
      } => {
        let radius = Conversion::radius_into(&attributes, &config.unit).unwrap_or(config.dot.pixels());

        let point = match source {
          Some(object) => {
            index.point_index(Some(object), &[]).unwrap()
          }
          None => *cursor
        };

        let mut bounds = Rect::from_xywh(point.x, point.y, 0., 0.);
        if let Some(caption) = &caption {
          let rect = Renderer::dot_offset_of(&point, &radius, caption);
          Self::bounds_from_rect(&mut bounds, rect);
        }

        index.insert(ShapeName::Dot, *id, bounds);

        let shape = Shape::Dot(point, radius, caption.clone());
        let node = Open(attrs, bounds, shape);
        Some((bounds, node))
      }
    }
  }

  fn flow_from<'a>(pair: Pair<'a, Rule>, cursor: &Point, config: &mut Config) -> Option<(Rect, Node<'a>)> {
    let length = Conversion::length_from(pair, &config.unit);
    let mut used = Rect::from_xywh(cursor.x, cursor.y, 0., 0.);
    if config.continuation.end.horizontal() {
      used.right += length.pixels();
    } else {
      used.bottom += length.pixels();
    }
    let node = Node::Move(used);
    Some((used, node))
  }

  fn move_from<'a>(pair: &Pair<'a, Rule>, cursor: &Point, unit: &Unit) -> Option<(Rect, Node<'a>)> {
    Conversion::displacements_from(pair, unit).map(|movements| {
      let mut used = Rect::from_xywh(cursor.x, cursor.y, 0., 0.);
      Index::offset_rect(&mut used, &movements);
      (used, Node::Move(used))
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
        Rule::radius => {
          let length = Conversion::length_from(pair, unit);
          config.radius = length.pixels();
        }
        Rule::space => {
          let length = Conversion::length_from(pair, unit);
          config.space = length.pixels();
        }
        _ => {
          warn!("Ignored {:?}", pair);
        }
      }
    });
  }

  fn source_movement_target_from_pair(pair: &Pair<Rule>, unit: &Unit) -> (ObjectEdge, Option<Displacement>, ObjectEdge) {
    let source = Conversion::fraction_edge_for(pair, Rule::source)
      .unwrap_or(ObjectEdge::new("source", "e"));

    let movement = Conversion::displacement_for(pair, Rule::rel_movement, unit);

    let target = Conversion::fraction_edge_for(pair, Rule::target)
      .unwrap_or(ObjectEdge::new("source", "w"));

    (source, movement, target)
  }

  fn displace_from_start(start: Point, movement: &Option<Displacement>, flow: &Continuation, default: f32) -> Point {
    movement.as_ref()
      .map(|movement| start.add(movement.offset()))
      .unwrap_or_else(|| {
        let movement = Displacement::new(default, Unit::Px, flow.end.clone());
        start.add(movement.offset())
      })
  }

  fn rect_from_points(start: Point, displacement: &Option<Displacement>, end: Point) -> (Rect, Rect) {
    let rect = Rect { left: start.x, top: start.y, right: end.x, bottom: end.y };
    let mut used = rect;
    if let Some(movement) = &displacement {
      used.offset(movement.offset());
    }
    (rect, used)
  }

  fn paragraph_sized(title: Option<&str>, width: &Option<f32>, height: &Option<f32>, config: &Config, shape: &ShapeConfig) -> (Option<Paragraph>, Size) {
    let width = width.unwrap_or(shape.width);
    let height = height.unwrap_or(shape.height);

    let paragraph = title.map(|title| {
      let (widths, height) = config.measure_strings(title, width - 2. * TEXT_PADDING);
      let size = Size::new(width, height);
      Paragraph { text: title.into(), widths, height, size }
    });

    let height = paragraph.as_ref().map(|paragraph| height.max(paragraph.height)).unwrap_or(height);
    (paragraph, Size::new(width, height))
  }

  fn paragraph_sized_(title: Option<&str>, size: Size, config: &Config) -> (Option<Paragraph>, Size) {
    let width = size.width;
    let height = size.height;

    let paragraph = title.map(|title| {
      let (widths, height) = config.measure_strings(title, width - 2. * TEXT_PADDING);
      let size = Size::new(width, height);
      Paragraph { text: title.into(), widths, height, size }
    });

    let height = paragraph.as_ref().map(|paragraph| height.max(paragraph.height)).unwrap_or(height);
    (paragraph, Size::new(width, height))
  }

  fn position_rect_on_edge(start: &Edge, location: &Option<(Edge, Vec<Displacement>, ObjectEdge)>, used: &mut Rect) {
    let start = match location {
      Some((edge, _, _)) => edge,
      None => start
    };
    start.offset(used);
  }

  fn adjust_topleft(continuation: &Continuation, used: &mut Rect) {
    continuation.start.offset(used);
  }

  /// Adjust bounds so that rect fits in it
  fn bounds_from_rect(bounds: &mut Rect, rect: Rect) {
    bounds.top = bounds.top.min(rect.top);
    bounds.left = bounds.left.min(rect.left);
    bounds.right = bounds.right.max(rect.right);
    bounds.bottom = bounds.bottom.max(rect.bottom);
  }

  fn bounds_from_point(bounds: &mut Rect, point: &Point) {
    bounds.top = bounds.top.min(point.y);
    bounds.bottom = bounds.bottom.max(point.y);
    bounds.left = bounds.left.min(point.x);
    bounds.right = bounds.right.max(point.x);
  }
  #[allow(dead_code)]
  fn find_node(&self, id: &str) -> Option<&Node> {
    Self::find_nodes(&self.nodes, id)
  }

  fn find_nodes<'a>(nodes: &'a [Node], node_id: &str) -> Option<&'a Node<'a>> {
    nodes.iter().find(|node| {
      match node {
        Primitive(common, ..) => {
          common.id == Some(node_id)
        }
        Container(Attributes::Closed { id, .. }, _, nodes) => {
          if let Some(id) = id {
            if id == &node_id {
              return true;
            }
          }
          Self::find_nodes(nodes, node_id).is_some()
        }
        _ => false,
      }
    })
  }

  pub fn node_mut(&mut self, id: &str, movements: Vec<Displacement>) {
    if let Primitive(ref mut rect, _) = Diagram::find_nodes_mut(&mut self.nodes, id).unwrap() {
      for movement in movements.iter() {
        rect.used.offset(movement.offset());
      }
    }
  }

  fn find_nodes_mut<'a: 'i>(nodes: &'i mut [Node<'a>], node_id: &str) -> Option<&'i mut Node<'a>> {
    for node in nodes.iter_mut() {
      match node {
        Primitive(common, _) => {
          if common.id == Some(node_id) {
            return Some(node);
          }
        }
        Container(_, _, nodes) => {
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
    let mut canvas = Canvas::new(self.size, None);
    self.write_to_file(filepath, &mut canvas);
  }

  pub fn shrink_to_file<P: AsRef<Path>>(&mut self, path: P, background: Option<Color>) {
    let size = self.bounds.with_outset(self.inset);
    let size = ISize::new(size.width() as i32, size.height() as i32);
    let mut canvas = Canvas::new(size, background);
    self.write_to_file(path, &mut canvas);
  }

  fn write_to_file<P: AsRef<Path>>(&mut self, filepath: P, canvas: &mut Canvas) {
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