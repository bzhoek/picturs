use skia_safe::{Point, Rect, scalar};

#[cfg(test)]
mod tests {
    use std::ops::{Add, Sub};

    use skia_safe::{Color, PaintStyle};

    use picturs::diagram::edges::{Edge, EdgeFinder};
    use picturs::skia::Canvas;
    use picturs::test::assert_canvas;
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
        assert_eq!(Edge::new((10, 10), (10, 0)), angle);
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
        assert_eq!(4, shape.edges.len());
        assert_eq!(Edge::new((0, 0), (20, 0)), shape.edges[0]);
    }

    #[test]
    fn rectangle_intersects() {
        let shape = EdgeFinder::rectangle(0., 0., 20., 20.);

        let angle = angle_from(shape.bounds.center(), 0., 11.);
        let intersect = angle.intersects(&shape.edges[0]);
        assert_eq!(Some(Point::new(10., -0.)), round(intersect));

        let intersect = angle.intersects(&shape.edges[1]);
        assert_eq!(None, intersect);

        let angle = angle_from(shape.bounds.center(), 90., 11.);
        let intersect = angle.intersects(&shape.edges[1]);
        assert_eq!(Some(Point::new(20., 10.)), round(intersect));
    }

    fn round(point: Option<Point>) -> Option<Point> {
        point.map(|p| Point::new(p.x.round(), p.y.round()))
    }

    #[test]
    fn triangle() {
        let shape = EdgeFinder::triangle(0., 0., 20., 20.);
        assert_eq!(3, shape.edges.len());
        assert_eq!(Edge::new((10, 0), (20, 20)), shape.edges[0]);
    }

    #[test]
    fn triangle_intersects() {
        let shape = EdgeFinder::triangle(0., 0., 20., 20.);

        let angle = angle_from(shape.bounds.center(), 45., 10.);
        let intersect = angle.intersects(&shape.edges[0]);
        assert_eq!(Some(Point::new(13., 7.)), round(intersect));
    }

    #[test]
    fn triangle_intersect() {
        let shape = EdgeFinder::triangle(0., 0., 20., 20.);
        let intersect = shape.intersect(45.);
        assert_eq!(Some(Point::new(13., 7.)), round(intersect));
    }

    #[test]
    fn diamond() {
        let shape = EdgeFinder::diamond(0., 0., 20., 20.);
        assert_eq!(4, shape.edges.len());
        assert_eq!(Edge::new((10, 0), (20, 10)), shape.edges[0]);
    }

    #[test]
    fn file() {
        let shape = EdgeFinder::file(0., 0., 20., 20., 5.);
        assert_eq!(5, shape.edges.len());
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
    fn arc_intersect() {
        let rect = Rect::from_xywh(8., 8., 40., 48.);
        let shape = EdgeFinder::cylinder(8., 8., 40., 48.);
        let top = Rect::from_xywh(rect.left, rect.top, rect.width(), rect.height() / 3.);
        let bottom = Rect::from_xywh(
            rect.left,
            rect.bottom - top.height(),
            rect.width(),
            top.height(),
        );

        let line = Edge::new(rect.center(), (rect.right + 1., rect.center_y()));

        let mut canvas = prepare_cylinder_canvas(&rect, &top, &bottom, &line);

        let result = if in_arc_range(&top, &line) {
            intersect_ellipse(&top, &line)
        } else if in_arc_range(&bottom, &line) {
            intersect_ellipse(&bottom, &line)
        } else {
            let intersect = shape.intersect(275.);
            if let Some(i) = intersect {
                canvas.paint.set_color(Color::GREEN);
                canvas.circle(&i, 1.);
            }
            None
        };

        if let Some((t1, t2)) = result {
            let i1 = line.interpolate(t1);
            let i2 = line.interpolate(t2);

            canvas.paint.set_color(Color::RED);
            canvas.circle(&i1, 1.);
            canvas.circle(&i2, 1.);
        }

        assert_canvas(canvas, "target/ellipse_intersect").unwrap();
    }

    fn prepare_cylinder_canvas(rect: &Rect, top: &Rect, bottom: &Rect, line: &Edge) -> Canvas {
        let mut canvas = Canvas::new((56, 64));

        canvas.paint.set_style(PaintStyle::Stroke);

        canvas.paint.set_color(Color::BLUE);
        canvas.rectangle(&rect, 0.);

        canvas.paint.set_color(Color::WHITE);
        canvas.circle(&line.from, 1.);
        canvas.ellipse(&top);
        canvas.ellipse(&bottom);
        canvas
    }

    #[allow(non_snake_case)]
    fn intersect_ellipse(ellipse: &Rect, line: &Edge) -> Option<(scalar, scalar)> {
        let e = ellipse.center();
        let w = ellipse.width() / 2.;
        let h = ellipse.height() / 2.;
        let p1 = line.from.sub(e);
        let p2 = line.to.sub(e);

        let d = p2.sub(p1);
        let A = d.x * d.x / w / w + d.y * d.y / h / h;
        let B = 2. * p1.x * (d.x) / w / w + 2. * p1.y * (d.y) / h / h;
        let C = p1.x * p1.x / w / w + p1.y * p1.y / h / h - 1.;
        let D = B * B - 4. * A * C;
        if D == 0. {
        } else if D > 0. {
            let t1 = (-B - D.sqrt()) / (2. * A);
            let t2 = (-B + D.sqrt()) / (2. * A);
            return Some((t1, t2));
        }
        None
    }

    fn in_arc_range(rect: &Rect, line: &Edge) -> bool {
        let mid_left = Point::new(rect.left, rect.center_y());
        let mid_right = Point::new(rect.right, rect.center_y());
        let left_angle = line.angle_to(&mid_left);
        let right_angle = line.angle_to(&mid_right);
        let line_angle = line.angle();
        return if line_angle < 0. {
            line_angle < right_angle && line_angle > left_angle
        } else {
            line_angle > right_angle && line_angle < left_angle
        };
    }
}
