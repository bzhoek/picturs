#![allow(dead_code)]

use pest::iterators::Pair;
use skia_safe::Color;

use crate::diagram::parser::{Radius, Rule};
use crate::diagram::rules::Rules;
use crate::diagram::types::{Displacement, Edge, Flow, Length, ObjectEdge};

const WIDTH: f32 = 120.;
const HEIGHT: f32 = 60.;

pub struct Conversion;

impl Conversion {
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
    Rules::find_rule(&pair, Rule::id)
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
  pub fn rule_to_color(pair: &Pair<Rule>, rule: Rule) -> Option<Color> {
    Rules::dig_rule(pair, rule)
      .and_then(|pair| Rules::find_rule(&pair, Rule::id))
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

  pub fn rule_to_string<'a>(pair: &Pair<'a, Rule>, rule: Rule) -> Option<&'a str> {
    Rules::dig_rule(pair, rule)
      .map(|p| p.as_str())
  }

  pub fn rule_to_length(pair: &Pair<Rule>, rule: Rule) -> Option<Length> {
    Rules::dig_rule(pair, rule).map(Self::pair_to_length)
  }

  fn pair_to_length(pair: Pair<Rule>) -> Length {
    let length = Rules::find_rule(&pair, Rule::length)
      .and_then(|p| p.as_str().parse::<usize>().ok())
      .unwrap();
    let unit = Self::rule_to_string(&pair, Rule::unit)
      .unwrap_or("px");
    Length::new(length as f32, unit.into())
  }

  pub fn parse_dimension(attributes: &Pair<Rule>) -> (f32, Option<f32>, Radius) {
    let width = Conversion::rule_to_length(attributes, Rule::width)
      .map(|length| length.pixels())
      .unwrap_or(WIDTH);
    let height = Conversion::rule_to_length(attributes, Rule::height).map(|length| length.pixels());
    let radius = Conversion::rule_to_radius(attributes);

    (width, height, radius)
  }

  pub fn width(attributes: &Pair<Rule>) -> Option<f32> {
    Conversion::rule_to_length(attributes, Rule::width).map(|length| length.pixels())
  }

  pub fn height(attributes: &Pair<Rule>) -> Option<f32> {
    Conversion::rule_to_length(attributes, Rule::width).map(|length| length.pixels())
  }

  pub fn padding(attributes: &Pair<Rule>) -> Option<f32> {
    Conversion::rule_to_length(attributes, Rule::padding).map(|length| length.pixels())
  }

  pub fn object_edge_from_pair(pair: &Pair<Rule>) -> Option<ObjectEdge> {
    Rules::find_rule(pair, Rule::object_edge).map(Self::pair_to_object_edge)
  }

  fn pair_to_object_edge(pair: Pair<Rule>) -> ObjectEdge {
    let id = Self::rule_to_string(&pair, Rule::id).unwrap();
    let edge = Self::rule_to_string(&pair, Rule::edge).unwrap();
    ObjectEdge::new(id, edge)
  }

  pub fn flow(pair: &Pair<Rule>) -> Option<Flow> {
    Rules::dig_rule(pair, Rule::flow)
      .map(|pair| Flow::new(pair.as_str()))
  }

  pub fn rule_to_radius(pair: &Pair<Rule>) -> Radius {
    Rules::dig_rule(pair, Rule::radius)
      .map(Self::pair_to_radius)
      .unwrap_or_default()
  }

  fn pair_to_radius(pair: Pair<Rule>) -> Radius {
    let length = Rules::find_rule(&pair, Rule::length)
      .and_then(|p| p.as_str().parse::<usize>().ok())
      .unwrap();
    let unit = Self::rule_to_string(&pair, Rule::unit)
      .unwrap();
    Radius::new(length as f32, unit.into())
  }

  fn pair_to_displacement(pair: Pair<Rule>) -> Displacement {
    let length = Rules::find_rule(&pair, Rule::length)
      .and_then(|p| p.as_str().parse::<usize>().ok())
      .unwrap();
    let unit = Self::rule_to_string(&pair, Rule::unit)
      .unwrap();
    let direction = Self::rule_to_string(&pair, Rule::direction).unwrap();
    let edge = Edge::new(direction);
    Displacement::new(length as f32, unit.into(), edge.vector())
  }

  pub fn location_to_edge(pair: &Pair<Rule>, rule: Rule) -> Option<ObjectEdge> {
    Rules::find_rule(pair, rule)
      .and_then(|pair| Rules::find_rule(&pair, Rule::object_edge))
      .map(Self::pair_to_object_edge)
  }

  pub fn rule_to_distance(pair: &Pair<Rule>, rule: Rule) -> Option<Displacement> {
    Rules::find_rule(pair, rule).map(Self::pair_to_displacement)
  }

  pub fn displacements_from_pair(pair: &Pair<Rule>) -> Option<Vec<Displacement>> {
    Rules::find_rule(pair, Rule::displacements)
      .map(|pair| {
        pair.into_inner()
          .map(|inner| Self::pair_to_displacement(inner))
          .collect::<Vec<_>>()
      })
  }

  pub fn location_from_pair(pair: &Pair<Rule>) -> Option<(Edge, Vec<Displacement>, ObjectEdge)> {
    Rules::dig_rule(pair, Rule::location)
      .map(|p| {
        let mut edge: Option<Edge> = None;
        let mut directions: Vec<Displacement> = vec![];
        let mut object: Option<ObjectEdge> = None;

        for rule in p.into_inner() {
          match rule.as_rule() {
            Rule::edge => { edge = Some(Edge::new(rule.as_str())); }
            Rule::displacement => {
              let distance = Self::pair_to_displacement(rule);
              directions.push(distance);
            }
            Rule::object_edge => { object = Some(Self::pair_to_object_edge(rule)); }
            _ => {}
          }
        };
        (edge.unwrap(), directions, object.unwrap())
      })
  }
}
