# picturs: A PIC like grammar in Rust

* https://pikchr.org/home/doc/trunk/homepage.md
* https://pikchr.org/home/doc/trunk/doc/grammar.md

## Relative placement

```pic
box "box"
circle "circle" 1 right
ellipse "ellipse" 1 right
oval "oval" 1 below first box
```

The relative `1 right` is shorthand for `.w 1cm right of last.e`. Since the direction is `right`, the polar opposite `west` is used. For `oval` the polar opposite `north` is assumed, because the placement is `below`.

![Relative placement](tests/visual/hello-direction_start_end.png)

The first relative component determines direction, `circle 1 right 1 down` will be horizontal, and `oval 1 down` will be vertical.

### Containers

Containers are always positioned relative to their top-left corner, because their size is determined by the content and thus not known in advance.

### Alignment

The position of captions on lines and arrows is determined by the alignment suffix, like `above` and `below` for horizontal lines.

![Above and below](tests/doc/align-above_below.png)

For vertical lines the alignment is `left` and `right`. The default is `center`.

![Left and right](tests/doc/align-left_right.png)

### Edges

The edges of a block object can be identified in four ways, each offering more granularity:
- 4 directions, `up`, `down`, `left`, and `right`
- 8 compass points `n`, `ne`, `e`, `se`, `s`, `sw`, `w`, and `nw`
- 12 clock hours `1:` - `12:`
- 360 degree angles `0` - `360`

In combination with the block `id`, they indicate exact positions on the block's edge.

The four directions indicate the midpoints on a block object, named `up`, `down`, `left`, and `right`. Compass points also add positions for the corners `nw`, `n`, `ne`, `e`, `se`, `s`, `sw`, and `w`.

![Degree angles](tests/doc/edges-all_edges.png)

Clock hours give you 12 positions on the edge of a block, and might be more natural. The `:` suffix is used to distinguish them from degree angles. Degree angles divide the edge in 360 parts, with 0 pointing up.

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
13. ~dot at edge~
14. ~Automatisch topleft bepalen~
15. ~Renderer uitsplitsen~
16. ~Index als type met #last~
17. ~Circle shape~
18. ~Ellipse shape~
19. ~Cylinder shape~
20. ~Oval shape~
21. ~Units met decimalen~

22. File shape
23. Tekstgrootte zonder canvas bepalen
24. Arc
25. Automatisch grootte bepalen
26. `arrow` met offset
27. `nnw` en uren op de klok, met horizontal en vertical
28. Richting binnen container, zoals flex

De topleft wordt het nieuwe centrum voor een rechter rij, waardoor er overlap ontstaat. De oplossing is om de container te positioneren nadat de inhoud is bepaald en de grootte bekend is.

Daarvoor moet de inhoud van de container relatief gepositioneerd worden.

Dat heeft wel gevolgen voor lookups, omdat de positie niet meer absoluut bekend is.

Een tweede pass lost het probleem niet op, omdat hier element relatief geplaatst kunnen zijn die weer gevolgen hebben voor andere elementen.

Een tweede pass kan werken als alleen wordt verwezen naar items die al hun absolute positie hebben.

Het makkelijkste is misschien om top alignment te houden.

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

## Setup

```sh
cargo add skia-safe --features metal
```

## Drawing

How to [draw an arrow](https://stackoverflow.com/questions/72714333/flutter-how-do-i-make-arrow-lines-with-canvas) with
Rust [trigonometry](https://rust-lang-nursery.github.io/rust-cookbook/science/mathematics/trigonometry.html).

### Timeline

https://speakerdeck.com/nihonbuson/agile-testinghaxin-siigai-nian-nanoka-pin-zhi-bao-zheng-noli-shi-wota-maetekao-eru-number-scrumniigata?slide=28