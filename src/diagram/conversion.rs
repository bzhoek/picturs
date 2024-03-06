#![allow(dead_code)]

use pest::iterators::{Pair, Pairs};
use pest::Parser;
use skia_safe::Color;

use crate::diagram::index::ShapeName;
use crate::diagram::parser::{DiagramParser, Rule};
use crate::diagram::rules::Rules;
use crate::diagram::types::{Caption, Config, Displacement, Edge, EdgeDirection, Ending, Endings, Flow, Length, Movement, ObjectEdge, Unit};

#[cfg(test)]
mod tests;

const INCH: f32 = 118.;

pub const WIDTH: f32 = INCH * 0.75;
pub const HEIGHT: f32 = INCH * 0.5;

pub(crate) struct Conversion;

impl Conversion {
  pub(crate) fn pair_for(rule: Rule, string: &str) -> Pair<Rule> {
    Self::pairs_for(rule, string).next().unwrap()
  }

  pub(crate) fn pairs_for(rule: Rule, string: &str) -> Pairs<Rule> {
    DiagramParser::parse(rule, string).unwrap()
  }

  fn next_to_f32(iter: &mut Pairs<Rule>) -> Option<f32> {
    iter.next().and_then(|p| p.as_str().parse::<f32>().ok())
  }

  fn next_to_string<'a>(iter: &mut Pairs<'a, Rule>) -> Option<&'a str> {
    iter.next().map(|p| p.as_str())
  }

  fn inner_string<'a>(iter: &mut Pairs<'a, Rule>) -> Option<&'a str> {
    let mut pairs = iter.next().unwrap().into_inner();
    pairs.next();
    pairs.next().map(|p| p.as_str())
  }

  pub(crate) fn colors_from(pair: &Pair<Rule>) -> (Color, Color, Color) {
    let stroke = Conversion::stroke_color_in(pair).unwrap_or(Color::BLUE);
    let fill = Conversion::fill_color_in(pair).unwrap_or(Color::TRANSPARENT);
    let text_color = Conversion::text_color_in(pair).unwrap_or(Color::BLACK);
    (stroke, fill, text_color)
  }

  pub(crate) fn stroke_color_in(pair: &Pair<Rule>) -> Option<Color> {
    Rules::dig_rule(pair, Rule::stroke).and_then(Self::color_from)
  }

  pub(crate) fn fill_color_in(pair: &Pair<Rule>) -> Option<Color> {
    Rules::dig_rule(pair, Rule::fill).and_then(Self::color_from)
  }

  pub(crate) fn text_color_in(pair: &Pair<Rule>) -> Option<Color> {
    Rules::dig_rule(pair, Rule::text_color).and_then(Self::color_from)
  }

  fn color_from(pair: Pair<Rule>) -> Option<Color> {
    let mut pairs = pair.into_inner();
    pairs.find_map(|pair| {
      match pair.as_rule() {
        Rule::id => {
          let str = pair.as_str();
          match str {
            "white" => Color::WHITE,
            "lgray" => Color::LIGHT_GRAY,
            "dgray" | "dgrey" => Color::DARK_GRAY,
            "black" => Color::BLACK,
            "yellow" => Color::YELLOW,
            "red" => Color::RED,
            "green" => Color::GREEN,
            "blue" => Color::BLUE,
            "gray" | "grey" => Color::GRAY,
            "cyan" => Color::CYAN,
            "magenta" => Color::MAGENTA,
            _ => panic!("unknown color {:?}", pair)
          }
        }
        Rule::rgb => {
          let str = pair.as_str();
          let hex = &str[1..];
          let hex = u32::from_str_radix(hex, 16).unwrap();
          let r = (hex >> 16) as u8;
          let g = (hex >> 8) as u8;
          let b = hex as u8;
          Color::from_argb(0xFF, r, g, b)
        }
        _ => panic!("unexpected rule {:?}", pair.as_rule())
      }.into()
    })
  }

  pub(crate) fn string_in<'a>(pair: &Pair<'a, Rule>, rule: Rule) -> Option<&'a str> {
    Rules::dig_rule(pair, rule)
      .map(|p| p.as_str())
  }

  pub(crate) fn string_for<'a>(pair: &Pair<'a, Rule>, rule: Rule) -> Option<&'a str> {
    Rules::find_rule(pair, rule)
      .map(|p| p.as_str())
  }

  pub(crate) fn identified_in<'a>(pair: &Pair<'a, Rule>) -> Option<&'a str> {
    Rules::dig_rule(pair, Rule::identified)
      .map(|p| p.into_inner().next().unwrap().as_str())
  }

  #[allow(clippy::unwrap_or_default)]
  pub(crate) fn caption<'a>(pair: &Pair<'a, Rule>, config: &Config) -> Option<Caption<'a>> {
    Rules::find_rule(pair, Rule::caption)
      .map(|caption| { Self::caption_from(caption, config) })
  }

  pub(crate) fn caption_from<'a>(pair: Pair<'a, Rule>, config: &Config) -> Caption<'a> {
    let mut text: Option<&str> = None;
    let mut alignment: Option<(Edge, Edge)> = None;
    let mut opaque = false;

    let pairs = pair.into_inner();
    pairs.for_each(|pair| match pair.as_rule() {
      Rule::string => {
        let str = pair.as_str();
        text = Some(&str[1..str.len() - 1]);
      }
      Rule::alignment => {
        let string = pair.as_str();
        alignment = match string {
          "ljust" => (Edge::right(), Edge::right()),
          _ => (string.into(), Edge::center())
        }.into();
      }
      Rule::opaque => { opaque = true }
      _ => panic!("unexpected rule {:?}", pair.as_rule())
    });

    let (inner, outer) = alignment.unwrap_or((Edge::center(), Edge::center()));
    let text = text.unwrap();
    let size = config.measure_string(text);
    Caption { text, inner: inner.mirror(), outer, size, opaque }
  }

  pub(crate) fn endings(pair: &Pair<Rule>) -> Endings {
    Rules::find_rule(pair, Rule::endings)
      .map(Self::endings_from)
      .unwrap_or_default()
  }

  pub(crate) fn movement_from(pair: Pair<Rule>, unit: &Unit) -> Movement {
    match pair.as_rule() {
      Rule::rel_movement => {
        Self::rel_movement_from(pair, unit)
      }
      Rule::abs_movement => {
        Self::abs_movement_from(pair)
      }
      _ => panic!("Unexpected {:?}", pair)
    }
  }

  pub(crate) fn rel_movement_from(pair: Pair<Rule>, unit: &Unit) -> Movement {
    let displacement = Self::displacement_from(pair, unit);
    Movement::Relative { displacement }
  }

  pub(crate) fn abs_movement_from(pair: Pair<Rule>) -> Movement {
    let object = pair.into_inner()
      .find(|pair| pair.as_rule() == Rule::object_edge)
      .map(Self::object_edge_from)
      .unwrap();
    Movement::Absolute { object }
  }

  pub(crate) fn length_in(pair: &Pair<Rule>, rule: Rule, unit: &Unit) -> Option<Length> {
    Rules::dig_rule(pair, rule)
      .map(|pair| Self::length_from(pair, unit))
  }

  pub(crate) fn length_from(pair: Pair<Rule>, unit: &Unit) -> Length {
    let mut width = pair.into_inner();
    let length = Self::next_to_f32(&mut width).unwrap();
    let unit = Self::next_to_string(&mut width).map(|str| str.into()).unwrap_or(*unit);
    Length::new(length, unit)
  }

  pub(crate) fn radius_into(attributes: &Pair<Rule>, unit: &Unit) -> Option<f32> {
    Conversion::length_in(attributes, Rule::radius, unit)
      .map(|length| length.pixels())
  }

  pub(crate) fn width_into(attributes: &Pair<Rule>, unit: &Unit) -> Option<f32> {
    Conversion::length_in(attributes, Rule::width, unit)
      .map(|length| length.pixels())
  }

  pub(crate) fn height_into(attributes: &Pair<Rule>, unit: &Unit) -> Option<f32> {
    Conversion::length_in(attributes, Rule::height, unit)
      .map(|length| length.pixels())
  }

  pub(crate) fn length_into(attributes: &Pair<Rule>, unit: &Unit) -> Option<f32> {
    Conversion::length_in(attributes, Rule::length, unit)
      .map(|length| length.pixels())
  }

  pub(crate) fn padding_into(attributes: &Pair<Rule>, unit: &Unit) -> Option<f32> {
    Conversion::length_in(attributes, Rule::padding, unit)
      .map(|length| length.pixels())
  }

  fn object_edge_from(pair: Pair<Rule>) -> ObjectEdge {
    Self::object_edge_from_default(pair, &Edge::default())
  }

  fn object_edge_from_default(pair: Pair<Rule>, default: &Edge) -> ObjectEdge {
    let mut inner = pair.into_inner();
    let id = Self::next_to_string(&mut inner).unwrap();
    let edge = Self::next_to_string(&mut inner);
    let edge = edge.map(Edge::from).unwrap_or(default.clone());
    ObjectEdge::new(id, edge)
  }

  fn object_edge_from_degrees(pair: Pair<Rule>) -> ObjectEdge {
    let mut inner = pair.into_inner();
    let id = Self::next_to_string(&mut inner).unwrap();

    let edge = inner.next();
    if edge.is_none() {
      return ObjectEdge::new(id, Edge::center());
    }

    let mut inner = edge.unwrap().into_inner();
    let next = inner.next().unwrap();
    let str = next.as_str();

    let degrees = match next.as_rule() {
      Rule::compass => {
        match str {
          "n" => 360,
          "ne" => 45,
          "e" => 90,
          "se" => 135,
          "s" => 180,
          "sw" => 225,
          "w" => 270,
          "nw" => 315,
          _ => panic!("unexpected compass direction")
        }
      }
      Rule::degrees => str.parse().unwrap(),
      Rule::hours => {
        let hour = str[0..str.len() - 1].parse::<i32>().unwrap();
        hour * 30
      }
      _ => panic!("unexpected rule")
    };
    ObjectEdge::new(id, Edge::from(degrees as f32))
  }

  pub(crate) fn flow_in(pair: &Pair<Rule>) -> Option<Flow> {
    Rules::dig_rule(pair, Rule::flow)
      .map(|pair| Flow::new(pair.as_str()))
  }

  pub(crate) fn fraction_edge_for(pair: &Pair<Rule>, rule: Rule) -> Option<ObjectEdge> {
    Rules::find_rule(pair, rule)
      .map(Self::fraction_edge_from)
  }

  pub(crate) fn fraction_edge_from(pair: Pair<Rule>) -> ObjectEdge {
    let mut fraction: Option<f32> = None;
    let mut object: Option<ObjectEdge> = None;

    pair.into_inner().for_each(|pair| match pair.as_rule() {
      Rule::fraction => {
        let mut inner = pair.into_inner();
        let x = Self::next_to_f32(&mut inner).unwrap();
        let y = Self::next_to_f32(&mut inner).unwrap();
        fraction = Some(x / y);
      }
      Rule::object_edge => {
        object = Some(Self::object_edge_from_degrees(pair));
      }
      _ => panic!("Unexpected rule {:?}", pair.as_rule())
    });

    let mut object = object.unwrap();
    if let Some(fraction) = fraction {
      match object.edge.direction {
        EdgeDirection::Horizontal => { object.edge.y = fraction - 0.5 }
        EdgeDirection::Vertical => { object.edge.x = fraction - 0.5 }
      }
    }
    object
  }

  pub(crate) fn displacement_for(pair: &Pair<Rule>, rule: Rule, unit: &Unit) -> Option<Displacement> {
    Rules::find_rule(pair, rule)
      .map(|pair| Self::displacement_from(pair, unit))
  }

  pub(crate) fn displacement_from(pair: Pair<Rule>, unit: &Unit) -> Displacement {
    let length = Rules::find_rule(&pair, Rule::offset)
      .map(|pair| Self::length_from(pair, unit)).unwrap();
    let direction = Self::string_for(&pair, Rule::direction).unwrap();
    Displacement { length, edge: direction.into() }
  }

  pub(crate) fn displacements_from(pair: &Pair<Rule>, unit: &Unit) -> Option<Vec<Displacement>> {
    Rules::find_rule(pair, Rule::movements)
      .map(|pair| {
        pair.into_inner()
          .map(|inner| Self::displacement_from(inner, unit))
          .collect::<Vec<_>>()
      })
  }

  pub(crate) fn location_for(pair: &Pair<Rule>, _end: &Edge, unit: &Unit) -> Option<(Edge, Vec<Displacement>, ObjectEdge)> {
    Rules::find_rule(pair, Rule::location)
      .map(|p| {
        let mut object: Option<ObjectEdge> = None;
        let mut directions: Vec<Displacement> = vec![];
        let mut edge: Option<Edge> = None;

        for pair in p.into_inner() {
          match pair.as_rule() {
            Rule::edge => { edge = Some(Edge::from(pair.as_str())); }
            Rule::rel_movement => {
              let movement = Self::displacement_from(pair, unit);
              directions.push(movement);
            }
            Rule::last_object => { object = Some(Self::fraction_edge_from(pair)); }
            Rule::from_object => { object = Some(Self::fraction_edge_from(pair)); }
            _ => {}
          }
        };

        if let Some(movement) = directions.first() {
          if let Some(object) = object.as_mut() {
            if ShapeName::some(object.id.as_str()).is_some() {
              object.edge = movement.edge.clone()
            }
          }
          if object.is_none() {
            object = Some(ObjectEdge::new("#last", movement.edge.clone()))
          }
          if edge.is_none() {
            edge = Some(movement.edge.flip())
          }
        }

        (edge.unwrap(), directions, object.unwrap())
      })
  }

  pub(crate) fn endings_from(pair: Pair<Rule>) -> Endings {
    let mut start = Ending::None;
    let mut end = Ending::None;

    pair.into_inner().for_each(|pair| match pair.as_rule() {
      Rule::left_end => start = Ending::from(pair.as_str()),
      Rule::right_end => end = Ending::from(pair.as_str()),
      _ => panic!("unexpected rule {:?}", pair.as_rule())
    });

    Endings { start, end }
  }
}
