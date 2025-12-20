//! JMT Proto - Protobuf definitions for client-server communication

// Include the generated protobuf code
include!("jmt.rs");

// Re-export commonly used types
pub use self::command::Command as CommandType;
pub use self::response::Response as ResponseType;

use jmt_core::geometry::{Color as CoreColor, Point as CorePoint, Rect as CoreRect};
use jmt_core::node::Side as CoreSide;

// Conversion implementations between protobuf and core types

impl From<CorePoint> for Point {
    fn from(p: CorePoint) -> Self {
        Point { x: p.x, y: p.y }
    }
}

impl From<Point> for CorePoint {
    fn from(p: Point) -> Self {
        CorePoint { x: p.x, y: p.y }
    }
}

impl From<CoreRect> for Rect {
    fn from(r: CoreRect) -> Self {
        Rect {
            x1: r.x1,
            y1: r.y1,
            x2: r.x2,
            y2: r.y2,
        }
    }
}

impl From<Rect> for CoreRect {
    fn from(r: Rect) -> Self {
        CoreRect {
            x1: r.x1,
            y1: r.y1,
            x2: r.x2,
            y2: r.y2,
        }
    }
}

impl From<CoreColor> for Color {
    fn from(c: CoreColor) -> Self {
        Color {
            r: c.r as u32,
            g: c.g as u32,
            b: c.b as u32,
            a: c.a as u32,
        }
    }
}

impl From<Color> for CoreColor {
    fn from(c: Color) -> Self {
        CoreColor {
            r: c.r as u8,
            g: c.g as u8,
            b: c.b as u8,
            a: c.a as u8,
        }
    }
}

impl From<CoreSide> for Side {
    fn from(s: CoreSide) -> Self {
        match s {
            CoreSide::None => Side::None,
            CoreSide::Top => Side::Top,
            CoreSide::Bottom => Side::Bottom,
            CoreSide::Left => Side::Left,
            CoreSide::Right => Side::Right,
        }
    }
}

impl From<Side> for CoreSide {
    fn from(s: Side) -> Self {
        match s {
            Side::None => CoreSide::None,
            Side::Top => CoreSide::Top,
            Side::Bottom => CoreSide::Bottom,
            Side::Left => CoreSide::Left,
            Side::Right => CoreSide::Right,
        }
    }
}
