#[cfg(test)]
mod real_world {
  use picturs::assert_diagram;

  #[test]
  fn pic8259_test() {
    let string = r#"
      continue down-left
      box.pic1 ht 2in wd 1in "Primary Interrupt Controller"
      line from 1/9 pic1.w 1.5in left "Timer" ljust opaque <-
      box.pic2 same "Secondary Interrupt Controller" 2.5in left from pic1
      "#;
    assert_diagram!(string);
  }

  #[test]
  fn pic8259_diagram() {
    let string = r#"
      continue down-left
      box.pic1 ht 2in wd 1in "Primary Interrupt Controller"
      line from 3/9 pic1.w 2in left <->
      line from 1/9 pic1.w 1.5in left "Timer" ljust opaque <-
      line from 2/9 pic1.w "Keyboard" same
      line from 4/9 pic1.w "Serial Port 2" same
      line from 5/9 pic1.w "Serial Port 1" same
      line from 6/9 pic1.w "Parallel Port 2/3" same
      line from 7/9 pic1.w "Floppy Disk" same
      line from 8/9 pic1.w "Parallel Port 1" same
      box.pic2 same "Secondary Interrupt Controller" 2.5in left from pic1
      line from 1/9 pic2.w "Real Time Clock" same
      line from 2/9 pic2.w "ACPI" same
      line from 3/9 pic2.w "Available" same
      line from 4/9 pic2.w "Available" same
      line from 5/9 pic2.w "Mouse" same
      line from 6/9 pic2.w "Co-Processor" same
      line from 7/9 pic2.w "Primary ATA" same
      line from 8/9 pic2.w "Secondary ATA" same
      "#;
    assert_diagram!(string);
  }

  #[test]
  fn dependencies_serial() {
    let string = r#"
      box.ui13 ht 1in wd 2in "jQuery"
      box.ui20 same wd 4in "Vue"
      box.ui23 same wd 1in "Revert"
      box.ui24 same wd 0.5in "UX"
      "#;
    assert_diagram!(string);
  }

  #[test]
  fn dependencies_parallel() {
    let string = r#"
      top
      path.ui13 1in up 3in right 0.5in down 2in left 0.5in down 1in left "jQuery 😇" top
      path.ui20 1in right 1in down 3in left 0.5in up 1in right "Vue" bottom
      path.fix 1in right to ui20.se 1in up "Improve" above
      path.ux 1in down 1in left to fix.ne "UX" below
      line from 1/3 ui20.e end fix.ne
      line from ui20.se end 2/3 fix.e
      "#;
    assert_diagram!(string);
  }

  #[test]
  fn timeline() {
    // TODO: edge.190 en edge.n is duidelijker dan .190 en .n
    let string = r#"
      set box rd 8pt ht .6in

      sline.time 4in right *-> color dgrey thick

      dot.y49 at 1/12 time.n "1949" below color #645590
      sline from y49.n .5in up color #645590
      box wd 1in .190 .5in up from y49.n color #645590 thick "QCリサーチ\nグループ\n結成"

      dot.y50 at 2/12 time.n "1950年代\n後半" above color #27A8BE
      sline from y50.s 1cm down color #27A8BE
      box wd 1in .20 1cm down from y50.s color #27A8BE normal "新製品開発\nの品質管理\nを始める"

      dot.y72 at 5/12 time.n "1972" below color #D6A02A
      sline from y72.n .5in up color #D6A02A
      box wd 1in .167 .5in up from y72.n color #D6A02A normal "ソフトウェアの 検査の考え方 発表"

      dot.y80 at 7/12 time.n "1980" above color #E47958
      sline from y80.s 1cm down color #E47958
      box wd 2.2in .8 1cm down from y80.s color #E47958 normal "ソフトウェア製品生産管理:\nソフトウェア工学における 品質管理(QC)と品質保証 (QA) 発表"

      dot.y81 at 8/12 time.n "1981" below color #E24B7E
      sline from y81.n .3in up color #E24B7E
      box wd 1.4in         .6:25 .3in up from y81.n color #E24B7E normal "ソフトウェアの 品質管理推進 について 発表"
      box wd 1.4in ht .2in .6:25 1in up from y81.n color #E24B7E normal "日本的品質管理刊行"

      dot.y98 at 11/12 time.n "1998" above color #214F79
      sline from y98.s 1cm down color #214F79 invisible
      box .n 1cm down from y98.s color #214F79 normal "ISTQBの 前身が 発足"
      "#;
    assert_diagram!(string);
  }

