#![allow(dead_code)]

use log::{debug, warn};
use pest::iterators::{Pair, Pairs};
use pest::Parser;
use skia_safe::{Color, Font, FontMgr, FontStyle};

use crate::diagram::index::ShapeName;
use crate::diagram::parser::{DiagramParser, Rule};
use crate::diagram::rules::Rules;
use crate::diagram::types::{Caption, Config, Displacement, Edge, EdgeDirection, Ending, Endings, Continuation, Length, Movement, ObjectEdge, Unit};
use crate::skia::Effect;

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

  pub(crate) fn colors_from(pair: &Pair<Rule>, stroke: &Color) -> (Color, Color, Color) {
    let stroke = Conversion::stroke_color_in(pair).unwrap_or(*stroke);
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

  // https://www.rapidtables.com/web/color/RGB_Color.html
  // https://www.colordic.org/w (Japanese)
  pub(crate) fn color_from(pair: Pair<Rule>) -> Option<Color> {
    let mut pairs = pair.into_inner();
    pairs.find_map(|pair| {
      match pair.as_rule() {
        Rule::id => {
          let str = pair.as_str();
          match str {
            "black" => Color::BLACK,
            "white" => Color::WHITE,
            "red" => Color::RED,
            "green" => Color::GREEN,
            "yellow" => Color::YELLOW,
            "blue" => Color::BLUE,
            "cyan" => Color::CYAN,
            "magenta" => Color::MAGENTA,
            "brown" => Color::new(0xFFA52A2A),
            "orange" => Color::new(0xFFFFA500),
            "pink" => Color::new(0xFFFFC0CB),
            "purple" => Color::new(0xFF800080),
            "gray" | "grey" => Color::GRAY,
            "dgray" | "dgrey" => Color::DARK_GRAY,
            "lgray" => Color::LIGHT_GRAY,
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
        _ => panic!("Unexpected rule for color {:?}", pair.as_rule())
      }.into()
    })
  }

  pub(crate) fn str_for<'a>(pair: &Pair<'a, Rule>, rule: Rule) -> Option<&'a str> {
    Rules::find_rule(pair, rule)
      .map(|p| p.as_str())
  }

  pub(crate) fn string_in<'a>(pair: &Pair<'a, Rule>, rule: Rule) -> Option<&'a str> {
    Rules::dig_rule(pair, rule)
      .map(|p| p.as_str())
  }

  pub(crate) fn string_for(pair: &Pair<Rule>, rule: Rule) -> Option<String> {
    Rules::find_rule(pair, rule)
      .map(Self::string_from)
  }

  pub(crate) fn strings_for(pair: &Pair<Rule>) -> Option<String> {
    let mut strings = vec![];

    let pairs = pair.clone().into_inner();
    pairs.for_each(|pair| match pair.as_rule() {
      Rule::string => {
        strings.push(Self::string_from(pair));
      }
      _ => debug!("Ignore rule {:?}", pair.as_rule())
    });

    if strings.is_empty() {
      None
    } else {
      Some(strings.join("\n"))
    }
  }

  pub(crate) fn string_from(pair: Pair<Rule>) -> String {
    let str = pair.clone().into_inner()
      .next().unwrap().as_str();
    str.replace("\\n", "\n")
  }

  pub(crate) fn identified_in<'a>(pair: &Pair<'a, Rule>) -> Option<&'a str> {
    Rules::dig_rule(pair, Rule::identified)
      .map(|p| p.into_inner().next().unwrap().as_str())
  }

  #[allow(clippy::unwrap_or_default)]
  pub(crate) fn caption(pair: &Pair<Rule>, config: &Config) -> Option<Caption> {
    Rules::find_rule(pair, Rule::caption)
      .map(|caption| { Self::caption_from(caption, config) })
  }

  pub(crate) fn font_from(pair: Pair<Rule>, unit: &Unit) -> Font {
    let mut family = "Helvetica".to_owned();
    let mut size = 17.;

    let pairs = pair.into_inner();
    pairs.for_each(|pair| match pair.as_rule() {
      Rule::string => {
        family = Self::string_from(pair);
      }
      Rule::size => {
        let length = Conversion::length_from(pair, unit);
        size = length.points();
      }
      _ => warn!("Unexpected rule for font {:?}", pair.as_rule())
    });

    let typeface = FontMgr::default().match_family_style(family, FontStyle::default()).unwrap();
    Font::from_typeface(typeface, size)
  }

  pub(crate) fn caption_from(pair: Pair<Rule>, config: &Config) -> Caption {
    let mut text: Option<String> = None;
    let mut alignment: Option<(Edge, Edge)> = None;
    let mut opaque = false;

    let pairs = pair.into_inner();
    pairs.for_each(|pair| match pair.as_rule() {
      Rule::string => {
        text = Self::string_from(pair).into();
      }
      Rule::alignment => {
        let string = pair.as_str();
        alignment = match string {
          "ljust" => (Edge::left(), Edge::left()),
          "top" => (Edge::above(), Edge::above()),
          "bottom" => (Edge::below(), Edge::below()),
          _ => (string.into(), Edge::from(string).mirror())
        }.into();
      }
      Rule::opaque => { opaque = true }
      _ => panic!("Unexpected rule for caption {:?}", pair.as_rule())
    });

    let (rect_edge, caption_edge) = alignment.unwrap_or((Edge::center(), Edge::center()));
    let text = text.unwrap();
    let bounds = config.measure_string(&text);
    Caption { text, rect_edge, caption_edge, bounds, opaque }
  }

  pub(crate) fn endings(pair: &Pair<Rule>) -> Option<Endings> {
    Rules::find_rule(pair, Rule::endings)
      .map(Self::endings_from)
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
    Movement::ObjectStart { object }
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

  pub(crate) fn space_into(attributes: &Pair<Rule>, unit: &Unit) -> Option<f32> {
    Conversion::length_in(attributes, Rule::space, unit)
      .map(|length| length.pixels())
  }

  pub(crate) fn width_into(attributes: &Pair<Rule>, unit: &Unit) -> Option<f32> {
    Conversion::length_in(attributes, Rule::width, unit)
      .map(|length| length.pixels())
  }

  pub(crate) fn width_into_(attributes: &Pair<Rule>, unit: &Unit, size: f32) -> Option<f32> {
    Conversion::length_in_(attributes, Rule::width, unit, size)
      .map(|length| length.pixels())
  }

  pub(crate) fn length_in_(pair: &Pair<Rule>, rule: Rule, unit: &Unit, size: f32) -> Option<Length> {
    Rules::dig_rule(pair, rule)
      .map(|pair| Self::length_from_(pair, unit, size))
  }

  pub(crate) fn length_from_(pair: Pair<Rule>, unit: &Unit, size: f32) -> Length {
    let mut pairs = pair.into_inner();
    let factor = Self::next_to_f32(&mut pairs).unwrap();
    match Self::next_to_string(&mut pairs).map(|str| str.into()) {
      Some(Unit::Unit) => {
        Length::new(factor * size, Unit::Px)
      }
      Some(unit) => {
        Length::new(factor, unit)
      }
      None => Length::new(factor, *unit)
    }
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
    let edge = inner.next().map(Self::edge_from).unwrap_or(Edge::center());

    ObjectEdge::new(id, edge)
  }

  pub(crate) fn continuation_in(pair: &Pair<Rule>) -> Option<Continuation> {
    Rules::dig_rule(pair, Rule::continuation)
      .map(|pair| Continuation::new(pair.as_str()))
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
    let direction = Self::str_for(&pair, Rule::direction).unwrap();
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

  pub(crate) fn edge_from(pair: Pair<Rule>) -> Edge {
    let next = pair.into_inner().next().unwrap();
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
      Rule::hours => Self::hours(str),
      _ => panic!("unexpected rule")
    };
    Edge::from(degrees as f32)
  }

  pub(crate) fn hours(str: &str) -> i32 {
    let colon = str.find(':').unwrap();
    let hour = str[0..colon].parse::<i32>().unwrap();
    let minutes = &str[(colon + 1)..];
    let minutes = if minutes.is_empty() {
      0.
    } else {
      minutes.parse::<f32>().unwrap() * 30. / 60.
    };
    hour * 30 + minutes as i32
  }

  pub(crate) fn location_for(pair: &Pair<Rule>, _end: &Edge, unit: &Unit) -> Option<(Edge, Vec<Displacement>, ObjectEdge)> {
    Rules::find_rule(pair, Rule::location)
      .map(|p| {
        let mut object: Option<ObjectEdge> = None;
        let mut directions: Vec<Displacement> = vec![];
        let mut edge: Option<Edge> = None;

        for pair in p.into_inner() {
          match pair.as_rule() {
            Rule::edge_point => { edge = Some(Self::edge_from(pair)); }
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
      _ => panic!(" {:?}", pair.as_rule())
    });

    Endings { start, end }
  }

  pub(crate) fn thickness_for(pair: &Pair<Rule>) -> f32 {
    Rules::find_rule(pair, Rule::thickness)
      .map(Self::thickness_from)
      .unwrap_or(1.0)
  }

  pub(crate) fn thickness_from(pair: Pair<Rule>) -> f32 {
    match pair.as_str() {
      "invisible" | "invis" | "nostroke" => 0.0,
      "thin" => 1.0,
      "normal" => 2.0,
      "thick" => 3.0,
      "thicker" => 4.0,
      "thickest" => 6.0,
      _ => panic!("unknown thickness {:?}", pair.as_str()),
    }
  }

  pub(crate) fn effect_for(pair: &Pair<Rule>) -> Effect {
    Rules::find_rule(pair, Rule::effect)
      .map(Self::effect_from)
      .unwrap_or(Effect::Solid)
  }

  pub(crate) fn effect_from(pair: Pair<Rule>) -> Effect {
    match pair.as_str() {
      "dashed" => Effect::Dashed,
      "dotted" => Effect::Dotted,
      _ => panic!("Unknown effect {:?}", pair.as_str()),
    }
  }
}
