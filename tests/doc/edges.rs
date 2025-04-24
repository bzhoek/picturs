#[cfg(test)]
mod edges {
  use picturs::assert_diagram;

  #[test]
  fn all_edges() {
    let string = r#"
      right
      set box wd 2.5cm ht 2cm

      box.a "direction" color black
      box.b "compass" 2cm right color black
      box.c "hours" 2cm right color black
      box.d "degrees" 2cm right color black

      dot at a.n color red rad 4pt ".up" above
      dot at a.s same ".down" below
      dot at a.e same ".right" right
      dot at a.w same ".left" left

      dot at b.n color red rad 4pt ".n" above
      dot at b.s same ".s" below
      dot at b.e same ".e" right
      dot at b.w same ".w" left
      dot at b.nw color blue rad 4pt ".nw" nw
      dot at b.ne same "ne" ne
      dot at b.sw same ".sw" sw
      dot at b.se same ".se" se

      dot at c.12: color red rad 4pt ".12:" above
      dot at c.1:  same ".1:"  above
      dot at c.2:  same ".2:"  right
      dot at c.3:  same ".3:"  right
      dot at c.4:  same ".4:"  right
      dot at c.5:  same ".5:"  below
      dot at c.6:  same ".6:"  below
      dot at c.7:  same ".7:"  below
      dot at c.8:  same ".8:"  left
      dot at c.9:  same ".9:"  left
      dot at c.10: same ".10:" left
      dot at c.11: same ".11:" above

      dot at d.360 color red rad 4pt ".360" above
      dot at d.30 color cyan rad 4pt ".30" above
      dot at d.45 same "45" ne
      dot at d.90 same ".90" right
      dot at d.135 same ".135" se
      dot at d.180 same ".180" below
      dot at d.245 color cyan rad 4pt ".245" left
      dot at d.225 same ".225" sw
      dot at d.270 same ".270" left
      dot at d.315 color blue rad 4pt ".315" nw
      "#;
    assert_diagram!(string);
  }

  #[test]
  fn direction_edges() {
    let string = r#"
      continue down-left
      box.a wd 1in ht 1in
      dot at a.n color red rad 4pt ".up" above
      dot at a.s same ".down" below
      dot at a.e same ".right" right
      dot at a.w same ".left" left
      "#;
    assert_diagram!(string);
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
    assert_diagram!(string);
  }

  #[test]
  fn hours_edges() {
    let string = r#"
      box.c wd 1in ht 1in
      dot at c.12: color red rad 4pt ".12:" above
      dot at c.1:  same ".1:"  above
      dot at c.2:  same ".2:"  right
      dot at c.3:  same ".3:"  right
      dot at c.4:  same ".4:"  right
      dot at c.5:  same ".5:"  below
      dot at c.6:  same ".6:"  below
      dot at c.7:  same ".7:"  below
      dot at c.8:  same ".8:"  left
      dot at c.9:  same ".9:"  left
      dot at c.10: same ".10:" left
      dot at c.11: same ".11:" above
      "#;
    assert_diagram!(string);
  }

  #[test]
  fn degrees_edges() {
    let string = r#"
      box.d wd 1in ht 1in
      dot at d.360 color red rad 4pt ".0" above
      dot at d.20 color cyan rad 4pt ".20" above
      dot at d.45 same "45" ne
      dot at d.90 same ".90" right
      dot at d.135 same ".135" se
      dot at d.180 same ".180" below
      dot at d.245 color cyan rad 4pt ".245" left
      dot at d.225 same ".225" sw
      dot at d.270 same ".270" left
      dot at d.315 color blue rad 4pt ".315" nw
      "#;
    assert_diagram!(string);
  }

  #[test]
  fn place_center_at_edge() {
    let string = r#"
      box.m1 wd 2in ht 1in
      box.m2 2cm right from m1.e wd 2in ht 1in
      box.c1 .c at from m1.ne stroke red
      box.c2 .c at from m2.ne stroke red dotted
      box.c3 .se at from m2.ne stroke red
      arrow from m1.c end m1.ne "ne" stroke black
      arrow from c3.c end m2.ne "sw" stroke black
      "#;
    assert_diagram!(string);
  }

}