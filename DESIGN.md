## Shapes

The shapes simplify finding the intersection from the center of a shape to the edge of a shape at a certain angle, so that lines can be drawn from any point of a shape to any other point.

Rectangles have four edges: top, bottom, left, and right. Diamonds also have four edges, all diagonal. A triangle has three: left, right, and bottom. 

Cylinders are a composite of two edges, left and right, and one and a half ellipses, top and bottom. Ovals are just the outline of a rotated cylinder.

### Edges

For straight edges, `intersect_factor` returns an interpolation factor somewhere on the line from the center of the shape to the edge of its bounding box where it crosses the edge of the shape.

Composite shapes have some straight edges, but also curved edges, for example the cylinder. The cylinder has two straight edges, left and right, and two curved edges, top and bottom. The top and bottom are ellipses.

Rectangles with rounded corners are also composite shapes, as the corners are curved.
