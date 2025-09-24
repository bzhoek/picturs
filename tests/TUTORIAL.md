
## Size

The size of the canvas is derived automatically from the bounds of all the shapes on it, plus the `inset`. You can also force the canvas to a certain size with the `canvas` directive.

```pic
canvas wd 0.75in ht 0.5in
grid
```

![](snapshots/placement-size.png)

## Placement

The origin for each shape is its center. 

![](snapshots/placement-grid_center.png)

Captions are also centered. Both in closed and open shapes.