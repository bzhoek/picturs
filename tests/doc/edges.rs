#[cfg(test)]
mod edges {
  use picturs::diagram::create_diagram;
  use picturs::test::assert_diagram;

  use crate::{assert_diagram};

  #[test]
  fn all_edges() {
    let string = r#"
      right
      set box wd 2.5cm ht 2cm

      box.a "direction"
      box.b "compass" 2cm right
      box.c "hours" 2cm right
      box.d "degrees" 2cm right

      dot at a.n color red rad 4pt ".up" above
      dot at a.s same ".down" below
      dot at a.e same ".right" right
      dot at a.w same ".left" left

      dot at c.n color red rad 4pt ".12:" above
      dot at c.e same ".3:" right
      dot at c.s same ".6:" below
      dot at c.w same ".9:" left

      dot at b.n color red rad 4pt ".n" above
      dot at b.s same ".s" below
      dot at b.e same ".e" right
      dot at b.w same ".w" left
      dot at b.nw color blue rad 4pt ".nw" nw
      dot at b.ne same "ne" ne
      dot at b.sw same ".sw" sw
      dot at b.se same ".se" se

      dot at d.n color red rad 4pt ".0" above
      dot at d.ne same "45" ne
      dot at d.e same ".90" right
      dot at d.se same ".135" se
      dot at d.s same ".180" below
      dot at d.sw same ".225" sw
      dot at d.w same ".270" left
      dot at d.nw color blue rad 4pt ".315" nw
      "#;
    let diagram = create_diagram(string);
    assert_diagram!(diagram);
  }

  #[test]
  fn direction_edges() {
    let string = r#"
      box.a wd 1in ht 1in
      dot at a.n color red rad 4pt ".up" above
      dot at a.s same ".down" below
      dot at a.e same ".right" right
      dot at a.w same ".left" left
      "#;
    let diagram = create_diagram(string);
    assert_diagram!(diagram);
  }

  #[test]
  fn compass_edges() {
    let string = r#"
      box.b wd 1in ht 1in
      dot at b.n color red rad 4pt ".n" above
      dot at b.s same ".s" below
      dot at b.e same ".e" right
      dot at b.w same ".w" left
      dot at b.nw color blue rad 4pt ".nw" nw
      dot at b.ne same "ne" ne
      dot at b.sw same ".sw" sw
      dot at b.se same ".se" se
      "#;
    let diagram = create_diagram(string);
    assert_diagram!(diagram);
  }

  #[test]
  fn hours_edges() {
    let string = r#"
      box.c wd 1in ht 1in
      dot at c.n color red rad 4pt ".12:" above
      dot at c.e same ".3:" right
      dot at c.s same ".6:" below
      dot at c.w same ".9:" left
      "#;
    let diagram = create_diagram(string);
    assert_diagram!(diagram);
  }

  #[test]
  fn degrees_edges() {
    let string = r#"
      box.d wd 1in ht 1in
      dot at d.n color red rad 4pt ".0" above
      dot at d.ne same "45" ne
      dot at d.e same ".90" right
      dot at d.se same ".135" se
      dot at d.s same ".180" below
      dot at d.sw same ".225" sw
      dot at d.w same ".270" left
      dot at d.nw color blue rad 4pt ".315" nw
      "#;
    let diagram = create_diagram(string);
    assert_diagram!(diagram);
  }

  #[test]
  fn degrees_edge() {
    let string = r#"
      box.d wd 1in ht 1in
      dot at d.0 color red rad 4pt ".0" above
      "#;
    let diagram = create_diagram(string);
    assert_diagram!(diagram);
  }
}