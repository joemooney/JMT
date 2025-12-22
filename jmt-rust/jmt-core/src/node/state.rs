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
    /// Whether to show activities in this state (None = use diagram default)
    #[serde(default)]
    pub show_activities: Option<bool>,
    /// Display title for the state (shown in title bar, separate from name)
    #[serde(default)]
    pub title: String,
    /// Path to sub-statemachine file, or empty string if embedded, or None if no sub-statemachine
    /// - Some("path/to/file.jmt") = external file
    /// - Some("") = embedded sub-statemachine (stored in regions)
    /// - None = no sub-statemachine
    #[serde(default)]
    pub substatemachine_path: Option<String>,
    /// Whether to show the sub-statemachine expanded inline (true) or as icon (false)
    #[serde(default)]
    pub show_expanded: bool,
    /// Whether this state is currently selected
    #[serde(skip)]
    pub has_focus: bool,
    /// Whether this node has a placement error (partially inside another node)
    #[serde(skip)]
    pub has_error: bool,
    /// Whether this node has been explicitly aligned (for crop grid-snapping)
    #[serde(default)]
    pub aligned: bool,
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
            show_activities: None, // Use diagram default
            title: String::new(),
            substatemachine_path: None,
            show_expanded: false,
            has_focus: false,
            has_error: false,
            aligned: false,
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
            show_activities: None, // Use diagram default
            title: String::new(),
            substatemachine_path: None,
            show_expanded: false,
            has_focus: false,
            has_error: false,
            aligned: false,
        }
    }

    /// Returns true if this state has a sub-statemachine (external or embedded)
    pub fn has_substatemachine(&self) -> bool {
        self.substatemachine_path.is_some()
    }

    /// Returns true if the sub-statemachine is stored as an external file
    pub fn is_external_substatemachine(&self) -> bool {
        self.substatemachine_path.as_ref().map(|p| !p.is_empty()).unwrap_or(false)
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

    /// Check if activities should be shown for this state
    /// If show_activities is None, uses the diagram default
    pub fn should_show_activities(&self, diagram_default: bool) -> bool {
        self.show_activities.unwrap_or(diagram_default) && self.has_activities()
    }

    /// Get the header height (area for name and activities)
    pub fn header_height(&self) -> f32 {
        if self.has_activities() {
            40.0
        } else {
            25.0
        }
    }

    /// Calculate the required height for this state based on content
    /// char_width: approximate character width in pixels
    /// line_height: height per line of text
    pub fn calculate_required_size(&self, show_activities: bool, char_width: f32, line_height: f32) -> (f32, f32) {
        let padding = 8.0;
        let name_height = line_height + 4.0; // Name + some padding
        let separator_height = 4.0;

        // Calculate required width based on name
        let name_width = self.name.len() as f32 * char_width + padding * 2.0;
        let mut required_width = name_width.max(60.0); // Minimum width

        // Calculate required height
        let mut required_height = name_height + padding;

        if show_activities && self.has_activities() {
            required_height += separator_height;

            // Calculate width and height for each activity
            if !self.entry_activity.is_empty() {
                let text = format!("entry / {}", self.entry_activity);
                let lines: Vec<&str> = text.lines().collect();
                for line in &lines {
                    let line_width = line.len() as f32 * char_width + padding * 2.0;
                    required_width = required_width.max(line_width);
                }
                required_height += lines.len() as f32 * line_height;
            }
            if !self.exit_activity.is_empty() {
                let text = format!("exit / {}", self.exit_activity);
                let lines: Vec<&str> = text.lines().collect();
                for line in &lines {
                    let line_width = line.len() as f32 * char_width + padding * 2.0;
                    required_width = required_width.max(line_width);
                }
                required_height += lines.len() as f32 * line_height;
            }
            if !self.do_activity.is_empty() {
                let text = format!("do / {}", self.do_activity);
                let lines: Vec<&str> = text.lines().collect();
                for line in &lines {
                    let line_width = line.len() as f32 * char_width + padding * 2.0;
                    required_width = required_width.max(line_width);
                }
                required_height += lines.len() as f32 * line_height;
            }

            required_height += padding;
        }

        (required_width, required_height)
    }

    /// Resize state to fit its content
    pub fn resize_to_fit(&mut self, show_activities: bool) {
        let (required_width, required_height) = self.calculate_required_size(show_activities, 7.0, 12.0);

        // Only grow, don't shrink below minimum
        let new_width = self.bounds.width().max(required_width);
        let new_height = self.bounds.height().max(required_height);

        self.bounds.x2 = self.bounds.x1 + new_width;
        self.bounds.y2 = self.bounds.y1 + new_height;
    }
}

impl Default for State {
    fn default() -> Self {
        Self::new("State", 0.0, 0.0, 100.0, 60.0)
    }
}
