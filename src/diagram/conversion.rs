#![allow(dead_code)]

use pest::iterators::{Pair, Pairs};
use pest::Parser;
use skia_safe::Color;

use crate::diagram::index::ShapeName;
use crate::diagram::parser::{DiagramParser, Rule};
use crate::diagram::rules::Rules;
use crate::diagram::types::{Arrows, Caption, Config, Edge, Flow, Length, Displacement, Movement, ObjectEdge, Unit};

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
    let stroke = Conversion::stroke_color(pair).unwrap_or(Color::BLUE);
    let fill = Conversion::fill_color(pair).unwrap_or(Color::TRANSPARENT);
    let text_color = Conversion::text_color(pair).unwrap_or(Color::BLACK);
    (stroke, fill, text_color)
  }

  pub(crate) fn stroke_color(pair: &Pair<Rule>) -> Option<Color> {
    Rules::dig_rule(pair, Rule::color).and_then(Self::color)
  }

  pub(crate) fn fill_color(pair: &Pair<Rule>) -> Option<Color> {
    Rules::dig_rule(pair, Rule::fill).and_then(Self::color)
  }

  pub(crate) fn text_color(pair: &Pair<Rule>) -> Option<Color> {
    Rules::dig_rule(pair, Rule::text_color).and_then(Self::color)
  }

  fn color(pair: Pair<Rule>) -> Option<Color> {
    Self::string_find(&pair, Rule::id)
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

  pub(crate) fn string_dig<'a>(pair: &Pair<'a, Rule>, rule: Rule) -> Option<&'a str> {
    Rules::dig_rule(pair, rule)
      .map(|p| p.as_str())
  }

  pub(crate) fn string_find<'a>(pair: &Pair<'a, Rule>, rule: Rule) -> Option<&'a str> {
    Rules::find_rule(pair, rule)
      .map(|p| p.as_str())
  }

  pub(crate) fn identified<'a>(pair: &Pair<'a, Rule>) -> Option<&'a str> {
    Rules::dig_rule(pair, Rule::identified)
      .map(|p| p.into_inner().next().unwrap().as_str())
  }

  #[allow(clippy::unwrap_or_default)]
  pub(crate) fn caption<'a>(pair: &Pair<'a, Rule>, config: &Config) -> Option<Caption<'a>> {
    Rules::find_rule(pair, Rule::caption).map(|caption| {
      Self::caption_from(caption, config)
    })
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

  pub(crate) fn arrows(pair: &Pair<Rule>) -> Arrows {
    Rules::find_rule(pair, Rule::arrows)
      .map(|pair| pair.as_str().into())
      .unwrap_or_default()
  }

  pub(crate) fn movement2_from(pair: Pair<Rule>, unit: &Unit) -> Movement {
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

  pub(crate) fn displacement_from(pair: Pair<Rule>, unit: &Unit) -> Displacement {
    let length = Rules::find_rule(&pair, Rule::offset)
      .map(|pair| Self::length_from(pair, unit)).unwrap();
    let direction = Self::string_find(&pair, Rule::direction).unwrap();
    Displacement { length, edge: direction.into() }
  }

  pub(crate) fn abs_movement_from(pair: Pair<Rule>) -> Movement {
    let object = pair.into_inner()
      .find(|pair| pair.as_rule() == Rule::object_edge)
      .map(|pair| Self::pair_to_object_edge(pair))
      .unwrap();
    Movement::Absolute { object }
  }

  pub(crate) fn length_dig(pair: &Pair<Rule>, rule: Rule, unit: &Unit) -> Option<Length> {
    Rules::dig_rule(pair, rule)
      .map(|pair| Self::length_from(pair, unit))
  }

  pub(crate) fn length_from(pair: Pair<Rule>, unit: &Unit) -> Length {
    let mut width = pair.into_inner();
    let length = Self::next_to_f32(&mut width).unwrap();
    let unit = Self::next_to_string(&mut width).map(|str| str.into()).unwrap_or(*unit);
    Length::new(length, unit)
  }

  pub(crate) fn radius(attributes: &Pair<Rule>, unit: &Unit) -> Option<Length> {
    Conversion::length_dig(attributes, Rule::radius, unit)
  }

  pub(crate) fn width(attributes: &Pair<Rule>, unit: &Unit) -> Option<f32> {
    Conversion::length_dig(attributes, Rule::width, unit).map(|length| length.pixels())
  }

  pub(crate) fn height(attributes: &Pair<Rule>, unit: &Unit) -> Option<f32> {
    Conversion::length_dig(attributes, Rule::height, unit).map(|length| length.pixels())
  }

  pub(crate) fn length(attributes: &Pair<Rule>, unit: &Unit) -> Option<f32> {
    Conversion::length_dig(attributes, Rule::length, unit).map(|length| length.pixels())
  }

  pub(crate) fn padding(attributes: &Pair<Rule>, unit: &Unit) -> Option<f32> {
    Conversion::length_dig(attributes, Rule::padding, unit).map(|length| length.pixels())
  }

  pub(crate) fn object_edge_from_pair(pair: &Pair<Rule>) -> Option<ObjectEdge> {
    Rules::dig_rule(pair, Rule::object_edge).map(Self::pair_to_object_edge)
  }

  fn pair_to_object_edge(pair: Pair<Rule>) -> ObjectEdge {
    let id = Self::string_dig(&pair, Rule::id).unwrap();
    let edge = Self::string_dig(&pair, Rule::edge).unwrap();
    ObjectEdge::new(id, edge)
  }

  fn object_edge_from(pair: Pair<Rule>, default: &Edge) -> ObjectEdge {
    let id = Self::string_dig(&pair, Rule::id).unwrap();
    let edge = Self::string_dig(&pair, Rule::edge)
      .map(Edge::from)
      .unwrap_or(default.clone());
    ObjectEdge::new(id, edge)
  }

  pub(crate) fn object_edge_in_degrees_from(pair: Pair<Rule>) -> (&str, Option<i32>) {
    let mut inner = pair.into_inner();
    let id = inner.next().unwrap().as_str();

    let edge = inner.next();
    if edge.is_none() {
      return (id, None);
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
    (id, Some(degrees))
  }

  pub(crate) fn flow(pair: &Pair<Rule>) -> Option<Flow> {
    Rules::dig_rule(pair, Rule::flow)
      .map(|pair| Flow::new(pair.as_str()))
  }

  pub(crate) fn location_to_edge(pair: &Pair<Rule>, rule: Rule) -> Option<ObjectEdge> {
    Rules::find_rule(pair, rule).map(|pair| {
      let mut fraction: Option<f32> = None;
      let mut object: Option<ObjectEdge> = None;

      pair.into_inner().for_each(|pair| match pair.as_rule() {
        Rule::fraction => {
          let mut inner = pair.into_inner();
          let x = Self::next_to_f32(&mut inner).unwrap();
          let y = Self::next_to_f32(&mut inner).unwrap();
          fraction = Some(x / y - 0.5 / y);
        }
        Rule::object_edge => {
          let mut inner = pair.into_inner();
          let id = Self::next_to_string(&mut inner).unwrap();
          let edge = Self::next_to_string(&mut inner).unwrap_or("c");
          object = Some(ObjectEdge::new(id, edge));
        }
        _ => panic!("Unexpected rule {:?}", pair.as_rule())
      });
      let mut object = object.unwrap();
      if let Some(fraction) = fraction {
        object.edge.y = fraction - 0.5;
      }
      object
    })
  }

  pub(crate) fn rule_to_displacement(pair: &Pair<Rule>, rule: Rule, unit: &Unit) -> Option<Displacement> {
    Rules::find_rule(pair, rule).map(|pair| Self::displacement_from(pair, unit))
  }

  pub(crate) fn displacements_from_pair(pair: &Pair<Rule>, unit: &Unit) -> Option<Vec<Displacement>> {
    Rules::find_rule(pair, Rule::movements)
      .map(|pair| {
        pair.into_inner()
          .map(|inner| Self::displacement_from(inner, unit))
          .collect::<Vec<_>>()
      })
  }

  pub(crate) fn location_from(pair: &Pair<Rule>, end: &Edge, unit: &Unit) -> Option<(Edge, Vec<Displacement>, ObjectEdge)> {
    Rules::find_rule(pair, Rule::location)
      .map(|p| {
        let mut object: Option<ObjectEdge> = None;
        let mut directions: Vec<Displacement> = vec![];
        let mut edge: Option<Edge> = None;

        for rule in p.into_inner() {
          match rule.as_rule() {
            Rule::edge => { edge = Some(Edge::from(rule.as_str())); }
            Rule::rel_movement => {
              let movement = Self::displacement_from(rule, unit);
              directions.push(movement);
            }
            Rule::object_edge => { object = Some(Self::object_edge_from(rule, end)); }
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
}