  #[test]
  fn risotto_timeline() {
    let string = r#"
      set box rd 8pt ht .6in
      set font 12pt

      continue right-top
      sline.time 6in right *-> color dgrey thick
      dot.water at 0/12 time.n
      dot.pbp at 1/16 time.n
      box .5: 1cm up from water.n <-> "1,2L water" "met 16g" "eekhoorntjes" "brood" color brown
      box "Pijnboom" "pitjes" "roosteren" color grey
      dot.bln at 4/16 time.n
      box 4cm up from bln.n "Zeven" color brown
      box "Fijnhakken" color brown
      dot.ui at 5/16 time.n
      box 1cm up from ui.n "Fruiten" color yellow
      box "Rijst bakken" color black
      box wd 2in "Bouillon toevoegen" color brown
      box "Parmezaan" color yellow
      box "Bakken" color orange

      box 1cm down from pbp.s dotted {
        down
        box "Parmezaan" "schafen" color yellow
        box "Ui snipperen" "Knoflook hakken" color yellow
      }
      dot.vis at 9/12 time.n
      box 1cm down from vis.s "Olie" color orange
      box "Paneren" color orange
      "#;
    assert_diagram!(string);
  }

  // Image from https://youtu.be/nBjEzQlJLHE?t=8
  // Sizes from https://intercom.help/omnitype/en/articles/5121683-keycap-sizes
  #[test]
  fn keyboard() {
    let string = r#"
      continue down-left
      set box rd 8pt ht .4in wd .4in sp 4 pd 0
      box nostroke pd 4 { // 13 x 1u, 1 x 1.5u
        continue right-top
        box "`"
        box "1"
        box "2"
        box "3"
        box "4"
        box "5"
        box "6"
        box "7"
        box "8"
        box "9"
        box "0"
        box "-"
        box "="
        box "delete" wd 1.5u
      }
      box invisible pd 4 { // 13 x 1u, 1 x 1.5u
        continue from right-top
        box "tab" wd 1.5u
        box "Q"
        box "W"
        box "E"
        box "R"
        box "T"
        box "Y"
        box "U"
        box "I"
        box "O"
        box "P"
        box "["
        box "]"
        box "\\"
      }
      box invis pd 4 { // 11 x 1u, 2 x 1.75u
        right-top
        box "caps" wd 1.75u
        box "A"
        box "S"
        box "D"
        box "F"
        box "G"
        box "H"
        box "J"
        box "K"
        box "L"
        box ";"
        box "'"
        box "enter" wd 1.75u
      }
      box invis pd 4 { // 10 x 1u, 2 x 2.25u
        right-top
        box "shift" wd 2.25u
        box "Z"
        box "X"
        box "C"
        box "V"
        box "B"
        box "N"
        box "M"
        box ","
        box "."
        box "/"
        box "shift" wd 2.25u
      }
      "#;
    assert_diagram!(string);
  }

  #[test]
  fn keyboard_small() {
    let string = r#"
      set box rd 8pt ht .4in wd .4in sp 4 pd 0
      box nostroke pd 4 { // 13 x 1u, 1 x 1.5u
        continue right-top
        box "1"
        box "2"
        box "3"
      }
      "#;
    assert_diagram!(string);
  }

  #[test]
  fn thinka() {
    let string = r#"
      box.thinka down {
        box.ui "UI"
        box.functions right {
          box "HAP"
          box "Alexa"
          box "Google"
          box "IFTTT"
          box "Olisto"
          box "Matter"
          box "Cloud"
          box "History"
        }
        box.model "Home Model"
        box.transport right {
          box "thinka(-knx)"
          box "thinka-zwave"
          box "Cloud"
        }
        box.boot "thinka-boot"
      }
    "#;
    assert_diagram!(string);
  }
}