use pest::iterators::Pair;

use crate::diagram::parser::Rule;

pub struct Rules;

impl Rules {

  pub fn get_rule<'a>(pair: &Pair<'a, Rule>, rule: Rule) -> Pair<'a, Rule> {
    Self::find_rule(pair, rule).unwrap()
  }

  pub fn find_rule<'a>(pair: &Pair<'a, Rule>, rule: Rule) -> Option<Pair<'a, Rule>> {
    pair.clone().into_inner()
      .find(|p| p.as_rule() == rule)
  }

  pub fn dig_rule<'a>(pair: &Pair<'a, Rule>, rule: Rule) -> Option<Pair<'a, Rule>> {
    for pair in pair.clone().into_inner() {
      if pair.as_rule() == rule {
        return Some(pair);
      }
      if let Some(pair) = Self::dig_rule(&pair, rule) {
        return Some(pair);
      }
    }
    None
  }
}