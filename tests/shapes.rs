use skia_safe::{Point, Rect};

#[cfg(test)]
mod shapes {
  use std::ops::Add;

  use skia_safe::{Color, PaintStyle};
  use picturs::assert_canvas;

  use picturs::diagram::edges::{Edge, EdgeFinder};
  use picturs::skia::Canvas;
  use picturs::test::test_canvas;
  use picturs::trig::angle_at;

  use super::*;

  fn angle_from(from: impl Into<Point>, angle: f32, length: f32) -> Edge {
    let from = from.into();
    let offset: Point = angle_at(angle, length).into();
    Edge::new(from, from.add(offset))
  }

  #[test]
  fn angle() {
    let angle = angle_from((10., 10.), 0., 10.);
    assert_eq!(angle, Edge::new((10, 10), (10, 0)))
  }

  #[test]
  fn intersect() {
    let edge = Edge::new((-10, -10), (5, -10));
    let angle = angle_from((0., 0.), 25., 40.);
    let intersect = angle.intersects(&edge);
    assert_eq!(round(intersect), Some(Point::new(5., -10.)));
  }

  #[test]
  fn rectangle() {
    let shape = EdgeFinder::rectangle(0., 0., 20., 20.);
    assert_eq!(shape.edges.len(), 4);
    assert_eq!(shape.edges[0], Edge::new((0, 0), (20, 0)))
  }

  #[test]
  fn rectangle_intersects() {
    let shape = EdgeFinder::rectangle(0., 0., 20., 20.);

    let angle = angle_from(shape.bounds.center(), 0., 11.);
    let intersect = angle.intersects(&shape.edges[0]);
    assert_eq!(round(intersect), Some(Point::new(10., -0.)));

    let intersect = angle.intersects(&shape.edges[1]);
    assert_eq!(intersect, None);

    let angle = angle_from(shape.bounds.center(), 90., 11.);
    let intersect = angle.intersects(&shape.edges[1]);
    assert_eq!(round(intersect), Some(Point::new(20., 10.)))
  }

  fn round(point: Option<Point>) -> Option<Point> {
    point.map(|p| Point::new(p.x.round(), p.y.round()))
  }

  #[test]
  fn triangle() {
    let shape = EdgeFinder::triangle(0., 0., 20., 20.);
    assert_eq!(shape.edges.len(), 3);
    assert_eq!(shape.edges[0], Edge::new((10, 0), (20, 20)))
  }

  #[test]
  fn triangle_intersects() {
    let shape = EdgeFinder::triangle(0., 0., 20., 20.);

    let angle = angle_from(shape.bounds.center(), 45., 10.);
    let intersect = angle.intersects(&shape.edges[0]);
    assert_eq!(round(intersect), Some(Point::new(13., 7.)))
  }

  #[test]
  fn triangle_intersect() {
    let shape = EdgeFinder::triangle(0., 0., 20., 20.);
    let intersect = shape.intersect(45.);
    assert_eq!(round(intersect), Some(Point::new(13., 7.)))
  }

  #[test]
  fn diamond() {
    let shape = EdgeFinder::diamond(0., 0., 20., 20.);
    assert_eq!(shape.edges.len(), 4);
    assert_eq!(shape.edges[0], Edge::new((10, 0), (20, 10)))
  }

  #[test]
  fn file() {
    let shape = EdgeFinder::file(0., 0., 20., 20., 5.);
    assert_eq!(shape.edges.len(), 5);
    assert_eq!(shape.edges[0], Edge::new((0, 0), (15, 0)));
    assert_eq!(shape.edges[1], Edge::new((15, 0), (20, 5)));
  }

  #[test]
  fn file_intersect() {
    let shape = EdgeFinder::file(-10., -10., 20., 20., 5.);
    assert_eq!(shape.edges[0], Edge::new((-10, -10), (5, -10)));

    let angle = angle_from(shape.bounds.center(), 45., 20.);
    let intersect = angle.intersects(&shape.edges[0]);
    assert_eq!(round(intersect), None);

    let intersect = shape.intersect(45.);
    assert_eq!(round(intersect), Some(Point::new(7., -8.)));
  }

  // http://www.csharphelper.com/howtos/howto_line_ellipse_intersection.html
  // https://github.com/davidfig/intersects/blob/master/ellipse-line.js
  #[test]
  fn ellipse_intersect() {
    let shape = EdgeFinder::cylinder(8., 8., 40., 48.);

    let lines = vec![
      shape.center_to_edge(45.),
      shape.center_to_edge(225.),
      shape.center_to_edge(275.),
    ];

    let mut canvas = prepare_cylinder_canvas(&shape.bounds, &shape.ellipses, &lines.first().unwrap());

    lines.iter().for_each(|line| {
      let intersect = shape.intersects(line);
      if let Some(i) = intersect {
        canvas.paint.set_color(Color::GREEN);
        canvas.circle(&i, 1.);
      }
    });

    assert_canvas!(canvas);
  }

  fn prepare_cylinder_canvas(rect: &Rect, top: &Vec<Rect>, line: &Edge) -> Canvas {
    let mut canvas = test_canvas((56, 64));

    canvas.paint.set_style(PaintStyle::Stroke);

    canvas.paint.set_color(Color::BLUE);
    canvas.rectangle(&rect, 0.);

    canvas.paint.set_color(Color::WHITE);
    canvas.circle(&line.from, 1.);
    top.iter().for_each(|ellipse|canvas.ellipse(ellipse));
    canvas
  }
}
