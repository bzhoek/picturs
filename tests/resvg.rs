#[cfg(test)]
mod tests {
  use resvg::usvg::{NodeExt, Options, Tree};
  use resvg::usvg::TreeParsing;

  #[test]
  fn parses_svg() {
    let svg =
      r#"<svg xmlns="http://www.w3.org/2000/svg" width="400" height="180">
      <g>
      <rect x="50" y="20" rx="20" ry="20" width="200" height="150" style="fill:red;stroke:black;stroke-width:5;opacity:0.5" />
      </g>
      </svg>
      "#;
    let options = Options::default();
    let tree = Tree::from_str(svg, &options).unwrap();
    let bbox = tree.root.calculate_bbox().unwrap();
    println!("{:?}", tree.root);
    assert_eq!(200.0, bbox.width());
  }
}