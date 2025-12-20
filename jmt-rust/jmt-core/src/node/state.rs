//! State node - represents a state in the state machine

use serde::{Deserialize, Serialize};
use uuid::Uuid;
use crate::geometry::{Color, Rect};
use super::region::Region;
use super::NodeId;

/// A state in the state machine
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct State {
    /// Unique identifier
    pub id: NodeId,
    /// Display name
    pub name: String,
    /// Bounding rectangle
    pub bounds: Rect,
    /// Optional fill color (uses default if None)
    pub fill_color: Option<Color>,
    /// Parent region ID (None for root state)
    pub parent_region_id: Option<Uuid>,
    /// Entry activity code
    pub entry_activity: String,
    /// Exit activity code
    pub exit_activity: String,
    /// Do activity code
    pub do_activity: String,
    /// Regions for composite states
    pub regions: Vec<Region>,
    /// Whether this state is currently selected
    #[serde(skip)]
    pub has_focus: bool,
}

impl State {
    /// Create a new state with default values
    pub fn new(name: &str, x: f32, y: f32, width: f32, height: f32) -> Self {
        Self {
            id: Uuid::new_v4(),
            name: name.to_string(),
            bounds: Rect::from_pos_size(x, y, width, height),
            fill_color: None,
            parent_region_id: None,
            entry_activity: String::new(),
            exit_activity: String::new(),
            do_activity: String::new(),
            regions: Vec::new(),
            has_focus: false,
        }
    }

    /// Create a new state with a specific ID
    pub fn with_id(id: NodeId, name: &str, x: f32, y: f32, width: f32, height: f32) -> Self {
        Self {
            id,
            name: name.to_string(),
            bounds: Rect::from_pos_size(x, y, width, height),
            fill_color: None,
            parent_region_id: None,
            entry_activity: String::new(),
            exit_activity: String::new(),
            do_activity: String::new(),
            regions: Vec::new(),
            has_focus: false,
        }
    }

    /// Returns true if this is a composite state (has regions with children)
    pub fn is_composite(&self) -> bool {
        !self.regions.is_empty()
    }

    /// Add a new region to this state
    pub fn add_region(&mut self, name: &str) -> Uuid {
        let region = Region::new(name, &self.bounds);
        let id = region.id;
        self.regions.push(region);
        self.recalculate_regions();
        id
    }

    /// Get the first (default) region
    pub fn first_region(&self) -> Option<&Region> {
        self.regions.first()
    }

    /// Get a mutable reference to the first region
    pub fn first_region_mut(&mut self) -> Option<&mut Region> {
        self.regions.first_mut()
    }

    /// Find a region by ID
    pub fn find_region(&self, id: Uuid) -> Option<&Region> {
        self.regions.iter().find(|r| r.id == id)
    }

    /// Find a mutable region by ID
    pub fn find_region_mut(&mut self, id: Uuid) -> Option<&mut Region> {
        self.regions.iter_mut().find(|r| r.id == id)
    }

    /// Recalculate region bounds based on state bounds
    pub fn recalculate_regions(&mut self) {
        if self.regions.is_empty() {
            return;
        }

        let count = self.regions.len() as f32;
        let header_height = 25.0; // Space for state name
        let available_height = self.bounds.height() - header_height;
        let region_height = available_height / count;

        for (i, region) in self.regions.iter_mut().enumerate() {
            region.bounds = Rect::new(
                self.bounds.x1,
                self.bounds.y1 + header_height + (i as f32 * region_height),
                self.bounds.x2,
                self.bounds.y1 + header_height + ((i + 1) as f32 * region_height),
            );
        }
    }

    /// Check if this state has any activities defined
    pub fn has_activities(&self) -> bool {
        !self.entry_activity.is_empty()
            || !self.exit_activity.is_empty()
            || !self.do_activity.is_empty()
    }

    /// Get the header height (area for name and activities)
    pub fn header_height(&self) -> f32 {
        if self.has_activities() {
            40.0
        } else {
            25.0
        }
    }
}

impl Default for State {
    fn default() -> Self {
        Self::new("State", 0.0, 0.0, 100.0, 60.0)
    }
}
