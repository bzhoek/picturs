
# picturs: pic grammar in Rust

* https://pikchr.org/home/doc/trunk/homepage.md
* https://pikchr.org/home/doc/trunk/doc/grammar.md

## Nested
1. ~Render all rectangles~
2. ~Wrap the text in a rectangle~
3. ~Save primitive bounds in AST on parsing first pass~
4. ~Onderscheid tussen `rect` en `used`, met eventueel een transform bij het renderen~
5. ~Helper functies in Diagram type~
6. ~Positioneren met `@nw 1cm from left.ne`~
7. ~Unit string in Distance vervangen door enum~
8. Layout als hele AST beschikbaar is
9. Bounds interface definieren zodat niet heel Canvas een dependency wordt

`.nw 1cm right 2cm down from left.ne`
`.nw=left.ne 1cr 2cd`

## pest
 
* https://docs.rs/pest/latest/pest/
* https://pest.rs/book/intro.html
* https://github.com/pest-parser/pest

https://blog.logrocket.com/understanding-rust-option-results-enums/

```
box.left "This goes to the left hand side"
box.right "While this goes to the right hand side" @nw 2cm right 2cm up from left.ne

box.now "Now" {
  box.step3 "What do we need to start doing now"
}
box.future "March" {
  box.step1 "Imagine it is four months into the future"
  box.step2 "What would you like to write about the past period"
  box.note "IMPORTANT: write in past tense"
}
line from now.n to future.n

balloon
list

```



```sh
cargo add skia-safe --features metal
```

