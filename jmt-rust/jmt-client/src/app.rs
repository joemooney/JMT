//! Main application state and update loop

use eframe::egui;
use jmt_core::{Diagram, EditMode, NodeType, DiagramType};
use jmt_core::geometry::{Point, Rect};
use jmt_core::node::{Corner, NodeId};
use std::time::Instant;

use crate::canvas::DiagramCanvas;
use crate::panels::{MenuBar, Toolbar, PropertiesPanel, StatusBar};

/// State for rectangular marquee selection
#[derive(Debug, Clone, Default)]
pub struct SelectionRect {
    /// Starting point of the selection (where drag began)
    pub start: Option<egui::Pos2>,
    /// Current end point (current mouse position during drag)
    pub current: Option<egui::Pos2>,
}

impl SelectionRect {
    /// Check if a selection is active
    pub fn is_active(&self) -> bool {
        self.start.is_some() && self.current.is_some()
    }

    /// Get the selection rectangle as an egui Rect
    pub fn to_egui_rect(&self) -> Option<egui::Rect> {
        if let (Some(start), Some(current)) = (self.start, self.current) {
            Some(egui::Rect::from_two_pos(start, current))
        } else {
            None
        }
    }

    /// Get the selection rectangle as a core Rect
    pub fn to_core_rect(&self) -> Option<Rect> {
        if let (Some(start), Some(current)) = (self.start, self.current) {
            let min_x = start.x.min(current.x);
            let min_y = start.y.min(current.y);
            let max_x = start.x.max(current.x);
            let max_y = start.y.max(current.y);
            Some(Rect::new(min_x, min_y, max_x, max_y))
        } else {
            None
        }
    }

    /// Clear the selection
    pub fn clear(&mut self) {
        self.start = None;
        self.current = None;
    }
}

/// State for resizing a node by its corner
#[derive(Debug, Clone, Default)]
pub struct ResizeState {
    /// The node being resized
    pub node_id: Option<NodeId>,
    /// Which corner is being dragged
    pub corner: Corner,
}

impl ResizeState {
    /// Check if a resize is active
    pub fn is_active(&self) -> bool {
        self.node_id.is_some() && self.corner != Corner::None
    }

    /// Start a resize operation
    pub fn start(&mut self, node_id: NodeId, corner: Corner) {
        self.node_id = Some(node_id);
        self.corner = corner;
    }

    /// Clear the resize state
    pub fn clear(&mut self) {
        self.node_id = None;
        self.corner = Corner::None;
    }
}

/// State for a single open diagram
pub struct DiagramState {
    pub diagram: Diagram,
    pub canvas: DiagramCanvas,
    pub modified: bool,
    /// File path where this diagram is saved (None if not yet saved)
    pub file_path: Option<std::path::PathBuf>,
}

impl DiagramState {
    pub fn new(diagram: Diagram) -> Self {
        Self {
            canvas: DiagramCanvas::new(),
            diagram,
            modified: false,
            file_path: None,
        }
    }

    /// Create with an existing file path (for opened files)
    pub fn with_path(diagram: Diagram, path: std::path::PathBuf) -> Self {
        Self {
            canvas: DiagramCanvas::new(),
            diagram,
            modified: false,
            file_path: Some(path),
        }
    }
}

/// Double-click detection threshold in milliseconds
const DOUBLE_CLICK_TIME_MS: u128 = 500;
/// Maximum distance (in pixels) between clicks to count as double-click
const DOUBLE_CLICK_DISTANCE: f32 = 10.0;
/// Minimum zoom level (25%)
const MIN_ZOOM: f32 = 0.25;
/// Maximum zoom level (400%)
const MAX_ZOOM: f32 = 4.0;
/// Zoom step for buttons
const ZOOM_STEP: f32 = 0.1;
/// Zoom step for mouse wheel
const ZOOM_WHEEL_STEP: f32 = 0.1;

/// The main JMT application
pub struct JmtApp {
    /// Open diagrams
    diagrams: Vec<DiagramState>,
    /// Currently active diagram index
    active_diagram: usize,
    /// Current edit mode
    pub edit_mode: EditMode,
    /// Status message
    status_message: String,
    /// Pending connection source (when in Connect mode)
    pending_connection_source: Option<uuid::Uuid>,
    /// Active selection rectangle for marquee selection
    pub selection_rect: SelectionRect,
    /// Whether we're currently dragging nodes (vs marquee selecting)
    dragging_nodes: bool,
    /// Current cursor position on canvas (for preview rendering)
    pub cursor_pos: Option<egui::Pos2>,
    /// Active resize state (when resizing a node by corner)
    resize_state: ResizeState,
    /// Lasso selection points (freeform polygon)
    lasso_points: Vec<egui::Pos2>,
    /// Time of last click (for custom double-click detection)
    last_click_time: Option<Instant>,
    /// Position of last click (for custom double-click detection)
    last_click_pos: Option<egui::Pos2>,
    /// Current zoom level (1.0 = 100%)
    pub zoom_level: f32,
}

impl Default for JmtApp {
    fn default() -> Self {
        // Create a default diagram to start with
        let diagram = Diagram::new("Untitled");
        let diagram_state = DiagramState::new(diagram);

        Self {
            diagrams: vec![diagram_state],
            active_diagram: 0,
            edit_mode: EditMode::Arrow,
            status_message: String::from("Ready"),
            pending_connection_source: None,
            selection_rect: SelectionRect::default(),
            dragging_nodes: false,
            cursor_pos: None,
            resize_state: ResizeState::default(),
            lasso_points: Vec::new(),
            last_click_time: None,
            last_click_pos: None,
            zoom_level: 1.0,
        }
    }
}

