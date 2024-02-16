#[cfg(test)]
mod units {
  use picturs::diagram::create_diagram;
  use picturs::test::assert_diagram;

  use crate::{assert_diagram};

  #[test]
  fn units() {
    let string = r#"
      down
      set unit cm
      box "A" wd 1in
      box "B" 10 right
      "#;
    let diagram = create_diagram(string);
    assert_diagram!(diagram);
  }

  #[test]
  fn text_fit() {
    let string = r#"
      set font "Courier"
      right
      text "ssh" fit
      text " -L " fit
      text.local "1025" fit
      text ":localhost:" fit
      text.remote "25" fit
      text.host " home.hoek.com" fit
      set font "Helvetica"
      line from local.n 1cm up "local port" above
      line from remote.s 1cm down "remote port" below
      line from host.n 1cm up "on host" above
      "#;
    let diagram = create_diagram(string);
    assert_diagram!(diagram);
  }
}