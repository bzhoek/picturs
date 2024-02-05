#![allow(dead_code)]

use pest::iterators::{Pair, Pairs};
use skia_safe::Color;
use crate::diagram::index::ShapeName;

use crate::diagram::parser::Rule;
use crate::diagram::rules::Rules;
use crate::diagram::types::{Displacement, Edge, Flow, Length, ObjectEdge};

pub const WIDTH: f32 = 120.;
pub const HEIGHT: f32 = 60.;

pub struct Conversion;

impl Conversion {
  fn next_to_f32(iter: &mut Pairs<Rule>) -> Option<f32> {
    iter.next().and_then(|p| p.as_str().parse::<f32>().ok())
  }

  fn next_to_string<'a>(iter: &mut Pairs<'a, Rule>) -> Option<&'a str> {
    iter.next().map(|p| p.as_str())
  }

  pub fn colors_from(pair: &Pair<Rule>) -> (Color, Color, Color) {
    let stroke = Conversion::stroke_color(pair).unwrap_or(Color::BLUE);
    let fill = Conversion::fill_color(pair).unwrap_or(Color::TRANSPARENT);
    let text_color = Conversion::text_color(pair).unwrap_or(Color::BLACK);
    (stroke, fill, text_color)
  }

  pub fn stroke_color(pair: &Pair<Rule>) -> Option<Color> {
    Rules::dig_rule(pair, Rule::color).and_then(Self::color)
  }

  pub fn fill_color(pair: &Pair<Rule>) -> Option<Color> {
    Rules::dig_rule(pair, Rule::fill).and_then(Self::color)
  }

  pub fn text_color(pair: &Pair<Rule>) -> Option<Color> {
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

  pub fn string_dig<'a>(pair: &Pair<'a, Rule>, rule: Rule) -> Option<&'a str> {
    Rules::dig_rule(pair, rule)
      .map(|p| p.as_str())
  }

  pub fn string_find<'a>(pair: &Pair<'a, Rule>, rule: Rule) -> Option<&'a str> {
    Rules::find_rule(pair, rule)
      .map(|p| p.as_str())
  }

  pub fn identified<'a>(pair: &Pair<'a, Rule>) -> Option<&'a str> {
    Rules::dig_rule(pair, Rule::identified)
      .map(|p| p.into_inner().next().unwrap().as_str())
  }

  fn displacement_from(pair: Pair<Rule>) -> Displacement {
    let length = Rules::find_rule(&pair, Rule::offset).map(Self::length_from).unwrap();
    let direction = Self::string_find(&pair, Rule::direction).unwrap();
    let direction = Edge::new(direction);
    Displacement { length, edge: direction }
  }

  pub fn length_dig(pair: &Pair<Rule>, rule: Rule) -> Option<Length> {
    Rules::dig_rule(pair, rule)
      .map(Self::length_from)
  }

  pub fn length_from(pair: Pair<Rule>) -> Length {
    let mut width = pair.into_inner();
    let length = Self::next_to_f32(&mut width).unwrap();
    let unit = Self::next_to_string(&mut width).unwrap_or("px");
    Length::new(length, unit.into())
  }

  pub fn radius(attributes: &Pair<Rule>) -> Option<Length> {
    Conversion::length_dig(attributes, Rule::radius)
  }

  pub fn width(attributes: &Pair<Rule>) -> Option<f32> {
    Conversion::length_dig(attributes, Rule::width).map(|length| length.pixels())
  }

  pub fn height(attributes: &Pair<Rule>) -> Option<f32> {
    Conversion::length_dig(attributes, Rule::height).map(|length| length.pixels())
  }

  pub fn padding(attributes: &Pair<Rule>) -> Option<f32> {
    Conversion::length_dig(attributes, Rule::padding).map(|length| length.pixels())
  }

  pub fn object_edge_from_pair(pair: &Pair<Rule>) -> Option<ObjectEdge> {
    Rules::find_rule(pair, Rule::object_edge).map(Self::pair_to_object_edge)
  }

  fn pair_to_object_edge(pair: Pair<Rule>) -> ObjectEdge {
    let id = Self::string_dig(&pair, Rule::id).unwrap();
    let edge = Self::string_dig(&pair, Rule::edge).unwrap();
    ObjectEdge::new(id, edge)
  }

  fn object_edge_from(pair: Pair<Rule>, default: &Edge) -> ObjectEdge {
    let id = Self::string_dig(&pair, Rule::id).unwrap();
    let edge = Self::string_dig(&pair, Rule::edge)
      .map(Edge::new)
      .unwrap_or(default.clone());
    ObjectEdge::edge(id, edge)
  }

  pub fn flow(pair: &Pair<Rule>) -> Option<Flow> {
    Rules::dig_rule(pair, Rule::flow)
      .map(|pair| Flow::new(pair.as_str()))
  }

  pub fn location_to_edge(pair: &Pair<Rule>, rule: Rule) -> Option<ObjectEdge> {
    Rules::find_rule(pair, rule)
      .and_then(|pair| Rules::find_rule(&pair, Rule::object_edge))
      .map(Self::pair_to_object_edge)
  }

  pub fn rule_to_distance(pair: &Pair<Rule>, rule: Rule) -> Option<Displacement> {
    Rules::find_rule(pair, rule).map(Self::displacement_from)
  }

  pub fn displacements_from_pair(pair: &Pair<Rule>) -> Option<Vec<Displacement>> {
    Rules::find_rule(pair, Rule::displacements)
      .map(|pair| {
        pair.into_inner()
          .map(|inner| Self::displacement_from(inner))
          .collect::<Vec<_>>()
      })
  }

  pub fn location_from(pair: &Pair<Rule>, end: &Edge) -> Option<(Edge, Vec<Displacement>, ObjectEdge)> {
    Rules::dig_rule(pair, Rule::location)
      .map(|p| {
        let mut object: Option<ObjectEdge> = None;
        let mut directions: Vec<Displacement> = vec![];
        let mut edge: Option<Edge> = None;

        for rule in p.into_inner() {
          match rule.as_rule() {
            Rule::edge => { edge = Some(Edge::new(rule.as_str())); }
            Rule::displacement => {
              let displacement = Self::displacement_from(rule);
              directions.push(displacement);
            }
            Rule::object_edge => { object = Some(Self::object_edge_from(rule, end)); }
            _ => {}
          }
        };
        if let Some(displacement) = directions.first() {
          if let Some(object) = object.as_mut() {
            if ShapeName::some(object.id.as_str()).is_some() {
              object.edge = displacement.edge.clone()
            }
          }
          if object.is_none() {
            object = Some(ObjectEdge::edge("#last", displacement.edge.clone()))
          }
          if edge.is_none() {
            edge = Some(displacement.edge.flip())
          }
        }

        (edge.unwrap(), directions, object.unwrap())
      })
  }
}
