
# picturs: pic grammar in Rust

* https://pikchr.org/home/doc/trunk/homepage.md
* https://pikchr.org/home/doc/trunk/doc/grammar.md

## Diagram
1. ~Render all rectangles~
2. ~Wrap the text in a rectangle~
3. ~Save primitive bounds in AST on parsing first pass~
4. ~Onderscheid tussen `rect` en `used`, met eventueel een transform bij het renderen~
5. ~Helper functies in Diagram type~
6. ~Positioneren met `@nw 1cm from left.ne`~
7. ~Unit string in Distance vervangen door enum~
8. ~Hele voorgaande layout beschikbaar~
9. ~Lijn trekken van `now` naar `future`~
10. ~Lijn met pijlpunt~
11. ~Move met meerdere offsets~
12. ~Tekst centreren in een rechthoek~
12. ~dot at edge~
13. Automatisch grootte bepalen
13. `arrow` met offset
13. `nnw` en uren op de klok, met horizontal en vertical

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

### Timeline

https://speakerdeck.com/nihonbuson/agile-testinghaxin-siigai-nian-nanoka-pin-zhi-bao-zheng-noli-shi-wota-maetekao-eru-number-scrumniigata?slide=28