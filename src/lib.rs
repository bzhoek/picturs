use std::sync::{Mutex, OnceLock};

use env_logger::Env;
use skia_safe::Rect;

pub mod skia;
pub mod diagram;
pub mod test;

pub mod pic;

pub mod trig;

#[allow(dead_code)]
fn debug_rect(used: &Rect) -> String {
  format!("x: {} y: {}, w: {}, h: {}", used.x(), used.y(), used.width(), used.height())
}

pub fn init_logging() -> &'static Mutex<()> {
  static LOGGER: OnceLock<Mutex<()>> = OnceLock::new();
  LOGGER.get_or_init(|| {
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();
    ().into()
  })
}
