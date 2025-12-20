//! Geometry primitives: Point, Rect, Color

use serde::{Deserialize, Serialize};

/// A 2D point with floating-point coordinates
#[derive(Debug, Clone, Copy, PartialEq, Default, Serialize, Deserialize)]
pub struct Point {
    pub x: f32,
    pub y: f32,
}

impl Point {
    pub const ZERO: Point = Point { x: 0.0, y: 0.0 };

    pub fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }

    pub fn distance_to(&self, other: Point) -> f32 {
        let dx = self.x - other.x;
        let dy = self.y - other.y;
        (dx * dx + dy * dy).sqrt()
    }
}

impl From<(f32, f32)> for Point {
    fn from((x, y): (f32, f32)) -> Self {
        Self { x, y }
    }
}

/// A rectangle defined by two corner points
#[derive(Debug, Clone, Copy, PartialEq, Default, Serialize, Deserialize)]
pub struct Rect {
    pub x1: f32,
    pub y1: f32,
    pub x2: f32,
    pub y2: f32,
}

impl Rect {
    pub fn new(x1: f32, y1: f32, x2: f32, y2: f32) -> Self {
        Self { x1, y1, x2, y2 }
    }

    pub fn from_pos_size(x: f32, y: f32, width: f32, height: f32) -> Self {
        Self {
            x1: x,
            y1: y,
            x2: x + width,
            y2: y + height,
        }
    }

    pub fn width(&self) -> f32 {
        self.x2 - self.x1
    }

    pub fn height(&self) -> f32 {
        self.y2 - self.y1
    }

    pub fn center(&self) -> Point {
        Point {
            x: (self.x1 + self.x2) / 2.0,
            y: (self.y1 + self.y2) / 2.0,
        }
    }

    pub fn top_left(&self) -> Point {
        Point { x: self.x1, y: self.y1 }
    }

    pub fn top_right(&self) -> Point {
        Point { x: self.x2, y: self.y1 }
    }

    pub fn bottom_left(&self) -> Point {
        Point { x: self.x1, y: self.y2 }
    }

    pub fn bottom_right(&self) -> Point {
        Point { x: self.x2, y: self.y2 }
    }

    /// Check if a point is inside this rectangle
    pub fn contains_point(&self, p: Point) -> bool {
        p.x >= self.x1 && p.x <= self.x2 && p.y >= self.y1 && p.y <= self.y2
    }

    /// Check if another rectangle is fully contained within this one
    pub fn contains_rect(&self, other: &Rect) -> bool {
        other.x1 >= self.x1 && other.x2 <= self.x2 && other.y1 >= self.y1 && other.y2 <= self.y2
    }

    /// Check if this rectangle overlaps with another
    pub fn overlaps(&self, other: &Rect) -> bool {
        self.x1 < other.x2 && self.x2 > other.x1 && self.y1 < other.y2 && self.y2 > other.y1
    }

    /// Expand the rectangle by a margin on all sides
    pub fn expand(&self, margin: f32) -> Rect {
        Rect {
            x1: self.x1 - margin,
            y1: self.y1 - margin,
            x2: self.x2 + margin,
            y2: self.y2 + margin,
        }
    }

    /// Move the rectangle by an offset
    pub fn translate(&self, dx: f32, dy: f32) -> Rect {
        Rect {
            x1: self.x1 + dx,
            y1: self.y1 + dy,
            x2: self.x2 + dx,
            y2: self.y2 + dy,
        }
    }
}

/// RGBA color
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl Color {
    pub const BLACK: Color = Color { r: 0, g: 0, b: 0, a: 255 };
    pub const WHITE: Color = Color { r: 255, g: 255, b: 255, a: 255 };
    pub const GRAY: Color = Color { r: 128, g: 128, b: 128, a: 255 };
    pub const ORANGE: Color = Color { r: 255, g: 165, b: 0, a: 255 };
    pub const BLUE: Color = Color { r: 0, g: 0, b: 255, a: 255 };

    /// Default state fill color (light yellow: #FFFFCC)
    pub const STATE_FILL: Color = Color { r: 255, g: 255, b: 204, a: 255 };

    pub fn new(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b, a: 255 }
    }

    pub fn with_alpha(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self { r, g, b, a }
    }

    pub fn from_hex(hex: u32) -> Self {
        Self {
            r: ((hex >> 16) & 0xFF) as u8,
            g: ((hex >> 8) & 0xFF) as u8,
            b: (hex & 0xFF) as u8,
            a: 255,
        }
    }

    pub fn to_rgba_u32(&self) -> u32 {
        ((self.r as u32) << 24) | ((self.g as u32) << 16) | ((self.b as u32) << 8) | (self.a as u32)
    }
}

impl Default for Color {
    fn default() -> Self {
        Self::BLACK
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_point_distance() {
        let p1 = Point::new(0.0, 0.0);
        let p2 = Point::new(3.0, 4.0);
        assert!((p1.distance_to(p2) - 5.0).abs() < 0.001);
    }

    #[test]
    fn test_rect_contains_point() {
        let rect = Rect::new(10.0, 10.0, 100.0, 100.0);
        assert!(rect.contains_point(Point::new(50.0, 50.0)));
        assert!(!rect.contains_point(Point::new(5.0, 50.0)));
    }

    #[test]
    fn test_rect_overlaps() {
        let r1 = Rect::new(0.0, 0.0, 50.0, 50.0);
        let r2 = Rect::new(25.0, 25.0, 75.0, 75.0);
        let r3 = Rect::new(100.0, 100.0, 150.0, 150.0);
        assert!(r1.overlaps(&r2));
        assert!(!r1.overlaps(&r3));
    }
}
