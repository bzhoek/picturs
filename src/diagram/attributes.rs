use crate::diagram::conversion::Conversion;
use crate::diagram::parser::Rule;
use crate::diagram::rules::Rules;
use crate::diagram::types::{Caption, Config, Displacement, Edge, Endings, Movement, ObjectEdge, Radius, ShapeConfig};
use crate::skia::Effect;
use pest::iterators::Pair;
use skia_safe::Color;

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
    effect: Effect,
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

#[derive(Clone, Debug, Default, PartialEq)]
struct OpenAttributes<'a> {
  id: Option<&'a str>,
  same: bool,
  caption: Option<Caption>,
  length: f32,
  endings: Endings,
  source: Option<ObjectEdge>,
  target: Option<ObjectEdge>,
  movement: Option<Displacement>,
  movements: Vec<Movement>,
  stroke: Color,
  thickness: f32,
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

#[cfg(test)]
mod tests {
  use skia_safe::Color;
  use crate::diagram::attributes::OpenAttributes;
  use crate::diagram::conversion::Conversion;
  use crate::diagram::parser::Rule;
  use crate::diagram::rules::Rules;
  use crate::diagram::types::{Caption, Config, Edge, Ending, Endings, ObjectEdge};
  use crate::diagram::types::EdgeDirection::Vertical;

  fn attrs_from(string: &str, config: Option<Config>) -> OpenAttributes {
    let config = config.unwrap_or_default();
    let mut top = Conversion::pairs_for(Rule::picture, string);
    let next = top.next().unwrap();
    let pairs = Rules::get_rule(&next, Rule::line_attributes);
    let mut attrs = OpenAttributes::default();
    pairs.into_inner().for_each(|pair| {
      match pair.as_rule() {
        Rule::endings => attrs.endings = Conversion::endings_from(pair),
        Rule::caption => attrs.caption = Some(Conversion::caption_from(pair, &config)),
        Rule::length => attrs.length = Conversion::length_from(pair, &config.unit).pixels(),
        Rule::same => attrs.same = true,
        Rule::source => attrs.source = Some(Conversion::fraction_edge_from(pair)),
        Rule::target => attrs.target = Some(Conversion::fraction_edge_from(pair)),
        Rule::stroke => attrs.stroke = Conversion::color_from(pair).unwrap_or(attrs.stroke),
        Rule::thickness => attrs.thickness = Conversion::thickness_from(pair),
        Rule::rel_movement | Rule::abs_movement => {
          let movement = Conversion::movement_from(pair, &config.unit);
          attrs.movements.push(movement)
        }
        _ => panic!("Unexpected {:?}", pair)
      }
    });
    attrs
  }

  #[test]
  fn test_attributes() {
    let string = r#"
      path.ui13 <-> ln 2cm from id1.n end id2.s same stroke red thick 1in up 3in right 0.5in down 2in left 0.5in down 1in left "jQuery 😇" above
      "#;

    let attrs = attrs_from(string, None);
    assert_eq!(Endings { start: Ending::Arrow, end: Ending::Arrow }, attrs.endings);
    assert_eq!(76., attrs.length);
    assert_eq!(Some(ObjectEdge { id: "id1".into(), edge: Edge { direction: Vertical, x: 0.0, y: -0.5 } }), attrs.source);
    assert_eq!(Some(ObjectEdge { id: "id2".into(), edge: Edge { direction: Vertical, x: 0.0, y: 0.5 } }), attrs.target);
    assert_eq!(true, attrs.same);
    assert_eq!(Color::RED, attrs.stroke);
    assert_eq!(3.0, attrs.thickness);
    assert_eq!(6, attrs.movements.len());
    assert!(matches!(&attrs.caption, Some(Caption { text, .. } ) if text == "jQuery 😇"));
    assert_eq!("jQuery 😇", attrs.caption.unwrap().text);
  }
}