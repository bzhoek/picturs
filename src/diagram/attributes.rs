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

impl Attributes<'_> {
  pub(crate) fn open_attributes<'a>(pair: &Pair<'a, Rule>, config: &Config, rule: Rule) -> (Attributes<'a>, Pair<'a, Rule>) {
    let attributes = Rules::get_rule(pair, rule);
    let (stroke, _fill, _text) = Conversion::colors_from(&attributes, &Color::BLACK);

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

  pub(crate) fn copy_attributes(&mut self, other: Option<&Attributes>) {
    match (self, other) {
      (Attributes::Closed {
        same,
        width,
        height,
        ..
      }, Some(Attributes::Closed {
        width: last_width,
        height: last_height,
        ..
      })
      ) => {
        if !*same {
          return;
        }
        if width.is_none() {
          *width = *last_width;
        }
        if height.is_none() {
          *height = *last_height;
        }
      }
      (Attributes::Open {
        same,
        endings,
        movement,
        caption,
        ..
      }, Some(Attributes::Open {
        endings: last_endings,
        movement: last_movement,
        caption: last_caption,
        ..
      })
      ) => {
        if !*same {
          return;
        }
        *endings = last_endings.clone();
        if movement.is_none() {
          movement.clone_from(last_movement);
        }
        if let Some(caption) = &mut *caption {
          if let Some(last) = last_caption.as_ref() {
            caption.rect_edge = last.rect_edge.clone();
            caption.caption_edge = last.caption_edge.clone();
            caption.opaque = last.opaque;
          }
        }
      }
      _ => {}
    }
  }
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct ClosedAttributes<'a> {
  pub(crate) id: Option<&'a str>,
  pub(crate) same: bool,
  pub(crate) width: Option<f32>,
  pub(crate) height: Option<f32>,
  pub(crate) padding: f32,
  pub(crate) radius: f32,
  pub(crate) space: f32,
  pub(crate) title: Option<String>,
  pub(crate) strings: Vec<String>,
  pub(crate) location: Option<EdgeMovement>,
  pub(crate) endings: Option<Endings>,
  pub(crate) stroke: Color,
  pub(crate) fill: Color,
  pub(crate) text: Color,
  pub(crate) thickness: f32,
  pub(crate) effect: Effect,
}

impl<'a> ClosedAttributes<'a> {
  pub(crate) fn from(pair: &Pair<'a, Rule>, config: &Config, shape: &ShapeConfig) -> Self {
    let mut attrs = ClosedAttributes::default();
    pair.clone().into_inner().for_each(|pair| {
      match pair.as_rule() {
        Rule::identified => attrs.id = Some(pair.into_inner().next().unwrap().as_str()),
        Rule::closed_attributes => Self::attributes(&pair, config, shape, &mut attrs),
        // _ => panic!("Unexpected {:?}", pair)
        _ => {}
      }
    });
    attrs
  }

  pub(crate) fn attributes(pair: &Pair<'a, Rule>, config: &Config, shape: &ShapeConfig, attrs: &mut ClosedAttributes<'a>) {
    attrs.fill = Color::TRANSPARENT;
    attrs.stroke = shape.stroke;
    attrs.thickness = 1.0;
    attrs.text = Color::BLACK;
    attrs.radius = shape.radius;
    attrs.space = shape.space;
    attrs.padding = shape.padding;

    pair.clone().into_inner().for_each(|pair| {
      match pair.as_rule() {
        Rule::string => {
          attrs.strings.push(Conversion::string_from(pair));
        }
        Rule::same => attrs.same = true,
        Rule::height => attrs.height = Conversion::length_from_(pair, &config.unit, shape.height).pixels().into(),
        Rule::width => attrs.width = Conversion::length_from_(pair, &config.unit, shape.width).pixels().into(),
        Rule::padding => attrs.padding = Conversion::length_from(pair, &config.unit).pixels(),
        Rule::location => attrs.location = Some(Conversion::location_from(pair, &config.unit)),
        Rule::stroke => attrs.stroke = Conversion::color_from(pair).unwrap_or(attrs.stroke),
        Rule::fill => attrs.fill = Conversion::color_from(pair).unwrap_or(attrs.fill),
        Rule::thickness => attrs.thickness = Conversion::thickness_from(pair),
        Rule::effect => attrs.effect = Conversion::effect_from(pair),
        Rule::radius => attrs.radius = Conversion::length_from(pair, &config.unit).pixels(),
        Rule::text_color => attrs.text = Conversion::color_from(pair).unwrap_or(attrs.text),
        Rule::endings => attrs.endings = Conversion::endings_from(pair).into(),
        Rule::continuation => {}
        _ => panic!("Unexpected {:?}", pair)
      }
    });
    if !attrs.strings.is_empty() {
      attrs.title = Some(attrs.strings.join("\n"));
    }
  }
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct OpenAttributes<'a> {
  pub(crate) id: Option<&'a str>,
  pub(crate) same: bool,
  pub(crate) route: bool,
  pub(crate) caption: Option<Caption>,
  length: f32,
  pub(crate) endings: Endings,
  source: Option<ObjectEdge>,
  target: Option<ObjectEdge>,
  movement: Option<Displacement>,
  pub(crate) movements: Vec<Movement>,
  stroke: Color,
  thickness: f32,
}

impl<'a> OpenAttributes<'a> {
  pub(crate) fn from(pair: &Pair<'a, Rule>, config: &Config) -> OpenAttributes<'a> {
    let mut attrs = OpenAttributes::default();
    pair.clone().into_inner().for_each(|pair| {
      match pair.as_rule() {
        Rule::identified => attrs.id = Some(pair.into_inner().next().unwrap().as_str()),
        Rule::open_attributes => OpenAttributes::attributes(&pair, config, &mut attrs),
        _ => panic!("Unexpected {:?}", pair)
      }
    });
    attrs
  }

  pub(crate) fn attributes(pair: &Pair<'a, Rule>, config: &Config, attrs: &mut OpenAttributes<'a>) {
    pair.clone().into_inner().for_each(|pair| {
      match pair.as_rule() {
        Rule::endings => attrs.endings = Conversion::endings_from(pair),
        Rule::caption => attrs.caption = Some(Conversion::caption_from(pair, config)),
        Rule::length => attrs.length = Conversion::length_from(pair, &config.unit).pixels(),
        Rule::same => attrs.same = true,
        Rule::route => attrs.route = true,
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
  }
}

#[cfg(test)]
mod tests {
  use crate::diagram::attributes::OpenAttributes;
  use crate::diagram::conversion::Conversion;
  use crate::diagram::parser::Rule;
  use crate::diagram::types::EdgeDirection::Vertical;
  use crate::diagram::types::{Caption, Config, Edge, Ending, Endings, ObjectEdge};
  use skia_safe::Color;

  fn attrs_from(string: &str, config: Option<Config>) -> OpenAttributes<'_> {
    let config = config.unwrap_or_default();
    let mut top = Conversion::pairs_for(Rule::picture, string);
    let next = top.next().unwrap();
    OpenAttributes::from(&next, &config)
  }

  #[test]
  fn test_attributes() {
    let string = r#"
      path.ui13 <-> ln 2cm from id1.n end id2.s same stroke red thick 1in up 3in right 0.5in down 2in left 0.5in down 1in left "jQuery 😇" above
      "#;

    let attrs = attrs_from(string, None);
    assert_eq!(Some("ui13"), attrs.id);
    assert_eq!(Endings { start: Ending::Arrow, end: Ending::Arrow }, attrs.endings);
    assert_eq!(78., attrs.length);
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