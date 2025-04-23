use skia_safe::{Point, Rect};
use crate::diagram::types::Displacement;

pub(crate) struct Bounds;

impl Bounds {

  /// Adjust bounds so that rect fits in it
  pub(crate) fn bounds_from_rect(bounds: &mut Rect, rect: Rect) {
    bounds.top = bounds.top.min(rect.top);
    bounds.left = bounds.left.min(rect.left);
    bounds.right = bounds.right.max(rect.right);
    bounds.bottom = bounds.bottom.max(rect.bottom);
  }

  pub(crate) fn bounds_from_points(points: &[Point]) -> Rect {
    let mut iter = points.iter();
    let first = iter.next().unwrap();
    let mut used = Rect::from_point_and_size(*first, (0, 0));
    for point in iter {
      Bounds::bounds_from_point(&mut used, point);
    }
    used
  }

  pub(crate) fn bounds_from_point(bounds: &mut Rect, point: &Point) {
    bounds.top = bounds.top.min(point.y);
    bounds.bottom = bounds.bottom.max(point.y);
    bounds.left = bounds.left.min(point.x);
    bounds.right = bounds.right.max(point.x);
  }

  pub(crate) fn rect_from_points(start: Point, displacement: &Option<Displacement>, end: Point) -> (Rect, Rect) {
    let rect = Rect { left: start.x, top: start.y, right: end.x, bottom: end.y };
    let mut used = rect;
    if let Some(movement) = &displacement {
      used.offset(movement.offset());
    }
    (rect, used)
  }

}
