//! ActivityPartition - represents an activity partition (swimlane container)

use serde::{Deserialize, Serialize};
use uuid::Uuid;
use crate::geometry::{Point, Rect, Color};
use super::swimlane::{Swimlane, SwimlaneOrientation};

/// Unique identifier for an activity partition
pub type ActivityPartitionId = Uuid;

/// An activity partition containing multiple swimlanes
/// Acts as a container that organizes swimlanes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActivityPartition {
    /// Unique identifier
    pub id: ActivityPartitionId,
    /// Name of the partition (optional, for 2D partitions)
    pub name: Option<String>,
    /// Bounding rectangle
    pub bounds: Rect,
    /// Orientation of swimlanes in this partition
    pub orientation: SwimlaneOrientation,
    /// Swimlanes in this partition
    pub swimlanes: Vec<Swimlane>,
    /// Whether this is a dimension (for 2D partitions)
    pub is_dimension: bool,
    /// Optional fill color
    pub fill_color: Option<Color>,
    /// Whether this partition is currently selected
    #[serde(skip)]
    pub has_focus: bool,
}

impl ActivityPartition {
    /// Create a new partition with vertical swimlanes
    pub fn new(x: f32, y: f32, width: f32, height: f32) -> Self {
        Self {
            id: Uuid::new_v4(),
            name: None,
            bounds: Rect::from_pos_size(x, y, width, height),
            orientation: SwimlaneOrientation::Vertical,
            swimlanes: Vec::new(),
            is_dimension: false,
            fill_color: None,
            has_focus: false,
        }
    }

    /// Create a partition with predefined swimlanes
    pub fn with_swimlanes(names: &[&str], x: f32, y: f32, lane_width: f32, height: f32) -> Self {
        let total_width = lane_width * names.len() as f32;
        let mut partition = Self::new(x, y, total_width, height);

        for (i, name) in names.iter().enumerate() {
            let lane_x = x + i as f32 * lane_width;
            let mut swimlane = Swimlane::new(name, lane_x, y, lane_width, height);
            swimlane.order = i as u32;
            partition.swimlanes.push(swimlane);
        }

        partition
    }

    /// Add a swimlane to the partition
    pub fn add_swimlane(&mut self, name: &str) -> Uuid {
        let lane_count = self.swimlanes.len();
        let lane_width = if lane_count > 0 {
            self.bounds.width() / (lane_count + 1) as f32
        } else {
            self.bounds.width()
        };

        // Resize existing swimlanes
        for (i, swimlane) in self.swimlanes.iter_mut().enumerate() {
            swimlane.bounds.x1 = self.bounds.x1 + i as f32 * lane_width;
            swimlane.bounds.x2 = swimlane.bounds.x1 + lane_width;
        }

        // Add new swimlane
        let lane_x = self.bounds.x1 + lane_count as f32 * lane_width;
        let mut swimlane = Swimlane::new(name, lane_x, self.bounds.y1, lane_width, self.bounds.height());
        swimlane.order = lane_count as u32;
        let id = swimlane.id;
        self.swimlanes.push(swimlane);
        id
    }

    /// Find swimlane at a given point
    pub fn find_swimlane_at(&self, p: Point) -> Option<&Swimlane> {
        self.swimlanes.iter().find(|s| s.contains_point(p))
    }

    /// Find mutable swimlane at a given point
    pub fn find_swimlane_at_mut(&mut self, p: Point) -> Option<&mut Swimlane> {
        self.swimlanes.iter_mut().find(|s| s.contains_point(p))
    }

    /// Get swimlane by ID
    pub fn find_swimlane(&self, id: Uuid) -> Option<&Swimlane> {
        self.swimlanes.iter().find(|s| s.id == id)
    }

    /// Get mutable swimlane by ID
    pub fn find_swimlane_mut(&mut self, id: Uuid) -> Option<&mut Swimlane> {
        self.swimlanes.iter_mut().find(|s| s.id == id)
    }

    /// Check if a point is inside the partition
    pub fn contains_point(&self, p: Point) -> bool {
        self.bounds.contains_point(p)
    }

    /// Translate the partition and all its swimlanes
    pub fn translate(&mut self, dx: f32, dy: f32) {
        self.bounds = self.bounds.translate(dx, dy);
        for swimlane in &mut self.swimlanes {
            swimlane.translate(dx, dy);
        }
    }

    /// Recalculate swimlane bounds to fill the partition
    pub fn recalculate_swimlanes(&mut self) {
        if self.swimlanes.is_empty() {
            return;
        }

        let lane_count = self.swimlanes.len() as f32;

        match self.orientation {
            SwimlaneOrientation::Vertical => {
                let lane_width = self.bounds.width() / lane_count;
                for (i, swimlane) in self.swimlanes.iter_mut().enumerate() {
                    swimlane.bounds = Rect::new(
                        self.bounds.x1 + i as f32 * lane_width,
                        self.bounds.y1,
                        self.bounds.x1 + (i + 1) as f32 * lane_width,
                        self.bounds.y2,
                    );
                    swimlane.orientation = SwimlaneOrientation::Vertical;
                }
            }
            SwimlaneOrientation::Horizontal => {
                let lane_height = self.bounds.height() / lane_count;
                for (i, swimlane) in self.swimlanes.iter_mut().enumerate() {
                    swimlane.bounds = Rect::new(
                        self.bounds.x1,
                        self.bounds.y1 + i as f32 * lane_height,
                        self.bounds.x2,
                        self.bounds.y1 + (i + 1) as f32 * lane_height,
                    );
                    swimlane.orientation = SwimlaneOrientation::Horizontal;
                }
            }
        }
    }
}

impl Default for ActivityPartition {
    fn default() -> Self {
        Self::new(50.0, 50.0, 600.0, 400.0)
    }
}
