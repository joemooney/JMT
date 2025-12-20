//! Activation - represents an activation box on a lifeline

use serde::{Deserialize, Serialize};
use uuid::Uuid;
use crate::geometry::Rect;
use super::lifeline::LifelineId;

/// Unique identifier for an activation
pub type ActivationId = Uuid;

/// An activation box on a lifeline
/// Shows the period during which an object is performing an action
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Activation {
    /// Unique identifier
    pub id: ActivationId,
    /// The lifeline this activation belongs to
    pub lifeline_id: LifelineId,
    /// Y position of the top of the activation box
    pub start_y: f32,
    /// Y position of the bottom of the activation box
    pub end_y: f32,
    /// Width of the activation box
    pub width: f32,
    /// Nesting level (for stacked activations)
    pub nesting_level: u32,
    /// Whether this activation is currently selected
    #[serde(skip)]
    pub selected: bool,
}

impl Activation {
    /// Create a new activation
    pub fn new(lifeline_id: LifelineId, start_y: f32, end_y: f32) -> Self {
        Self {
            id: Uuid::new_v4(),
            lifeline_id,
            start_y,
            end_y,
            width: 10.0,
            nesting_level: 0,
            selected: false,
        }
    }

    /// Get the bounds of this activation box given the lifeline X position
    pub fn bounds(&self, lifeline_x: f32) -> Rect {
        // Offset based on nesting level
        let offset = self.nesting_level as f32 * 4.0;
        Rect::new(
            lifeline_x - self.width / 2.0 + offset,
            self.start_y,
            lifeline_x + self.width / 2.0 + offset,
            self.end_y,
        )
    }

    /// Get the height of the activation
    pub fn height(&self) -> f32 {
        self.end_y - self.start_y
    }

    /// Extend the activation to a new end Y
    pub fn extend_to(&mut self, new_end_y: f32) {
        if new_end_y > self.end_y {
            self.end_y = new_end_y;
        }
    }
}

impl Default for Activation {
    fn default() -> Self {
        Self::new(Uuid::nil(), 100.0, 150.0)
    }
}
