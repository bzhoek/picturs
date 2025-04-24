#[cfg(test)]
mod units {
  use picturs::assert_diagram;

  #[test]
  fn units() {
    let string = r#"
      down
      set unit cm
      box "A" wd 1in
      box "B" 10 right
      "#;
    assert_diagram!(string);
  }

  #[test]
  fn text_fit() {
    let string = r#"
      set font "Courier"
      right
      text "ssh " fit
      text "-L " fit
      text.local "1025" fit
      text ":localhost:" fit
      text.remote "25 " fit
      text.host "home.hoek.com" fit
      set font "Helvetica"
      line from local.n 1cm up "local port" above
      line from remote.s 1cm down "remote port" below
      line from host.n 1cm up "on host" above
      "#;
    assert_diagram!(string);
  }
}