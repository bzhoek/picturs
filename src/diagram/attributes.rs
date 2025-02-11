use pest::iterators::Pair;
use skia_safe::Color;
use crate::diagram::conversion::Conversion;
use crate::diagram::parser::Rule;
use crate::diagram::rules::Rules;
use crate::diagram::types::{Caption, Config, Displacement, Edge, Endings, ObjectEdge, Radius, ShapeConfig};
use crate::skia::Effect;

pub(crate) type EdgeMovement = (Edge, Vec<Displacement>, ObjectEdge);

#[derive(Clone, Debug, PartialEq)]
pub enum Attributes<'a> {
  Closed {
    id: Option<&'a str>,
    same: bool,
    width: Option<f32>,
    height: Option<f32>,
    padding: f32,
    radius: Radius,
    space: f32,
    title: Option<String>,
    location: Option<EdgeMovement>,
    endings: Option<Endings>,
    stroke: Color,
    fill: Color,
    text: Color,
    thickness: f32,
    effect: Effect
  },
  Open {
    id: Option<&'a str>,
    same: bool,
    caption: Option<Caption>,
    length: f32,
    endings: Endings,
    source: Option<ObjectEdge>,
    target: Option<ObjectEdge>,
    movement: Option<Displacement>,
    stroke: Color,
    thickness: f32,
  },
}

impl Attributes<'_> {
  pub(crate) fn open_attributes<'a>(pair: &Pair<'a, Rule>, config: &Config, rule: Rule) -> (Attributes<'a>, Pair<'a, Rule>) {
    let attributes = Rules::get_rule(pair, rule);
    let (stroke, _fill, _text) = Conversion::colors_from(&attributes, &Color::BLUE);

    (Attributes::Open {
      id: Conversion::identified_in(pair),
      caption: Conversion::caption(&attributes, config),
      length: Conversion::length_into(&attributes, &config.unit).unwrap_or(config.line.pixels()),
      endings: Conversion::endings(&attributes).unwrap_or_default(),
      source: Conversion::fraction_edge_for(&attributes, Rule::source),
      target: Conversion::fraction_edge_for(&attributes, Rule::target),
      movement: Conversion::displacement_for(&attributes, Rule::rel_movement, &config.unit),
      same: Rules::find_rule(&attributes, Rule::same).is_some(),
      stroke,
      thickness: Conversion::thickness_for(&attributes),
    }, attributes)
  }

  pub(crate) fn closed_attributes<'a>(pair: &Pair<'a, Rule>, config: &Config, shape: &ShapeConfig) -> (Attributes<'a>, Pair<'a, Rule>) {
    let attributes = Rules::get_rule(pair, Rule::attributes);
    let (stroke, fill, text) = Conversion::colors_from(&attributes, &shape.stroke);

    (Attributes::Closed {
      id: Conversion::identified_in(pair),
      same: Rules::find_rule(&attributes, Rule::same).is_some(),
      width: Conversion::width_into_(&attributes, &config.unit, shape.width),
      height: Conversion::height_into(&attributes, &config.unit),
      padding: Conversion::padding_into(&attributes, &config.unit).unwrap_or(shape.padding),
      radius: Conversion::radius_into(&attributes, &config.unit).unwrap_or(shape.radius),
      space: Conversion::space_into(&attributes, &config.unit).unwrap_or(shape.space),
      title: Conversion::strings_for(&attributes),
      location: Conversion::location_for(&attributes, &Edge::default(), &config.unit),
      endings: Conversion::endings(&attributes),
      stroke,
      fill,
      text,
      thickness: Conversion::thickness_for(&attributes),
      effect: Conversion::effect_for(&attributes),
    }, attributes)
  }
}