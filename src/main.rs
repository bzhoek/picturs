use std::fs;

use pest::error::Error;
use pest::Parser;
use pest_derive::Parser;

#[derive(Parser)]
#[grammar = "pic.pest"] // relative to project `src`
struct PicParser;

fn main() {
  let pic = fs::read_to_string("pikchr.pic").unwrap();
  match PicParser::parse(Rule::statements, &*pic) {
    Ok(mut pairs) => {
      let next = pairs.next().unwrap();
      for pair in next.into_inner() {
        println!("{:?}", pair);
      }
      // println!("{:?}", pairs);
    }
    Err(e) => { println!("{}", e) }
  };
}

#[cfg(test)]
mod tests {
  #[test]
  fn it_works() {
    let result = 2 + 2;
    assert_eq!(result, 4);
  }
}