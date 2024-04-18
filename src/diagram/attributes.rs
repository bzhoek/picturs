use pest::iterators::Pair;
use skia_safe::Color;
use crate::diagram::conversion::Conversion;
use crate::diagram::parser::Rule;
use crate::diagram::rules::Rules;
use crate::diagram::types::{Caption, Config, Displacement, Edge, Endings, ObjectEdge, Radius, ShapeConfig};

#[derive(Clone, Debug, PartialEq)]
pub enum Attributes<'a> {
  Open {
    id: Option<&'a str>,
    same: bool,
    caption: Option<Caption>,
    length: f32,
    arrows: Endings,
    source: Option<ObjectEdge>,
    target: Option<ObjectEdge>,
    movement: Option<Displacement>,
    stroke: Color,
    thickness: f32,
  },
  Closed {
    id: Option<&'a str>,
    same: bool,
    width: Option<f32>,
    height: Option<f32>,
    padding: f32,
    radius: Radius,
    title: Option<String>,
    location: Option<(Edge, Vec<Displacement>, ObjectEdge)>,
    stroke: Color,
    fill: Color,
    text: Color,
    thickness: f32,
  },
}

impl<'i> Attributes<'i> {
  pub(crate) fn open_attributes<'a>(pair: &Pair<'a, Rule>, config: &Config, rule: Rule) -> (Attributes<'a>, Pair<'a, Rule>) {
    let attributes = Rules::get_rule(pair, rule);
    let (stroke, _fill, _text) = Conversion::colors_from(&attributes);

    (Attributes::Open {
      id: Conversion::identified_in(pair),
      caption: Conversion::caption(&attributes, config),
      length: Conversion::length_into(&attributes, &config.unit).unwrap_or(config.line.pixels()),
      arrows: Conversion::endings(&attributes),
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
    let (stroke, fill, text) = Conversion::colors_from(&attributes);

    (Attributes::Closed {
      id: Conversion::identified_in(pair),
      same: Rules::find_rule(&attributes, Rule::same).is_some(),
      width: Conversion::width_into(&attributes, &config.unit),
      height: Conversion::height_into(&attributes, &config.unit),
      padding: Conversion::padding_into(&attributes, &config.unit).unwrap_or(shape.padding),
      radius: Conversion::radius_into(&attributes, &config.unit).unwrap_or(shape.radius),
      title: Conversion::string_for(&attributes, Rule::string),
      location: Conversion::location_for(&attributes, &Edge::default(), &config.unit),
      stroke,
      fill,
      text,
      thickness: Conversion::thickness_for(&attributes),
    }, attributes)
  }
}