## Shapes

The shapes simplify finding the intersection from the center of a shape to the edge of a shape at a certain angle, so that lines can be drawn from any point of a shape to any other point.

Rectangles have four edges: top, bottom, left, and right. Diamonds also have four edges, all diagonal. A triangle has three: left, right, and bottom. 

Cylinders are a composite of two edges, left and right, and one and a half ellipses, top and bottom. Ovals are just the outline of a rotated cylinder.

### Edges

For straight edges, `intersect_factor` returns an interpolation factor somewhere on the line from the center of the shape to the edge of its bounding box where it crosses the edge of the shape.

Composite shapes have some straight edges, but also curved edges, for example the cylinder. The cylinder has two straight edges, left and right, and two curved edges, top and bottom. The top and bottom are ellipses.

Rectangles with rounded corners are also composite shapes, as the corners are curved.

## DRY

The `same` attribute tries to cut down on duplicating the same attributes over and over.

```
box.pic1 ht 2in wd 1in "Primary Interrupt Controller"
line from 1/8 pic1.w 1.5in left "Timer" ljust opaque ->
line from 2/8 pic1.w same "Keyboard"

box.pic2 same "Primary Interrupt Controller"
```

In this example, the second `line` gets the `1.5in left` movement and `ljust opaque ->` from the first `line`. The `same` attributes only takes values for empty `Option`s