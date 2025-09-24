
## Size

The size of the canvas is derived automatically from the bounds of all the shapes on it, plus the `inset`. You can also force the canvas to a certain size with the `canvas` directive. 

```pic
canvas 0.75x0.5in
grid
```

![](snapshots/placement-sized.png)

The `grid` directive divides the canvas in cells relative from the `inset`. It will be omitted from further examples.

## Placement

The origin for each shape is its center. 

![](snapshots/placement-grid_center.png)

Captions are also centered. Both in closed and open shapes.

When you add more shapes, they continue in a `flow` direction. The default flows right from the `center` of the last shape.

```pic
line
box "Hello"
arrow
```
![](snapshots/placement-flow_right.png)