impl JmtApp {
    /// Create a new application instance
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        Self::default()
    }

    /// Get the current diagram
    pub fn current_diagram(&self) -> Option<&DiagramState> {
        self.diagrams.get(self.active_diagram)
    }

    /// Get the current diagram mutably
    pub fn current_diagram_mut(&mut self) -> Option<&mut DiagramState> {
        self.diagrams.get_mut(self.active_diagram)
    }

    /// Create a new diagram
    pub fn new_diagram(&mut self) {
        self.new_diagram_of_type(DiagramType::StateMachine);
    }

    /// Create a new diagram of a specific type
    pub fn new_diagram_of_type(&mut self, diagram_type: DiagramType) {
        let type_name = diagram_type.display_name();
        let name = format!("{} {}", type_name, self.diagrams.len() + 1);
        let mut diagram = Diagram::new(&name);
        diagram.diagram_type = diagram_type;
        self.diagrams.push(DiagramState::new(diagram));
        self.active_diagram = self.diagrams.len() - 1;
        self.edit_mode = EditMode::Arrow;
        self.status_message = format!("Created new {}", type_name);
    }

    /// Close the current diagram
    pub fn close_diagram(&mut self) {
        if self.diagrams.len() > 1 {
            self.diagrams.remove(self.active_diagram);
            if self.active_diagram >= self.diagrams.len() {
                self.active_diagram = self.diagrams.len() - 1;
            }
        }
    }

    /// Save the current diagram (prompts for path if not yet saved)
    #[cfg(not(target_arch = "wasm32"))]
    pub fn save(&mut self) {
        if let Some(state) = self.current_diagram() {
            if state.file_path.is_some() {
                // Has a path, save directly
                self.save_to_current_path();
            } else {
                // No path yet, do Save As
                self.save_as();
            }
        }
    }

    /// Save the current diagram to its current path
    #[cfg(not(target_arch = "wasm32"))]
    fn save_to_current_path(&mut self) {
        if let Some(state) = self.current_diagram() {
            if let Some(path) = &state.file_path {
                let json = serde_json::to_string_pretty(&state.diagram);
                match json {
                    Ok(content) => {
                        match std::fs::write(path, &content) {
                            Ok(_) => {
                                let filename = path.file_name()
                                    .map(|s| s.to_string_lossy().to_string())
                                    .unwrap_or_else(|| "file".to_string());
                                self.status_message = format!("Saved: {}", filename);
                                // Mark as not modified after successful save
                                if let Some(state) = self.current_diagram_mut() {
                                    state.modified = false;
                                }
                            }
                            Err(e) => {
                                self.status_message = format!("Error saving: {}", e);
                            }
                        }
                    }
                    Err(e) => {
                        self.status_message = format!("Error serializing: {}", e);
                    }
                }
            }
        }
    }

    /// Save As - always prompts for a new file path
    #[cfg(not(target_arch = "wasm32"))]
    pub fn save_as(&mut self) {
        use rfd::FileDialog;

        if self.current_diagram().is_none() {
            return;
        }

        // Get diagram name for default filename
        let default_name = self.current_diagram()
            .map(|s| s.diagram.settings.name.clone())
            .unwrap_or_else(|| "diagram".to_string());

        let dialog = FileDialog::new()
            .set_title("Save Diagram As")
            .add_filter("JMT Diagram", &["jmt"])
            .add_filter("JSON", &["json"])
            .set_file_name(&format!("{}.jmt", default_name));

        if let Some(path) = dialog.save_file() {
            // Update the file path
            if let Some(state) = self.current_diagram_mut() {
                state.file_path = Some(path);
            }
            // Now save to this path
            self.save_to_current_path();
        }
    }

    /// Open a diagram from file
    #[cfg(not(target_arch = "wasm32"))]
    pub fn open(&mut self) {
        use rfd::FileDialog;

        let dialog = FileDialog::new()
            .set_title("Open Diagram")
            .add_filter("JMT Diagram", &["jmt"])
            .add_filter("JSON", &["json"])
            .add_filter("All Files", &["*"]);

        if let Some(path) = dialog.pick_file() {
            match std::fs::read_to_string(&path) {
                Ok(content) => {
                    match serde_json::from_str::<Diagram>(&content) {
                        Ok(mut diagram) => {
                            // Recalculate connection routing (segments are not serialized)
                            diagram.recalculate_connections();
                            let state = DiagramState::with_path(diagram, path.clone());
                            self.diagrams.push(state);
                            self.active_diagram = self.diagrams.len() - 1;
                            let filename = path.file_name()
                                .map(|s| s.to_string_lossy().to_string())
                                .unwrap_or_else(|| "file".to_string());
                            self.status_message = format!("Opened: {}", filename);
                        }
                        Err(e) => {
                            self.status_message = format!("Error parsing file: {}", e);
                        }
                    }
                }
                Err(e) => {
                    self.status_message = format!("Error reading file: {}", e);
                }
            }
        }
    }

    /// Export the current diagram as PNG
    #[cfg(not(target_arch = "wasm32"))]
    pub fn export_png(&mut self, autocrop: bool) {
        use rfd::FileDialog;
        use image::{ImageBuffer, Rgba};

        let state = match self.current_diagram() {
            Some(s) => s,
            None => return,
        };

        // Get diagram name for default filename
        let default_name = state.diagram.settings.name.clone();

        let dialog = FileDialog::new()
            .set_title("Export as PNG")
            .add_filter("PNG Image", &["png"])
            .set_file_name(&format!("{}.png", default_name));

        if let Some(path) = dialog.save_file() {
            // Calculate diagram bounds
            let (min_x, min_y, max_x, max_y) = self.calculate_diagram_bounds();

            if max_x <= min_x || max_y <= min_y {
                self.status_message = "No elements to export".to_string();
                return;
            }

            // Add margin
            let margin = if autocrop { 20.0 } else { 50.0 };
            let (x_offset, y_offset, width, height) = if autocrop {
                (min_x - margin, min_y - margin,
                 (max_x - min_x + margin * 2.0) as u32,
                 (max_y - min_y + margin * 2.0) as u32)
            } else {
                // Use fixed canvas size with content positioned
                let canvas_width = (max_x + margin * 2.0).max(800.0) as u32;
                let canvas_height = (max_y + margin * 2.0).max(600.0) as u32;
                (0.0, 0.0, canvas_width, canvas_height)
            };

            // Create image buffer with white background
            let mut img: ImageBuffer<Rgba<u8>, Vec<u8>> = ImageBuffer::from_pixel(
                width, height,
                Rgba([255, 255, 255, 255])
            );

            // Render diagram elements to the image
            if let Some(state) = self.current_diagram() {
                Self::render_diagram_to_image(&mut img, &state.diagram, x_offset, y_offset);
            }

            // Save the image
            match img.save(&path) {
                Ok(_) => {
                    let filename = path.file_name()
                        .map(|s| s.to_string_lossy().to_string())
                        .unwrap_or_else(|| "file".to_string());
                    self.status_message = format!("Exported: {}", filename);
                }
                Err(e) => {
                    self.status_message = format!("Error exporting: {}", e);
                }
            }
        }
    }

    /// Calculate the bounding box of all diagram elements
    #[cfg(not(target_arch = "wasm32"))]
    fn calculate_diagram_bounds(&self) -> (f32, f32, f32, f32) {
        let state = match self.current_diagram() {
            Some(s) => s,
            None => return (0.0, 0.0, 0.0, 0.0),
        };

        let mut min_x = f32::MAX;
        let mut min_y = f32::MAX;
        let mut max_x = f32::MIN;
        let mut max_y = f32::MIN;

        // State machine nodes
        for node in state.diagram.nodes() {
            let bounds = node.bounds();
            min_x = min_x.min(bounds.x1);
            min_y = min_y.min(bounds.y1);
            max_x = max_x.max(bounds.x2);
            max_y = max_y.max(bounds.y2);
        }

        // Connection endpoints
        for conn in state.diagram.connections() {
            for seg in &conn.segments {
                min_x = min_x.min(seg.start.x).min(seg.end.x);
                min_y = min_y.min(seg.start.y).min(seg.end.y);
                max_x = max_x.max(seg.start.x).max(seg.end.x);
                max_y = max_y.max(seg.start.y).max(seg.end.y);
            }
        }

        (min_x, min_y, max_x, max_y)
    }

    /// Render diagram to an image buffer
    #[cfg(not(target_arch = "wasm32"))]
    fn render_diagram_to_image(
        img: &mut image::ImageBuffer<image::Rgba<u8>, Vec<u8>>,
        diagram: &Diagram,
        x_offset: f32,
        y_offset: f32,
    ) {
        use image::Rgba;

        let state_fill = Rgba([255, 255, 204, 255]); // Light yellow
        let state_stroke = Rgba([0, 0, 0, 255]); // Black
        let pseudo_fill = Rgba([0, 0, 0, 255]); // Black for initial/final
        let connection_color = Rgba([100, 100, 100, 255]); // Gray

        // Draw states
        for node in diagram.nodes() {
            let bounds = node.bounds();
            let x1 = (bounds.x1 - x_offset) as i32;
            let y1 = (bounds.y1 - y_offset) as i32;
            let x2 = (bounds.x2 - x_offset) as i32;
            let y2 = (bounds.y2 - y_offset) as i32;

            match node {
                jmt_core::node::Node::State(_) => {
                    // Draw filled rounded rectangle (simplified to rectangle)
                    Self::draw_filled_rect(img, x1, y1, x2, y2, state_fill);
                    Self::draw_rect_outline(img, x1, y1, x2, y2, state_stroke);

                    // Draw state name
                    let name = node.name();
                    let cx = (x1 + x2) / 2;
                    let cy = (y1 + y2) / 2;
                    Self::draw_text_centered(img, cx, cy, name, state_stroke);
                }
                jmt_core::node::Node::Pseudo(ps) => {
                    let cx = ((bounds.x1 + bounds.x2) / 2.0 - x_offset) as i32;
                    let cy = ((bounds.y1 + bounds.y2) / 2.0 - y_offset) as i32;
                    let radius = ((bounds.x2 - bounds.x1) / 2.0) as i32;

                    match ps.kind {
                        jmt_core::node::PseudoStateKind::Initial => {
                            Self::draw_filled_circle(img, cx, cy, radius, pseudo_fill);
                        }
                        jmt_core::node::PseudoStateKind::Final => {
                            Self::draw_circle_outline(img, cx, cy, radius, pseudo_fill);
                            Self::draw_filled_circle(img, cx, cy, radius - 4, pseudo_fill);
                        }
                        jmt_core::node::PseudoStateKind::Choice | jmt_core::node::PseudoStateKind::Junction => {
                            // Diamond shape
                            Self::draw_diamond(img, cx, cy, radius, state_stroke);
                        }
                        jmt_core::node::PseudoStateKind::Fork | jmt_core::node::PseudoStateKind::Join => {
                            // Thick bar
                            Self::draw_filled_rect(img, x1, y1, x2, y2, pseudo_fill);
                        }
                    }
                }
            }
        }

        // Draw connections
        for conn in diagram.connections() {
            for seg in &conn.segments {
                let x1 = (seg.start.x - x_offset) as i32;
                let y1 = (seg.start.y - y_offset) as i32;
                let x2 = (seg.end.x - x_offset) as i32;
                let y2 = (seg.end.y - y_offset) as i32;
                Self::draw_line(img, x1, y1, x2, y2, connection_color);
            }

            // Draw arrowhead at target
            if let Some(last_seg) = conn.segments.last() {
                let x2 = (last_seg.end.x - x_offset) as i32;
                let y2 = (last_seg.end.y - y_offset) as i32;
                let x1 = (last_seg.start.x - x_offset) as i32;
                let y1 = (last_seg.start.y - y_offset) as i32;
                Self::draw_arrowhead(img, x1, y1, x2, y2, connection_color);
            }
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn draw_filled_rect(img: &mut image::ImageBuffer<image::Rgba<u8>, Vec<u8>>, x1: i32, y1: i32, x2: i32, y2: i32, color: image::Rgba<u8>) {
        let (width, height) = img.dimensions();
        for y in y1.max(0)..y2.min(height as i32) {
            for x in x1.max(0)..x2.min(width as i32) {
                img.put_pixel(x as u32, y as u32, color);
            }
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn draw_rect_outline(img: &mut image::ImageBuffer<image::Rgba<u8>, Vec<u8>>, x1: i32, y1: i32, x2: i32, y2: i32, color: image::Rgba<u8>) {
        Self::draw_line(img, x1, y1, x2, y1, color); // Top
        Self::draw_line(img, x1, y2, x2, y2, color); // Bottom
        Self::draw_line(img, x1, y1, x1, y2, color); // Left
        Self::draw_line(img, x2, y1, x2, y2, color); // Right
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn draw_line(img: &mut image::ImageBuffer<image::Rgba<u8>, Vec<u8>>, x1: i32, y1: i32, x2: i32, y2: i32, color: image::Rgba<u8>) {
        let (width, height) = img.dimensions();
        let dx = (x2 - x1).abs();
        let dy = (y2 - y1).abs();
        let sx = if x1 < x2 { 1 } else { -1 };
        let sy = if y1 < y2 { 1 } else { -1 };
        let mut err = dx - dy;
        let mut x = x1;
        let mut y = y1;

        loop {
            if x >= 0 && x < width as i32 && y >= 0 && y < height as i32 {
                img.put_pixel(x as u32, y as u32, color);
            }
            if x == x2 && y == y2 { break; }
            let e2 = 2 * err;
            if e2 > -dy { err -= dy; x += sx; }
            if e2 < dx { err += dx; y += sy; }
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn draw_filled_circle(img: &mut image::ImageBuffer<image::Rgba<u8>, Vec<u8>>, cx: i32, cy: i32, radius: i32, color: image::Rgba<u8>) {
        let (width, height) = img.dimensions();
        for dy in -radius..=radius {
            for dx in -radius..=radius {
                if dx * dx + dy * dy <= radius * radius {
                    let x = cx + dx;
                    let y = cy + dy;
                    if x >= 0 && x < width as i32 && y >= 0 && y < height as i32 {
                        img.put_pixel(x as u32, y as u32, color);
                    }
                }
            }
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn draw_circle_outline(img: &mut image::ImageBuffer<image::Rgba<u8>, Vec<u8>>, cx: i32, cy: i32, radius: i32, color: image::Rgba<u8>) {
        let (width, height) = img.dimensions();
        let r2_outer = radius * radius;
        let r2_inner = (radius - 2) * (radius - 2);
        for dy in -radius..=radius {
            for dx in -radius..=radius {
                let d2 = dx * dx + dy * dy;
                if d2 <= r2_outer && d2 >= r2_inner {
                    let x = cx + dx;
                    let y = cy + dy;
                    if x >= 0 && x < width as i32 && y >= 0 && y < height as i32 {
                        img.put_pixel(x as u32, y as u32, color);
                    }
                }
            }
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn draw_diamond(img: &mut image::ImageBuffer<image::Rgba<u8>, Vec<u8>>, cx: i32, cy: i32, radius: i32, color: image::Rgba<u8>) {
        // Draw diamond outline
        Self::draw_line(img, cx, cy - radius, cx + radius, cy, color); // Top to right
        Self::draw_line(img, cx + radius, cy, cx, cy + radius, color); // Right to bottom
        Self::draw_line(img, cx, cy + radius, cx - radius, cy, color); // Bottom to left
        Self::draw_line(img, cx - radius, cy, cx, cy - radius, color); // Left to top
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn draw_arrowhead(img: &mut image::ImageBuffer<image::Rgba<u8>, Vec<u8>>, x1: i32, y1: i32, x2: i32, y2: i32, color: image::Rgba<u8>) {
        let dx = (x2 - x1) as f32;
        let dy = (y2 - y1) as f32;
        let len = (dx * dx + dy * dy).sqrt();
        if len < 1.0 { return; }

        let ux = dx / len;
        let uy = dy / len;

        let arrow_len = 10.0;
        let arrow_width = 5.0;

        let ax = x2 as f32 - ux * arrow_len;
        let ay = y2 as f32 - uy * arrow_len;

        let p1x = (ax - uy * arrow_width) as i32;
        let p1y = (ay + ux * arrow_width) as i32;
        let p2x = (ax + uy * arrow_width) as i32;
        let p2y = (ay - ux * arrow_width) as i32;

        Self::draw_line(img, x2, y2, p1x, p1y, color);
        Self::draw_line(img, x2, y2, p2x, p2y, color);
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn draw_text_centered(img: &mut image::ImageBuffer<image::Rgba<u8>, Vec<u8>>, cx: i32, cy: i32, text: &str, color: image::Rgba<u8>) {
        // Simple text rendering - draw each character as a small pattern
        // This is a simplified version; for production, use a proper font rendering library
        let char_width = 6;
        let char_height = 10;
        let text_width = text.len() as i32 * char_width;
        let start_x = cx - text_width / 2;
        let start_y = cy - char_height / 2;

        for (i, c) in text.chars().enumerate() {
            let x = start_x + i as i32 * char_width;
            Self::draw_simple_char(img, x, start_y, c, color);
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn draw_simple_char(img: &mut image::ImageBuffer<image::Rgba<u8>, Vec<u8>>, x: i32, y: i32, c: char, color: image::Rgba<u8>) {
        // Very simple bitmap font for basic characters
        let (width, height) = img.dimensions();
        let patterns: &[(char, &[u8])] = &[
            ('A', &[0b01110, 0b10001, 0b10001, 0b11111, 0b10001, 0b10001, 0b10001]),
            ('B', &[0b11110, 0b10001, 0b10001, 0b11110, 0b10001, 0b10001, 0b11110]),
            ('C', &[0b01110, 0b10001, 0b10000, 0b10000, 0b10000, 0b10001, 0b01110]),
            ('D', &[0b11110, 0b10001, 0b10001, 0b10001, 0b10001, 0b10001, 0b11110]),
            ('E', &[0b11111, 0b10000, 0b10000, 0b11110, 0b10000, 0b10000, 0b11111]),
            ('F', &[0b11111, 0b10000, 0b10000, 0b11110, 0b10000, 0b10000, 0b10000]),
            ('G', &[0b01110, 0b10001, 0b10000, 0b10111, 0b10001, 0b10001, 0b01110]),
            ('H', &[0b10001, 0b10001, 0b10001, 0b11111, 0b10001, 0b10001, 0b10001]),
            ('I', &[0b01110, 0b00100, 0b00100, 0b00100, 0b00100, 0b00100, 0b01110]),
            ('J', &[0b00111, 0b00010, 0b00010, 0b00010, 0b00010, 0b10010, 0b01100]),
            ('K', &[0b10001, 0b10010, 0b10100, 0b11000, 0b10100, 0b10010, 0b10001]),
            ('L', &[0b10000, 0b10000, 0b10000, 0b10000, 0b10000, 0b10000, 0b11111]),
            ('M', &[0b10001, 0b11011, 0b10101, 0b10101, 0b10001, 0b10001, 0b10001]),
            ('N', &[0b10001, 0b11001, 0b10101, 0b10011, 0b10001, 0b10001, 0b10001]),
            ('O', &[0b01110, 0b10001, 0b10001, 0b10001, 0b10001, 0b10001, 0b01110]),
            ('P', &[0b11110, 0b10001, 0b10001, 0b11110, 0b10000, 0b10000, 0b10000]),
            ('Q', &[0b01110, 0b10001, 0b10001, 0b10001, 0b10101, 0b10010, 0b01101]),
            ('R', &[0b11110, 0b10001, 0b10001, 0b11110, 0b10100, 0b10010, 0b10001]),
            ('S', &[0b01110, 0b10001, 0b10000, 0b01110, 0b00001, 0b10001, 0b01110]),
            ('T', &[0b11111, 0b00100, 0b00100, 0b00100, 0b00100, 0b00100, 0b00100]),
            ('U', &[0b10001, 0b10001, 0b10001, 0b10001, 0b10001, 0b10001, 0b01110]),
            ('V', &[0b10001, 0b10001, 0b10001, 0b10001, 0b10001, 0b01010, 0b00100]),
            ('W', &[0b10001, 0b10001, 0b10001, 0b10101, 0b10101, 0b11011, 0b10001]),
            ('X', &[0b10001, 0b10001, 0b01010, 0b00100, 0b01010, 0b10001, 0b10001]),
            ('Y', &[0b10001, 0b10001, 0b01010, 0b00100, 0b00100, 0b00100, 0b00100]),
            ('Z', &[0b11111, 0b00001, 0b00010, 0b00100, 0b01000, 0b10000, 0b11111]),
            ('0', &[0b01110, 0b10001, 0b10011, 0b10101, 0b11001, 0b10001, 0b01110]),
            ('1', &[0b00100, 0b01100, 0b00100, 0b00100, 0b00100, 0b00100, 0b01110]),
            ('2', &[0b01110, 0b10001, 0b00001, 0b00110, 0b01000, 0b10000, 0b11111]),
            ('3', &[0b01110, 0b10001, 0b00001, 0b00110, 0b00001, 0b10001, 0b01110]),
            ('4', &[0b00010, 0b00110, 0b01010, 0b10010, 0b11111, 0b00010, 0b00010]),
            ('5', &[0b11111, 0b10000, 0b11110, 0b00001, 0b00001, 0b10001, 0b01110]),
            ('6', &[0b00110, 0b01000, 0b10000, 0b11110, 0b10001, 0b10001, 0b01110]),
            ('7', &[0b11111, 0b00001, 0b00010, 0b00100, 0b01000, 0b01000, 0b01000]),
            ('8', &[0b01110, 0b10001, 0b10001, 0b01110, 0b10001, 0b10001, 0b01110]),
            ('9', &[0b01110, 0b10001, 0b10001, 0b01111, 0b00001, 0b00010, 0b01100]),
            ('a', &[0b00000, 0b00000, 0b01110, 0b00001, 0b01111, 0b10001, 0b01111]),
            ('b', &[0b10000, 0b10000, 0b10110, 0b11001, 0b10001, 0b10001, 0b11110]),
            ('c', &[0b00000, 0b00000, 0b01110, 0b10000, 0b10000, 0b10001, 0b01110]),
            ('d', &[0b00001, 0b00001, 0b01101, 0b10011, 0b10001, 0b10001, 0b01111]),
            ('e', &[0b00000, 0b00000, 0b01110, 0b10001, 0b11111, 0b10000, 0b01110]),
            ('f', &[0b00110, 0b01001, 0b01000, 0b11100, 0b01000, 0b01000, 0b01000]),
            ('g', &[0b00000, 0b01111, 0b10001, 0b10001, 0b01111, 0b00001, 0b01110]),
            ('h', &[0b10000, 0b10000, 0b10110, 0b11001, 0b10001, 0b10001, 0b10001]),
            ('i', &[0b00100, 0b00000, 0b01100, 0b00100, 0b00100, 0b00100, 0b01110]),
            ('j', &[0b00010, 0b00000, 0b00110, 0b00010, 0b00010, 0b10010, 0b01100]),
            ('k', &[0b10000, 0b10000, 0b10010, 0b10100, 0b11000, 0b10100, 0b10010]),
            ('l', &[0b01100, 0b00100, 0b00100, 0b00100, 0b00100, 0b00100, 0b01110]),
            ('m', &[0b00000, 0b00000, 0b11010, 0b10101, 0b10101, 0b10001, 0b10001]),
            ('n', &[0b00000, 0b00000, 0b10110, 0b11001, 0b10001, 0b10001, 0b10001]),
            ('o', &[0b00000, 0b00000, 0b01110, 0b10001, 0b10001, 0b10001, 0b01110]),
            ('p', &[0b00000, 0b00000, 0b11110, 0b10001, 0b11110, 0b10000, 0b10000]),
            ('q', &[0b00000, 0b00000, 0b01101, 0b10011, 0b01111, 0b00001, 0b00001]),
            ('r', &[0b00000, 0b00000, 0b10110, 0b11001, 0b10000, 0b10000, 0b10000]),
            ('s', &[0b00000, 0b00000, 0b01110, 0b10000, 0b01110, 0b00001, 0b11110]),
            ('t', &[0b01000, 0b01000, 0b11100, 0b01000, 0b01000, 0b01001, 0b00110]),
            ('u', &[0b00000, 0b00000, 0b10001, 0b10001, 0b10001, 0b10011, 0b01101]),
            ('v', &[0b00000, 0b00000, 0b10001, 0b10001, 0b10001, 0b01010, 0b00100]),
            ('w', &[0b00000, 0b00000, 0b10001, 0b10001, 0b10101, 0b10101, 0b01010]),
            ('x', &[0b00000, 0b00000, 0b10001, 0b01010, 0b00100, 0b01010, 0b10001]),
            ('y', &[0b00000, 0b00000, 0b10001, 0b10001, 0b01111, 0b00001, 0b01110]),
            ('z', &[0b00000, 0b00000, 0b11111, 0b00010, 0b00100, 0b01000, 0b11111]),
            (' ', &[0b00000, 0b00000, 0b00000, 0b00000, 0b00000, 0b00000, 0b00000]),
            ('_', &[0b00000, 0b00000, 0b00000, 0b00000, 0b00000, 0b00000, 0b11111]),
        ];

        let pattern = patterns.iter().find(|(ch, _)| *ch == c.to_ascii_uppercase() || *ch == c);
        if let Some((_, bits)) = pattern {
            for (row, &byte) in bits.iter().enumerate() {
                for col in 0..5 {
                    if (byte >> (4 - col)) & 1 == 1 {
                        let px = x + col;
                        let py = y + row as i32;
                        if px >= 0 && px < width as i32 && py >= 0 && py < height as i32 {
                            img.put_pixel(px as u32, py as u32, color);
                        }
                    }
                }
            }
        }
    }

    /// Set the edit mode
    pub fn set_edit_mode(&mut self, mode: EditMode) {
        // Special handling: If switching to Connect mode with multiple nodes selected,
        // connect them in sequence automatically
        if mode == EditMode::Connect {
            if let Some(state) = self.current_diagram_mut() {
                let selected = state.diagram.selected_nodes_in_order();
                if selected.len() >= 2 {
                    // Determine ordering: explicit (Ctrl+Click) or position-based (marquee)
                    let use_selection_order = state.diagram.has_explicit_selection_order();

                    // Get nodes with positions for sorting
                    let mut nodes_with_pos: Vec<_> = selected.iter()
                        .filter_map(|id| {
                            state.diagram.find_node(*id).map(|n| {
                                let center = n.bounds().center();
                                (*id, center.x)
                            })
                        })
                        .collect();

                    // Sort by x position if marquee selection (not explicit order)
                    if !use_selection_order {
                        nodes_with_pos.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());
                    }

                    let ordered_ids: Vec<_> = nodes_with_pos.iter().map(|(id, _)| *id).collect();

                    if ordered_ids.len() >= 2 {
                        // Connect nodes in sequence: 1->2, 2->3, 3->4, etc.
                        state.diagram.push_undo();
                        let mut connections_made = 0;
                        for i in 0..ordered_ids.len() - 1 {
                            let source = ordered_ids[i];
                            let target = ordered_ids[i + 1];
                            if state.diagram.add_connection(source, target).is_some() {
                                connections_made += 1;
                            }
                        }
                        if connections_made > 0 {
                            state.modified = true;
                            self.status_message = format!("Created {} connection(s)", connections_made);
                            // Stay in Arrow mode after auto-connecting
                            self.edit_mode = EditMode::Arrow;
                            self.pending_connection_source = None;
                            return;
                        }
                    }
                }
            }
        }

        self.edit_mode = mode;
        self.pending_connection_source = None;
        self.status_message = format!("Mode: {}", mode.display_name());
    }

    /// Zoom in by one step
    pub fn zoom_in(&mut self) {
        self.zoom_level = (self.zoom_level + ZOOM_STEP).min(MAX_ZOOM);
        self.status_message = format!("Zoom: {:.0}%", self.zoom_level * 100.0);
    }

    /// Zoom out by one step
    pub fn zoom_out(&mut self) {
        self.zoom_level = (self.zoom_level - ZOOM_STEP).max(MIN_ZOOM);
        self.status_message = format!("Zoom: {:.0}%", self.zoom_level * 100.0);
    }

    /// Reset zoom to 100%
    pub fn reset_zoom(&mut self) {
        self.zoom_level = 1.0;
        self.status_message = "Zoom: 100%".to_string();
    }

    /// Zoom by a delta (positive = zoom in, negative = zoom out)
    pub fn zoom_by(&mut self, delta: f32) {
        self.zoom_level = (self.zoom_level + delta).clamp(MIN_ZOOM, MAX_ZOOM);
        self.status_message = format!("Zoom: {:.0}%", self.zoom_level * 100.0);
    }

    /// Handle canvas click
    /// If `switch_to_arrow` is true, switch back to Arrow mode after adding element
    /// If `ctrl_held` is true, toggle selection instead of replacing it
    fn handle_canvas_click(&mut self, pos: egui::Pos2, switch_to_arrow: bool, ctrl_held: bool) {
        let point = Point::new(pos.x, pos.y);
        let edit_mode = self.edit_mode;
        let pending_source = self.pending_connection_source;

        let Some(state) = self.current_diagram_mut() else {
            return;
        };

        match edit_mode {
            EditMode::Arrow => {
                // Try to select any element (node, lifeline, actor, use case, action, etc.)
                if let Some(element_id) = state.diagram.find_element_at(point) {
                    let name = state.diagram.get_element_name(element_id)
                        .unwrap_or_default();

                    if ctrl_held {
                        // Ctrl+Click: Toggle selection
                        state.diagram.toggle_element_selection(element_id);
                        let selected_count = state.diagram.selected_nodes().len();
                        self.status_message = format!("Selected {} element(s)", selected_count);
                    } else {
                        // Regular click: Select only this element
                        state.diagram.select_element(element_id);
                        self.status_message = format!("Selected: {}", name);
                    }
                } else if let Some(conn_id) = state.diagram.find_connection_at(point, 10.0) {
                    state.diagram.select_connection(conn_id);
                    self.status_message = "Selected connection".to_string();
                } else {
                    // Only clear selection if Ctrl is not held
                    if !ctrl_held {
                        state.diagram.clear_selection();
                        self.status_message = "Ready".to_string();
                    }
                }
            }
            EditMode::AddState => {
                state.diagram.push_undo();
                let id = state.diagram.add_node(NodeType::State, pos.x, pos.y);
                state.diagram.select_node(id);
                state.modified = true;
                self.status_message = "Added state".to_string();
                if switch_to_arrow {
                    self.edit_mode = EditMode::Arrow;
                }
            }
            EditMode::AddInitial => {
                state.diagram.push_undo();
                let id = state.diagram.add_node(NodeType::Initial, pos.x, pos.y);
                state.diagram.select_node(id);
                state.modified = true;
                self.status_message = "Added initial pseudo-state".to_string();
                // Auto-switch back to Arrow mode (typically only one initial state)
                self.edit_mode = EditMode::Arrow;
            }
            EditMode::AddFinal => {
                state.diagram.push_undo();
                let id = state.diagram.add_node(NodeType::Final, pos.x, pos.y);
                state.diagram.select_node(id);
                state.modified = true;
                self.status_message = "Added final pseudo-state".to_string();
                // Auto-switch back to Arrow mode (typically only one final state)
                self.edit_mode = EditMode::Arrow;
            }
            EditMode::AddChoice => {
                state.diagram.push_undo();
                let id = state.diagram.add_node(NodeType::Choice, pos.x, pos.y);
                state.diagram.select_node(id);
                state.modified = true;
                self.status_message = "Added choice pseudo-state".to_string();
                if switch_to_arrow {
                    self.edit_mode = EditMode::Arrow;
                }
            }
            EditMode::AddJunction => {
                state.diagram.push_undo();
                let id = state.diagram.add_node(NodeType::Junction, pos.x, pos.y);
                state.diagram.select_node(id);
                state.modified = true;
                self.status_message = "Added junction pseudo-state".to_string();
                if switch_to_arrow {
                    self.edit_mode = EditMode::Arrow;
                }
            }
            EditMode::AddFork => {
                state.diagram.push_undo();
                let id = state.diagram.add_node(NodeType::Fork, pos.x, pos.y);
                state.diagram.select_node(id);
                state.modified = true;
                self.status_message = "Added fork pseudo-state".to_string();
                if switch_to_arrow {
                    self.edit_mode = EditMode::Arrow;
                }
            }
            EditMode::AddJoin => {
                state.diagram.push_undo();
                let id = state.diagram.add_node(NodeType::Join, pos.x, pos.y);
                state.diagram.select_node(id);
                state.modified = true;
                self.status_message = "Added join pseudo-state".to_string();
                if switch_to_arrow {
                    self.edit_mode = EditMode::Arrow;
                }
            }
            EditMode::Connect => {
                if let Some(node_id) = state.diagram.find_node_at(point) {
                    if let Some(source_id) = pending_source {
                        // Complete the connection
                        state.diagram.push_undo();
                        if let Some(conn_id) = state.diagram.add_connection(source_id, node_id) {
                            state.diagram.select_connection(conn_id);
                            state.modified = true;
                            self.status_message = "Connection created".to_string();
                        } else {
                            self.status_message = "Cannot connect these nodes".to_string();
                        }
                        self.pending_connection_source = None;
                    } else {
                        // Start the connection
                        self.pending_connection_source = Some(node_id);
                        self.status_message = "Click target node to complete connection".to_string();
                    }
                } else {
                    // Clicked outside any node - switch back to Arrow mode
                    self.pending_connection_source = None;
                    self.edit_mode = EditMode::Arrow;
                    self.status_message = "Ready".to_string();
                }
            }

            // === Sequence Diagram Elements ===
            EditMode::AddLifeline => {
                state.diagram.push_undo();
                let _id = state.diagram.add_lifeline("Object", pos.x, pos.y);
                state.modified = true;
                self.status_message = "Added lifeline".to_string();
                if switch_to_arrow {
                    self.edit_mode = EditMode::Arrow;
                }
            }
            EditMode::AddActivation => {
                // TODO: Activations need to be attached to lifelines
                self.status_message = "Click on a lifeline to add activation".to_string();
            }
            EditMode::AddFragment => {
                state.diagram.push_undo();
                state.diagram.add_combined_fragment(pos.x, pos.y, 200.0, 150.0);
                state.modified = true;
                self.status_message = "Added combined fragment".to_string();
                if switch_to_arrow {
                    self.edit_mode = EditMode::Arrow;
                }
            }
            EditMode::AddSyncMessage | EditMode::AddAsyncMessage |
            EditMode::AddReturnMessage | EditMode::AddSelfMessage | EditMode::AddMessage => {
                // TODO: Messages need source and target lifelines
                self.status_message = "Click on source lifeline, then target lifeline".to_string();
            }

            // === Use Case Diagram Elements ===
            EditMode::AddActor => {
                state.diagram.push_undo();
                let _id = state.diagram.add_actor("Actor", pos.x, pos.y);
                state.modified = true;
                self.status_message = "Added actor".to_string();
                if switch_to_arrow {
                    self.edit_mode = EditMode::Arrow;
                }
            }
            EditMode::AddUseCase => {
                state.diagram.push_undo();
                let _id = state.diagram.add_use_case("Use Case", pos.x, pos.y);
                state.modified = true;
                self.status_message = "Added use case".to_string();
                if switch_to_arrow {
                    self.edit_mode = EditMode::Arrow;
                }
            }
            EditMode::AddSystemBoundary => {
                state.diagram.push_undo();
                let _id = state.diagram.add_system_boundary("System", pos.x, pos.y, 300.0, 400.0);
                state.modified = true;
                self.status_message = "Added system boundary".to_string();
                if switch_to_arrow {
                    self.edit_mode = EditMode::Arrow;
                }
            }
            EditMode::AddAssociation | EditMode::AddInclude |
            EditMode::AddExtend | EditMode::AddGeneralization => {
                // TODO: Relationships need source and target elements
                self.status_message = "Click on source element, then target element".to_string();
            }

            // === Activity Diagram Elements ===
            EditMode::AddAction => {
                state.diagram.push_undo();
                let _id = state.diagram.add_action("Action", pos.x, pos.y);
                state.modified = true;
                self.status_message = "Added action".to_string();
                if switch_to_arrow {
                    self.edit_mode = EditMode::Arrow;
                }
            }
            EditMode::AddDecision => {
                state.diagram.push_undo();
                state.diagram.add_decision_node(pos.x, pos.y);
                state.modified = true;
                self.status_message = "Added decision node".to_string();
                if switch_to_arrow {
                    self.edit_mode = EditMode::Arrow;
                }
            }
            EditMode::AddSendSignal => {
                state.diagram.push_undo();
                state.diagram.add_send_signal("Send", pos.x, pos.y);
                state.modified = true;
                self.status_message = "Added send signal".to_string();
                if switch_to_arrow {
                    self.edit_mode = EditMode::Arrow;
                }
            }
            EditMode::AddAcceptEvent => {
                state.diagram.push_undo();
                state.diagram.add_accept_event("Accept", pos.x, pos.y);
                state.modified = true;
                self.status_message = "Added accept event".to_string();
                if switch_to_arrow {
                    self.edit_mode = EditMode::Arrow;
                }
            }
            EditMode::AddTimeEvent => {
                state.diagram.push_undo();
                state.diagram.add_time_event("Time", pos.x, pos.y);
                state.modified = true;
                self.status_message = "Added time event".to_string();
                if switch_to_arrow {
                    self.edit_mode = EditMode::Arrow;
                }
            }
            EditMode::AddSwimlane => {
                state.diagram.push_undo();
                let _id = state.diagram.add_swimlane("Lane", pos.x, pos.y, 200.0, 400.0);
                state.modified = true;
                self.status_message = "Added swimlane".to_string();
                if switch_to_arrow {
                    self.edit_mode = EditMode::Arrow;
                }
            }
            EditMode::AddObjectNode => {
                state.diagram.push_undo();
                state.diagram.add_object_node("Object", pos.x, pos.y);
                state.modified = true;
                self.status_message = "Added object node".to_string();
                if switch_to_arrow {
                    self.edit_mode = EditMode::Arrow;
                }
            }
            EditMode::AddDataStore => {
                state.diagram.push_undo();
                state.diagram.add_data_store("DataStore", pos.x, pos.y);
                state.modified = true;
                self.status_message = "Added data store".to_string();
                if switch_to_arrow {
                    self.edit_mode = EditMode::Arrow;
                }
            }

            _ => {}
        }
    }

    /// Render cursor preview for add modes
    fn render_cursor_preview(&self, painter: &egui::Painter, pos: egui::Pos2, zoom: f32) {
        let preview_alpha = 128u8; // Semi-transparent

        match self.edit_mode {
            EditMode::AddState => {
                // Draw a ghost state rectangle
                let width = self.current_diagram()
                    .map(|s| s.diagram.settings.default_state_width)
                    .unwrap_or(100.0) * zoom;
                let height = self.current_diagram()
                    .map(|s| s.diagram.settings.default_state_height)
                    .unwrap_or(60.0) * zoom;
                let rect = egui::Rect::from_center_size(pos, egui::Vec2::new(width, height));
                let rounding = self.current_diagram()
                    .map(|s| s.diagram.settings.corner_rounding)
                    .unwrap_or(12.0) * zoom;

                painter.rect(
                    rect,
                    egui::Rounding::same(rounding),
                    egui::Color32::from_rgba_unmultiplied(255, 255, 204, preview_alpha),
                    egui::Stroke::new(zoom, egui::Color32::from_rgba_unmultiplied(0, 0, 0, preview_alpha)),
                );
                painter.text(
                    rect.center(),
                    egui::Align2::CENTER_CENTER,
                    "State",
                    egui::FontId::proportional(12.0 * zoom),
                    egui::Color32::from_rgba_unmultiplied(0, 0, 0, preview_alpha),
                );
            }
            EditMode::AddInitial => {
                // Draw a ghost initial state (filled circle)
                let radius = 8.0 * zoom;
                painter.circle_filled(
                    pos,
                    radius,
                    egui::Color32::from_rgba_unmultiplied(0, 0, 0, preview_alpha),
                );
            }
            EditMode::AddFinal => {
                // Draw a ghost final state (double circle)
                let outer_radius = 10.0 * zoom;
                let inner_radius = 6.0 * zoom;
                painter.circle_stroke(
                    pos,
                    outer_radius,
                    egui::Stroke::new(zoom, egui::Color32::from_rgba_unmultiplied(0, 0, 0, preview_alpha)),
                );
                painter.circle_filled(
                    pos,
                    inner_radius,
                    egui::Color32::from_rgba_unmultiplied(0, 0, 0, preview_alpha),
                );
            }
            EditMode::AddChoice | EditMode::AddJunction => {
                // Draw a ghost diamond
                let size = 10.0 * zoom;
                let points = vec![
                    egui::Pos2::new(pos.x, pos.y - size),
                    egui::Pos2::new(pos.x + size, pos.y),
                    egui::Pos2::new(pos.x, pos.y + size),
                    egui::Pos2::new(pos.x - size, pos.y),
                    egui::Pos2::new(pos.x, pos.y - size),
                ];
                painter.add(egui::Shape::line(
                    points,
                    egui::Stroke::new(zoom, egui::Color32::from_rgba_unmultiplied(0, 0, 0, preview_alpha)),
                ));
            }
            EditMode::AddFork | EditMode::AddJoin => {
                // Draw a ghost bar
                let width = 60.0 * zoom;
                let height = 6.0 * zoom;
                let rect = egui::Rect::from_center_size(pos, egui::Vec2::new(width, height));
                painter.rect_filled(
                    rect,
                    egui::Rounding::ZERO,
                    egui::Color32::from_rgba_unmultiplied(0, 0, 0, preview_alpha),
                );
            }
            EditMode::Connect => {
                // Draw a small arrow icon at cursor
                if self.pending_connection_source.is_some() {
                    // Show we're waiting for target
                    painter.circle_stroke(
                        pos,
                        8.0 * zoom,
                        egui::Stroke::new(2.0 * zoom, egui::Color32::from_rgba_unmultiplied(255, 165, 0, preview_alpha)),
                    );
                } else {
                    // Show connection start indicator
                    let size = 6.0 * zoom;
                    painter.line_segment(
                        [egui::Pos2::new(pos.x - size, pos.y), egui::Pos2::new(pos.x + size, pos.y)],
                        egui::Stroke::new(2.0 * zoom, egui::Color32::from_rgba_unmultiplied(0, 0, 0, preview_alpha)),
                    );
                    painter.line_segment(
                        [egui::Pos2::new(pos.x, pos.y - size), egui::Pos2::new(pos.x, pos.y + size)],
                        egui::Stroke::new(2.0 * zoom, egui::Color32::from_rgba_unmultiplied(0, 0, 0, preview_alpha)),
                    );
                }
            }
            EditMode::Arrow => {
                // No preview needed for selection mode
            }

            // === Use Case Diagram Previews ===
            EditMode::AddActor => {
                // Draw a ghost stick figure
                let stroke = egui::Stroke::new(1.5, egui::Color32::from_rgba_unmultiplied(0, 0, 0, preview_alpha));
                let head_y = pos.y - 20.0;
                let head_r = 8.0;
                let body_top = head_y + head_r;
                let body_bottom = body_top + 20.0;
                let arm_y = body_top + 8.0;
                let leg_bottom = body_bottom + 18.0;

                painter.circle_stroke(egui::Pos2::new(pos.x, head_y), head_r, stroke);
                painter.line_segment([egui::Pos2::new(pos.x, body_top), egui::Pos2::new(pos.x, body_bottom)], stroke);
                painter.line_segment([egui::Pos2::new(pos.x - 15.0, arm_y), egui::Pos2::new(pos.x + 15.0, arm_y)], stroke);
                painter.line_segment([egui::Pos2::new(pos.x, body_bottom), egui::Pos2::new(pos.x - 12.0, leg_bottom)], stroke);
                painter.line_segment([egui::Pos2::new(pos.x, body_bottom), egui::Pos2::new(pos.x + 12.0, leg_bottom)], stroke);
            }
            EditMode::AddUseCase => {
                // Draw a ghost ellipse
                let radius = egui::Vec2::new(50.0, 30.0);
                painter.add(egui::Shape::ellipse_stroke(
                    pos,
                    radius,
                    egui::Stroke::new(1.0, egui::Color32::from_rgba_unmultiplied(0, 0, 0, preview_alpha)),
                ));
                painter.text(
                    pos,
                    egui::Align2::CENTER_CENTER,
                    "Use Case",
                    egui::FontId::proportional(11.0),
                    egui::Color32::from_rgba_unmultiplied(0, 0, 0, preview_alpha),
                );
            }
            EditMode::AddSystemBoundary => {
                // Draw a ghost rectangle
                let rect = egui::Rect::from_center_size(pos, egui::Vec2::new(150.0, 200.0));
                painter.rect(
                    rect,
                    egui::Rounding::same(4.0),
                    egui::Color32::from_rgba_unmultiplied(245, 245, 245, 80),
                    egui::Stroke::new(1.0, egui::Color32::from_rgba_unmultiplied(0, 0, 0, preview_alpha)),
                );
                painter.text(
                    egui::Pos2::new(pos.x, rect.top() + 15.0),
                    egui::Align2::CENTER_CENTER,
                    "System",
                    egui::FontId::proportional(12.0),
                    egui::Color32::from_rgba_unmultiplied(0, 0, 0, preview_alpha),
                );
            }

            // === Sequence Diagram Previews ===
            EditMode::AddLifeline => {
                // Draw a ghost lifeline
                let head_rect = egui::Rect::from_center_size(
                    egui::Pos2::new(pos.x, pos.y - 40.0),
                    egui::Vec2::new(80.0, 30.0),
                );
                painter.rect(
                    head_rect,
                    egui::Rounding::same(2.0),
                    egui::Color32::from_rgba_unmultiplied(255, 255, 255, preview_alpha),
                    egui::Stroke::new(1.0, egui::Color32::from_rgba_unmultiplied(0, 0, 0, preview_alpha)),
                );
                // Dashed line
                let line_top = head_rect.bottom();
                let line_bottom = pos.y + 60.0;
                let mut y = line_top;
                while y < line_bottom {
                    let end_y = (y + 6.0).min(line_bottom);
                    painter.line_segment(
                        [egui::Pos2::new(pos.x, y), egui::Pos2::new(pos.x, end_y)],
                        egui::Stroke::new(1.0, egui::Color32::from_rgba_unmultiplied(0, 0, 0, preview_alpha)),
                    );
                    y += 10.0;
                }
                painter.text(
                    head_rect.center(),
                    egui::Align2::CENTER_CENTER,
                    "Object",
                    egui::FontId::proportional(11.0),
                    egui::Color32::from_rgba_unmultiplied(0, 0, 0, preview_alpha),
                );
            }

            // === Activity Diagram Previews ===
            EditMode::AddAction => {
                // Draw a ghost action (rounded rectangle)
                let rect = egui::Rect::from_center_size(pos, egui::Vec2::new(100.0, 40.0));
                painter.rect(
                    rect,
                    egui::Rounding::same(10.0),
                    egui::Color32::from_rgba_unmultiplied(200, 230, 255, preview_alpha),
                    egui::Stroke::new(1.0, egui::Color32::from_rgba_unmultiplied(0, 0, 0, preview_alpha)),
                );
                painter.text(
                    pos,
                    egui::Align2::CENTER_CENTER,
                    "Action",
                    egui::FontId::proportional(11.0),
                    egui::Color32::from_rgba_unmultiplied(0, 0, 0, preview_alpha),
                );
            }
            EditMode::AddDecision => {
                // Draw a ghost diamond
                let size = 15.0;
                let points = vec![
                    egui::Pos2::new(pos.x, pos.y - size),
                    egui::Pos2::new(pos.x + size, pos.y),
                    egui::Pos2::new(pos.x, pos.y + size),
                    egui::Pos2::new(pos.x - size, pos.y),
                ];
                painter.add(egui::Shape::convex_polygon(
                    points,
                    egui::Color32::from_rgba_unmultiplied(255, 255, 255, preview_alpha),
                    egui::Stroke::new(1.0, egui::Color32::from_rgba_unmultiplied(0, 0, 0, preview_alpha)),
                ));
            }
            EditMode::AddSendSignal => {
                // Draw a ghost pentagon (send signal)
                let w = 50.0;
                let h = 25.0;
                let points = vec![
                    egui::Pos2::new(pos.x - w/2.0, pos.y - h/2.0),
                    egui::Pos2::new(pos.x + w/2.0 - 10.0, pos.y - h/2.0),
                    egui::Pos2::new(pos.x + w/2.0, pos.y),
                    egui::Pos2::new(pos.x + w/2.0 - 10.0, pos.y + h/2.0),
                    egui::Pos2::new(pos.x - w/2.0, pos.y + h/2.0),
                ];
                painter.add(egui::Shape::convex_polygon(
                    points,
                    egui::Color32::from_rgba_unmultiplied(255, 230, 200, preview_alpha),
                    egui::Stroke::new(1.0, egui::Color32::from_rgba_unmultiplied(0, 0, 0, preview_alpha)),
                ));
            }
            EditMode::AddAcceptEvent => {
                // Draw a ghost concave pentagon
                let w = 50.0;
                let h = 25.0;
                let points = vec![
                    egui::Pos2::new(pos.x - w/2.0, pos.y - h/2.0),
                    egui::Pos2::new(pos.x + w/2.0, pos.y - h/2.0),
                    egui::Pos2::new(pos.x + w/2.0, pos.y + h/2.0),
                    egui::Pos2::new(pos.x - w/2.0, pos.y + h/2.0),
                    egui::Pos2::new(pos.x - w/2.0 + 10.0, pos.y),
                ];
                painter.add(egui::Shape::convex_polygon(
                    points,
                    egui::Color32::from_rgba_unmultiplied(200, 255, 200, preview_alpha),
                    egui::Stroke::new(1.0, egui::Color32::from_rgba_unmultiplied(0, 0, 0, preview_alpha)),
                ));
            }
            EditMode::AddSwimlane => {
                // Draw a ghost swimlane
                let rect = egui::Rect::from_center_size(pos, egui::Vec2::new(100.0, 200.0));
                painter.rect(
                    rect,
                    egui::Rounding::ZERO,
                    egui::Color32::from_rgba_unmultiplied(230, 230, 255, 80),
                    egui::Stroke::new(1.0, egui::Color32::from_rgba_unmultiplied(0, 0, 0, preview_alpha)),
                );
                // Header line
                painter.line_segment(
                    [egui::Pos2::new(rect.left(), rect.top() + 25.0), egui::Pos2::new(rect.right(), rect.top() + 25.0)],
                    egui::Stroke::new(1.0, egui::Color32::from_rgba_unmultiplied(0, 0, 0, preview_alpha)),
                );
                painter.text(
                    egui::Pos2::new(pos.x, rect.top() + 12.0),
                    egui::Align2::CENTER_CENTER,
                    "Lane",
                    egui::FontId::proportional(11.0),
                    egui::Color32::from_rgba_unmultiplied(0, 0, 0, preview_alpha),
                );
            }

            _ => {}
        }
    }

    /// Handle keyboard input
    fn handle_keyboard(&mut self, ctx: &egui::Context) {
        // Don't handle Delete/Backspace if a text field has focus (e.g., properties panel)
        let text_edit_has_focus = ctx.memory(|m| m.focused().is_some()) && ctx.wants_keyboard_input();

        if !text_edit_has_focus && ctx.input(|i| i.key_pressed(egui::Key::Delete) || i.key_pressed(egui::Key::Backspace)) {
            if let Some(state) = self.current_diagram_mut() {
                state.diagram.push_undo();
                state.diagram.delete_selected();
                state.modified = true;
                self.status_message = "Deleted".to_string();
            }
        }

        if ctx.input(|i| i.modifiers.command && i.key_pressed(egui::Key::Z)) {
            if ctx.input(|i| i.modifiers.shift) {
                // Redo
                if let Some(state) = self.current_diagram_mut() {
                    if state.diagram.redo() {
                        self.status_message = "Redo".to_string();
                    }
                }
            } else {
                // Undo
                if let Some(state) = self.current_diagram_mut() {
                    if state.diagram.undo() {
                        self.status_message = "Undo".to_string();
                    }
                }
            }
        }

        // Escape to cancel connection
        if ctx.input(|i| i.key_pressed(egui::Key::Escape)) {
            self.pending_connection_source = None;
            self.set_edit_mode(EditMode::Arrow);
        }

        // Ctrl+S to Save, Ctrl+Shift+S to Save As
        #[cfg(not(target_arch = "wasm32"))]
        if ctx.input(|i| i.modifiers.command && i.key_pressed(egui::Key::S)) {
            if ctx.input(|i| i.modifiers.shift) {
                self.save_as();
            } else {
                self.save();
            }
        }

        // Ctrl+O to Open
        #[cfg(not(target_arch = "wasm32"))]
        if ctx.input(|i| i.modifiers.command && i.key_pressed(egui::Key::O)) {
            self.open();
        }
    }
}

impl eframe::App for JmtApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Handle keyboard input
        self.handle_keyboard(ctx);

        // Top panel - Menu bar
        egui::TopBottomPanel::top("menu_bar").show(ctx, |ui| {
            MenuBar::show(ui, self);
        });

        // Top panel - Toolbar
        egui::TopBottomPanel::top("toolbar").show(ctx, |ui| {
            Toolbar::show(ui, self);
        });

        // Bottom panel - Status bar
        egui::TopBottomPanel::bottom("status_bar").show(ctx, |ui| {
            StatusBar::show(ui, &self.status_message);
        });

        // Right panel - Properties
        egui::SidePanel::right("properties")
            .min_width(200.0)
            .show(ctx, |ui| {
                PropertiesPanel::show(ui, self);
            });

        // Central panel - Canvas with tabs
        egui::CentralPanel::default().show(ctx, |ui| {
            // Diagram tabs
            ui.horizontal(|ui| {
                for (i, diagram_state) in self.diagrams.iter().enumerate() {
                    let name = &diagram_state.diagram.settings.name;
                    let type_icon = diagram_state.diagram.diagram_type.icon();
                    let label = if diagram_state.modified {
                        format!("{} {}*", type_icon, name)
                    } else {
                        format!("{} {}", type_icon, name)
                    };

                    if ui.selectable_label(i == self.active_diagram, &label).clicked() {
                        self.active_diagram = i;
                    }
                }

                // New diagram dropdown
                egui::menu::menu_button(ui, "+ New", |ui| {
                    if ui.button("State Machine").clicked() {
                        self.new_diagram_of_type(DiagramType::StateMachine);
                        ui.close_menu();
                    }
                    if ui.button("Sequence").clicked() {
                        self.new_diagram_of_type(DiagramType::Sequence);
                        ui.close_menu();
                    }
                    if ui.button("Use Case").clicked() {
                        self.new_diagram_of_type(DiagramType::UseCase);
                        ui.close_menu();
                    }
                    if ui.button("Activity").clicked() {
                        self.new_diagram_of_type(DiagramType::Activity);
                        ui.close_menu();
                    }
                });
            });

            ui.separator();

            // Handle Ctrl+MouseWheel for zooming (before ScrollArea consumes the scroll)
            let ctrl_held_for_zoom = ui.input(|i| i.modifiers.ctrl);
            if ctrl_held_for_zoom {
                let scroll_delta = ui.input(|i| i.raw_scroll_delta.y);
                if scroll_delta != 0.0 {
                    // Zoom in/out based on scroll direction
                    let zoom_delta = scroll_delta * ZOOM_WHEEL_STEP / 50.0;
                    self.zoom_by(zoom_delta);
                }
            }

            // Calculate content bounds to determine scroll area size
            let content_bounds = self.current_diagram()
                .map(|s| s.diagram.content_bounds())
                .unwrap_or(jmt_core::geometry::Rect::new(0.0, 0.0, 800.0, 600.0));

            let zoom = self.zoom_level;

            // Canvas with scroll bars
            egui::ScrollArea::both()
                .auto_shrink([false, false])
                .scroll_bar_visibility(egui::scroll_area::ScrollBarVisibility::AlwaysVisible)
                .show(ui, |ui| {
                    // Make the canvas at least as big as content bounds, scaled by zoom
                    let canvas_width = (content_bounds.x2 * zoom).max(ui.available_width());
                    let canvas_height = (content_bounds.y2 * zoom).max(ui.available_height());
                    let canvas_size = egui::vec2(canvas_width, canvas_height);

                    let (response, painter) = ui.allocate_painter(canvas_size, egui::Sense::click_and_drag());

                    // Draw background with light gray to distinguish from scrollbar
                    painter.rect_filled(response.rect, 0.0, egui::Color32::from_gray(252));

            // Track cursor position for preview
            self.cursor_pos = response.hover_pos();

            // Draw the diagram with zoom
            if let Some(state) = self.current_diagram_mut() {
                state.canvas.render_with_zoom(&state.diagram, &painter, response.rect, zoom);
            }

            // Draw cursor preview for add modes
            if let Some(pos) = self.cursor_pos {
                self.render_cursor_preview(&painter, pos, zoom);
            }

            // Handle right-click to exit add/connect mode
            if response.secondary_clicked() {
                if self.edit_mode.is_add_node() || self.edit_mode == EditMode::Connect {
                    self.pending_connection_source = None; // Clear any pending connection
                    self.set_edit_mode(EditMode::Arrow);
                    self.status_message = "Switched to Arrow mode".to_string();
                }
            }

            // Canvas origin in screen space (for coordinate transformation)
            let canvas_origin = response.rect.min;

            // Handle mouse clicks with custom double-click detection (500ms window)
            // Double-click in add mode will add element AND switch back to arrow mode
            let ctrl_held = ui.input(|i| i.modifiers.ctrl);
            if response.clicked() {
                if let Some(pos) = response.interact_pointer_pos() {
                    // Transform screen coordinates to diagram coordinates
                    let diagram_pos = egui::Pos2::new(
                        (pos.x - canvas_origin.x) / zoom,
                        (pos.y - canvas_origin.y) / zoom
                    );
                    let now = Instant::now();

                    // Check if this is a double-click (within time and distance threshold)
                    let is_double_click = if let (Some(last_time), Some(last_pos)) = (self.last_click_time, self.last_click_pos) {
                        let time_diff = now.duration_since(last_time).as_millis();
                        let distance = ((pos.x - last_pos.x).powi(2) + (pos.y - last_pos.y).powi(2)).sqrt();
                        time_diff <= DOUBLE_CLICK_TIME_MS && distance <= DOUBLE_CLICK_DISTANCE
                    } else {
                        false
                    };

                    if is_double_click {
                        // Double-click detected: the first click already added the node,
                        // so just switch to Arrow mode without adding another node
                        self.last_click_time = None;
                        self.last_click_pos = None;
                        if self.edit_mode.is_add_node() {
                            self.set_edit_mode(EditMode::Arrow);
                            self.status_message = "Switched to Arrow mode".to_string();
                        } else {
                            // For non-add modes, handle normally (e.g., Arrow mode selection)
                            self.handle_canvas_click(diagram_pos, true, ctrl_held);
                        }
                    } else {
                        // Single click: record time/pos for potential double-click detection
                        self.last_click_time = Some(now);
                        self.last_click_pos = Some(pos);
                        self.handle_canvas_click(diagram_pos, false, ctrl_held);
                    }
                }
            }

            // Handle drag start - determine if we're resizing, dragging nodes, or marquee selecting
            if response.drag_started() {
                if let Some(pos) = response.interact_pointer_pos() {
                    // Transform screen coordinates to diagram coordinates
                    let diagram_pos = egui::Pos2::new(
                        (pos.x - canvas_origin.x) / zoom,
                        (pos.y - canvas_origin.y) / zoom
                    );
                    let point = Point::new(diagram_pos.x, diagram_pos.y);
                    let corner_margin = 10.0; // Size of corner hit area

                    // First, check if we clicked on a corner of a selected resizable node
                    let mut corner_info: Option<(NodeId, Corner)> = None;
                    if let Some(state) = self.current_diagram() {
                        for node_id in state.diagram.selected_nodes() {
                            if let Some(node) = state.diagram.find_node(node_id) {
                                if node.can_resize() {
                                    let corner = node.get_corner(point, corner_margin);
                                    if corner != Corner::None {
                                        corner_info = Some((node_id, corner));
                                        break;
                                    }
                                }
                            }
                        }
                    }

                    if let Some((node_id, corner)) = corner_info {
                        // We're starting a resize operation
                        if self.edit_mode != EditMode::Arrow {
                            self.set_edit_mode(EditMode::Arrow);
                        }
                        if let Some(state) = self.current_diagram_mut() {
                            state.diagram.push_undo();
                        }
                        self.resize_state.start(node_id, corner);
                        self.dragging_nodes = false;
                        self.selection_rect.clear();
                        self.status_message = "Resizing...".to_string();
                    } else {
                        // Check if we clicked on any element (for dragging)
                        let clicked_element_id = self.current_diagram()
                            .and_then(|state| state.diagram.find_element_at(point));

                        if let Some(element_id) = clicked_element_id {
                            // Dragging on an element - switch to Arrow mode and start dragging
                            if self.edit_mode != EditMode::Arrow {
                                self.set_edit_mode(EditMode::Arrow);
                            }

                            // Select the element if not already selected
                            if let Some(state) = self.current_diagram_mut() {
                                let already_selected = state.diagram.selected_elements_in_order().contains(&element_id);
                                if !already_selected {
                                    // Select this element (this allows click-and-drag in one motion)
                                    state.diagram.select_element(element_id);
                                }
                                // Push undo before we start moving
                                state.diagram.push_undo();
                            }
                            self.dragging_nodes = true;
                            self.selection_rect.clear();
                        } else if self.edit_mode == EditMode::Arrow {
                            // We're starting a marquee selection (only in Arrow mode)
                            // Store screen coordinates for the selection rectangle
                            self.dragging_nodes = false;
                            self.selection_rect.start = Some(diagram_pos);
                            self.selection_rect.current = Some(diagram_pos);
                            // Clear current selection when starting a new marquee
                            if let Some(state) = self.current_diagram_mut() {
                                state.diagram.clear_selection();
                            }
                        } else if self.edit_mode == EditMode::Lasso {
                            // We're starting a lasso selection
                            self.dragging_nodes = false;
                            self.lasso_points.clear();
                            self.lasso_points.push(diagram_pos);
                            // Clear current selection when starting a new lasso
                            if let Some(state) = self.current_diagram_mut() {
                                state.diagram.clear_selection();
                            }
                        }
                    }
                }
            }

            // Handle dragging
            if response.dragged() {
                if let Some(pos) = response.interact_pointer_pos() {
                    // Transform screen coordinates to diagram coordinates
                    let diagram_pos = egui::Pos2::new(
                        (pos.x - canvas_origin.x) / zoom,
                        (pos.y - canvas_origin.y) / zoom
                    );
                    let delta = response.drag_delta();
                    // Scale delta to diagram space
                    let diagram_delta_x = delta.x / zoom;
                    let diagram_delta_y = delta.y / zoom;

                    if self.resize_state.is_active() {
                        // Handle resize
                        let node_id = self.resize_state.node_id.unwrap();
                        let corner = self.resize_state.corner;
                        let min_width = 40.0;
                        let min_height = 30.0;

                        if let Some(state) = self.current_diagram_mut() {
                            if let Some(node) = state.diagram.find_node_mut(node_id) {
                                node.resize_from_corner(corner, diagram_delta_x, diagram_delta_y, min_width, min_height);
                            }
                            state.diagram.recalculate_connections();
                            state.modified = true;
                        }
                    } else if self.edit_mode == EditMode::Arrow {
                        if self.dragging_nodes {
                            // Move selected elements (nodes, lifelines, actors, etc.)
                            if let Some(state) = self.current_diagram_mut() {
                                let selected = state.diagram.selected_elements_in_order();
                                if !selected.is_empty() {
                                    for id in selected {
                                        state.diagram.translate_element(id, diagram_delta_x, diagram_delta_y);
                                    }
                                    state.diagram.recalculate_connections();
                                    state.modified = true;
                                }
                            }
                        } else {
                            // Update marquee selection rectangle (in diagram space)
                            self.selection_rect.current = Some(diagram_pos);
                        }
                    } else if self.edit_mode == EditMode::Lasso {
                        // Add points to the lasso path (with some distance threshold to avoid too many points)
                        if let Some(last) = self.lasso_points.last() {
                            let dist = ((diagram_pos.x - last.x).powi(2) + (diagram_pos.y - last.y).powi(2)).sqrt();
                            if dist > 3.0 {
                                self.lasso_points.push(diagram_pos);
                            }
                        }
                    }
                }
            }

            // Draw selection rectangle if active (scale to screen space)
            if self.selection_rect.is_active() {
                if let Some(rect) = self.selection_rect.to_egui_rect() {
                    // Scale the rectangle to screen space and add canvas offset
                    let screen_rect = egui::Rect::from_min_max(
                        egui::Pos2::new(rect.min.x * zoom + canvas_origin.x, rect.min.y * zoom + canvas_origin.y),
                        egui::Pos2::new(rect.max.x * zoom + canvas_origin.x, rect.max.y * zoom + canvas_origin.y),
                    );
                    // Draw selection rectangle with semi-transparent fill
                    painter.rect(
                        screen_rect,
                        egui::Rounding::ZERO,
                        egui::Color32::from_rgba_unmultiplied(100, 150, 255, 50),
                        egui::Stroke::new(zoom, egui::Color32::from_rgb(100, 150, 255)),
                    );
                }
            }

            // Draw lasso path if active (scale to screen space)
            if self.lasso_points.len() > 1 {
                // Scale lasso points to screen space and add canvas offset
                let screen_points: Vec<egui::Pos2> = self.lasso_points.iter()
                    .map(|p| egui::Pos2::new(p.x * zoom + canvas_origin.x, p.y * zoom + canvas_origin.y))
                    .collect();

                // Draw the lasso line
                painter.add(egui::Shape::line(
                    screen_points.clone(),
                    egui::Stroke::new(2.0 * zoom, egui::Color32::from_rgb(100, 150, 255)),
                ));

                // Draw closing line (dashed effect using dotted line to start point)
                if screen_points.len() > 2 {
                    if let (Some(first), Some(last)) = (screen_points.first(), screen_points.last()) {
                        painter.line_segment(
                            [*last, *first],
                            egui::Stroke::new(zoom, egui::Color32::from_rgba_unmultiplied(100, 150, 255, 128)),
                        );
                    }
                }
            }

            // Handle drag end
            if response.drag_stopped() {
                if self.resize_state.is_active() {
                    // Finished resizing
                    self.resize_state.clear();
                    self.status_message = "Ready".to_string();
                } else if self.edit_mode == EditMode::Arrow {
                    if self.dragging_nodes {
                        // Undo was already pushed at drag start, nothing to do here
                    } else {
                        // Complete marquee selection
                        if let Some(rect) = self.selection_rect.to_core_rect() {
                            if let Some(state) = self.current_diagram_mut() {
                                state.diagram.select_elements_in_rect(&rect);
                                let count = state.diagram.selected_elements_in_order().len();
                                if count > 0 {
                                    self.status_message = format!("Selected {} node(s)", count);
                                } else {
                                    self.status_message = "Ready".to_string();
                                }
                            }
                        }
                    }
                } else if self.edit_mode == EditMode::Lasso {
                    // Complete lasso selection
                    if self.lasso_points.len() >= 3 {
                        // Convert lasso points to core Points
                        let polygon: Vec<Point> = self.lasso_points
                            .iter()
                            .map(|p| Point::new(p.x, p.y))
                            .collect();

                        if let Some(state) = self.current_diagram_mut() {
                            state.diagram.select_elements_in_polygon(&polygon);
                            let count = state.diagram.selected_elements_in_order().len();
                            if count > 0 {
                                self.status_message = format!("Selected {} element(s)", count);
                            } else {
                                self.status_message = "Ready".to_string();
                            }
                        }
                    }
                    self.lasso_points.clear();
                }

                // Clear selection rect and reset state
                self.selection_rect.clear();
                self.dragging_nodes = false;
            }
            }); // End ScrollArea
        });
    }
}
