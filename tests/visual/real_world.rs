#[cfg(test)]
mod real_world {
  use picturs::assert_diagram;
  use picturs::diagram::create_diagram;

  #[test]
  fn pic8259_diagram() {
    let string = r#"
      box.pic1 ht 2in wd 1in "Primary Interrupt Controller"
      line from 1/8 pic1.w 1.5in left "Timer" above
      line from 2/8 pic1.w 1.5in left "Keyboard" above
      line from 3/8 pic1.w 1.5in left
      line from 4/8 pic1.w 1.5in left "Serial Port 2" above
      line from 5/8 pic1.w 1.5in left "Serial Port 1" above
      line from 6/8 pic1.w 1.5in left "Parallel Port 2/3" above
      line from 7/8 pic1.w 1.5in left "Floppy Disk" above
      line from 8/8 pic1.w 1.5in left "Parallel Port 1" above
      box.pic2 ht 2in wd 1in "Secondary Interrupt Controller" 2.5in left from pic1
      "#;
    let diagram = create_diagram(string);
    assert_diagram!(diagram);
  }
}