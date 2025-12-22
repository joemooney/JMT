//! Diagram - the top-level container for UML diagrams

use serde::{Deserialize, Serialize};
use uuid::Uuid;
use crate::geometry::{Point, Rect};
use crate::node::{Node, NodeId, NodeType, State, PseudoState, PseudoStateKind, Region};
use crate::connection::{Connection, ConnectionId};
use crate::settings::DiagramSettings;
use crate::diagram_type::DiagramType;

// Sequence diagram imports
use crate::sequence::{Lifeline, Message, Activation, CombinedFragment};

// Use case diagram imports
use crate::usecase::{Actor, UseCase, SystemBoundary, UseCaseRelationship};

// Activity diagram imports
use crate::activity::{Action, Swimlane, ControlFlow, ObjectNode, ActivityPartition};

/// How to display the diagram title
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum TitleStyle {
    /// Don't display the title
    #[default]
    None,
    /// Display as a simple header at the top
    Header,
    /// Display in a UML frame notation (dog-eared rectangle in top-left)
    Frame,
}

impl TitleStyle {
    /// Get display name for UI
    pub fn display_name(&self) -> &'static str {
        match self {
            TitleStyle::None => "None",
            TitleStyle::Header => "Header",
            TitleStyle::Frame => "Frame (UML)",
        }
    }

    /// Get all variants for UI iteration
    pub fn all() -> &'static [TitleStyle] {
        &[TitleStyle::None, TitleStyle::Header, TitleStyle::Frame]
    }
}

/// Reference to a parent diagram that uses this diagram as a sub-statemachine
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParentReference {
    /// Path to the parent diagram file
    pub diagram_path: String,
    /// Name of the state in the parent diagram that references this sub-statemachine
    pub state_name: String,
}

/// A complete UML diagram (state machine, sequence, use case, or activity)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Diagram {
    /// Unique identifier
    pub id: Uuid,
    /// Diagram title
    #[serde(default)]
    pub title: String,
    /// How to display the title
    #[serde(default)]
    pub title_style: TitleStyle,
    /// Diagram type
    #[serde(default)]
    pub diagram_type: DiagramType,
    /// Diagram settings
    pub settings: DiagramSettings,
    /// Root state (contains all other nodes) - for state machine diagrams
    pub root_state: State,
    /// All nodes in the diagram (state machine nodes)
    nodes: Vec<Node>,
    /// All connections in the diagram (state machine transitions)
    connections: Vec<Connection>,

    // === Sequence Diagram elements ===
    /// Lifelines in sequence diagrams
    #[serde(default)]
    pub lifelines: Vec<Lifeline>,
    /// Messages between lifelines
    #[serde(default)]
    pub messages: Vec<Message>,
    /// Activation boxes on lifelines
    #[serde(default)]
    pub activations: Vec<Activation>,
    /// Combined fragments (alt, opt, loop, etc.)
    #[serde(default)]
    pub fragments: Vec<CombinedFragment>,

    // === Use Case Diagram elements ===
    /// Actors in use case diagrams
    #[serde(default)]
    pub actors: Vec<Actor>,
    /// Use cases
    #[serde(default)]
    pub use_cases: Vec<UseCase>,
    /// System boundaries
    #[serde(default)]
    pub system_boundaries: Vec<SystemBoundary>,
    /// Use case relationships (association, include, extend, generalization)
    #[serde(default)]
    pub uc_relationships: Vec<UseCaseRelationship>,

    // === Activity Diagram elements ===
    /// Actions in activity diagrams
    #[serde(default)]
    pub actions: Vec<Action>,
    /// Swimlanes/partitions
    #[serde(default)]
    pub swimlanes: Vec<Swimlane>,
    /// Activity partitions (containers for swimlanes)
    #[serde(default)]
    pub partitions: Vec<ActivityPartition>,
    /// Object nodes (data stores, pins, etc.)
    #[serde(default)]
    pub object_nodes: Vec<ObjectNode>,
    /// Control flows between activity nodes
    #[serde(default)]
    pub control_flows: Vec<ControlFlow>,

    /// References to parent diagrams that use this as a sub-statemachine
    #[serde(default)]
    pub parent_references: Vec<ParentReference>,

    /// Undo history (serialized diagram snapshots)
    #[serde(skip)]
    undo_stack: Vec<String>,
    /// Redo history
    #[serde(skip)]
    redo_stack: Vec<String>,
    /// Maximum undo levels
    #[serde(skip)]
    max_undo_levels: usize,
    /// Selection order - tracks the order in which nodes were selected
    #[serde(skip)]
    selection_order: Vec<NodeId>,
    /// True if selection order was explicitly set via Ctrl+Click (use selection order)
    /// False if selection was via marquee/lasso (use position order for align/distribute)
    #[serde(skip)]
    explicit_selection_order: bool,
}

impl Diagram {
    /// Create a new empty state machine diagram
    pub fn new(name: &str) -> Self {
        Self::new_with_type(name, DiagramType::StateMachine)
    }

    /// Create a new empty diagram of a specific type
    pub fn new_with_type(name: &str, diagram_type: DiagramType) -> Self {
        let mut root_state = State::new("Root", 50.0, 50.0, 700.0, 500.0);
        root_state.add_region("default");

        Self {
            id: Uuid::new_v4(),
            title: String::new(),
            title_style: TitleStyle::None,
            diagram_type,
            settings: DiagramSettings::new(name),
            root_state,
            nodes: Vec::new(),
            connections: Vec::new(),
            // Sequence
            lifelines: Vec::new(),
            messages: Vec::new(),
            activations: Vec::new(),
            fragments: Vec::new(),
            // Use case
            actors: Vec::new(),
            use_cases: Vec::new(),
            system_boundaries: Vec::new(),
            uc_relationships: Vec::new(),
            // Activity
            actions: Vec::new(),
            swimlanes: Vec::new(),
            partitions: Vec::new(),
            object_nodes: Vec::new(),
            control_flows: Vec::new(),
            // Sub-statemachine parent tracking
            parent_references: Vec::new(),
            // Undo
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
            max_undo_levels: 50,
            // Selection tracking
            selection_order: Vec::new(),
            explicit_selection_order: false,
        }
    }

    /// Create a new sequence diagram
    pub fn new_sequence(name: &str) -> Self {
        Self::new_with_type(name, DiagramType::Sequence)
    }

    /// Create a new use case diagram
    pub fn new_use_case(name: &str) -> Self {
        Self::new_with_type(name, DiagramType::UseCase)
    }

    /// Create a new activity diagram
    pub fn new_activity(name: &str) -> Self {
        Self::new_with_type(name, DiagramType::Activity)
    }

    /// Calculate the bounding rectangle of all content in the diagram
    /// Returns (min_x, min_y, max_x, max_y) with some padding
    pub fn content_bounds(&self) -> Rect {
        const PADDING: f32 = 50.0;
        const MIN_SIZE: f32 = 800.0;

        let mut min_x = f32::MAX;
        let mut min_y = f32::MAX;
        let mut max_x = f32::MIN;
        let mut max_y = f32::MIN;

        let mut has_content = false;

        // State machine nodes
        for node in &self.nodes {
            let bounds = node.bounds();
            min_x = min_x.min(bounds.x1);
            min_y = min_y.min(bounds.y1);
            max_x = max_x.max(bounds.x2);
            max_y = max_y.max(bounds.y2);
            has_content = true;
        }

        // Sequence diagram lifelines
        for lifeline in &self.lifelines {
            let bounds = lifeline.head_bounds();
            min_x = min_x.min(bounds.x1);
            min_y = min_y.min(bounds.y1);
            max_x = max_x.max(bounds.x2);
            // Lifelines extend downward
            max_y = max_y.max(lifeline.y + lifeline.head_height + lifeline.line_length);
            has_content = true;
        }

        // Use case actors
        for actor in &self.actors {
            min_x = min_x.min(actor.x - 20.0);
            min_y = min_y.min(actor.y);
            max_x = max_x.max(actor.x + 20.0);
            max_y = max_y.max(actor.y + actor.height);
            has_content = true;
        }

        // Use cases
        for uc in &self.use_cases {
            min_x = min_x.min(uc.bounds.x1);
            min_y = min_y.min(uc.bounds.y1);
            max_x = max_x.max(uc.bounds.x2);
            max_y = max_y.max(uc.bounds.y2);
            has_content = true;
        }

        // System boundaries
        for sb in &self.system_boundaries {
            min_x = min_x.min(sb.bounds.x1);
            min_y = min_y.min(sb.bounds.y1);
            max_x = max_x.max(sb.bounds.x2);
            max_y = max_y.max(sb.bounds.y2);
            has_content = true;
        }

        // Activity actions
        for action in &self.actions {
            min_x = min_x.min(action.bounds.x1);
            min_y = min_y.min(action.bounds.y1);
            max_x = max_x.max(action.bounds.x2);
            max_y = max_y.max(action.bounds.y2);
            has_content = true;
        }

        // Swimlanes
        for swimlane in &self.swimlanes {
            min_x = min_x.min(swimlane.bounds.x1);
            min_y = min_y.min(swimlane.bounds.y1);
            max_x = max_x.max(swimlane.bounds.x2);
            max_y = max_y.max(swimlane.bounds.y2);
            has_content = true;
        }

        // Object nodes
        for obj in &self.object_nodes {
            min_x = min_x.min(obj.bounds.x1);
            min_y = min_y.min(obj.bounds.y1);
            max_x = max_x.max(obj.bounds.x2);
            max_y = max_y.max(obj.bounds.y2);
            has_content = true;
        }

        if !has_content {
            // Default bounds if no content
            return Rect::new(0.0, 0.0, MIN_SIZE, MIN_SIZE);
        }

        // Add padding and ensure minimum size
        Rect::new(
            (min_x - PADDING).min(0.0),
            (min_y - PADDING).min(0.0),
            (max_x + PADDING).max(MIN_SIZE),
            (max_y + PADDING).max(MIN_SIZE),
        )
    }

    /// Get all nodes
    pub fn nodes(&self) -> &[Node] {
        &self.nodes
    }

    /// Check if a node is visible (not hidden inside a collapsed sub-statemachine)
    pub fn is_node_visible(&self, node_id: NodeId) -> bool {
        let Some(node) = self.find_node(node_id) else {
            return false;
        };

        // Check if this node's parent region belongs to a collapsed sub-statemachine
        if let Some(region_id) = node.parent_region_id() {
            if let Some(parent_state) = self.find_region_parent_state(region_id) {
                // If parent has sub-statemachine and is not expanded, hide this node
                if parent_state.has_substatemachine() && !parent_state.show_expanded {
                    return false;
                }
            }
        }
        true
    }

    /// Check if a connection is visible (both endpoints are visible)
    pub fn is_connection_visible(&self, conn: &Connection) -> bool {
        self.is_node_visible(conn.source_id) && self.is_node_visible(conn.target_id)
    }

    /// Get connections in render order, excluding those to hidden nodes
    pub fn connections_in_render_order(&self) -> Vec<&Connection> {
        self.connections.iter()
            .filter(|c| self.is_connection_visible(c))
            .collect()
    }

    /// Get nodes in proper render order (largest first, smallest last)
    /// This ensures smaller nodes are drawn on top of larger nodes
    /// Excludes children of collapsed sub-statemachines
    pub fn nodes_in_render_order(&self) -> Vec<&Node> {
        let mut sorted_nodes: Vec<&Node> = self.nodes.iter()
            .filter(|n| self.is_node_visible(n.id()))
            .collect();
        // Sort by area descending (largest first = rendered in background)
        sorted_nodes.sort_by(|a, b| {
            let area_a = a.bounds().width() * a.bounds().height();
            let area_b = b.bounds().width() * b.bounds().height();
            // Descending order: larger nodes first
            area_b.partial_cmp(&area_a).unwrap_or(std::cmp::Ordering::Equal)
        });
        sorted_nodes
    }

    /// Get all connections
    pub fn connections(&self) -> &[Connection] {
        &self.connections
    }

    /// Get a mutable reference to all connections
    pub fn connections_mut(&mut self) -> &mut Vec<Connection> {
        &mut self.connections
    }

    /// Find a node by ID
    pub fn find_node(&self, id: NodeId) -> Option<&Node> {
        self.nodes.iter().find(|n| n.id() == id)
    }

    /// Find a mutable node by ID
    pub fn find_node_mut(&mut self, id: NodeId) -> Option<&mut Node> {
        self.nodes.iter_mut().find(|n| n.id() == id)
    }

    /// Find a connection by ID
    pub fn find_connection(&self, id: ConnectionId) -> Option<&Connection> {
        self.connections.iter().find(|c| c.id == id)
    }

    /// Find a mutable connection by ID
    pub fn find_connection_mut(&mut self, id: ConnectionId) -> Option<&mut Connection> {
        self.connections.iter_mut().find(|c| c.id == id)
    }

    /// Find a node at the given position
    pub fn find_node_at(&self, pos: Point) -> Option<NodeId> {
        // Find the innermost (smallest) node at this point
        // This ensures clicking on a substate selects it, not its parent
        let mut best_match: Option<(NodeId, f32)> = None; // (node_id, area)

        for node in &self.nodes {
            if node.contains_point(pos) {
                let bounds = node.bounds();
                let area = bounds.width() * bounds.height();
                if best_match.is_none() || area < best_match.unwrap().1 {
                    best_match = Some((node.id(), area));
                }
            }
        }

        best_match.map(|(id, _)| id)
    }

    /// Find a connection at the given position
    pub fn find_connection_at(&self, pos: Point, tolerance: f32) -> Option<ConnectionId> {
        self.connections
            .iter()
            .find(|c| c.is_near_point(pos, tolerance))
            .map(|c| c.id)
    }

    /// Find a connection label at the given position
    /// Returns the connection ID if a label was clicked
    pub fn find_connection_label_at(&self, pos: Point) -> Option<ConnectionId> {
        // Check each connection's label
        for conn in &self.connections {
            let label = conn.label();
            if !label.is_empty() {
                // Estimate label dimensions (10px font, ~6px per character)
                let label_width = label.len() as f32 * 6.0;
                let label_height = 12.0;

                if conn.is_near_label(pos, label_width, label_height) {
                    return Some(conn.id);
                }
            }
        }
        None
    }

    /// Select a connection label (deselects everything else)
    pub fn select_connection_label(&mut self, id: ConnectionId) {
        self.clear_selection();
        if let Some(conn) = self.find_connection_mut(id) {
            conn.label_selected = true;
            conn.selected = true;  // Also select the connection
        }
    }

    /// Get the connection with a selected label
    pub fn selected_connection_label(&self) -> Option<ConnectionId> {
        self.connections.iter()
            .find(|c| c.label_selected)
            .map(|c| c.id)
    }

    /// Update a connection's label offset
    pub fn set_connection_label_offset(&mut self, id: ConnectionId, offset: Option<(f32, f32)>) {
        if let Some(conn) = self.find_connection_mut(id) {
            conn.set_label_offset(offset);
        }
    }

    /// Find all nodes fully contained within a rectangle (all four corners inside)
    pub fn find_nodes_in_rect(&self, rect: &Rect) -> Vec<NodeId> {
        self.nodes
            .iter()
            .filter(|n| rect.contains_rect(n.bounds()))
            .map(|n| n.id())
            .collect()
    }

    /// Find any selectable element at the given position based on diagram type
    /// Returns the element ID if found
    pub fn find_element_at(&self, pos: Point) -> Option<Uuid> {
        match self.diagram_type {
            DiagramType::StateMachine => {
                self.find_node_at(pos)
            }
            DiagramType::Sequence => {
                // Check lifelines
                if let Some(id) = self.find_lifeline_at(pos) {
                    return Some(id);
                }
                // TODO: Check fragments, activations
                None
            }
            DiagramType::UseCase => {
                // Check actors
                if let Some(id) = self.find_actor_at(pos) {
                    return Some(id);
                }
                // Check use cases
                if let Some(id) = self.find_use_case_at(pos) {
                    return Some(id);
                }
                // Check system boundaries
                if let Some(id) = self.find_system_boundary_at(pos) {
                    return Some(id);
                }
                None
            }
            DiagramType::Activity => {
                // Check actions
                if let Some(id) = self.find_action_at(pos) {
                    return Some(id);
                }
                // Check swimlanes
                if let Some(id) = self.find_swimlane_at(pos) {
                    return Some(id);
                }
                // Check object nodes
                if let Some(id) = self.find_object_node_at(pos) {
                    return Some(id);
                }
                None
            }
        }
    }

    /// Select any element by ID (works across all diagram types)
    pub fn select_element(&mut self, id: Uuid) {
        self.clear_selection();

        // Try state machine nodes
        if let Some(node) = self.find_node_mut(id) {
            node.set_focus(true);
            self.selection_order.push(id);
            return;
        }

        // Try sequence elements
        if let Some(lifeline) = self.lifelines.iter_mut().find(|l| l.id == id) {
            lifeline.has_focus = true;
            self.selection_order.push(id);
            return;
        }

        // Try use case elements
        if let Some(actor) = self.actors.iter_mut().find(|a| a.id == id) {
            actor.has_focus = true;
            self.selection_order.push(id);
            return;
        }
        if let Some(uc) = self.use_cases.iter_mut().find(|u| u.id == id) {
            uc.has_focus = true;
            self.selection_order.push(id);
            return;
        }
        if let Some(sb) = self.system_boundaries.iter_mut().find(|s| s.id == id) {
            sb.has_focus = true;
            self.selection_order.push(id);
            return;
        }

        // Try activity elements
        if let Some(action) = self.actions.iter_mut().find(|a| a.id == id) {
            action.has_focus = true;
            self.selection_order.push(id);
            return;
        }
        if let Some(swimlane) = self.swimlanes.iter_mut().find(|s| s.id == id) {
            swimlane.has_focus = true;
            self.selection_order.push(id);
            return;
        }
        if let Some(obj) = self.object_nodes.iter_mut().find(|o| o.id == id) {
            obj.has_focus = true;
            self.selection_order.push(id);
            return;
        }
    }

    /// Toggle element selection by ID (works across all diagram types)
    /// When adding via Ctrl+Click, marks as explicit ordering
    pub fn toggle_element_selection(&mut self, id: Uuid) {
        // Try state machine nodes
        if let Some(node) = self.find_node_mut(id) {
            let selected = node.has_focus();
            if selected {
                node.set_focus(false);
                self.selection_order.retain(|&x| x != id);
            } else {
                node.set_focus(true);
                self.selection_order.push(id);
                self.explicit_selection_order = true;
            }
            return;
        }

        // Try sequence elements
        if let Some(lifeline) = self.lifelines.iter_mut().find(|l| l.id == id) {
            if lifeline.has_focus {
                lifeline.has_focus = false;
                self.selection_order.retain(|&x| x != id);
            } else {
                lifeline.has_focus = true;
                self.selection_order.push(id);
                self.explicit_selection_order = true;
            }
            return;
        }

        // Try use case elements
        if let Some(actor) = self.actors.iter_mut().find(|a| a.id == id) {
            if actor.has_focus {
                actor.has_focus = false;
                self.selection_order.retain(|&x| x != id);
            } else {
                actor.has_focus = true;
                self.selection_order.push(id);
                self.explicit_selection_order = true;
            }
            return;
        }
        if let Some(uc) = self.use_cases.iter_mut().find(|u| u.id == id) {
            if uc.has_focus {
                uc.has_focus = false;
                self.selection_order.retain(|&x| x != id);
            } else {
                uc.has_focus = true;
                self.selection_order.push(id);
                self.explicit_selection_order = true;
            }
            return;
        }
        if let Some(sb) = self.system_boundaries.iter_mut().find(|s| s.id == id) {
            if sb.has_focus {
                sb.has_focus = false;
                self.selection_order.retain(|&x| x != id);
            } else {
                sb.has_focus = true;
                self.selection_order.push(id);
                self.explicit_selection_order = true;
            }
            return;
        }

        // Try activity elements
        if let Some(action) = self.actions.iter_mut().find(|a| a.id == id) {
            if action.has_focus {
                action.has_focus = false;
                self.selection_order.retain(|&x| x != id);
            } else {
                action.has_focus = true;
                self.selection_order.push(id);
                self.explicit_selection_order = true;
            }
            return;
        }
        if let Some(swimlane) = self.swimlanes.iter_mut().find(|s| s.id == id) {
            if swimlane.has_focus {
                swimlane.has_focus = false;
                self.selection_order.retain(|&x| x != id);
            } else {
                swimlane.has_focus = true;
                self.selection_order.push(id);
                self.explicit_selection_order = true;
            }
            return;
        }
        if let Some(obj) = self.object_nodes.iter_mut().find(|o| o.id == id) {
            if obj.has_focus {
                obj.has_focus = false;
                self.selection_order.retain(|&x| x != id);
            } else {
                obj.has_focus = true;
                self.selection_order.push(id);
                self.explicit_selection_order = true;
            }
            return;
        }
    }

    /// Translate an element by ID (works across all diagram types)
    /// Returns true if element was found and translated
    pub fn translate_element(&mut self, id: Uuid, dx: f32, dy: f32) -> bool {
        // State machine nodes
        if let Some(node) = self.find_node_mut(id) {
            node.translate(dx, dy);
            return true;
        }
        // Sequence
        if let Some(l) = self.lifelines.iter_mut().find(|l| l.id == id) {
            l.translate(dx, dy);
            return true;
        }
        // Use case
        if let Some(a) = self.actors.iter_mut().find(|a| a.id == id) {
            a.translate(dx, dy);
            return true;
        }
        if let Some(u) = self.use_cases.iter_mut().find(|u| u.id == id) {
            u.translate(dx, dy);
            return true;
        }
        if let Some(s) = self.system_boundaries.iter_mut().find(|s| s.id == id) {
            s.translate(dx, dy);
            return true;
        }
        // Activity
        if let Some(a) = self.actions.iter_mut().find(|a| a.id == id) {
            a.translate(dx, dy);
            return true;
        }
        if let Some(s) = self.swimlanes.iter_mut().find(|s| s.id == id) {
            s.translate(dx, dy);
            return true;
        }
        if let Some(o) = self.object_nodes.iter_mut().find(|o| o.id == id) {
            o.translate(dx, dy);
            return true;
        }
        false
    }

    /// Translate a node and all its children (recursive)
    /// Used when dragging a parent state - children should move with it
    pub fn translate_node_with_children(&mut self, node_id: NodeId, dx: f32, dy: f32) {
        // First, collect all children IDs (from all regions of this node)
        let child_ids: Vec<NodeId> = self.find_node(node_id)
            .and_then(|n| n.as_state())
            .map(|state| {
                state.regions.iter()
                    .flat_map(|r| r.children.iter().copied())
                    .collect()
            })
            .unwrap_or_default();

        // Translate the node itself
        if let Some(node) = self.find_node_mut(node_id) {
            node.translate(dx, dy);
        }

        // Recursively translate all children
        for child_id in child_ids {
            self.translate_node_with_children(child_id, dx, dy);
        }
    }

    /// Get direct children of a state (nodes in its regions)
    pub fn get_children_of_node(&self, node_id: NodeId) -> Vec<NodeId> {
        self.find_node(node_id)
            .and_then(|n| n.as_state())
            .map(|state| {
                state.regions.iter()
                    .flat_map(|r| r.children.iter().copied())
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Get all descendants of a state (children, grandchildren, etc.)
    pub fn get_all_descendants(&self, node_id: NodeId) -> Vec<NodeId> {
        let mut descendants = Vec::new();
        let mut to_visit = vec![node_id];

        while let Some(current) = to_visit.pop() {
            let children = self.get_children_of_node(current);
            for child in children {
                descendants.push(child);
                to_visit.push(child);
            }
        }

        descendants
    }

    /// Check if a node is a descendant of another node
    pub fn is_descendant_of(&self, node_id: NodeId, ancestor_id: NodeId) -> bool {
        let descendants = self.get_all_descendants(ancestor_id);
        descendants.contains(&node_id)
    }

    /// Extract child nodes and connections for a state's sub-statemachine.
    /// Returns (nodes, connections) with positions translated so the bounding box starts at (margin, margin).
    /// The nodes are cloned with their parent_region_id cleared (they become top-level nodes).
    pub fn extract_substatemachine_contents(&self, state_id: NodeId) -> (Vec<Node>, Vec<Connection>) {
        use std::collections::HashSet;

        // Get all child node IDs (direct children only, not descendants)
        let child_ids: HashSet<NodeId> = self.get_children_of_node(state_id)
            .into_iter()
            .collect();

        if child_ids.is_empty() {
            return (Vec::new(), Vec::new());
        }

        // Clone the child nodes
        let mut nodes: Vec<Node> = child_ids.iter()
            .filter_map(|&id| self.find_node(id).cloned())
            .collect();

        // Calculate the bounding box of all child nodes
        let min_x = nodes.iter().map(|n| n.bounds().x1).fold(f32::MAX, f32::min);
        let min_y = nodes.iter().map(|n| n.bounds().y1).fold(f32::MAX, f32::min);

        // Translate nodes so they start at a margin from origin
        let margin = 50.0;
        let offset_x = margin - min_x;
        let offset_y = margin - min_y;

        for node in &mut nodes {
            // Translate bounds
            let bounds = node.bounds_mut();
            bounds.x1 += offset_x;
            bounds.y1 += offset_y;
            bounds.x2 += offset_x;
            bounds.y2 += offset_y;

            // Clear parent region ID (they become top-level nodes in the sub-diagram)
            node.set_parent_region_id(None);
        }

        // Clone connections where both source and target are in the child set
        // Note: segments are recalculated via recalculate_connections() so we don't translate them
        let connections: Vec<Connection> = self.connections.iter()
            .filter(|c| child_ids.contains(&c.source_id) && child_ids.contains(&c.target_id))
            .cloned()
            .collect();

        (nodes, connections)
    }

    /// Import nodes and connections into this diagram.
    /// Nodes are added to the root region.
    pub fn import_nodes_and_connections(&mut self, nodes: Vec<Node>, connections: Vec<Connection>) {
        let root_region_id = self.root_region_id();

        for mut node in nodes {
            let node_id = node.id();
            // Assign to root region
            node.set_parent_region_id(Some(root_region_id));
            self.nodes.push(node);

            // Add to root region's children
            if let Some(region) = self.root_state.regions.iter_mut().find(|r| r.id == root_region_id) {
                region.children.push(node_id);
            }
        }

        for conn in connections {
            self.connections.push(conn);
        }

        // Recalculate connections to ensure proper rendering
        self.recalculate_connections();
    }

    /// Check if an adjoined label overlaps target node and shift target if needed
    /// Returns true if a shift was performed
    pub fn adjust_for_label_overlap(&mut self, conn_id: ConnectionId) -> bool {
        // Gather connection info first to avoid borrow issues
        let conn_info = self.find_connection(conn_id).map(|c| {
            (
                c.text_adjoined,
                c.source_id,
                c.target_id,
                c.label(),
                c.label_bounds(),
            )
        });

        let Some((text_adjoined, source_id, target_id, label, label_bounds)) = conn_info else {
            return false;
        };

        // Only process if text is adjoined and label exists
        if !text_adjoined || label.is_empty() {
            return false;
        }

        let Some(label_bounds) = label_bounds else {
            return false;
        };

        // Skip self-loop connections
        if source_id == target_id {
            return false;
        }

        // Get node bounds and centers
        let target_bounds = self.find_node(target_id).map(|n| n.bounds().clone());
        let source_center = self.find_node(source_id).map(|n| n.bounds().center());
        let target_center = self.find_node(target_id).map(|n| n.bounds().center());

        let Some(target_bounds) = target_bounds else { return false; };
        let Some(source_center) = source_center else { return false; };
        let Some(target_center) = target_center else { return false; };

        // Check if label overlaps target node
        if !label_bounds.overlaps(&target_bounds) {
            return false;
        }

        // Calculate shift direction and amount
        let (shift_dx, shift_dy) = Self::calculate_target_shift_for_label(
            &label_bounds,
            &target_bounds,
            source_center,
            target_center,
        );

        // Shift the target node (only if there's actually a shift to make)
        if shift_dx.abs() > 0.1 || shift_dy.abs() > 0.1 {
            self.translate_node_with_children(target_id, shift_dx, shift_dy);
            self.recalculate_connections();
            return true;
        }

        false
    }

    /// Calculate how much to shift the target node to avoid label overlap
    fn calculate_target_shift_for_label(
        label_bounds: &Rect,
        target_bounds: &Rect,
        source_center: Point,
        target_center: Point,
    ) -> (f32, f32) {
        const MARGIN: f32 = 5.0;

        let dx = target_center.x - source_center.x;
        let dy = target_center.y - source_center.y;

        if dx.abs() > dy.abs() {
            // Horizontal connection - shift target horizontally
            let shift = if dx > 0.0 {
                // Target is to the right of source, shift target further right
                (label_bounds.x2 - target_bounds.x1 + MARGIN).max(0.0)
            } else {
                // Target is to the left of source, shift target further left
                (label_bounds.x1 - target_bounds.x2 - MARGIN).min(0.0)
            };
            (shift, 0.0)
        } else {
            // Vertical connection - shift target vertically
            let shift = if dy > 0.0 {
                // Target is below source, shift target further down
                (label_bounds.y2 - target_bounds.y1 + MARGIN).max(0.0)
            } else {
                // Target is above source, shift target further up
                (label_bounds.y1 - target_bounds.y2 - MARGIN).min(0.0)
            };
            (0.0, shift)
        }
    }

    /// Get selected elements in order (across all diagram types)
    pub fn selected_elements_in_order(&self) -> Vec<Uuid> {
        self.selection_order.clone()
    }

    /// Get element name by ID (works across all diagram types)
    pub fn get_element_name(&self, id: Uuid) -> Option<String> {
        // State machine nodes
        if let Some(node) = self.find_node(id) {
            return Some(node.name().to_string());
        }
        // Sequence
        if let Some(l) = self.lifelines.iter().find(|l| l.id == id) {
            return Some(l.name.clone());
        }
        // Use case
        if let Some(a) = self.actors.iter().find(|a| a.id == id) {
            return Some(a.name.clone());
        }
        if let Some(u) = self.use_cases.iter().find(|u| u.id == id) {
            return Some(u.name.clone());
        }
        if let Some(s) = self.system_boundaries.iter().find(|s| s.id == id) {
            return Some(s.name.clone());
        }
        // Activity
        if let Some(a) = self.actions.iter().find(|a| a.id == id) {
            return Some(a.name.clone());
        }
        if let Some(s) = self.swimlanes.iter().find(|s| s.id == id) {
            return Some(s.name.clone());
        }
        if let Some(o) = self.object_nodes.iter().find(|o| o.id == id) {
            return Some(o.name.clone());
        }
        None
    }

    /// Add a new state at the given position
    pub fn add_state(&mut self, name: &str, x: f32, y: f32) -> NodeId {
        // Center the state on the given position
        let width = self.settings.default_state_width;
        let height = self.settings.default_state_height;
        let state = State::new(
            name,
            x - width / 2.0,
            y - height / 2.0,
            width,
            height,
        );
        let id = state.id;
        self.nodes.push(Node::State(state));

        // Assign to appropriate region based on position
        // This auto-creates regions for containing states if needed
        self.update_node_region(id);

        id
    }

    /// Add a new pseudo-state at the given position (centered on the position)
    pub fn add_pseudo_state(&mut self, kind: PseudoStateKind, x: f32, y: f32) -> NodeId {
        let (width, height) = kind.default_size();
        // Center on the given position
        let pseudo = PseudoState::new(kind, x - width / 2.0, y - height / 2.0);
        let id = pseudo.id;
        self.nodes.push(Node::Pseudo(pseudo));

        // Assign to appropriate region based on position
        // This auto-creates regions for containing states if needed
        self.update_node_region(id);

        id
    }

    /// Add a node based on type
    pub fn add_node(&mut self, node_type: NodeType, x: f32, y: f32) -> NodeId {
        match node_type {
            NodeType::State => self.add_state(&format!("State{}", self.nodes.len() + 1), x, y),
            NodeType::Initial => self.add_pseudo_state(PseudoStateKind::Initial, x, y),
            NodeType::Final => self.add_pseudo_state(PseudoStateKind::Final, x, y),
            NodeType::Choice => self.add_pseudo_state(PseudoStateKind::Choice, x, y),
            NodeType::Fork => self.add_pseudo_state(PseudoStateKind::Fork, x, y),
            NodeType::Join => self.add_pseudo_state(PseudoStateKind::Join, x, y),
            NodeType::Junction => self.add_pseudo_state(PseudoStateKind::Junction, x, y),
        }
    }

    /// Remove a node by ID
    pub fn remove_node(&mut self, id: NodeId) {
        // Remove from parent region first
        self.remove_node_from_region(id);

        // Remove any connections involving this node
        self.connections.retain(|c| c.source_id != id && c.target_id != id);

        // Remove the node
        self.nodes.retain(|n| n.id() != id);
    }

    // ===== Region Management Methods =====

    /// Get the root/default region ID for top-level nodes
    pub fn root_region_id(&self) -> Uuid {
        self.root_state.regions.first()
            .map(|r| r.id)
            .expect("Root state should always have a default region")
    }

    /// Find a region by ID (searches root_state and all state nodes)
    pub fn find_region(&self, region_id: Uuid) -> Option<&Region> {
        // Check root state's regions
        if let Some(region) = self.root_state.regions.iter().find(|r| r.id == region_id) {
            return Some(region);
        }

        // Check all state nodes' regions
        for node in &self.nodes {
            if let Node::State(state) = node {
                if let Some(region) = state.regions.iter().find(|r| r.id == region_id) {
                    return Some(region);
                }
            }
        }

        None
    }

    /// Find a mutable region by ID (searches root_state and all state nodes)
    pub fn find_region_mut(&mut self, region_id: Uuid) -> Option<&mut Region> {
        // Check root state's regions
        if self.root_state.regions.iter().any(|r| r.id == region_id) {
            return self.root_state.regions.iter_mut().find(|r| r.id == region_id);
        }

        // Check all state nodes' regions
        for node in &mut self.nodes {
            if let Node::State(state) = node {
                if let Some(region) = state.regions.iter_mut().find(|r| r.id == region_id) {
                    return Some(region);
                }
            }
        }

        None
    }

    /// Find a region by ID and return its name
    pub fn find_region_name(&self, region_id: Uuid) -> Option<String> {
        self.find_region(region_id).map(|r| r.name.clone())
    }

    /// Find the parent state that contains a region
    pub fn find_region_parent_state(&self, region_id: Uuid) -> Option<&State> {
        // Check root state
        if self.root_state.regions.iter().any(|r| r.id == region_id) {
            return Some(&self.root_state);
        }

        // Check all state nodes
        for node in &self.nodes {
            if let Node::State(state) = node {
                if state.regions.iter().any(|r| r.id == region_id) {
                    return Some(state);
                }
            }
        }

        None
    }

    /// Find which region contains a point (checks all states' regions, innermost first)
    /// Returns the region_id of the deepest region containing the point
    /// Find which region contains a point, excluding a specific node's regions
    pub fn find_region_at_point_excluding(&self, x: f32, y: f32, exclude_id: Option<NodeId>) -> Option<Uuid> {
        // Check state nodes' regions first (inner states before outer)
        // We want the innermost region that contains the point
        let mut best_match: Option<(Uuid, f32)> = None; // (region_id, area)

        for node in &self.nodes {
            if let Node::State(state) = node {
                // Skip the excluded node's regions
                if Some(state.id) == exclude_id {
                    continue;
                }
                for region in &state.regions {
                    if region.contains_point(x, y) {
                        let area = region.bounds.width() * region.bounds.height();
                        if best_match.is_none() || area < best_match.unwrap().1 {
                            best_match = Some((region.id, area));
                        }
                    }
                }
            }
        }

        // If found a region in a state node, return it
        if let Some((region_id, _)) = best_match {
            return Some(region_id);
        }

        // Fall back to root state's regions
        for region in &self.root_state.regions {
            if region.contains_point(x, y) {
                return Some(region.id);
            }
        }

        // Default to root region if point is anywhere
        Some(self.root_region_id())
    }

    pub fn find_region_at_point(&self, x: f32, y: f32) -> Option<Uuid> {
        self.find_region_at_point_excluding(x, y, None)
    }

    /// Find if a point is on a region separator line within a state
    /// Returns (state_id, region_index) where region_index is the region whose top edge this is
    pub fn find_region_separator_at(&self, x: f32, y: f32, tolerance: f32) -> Option<(NodeId, usize)> {
        for node in &self.nodes {
            if let Node::State(state) = node {
                // Only check states with multiple regions
                if state.regions.len() > 1 {
                    // Check each region separator (skip first region - no separator above it)
                    for (i, region) in state.regions.iter().enumerate().skip(1) {
                        let separator_y = region.bounds.y1;
                        // Check if point is on this horizontal separator line
                        if (y - separator_y).abs() <= tolerance
                            && x >= state.bounds.x1
                            && x <= state.bounds.x2
                        {
                            return Some((state.id, i));
                        }
                    }
                }
            }
        }
        None
    }

    /// Resize regions by moving a separator line
    /// region_index is the region whose top edge is being moved
    /// delta_y is how much to move the separator (positive = down)
    pub fn move_region_separator(&mut self, state_id: NodeId, region_index: usize, delta_y: f32) {
        if let Some(Node::State(state)) = self.find_node_mut(state_id) {
            if region_index > 0 && region_index < state.regions.len() {
                // Get the regions involved
                let min_region_height = 20.0;

                let curr_region_top = state.regions[region_index].bounds.y1;
                let prev_region_top = state.regions[region_index - 1].bounds.y1;
                let curr_region_bottom = state.regions[region_index].bounds.y2;

                // Calculate new separator position
                let new_separator_y = curr_region_top + delta_y;

                // Ensure both regions maintain minimum height
                let prev_new_height = new_separator_y - prev_region_top;
                let curr_new_height = curr_region_bottom - new_separator_y;

                if prev_new_height >= min_region_height && curr_new_height >= min_region_height {
                    // Update the previous region's bottom
                    state.regions[region_index - 1].bounds.y2 = new_separator_y;
                    // Update the current region's top
                    state.regions[region_index].bounds.y1 = new_separator_y;
                }
            }
        }
    }

    /// Select a region (for visual feedback)
    pub fn select_region(&mut self, state_id: NodeId, region_index: usize) {
        // Clear all region selections first
        for node in &mut self.nodes {
            if let Node::State(state) = node {
                for region in &mut state.regions {
                    region.has_focus = false;
                }
            }
        }
        // Select the specified region
        if let Some(Node::State(state)) = self.find_node_mut(state_id) {
            if region_index < state.regions.len() {
                state.regions[region_index].has_focus = true;
            }
        }
    }

    /// Clear all region selections
    pub fn clear_region_selection(&mut self) {
        for node in &mut self.nodes {
            if let Node::State(state) = node {
                for region in &mut state.regions {
                    region.has_focus = false;
                }
            }
        }
    }

    /// Assign a node to a region (updates both node.parent_region_id and region.children)
    pub fn assign_node_to_region(&mut self, node_id: NodeId, region_id: Uuid) {
        // First, remove from any existing region
        self.remove_node_from_region(node_id);

        // Update the node's parent_region_id
        if let Some(node) = self.find_node_mut(node_id) {
            node.set_parent_region_id(Some(region_id));
        }

        // Add to the region's children list
        if let Some(region) = self.find_region_mut(region_id) {
            region.add_child(node_id);
        }
    }

    /// Remove a node from its current parent region
    pub fn remove_node_from_region(&mut self, node_id: NodeId) {
        // Get the node's current parent region
        let parent_region_id = self.find_node(node_id)
            .and_then(|n| n.parent_region_id());

        if let Some(region_id) = parent_region_id {
            // Remove from region's children
            if let Some(region) = self.find_region_mut(region_id) {
                region.remove_child(node_id);
            }
        }

        // Clear the node's parent_region_id
        if let Some(node) = self.find_node_mut(node_id) {
            node.set_parent_region_id(None);
        }
    }

    /// Expand a node's parent state to fully contain the node
    /// This preserves parent-child relationships when nodes are moved (e.g., by alignment)
    /// Also shifts sibling nodes that would overlap with the expanded parent
    /// Returns true if the parent was expanded
    pub fn expand_parent_to_contain(&mut self, node_id: NodeId) -> bool {
        const MARGIN: f32 = 10.0; // Margin around child within parent

        // Get node bounds and parent region ID
        let (node_bounds, parent_region_id) = match self.find_node(node_id) {
            Some(n) => (n.bounds().clone(), n.parent_region_id()),
            None => return false,
        };

        // Get the parent region ID (skip if it's the root region)
        let region_id = match parent_region_id {
            Some(rid) if rid != self.root_region_id() => rid,
            _ => return false, // Node is in root region, no parent to expand
        };

        // Find which state owns this region
        let parent_state_id = self.nodes.iter()
            .filter_map(|n| n.as_state())
            .find(|s| s.regions.iter().any(|r| r.id == region_id))
            .map(|s| s.id);

        let parent_state_id = match parent_state_id {
            Some(id) => id,
            None => return false,
        };

        // Get parent state bounds
        let parent_bounds = match self.find_node(parent_state_id) {
            Some(n) => n.bounds().clone(),
            None => return false,
        };

        // Check if node is fully contained (with margin for header)
        let header_height = 25.0;
        let effective_parent = Rect::new(
            parent_bounds.x1 + MARGIN,
            parent_bounds.y1 + header_height + MARGIN,
            parent_bounds.x2 - MARGIN,
            parent_bounds.y2 - MARGIN,
        );

        // Calculate required expansion
        let mut new_bounds = parent_bounds.clone();
        let mut expand_left = 0.0;
        let mut expand_up = 0.0;
        let mut expand_right = 0.0;
        let mut expand_down = 0.0;

        if node_bounds.x1 < effective_parent.x1 {
            expand_left = effective_parent.x1 - node_bounds.x1 + MARGIN;
            new_bounds.x1 = node_bounds.x1 - MARGIN;
        }
        if node_bounds.x2 > effective_parent.x2 {
            expand_right = node_bounds.x2 - effective_parent.x2 + MARGIN;
            new_bounds.x2 = node_bounds.x2 + MARGIN;
        }
        if node_bounds.y1 < effective_parent.y1 {
            expand_up = effective_parent.y1 - node_bounds.y1 + MARGIN;
            new_bounds.y1 = node_bounds.y1 - header_height - MARGIN;
        }
        if node_bounds.y2 > effective_parent.y2 {
            expand_down = node_bounds.y2 - effective_parent.y2 + MARGIN;
            new_bounds.y2 = node_bounds.y2 + MARGIN;
        }

        let expanded = expand_left > 0.0 || expand_up > 0.0 || expand_right > 0.0 || expand_down > 0.0;

        // Apply expansion if needed
        if expanded {
            // Collect sibling node IDs that need to be shifted
            // (nodes at the same level as parent, not children of parent)
            let parent_parent_region = self.find_node(parent_state_id)
                .and_then(|n| n.parent_region_id());

            let siblings_to_shift: Vec<(NodeId, f32, f32)> = self.nodes.iter()
                .filter(|n| {
                    let nid = n.id();
                    if nid == parent_state_id || nid == node_id {
                        return false;
                    }
                    // Same parent region as the expanding state
                    n.parent_region_id() == parent_parent_region
                })
                .filter_map(|n| {
                    let nb = n.bounds();
                    let mut dx = 0.0;
                    let mut dy = 0.0;

                    // If parent expands left and sibling is to the left, shift left
                    if expand_left > 0.0 && nb.x2 <= parent_bounds.x1 {
                        dx = -expand_left;
                    }
                    // If parent expands up and sibling is above, shift up
                    if expand_up > 0.0 && nb.y2 <= parent_bounds.y1 {
                        dy = -expand_up;
                    }
                    // If parent expands right and sibling is to the right, shift right
                    if expand_right > 0.0 && nb.x1 >= parent_bounds.x2 {
                        dx = expand_right;
                    }
                    // If parent expands down and sibling is below, shift down
                    if expand_down > 0.0 && nb.y1 >= parent_bounds.y2 {
                        dy = expand_down;
                    }

                    if dx != 0.0 || dy != 0.0 {
                        Some((n.id(), dx, dy))
                    } else {
                        None
                    }
                })
                .collect();

            // Shift siblings
            for (sibling_id, dx, dy) in siblings_to_shift {
                self.translate_node_with_children(sibling_id, dx, dy);
            }

            // Expand the parent
            if let Some(node) = self.find_node_mut(parent_state_id) {
                *node.bounds_mut() = new_bounds;
                // Recalculate regions for the parent state
                if let Some(state) = node.as_state_mut() {
                    state.recalculate_regions();
                }
            }

            // Recursively expand grandparent if needed
            self.expand_parent_to_contain(parent_state_id);
        }

        expanded
    }

    /// Expand all parent states to contain their children
    /// Call this after alignment/distribution to preserve parentage
    pub fn expand_parents_to_contain_children(&mut self) {
        // Get all node IDs
        let node_ids: Vec<NodeId> = self.nodes.iter().map(|n| n.id()).collect();

        // Expand parents for each node (smallest to largest would be ideal but this works)
        for node_id in node_ids {
            self.expand_parent_to_contain(node_id);
        }
    }

    /// Crop a parent state to remove blank margins around its children
    /// Returns true if the state was cropped
    /// For collapsed sub-statemachines (show_expanded=false), crops to simple state size
    pub fn crop_state(&mut self, state_id: NodeId) -> bool {
        const MARGIN: f32 = 10.0;
        const HEADER_HEIGHT: f32 = 25.0;

        // Check if this is a collapsed sub-statemachine
        let is_collapsed_substatemachine = self.find_node(state_id)
            .and_then(|n| n.as_state())
            .map(|s| s.has_substatemachine() && !s.show_expanded)
            .unwrap_or(false);

        if is_collapsed_substatemachine {
            // Crop to simple state size (default state dimensions)
            let simple_width = self.settings.default_state_width;
            let simple_height = self.settings.default_state_height;

            // Get current bounds to preserve position
            let current_bounds = match self.find_node(state_id) {
                Some(n) => n.bounds().clone(),
                None => return false,
            };

            // Keep top-left corner, shrink to simple size
            let new_bounds = Rect::new(
                current_bounds.x1,
                current_bounds.y1,
                current_bounds.x1 + simple_width,
                current_bounds.y1 + simple_height,
            );

            // Check if cropping would change anything
            let would_crop = (new_bounds.width() - current_bounds.width()).abs() > 0.1
                || (new_bounds.height() - current_bounds.height()).abs() > 0.1;

            if !would_crop {
                return false;
            }

            // Apply the crop
            if let Some(node) = self.find_node_mut(state_id) {
                *node.bounds_mut() = new_bounds;
                if let Some(state) = node.as_state_mut() {
                    state.recalculate_regions();
                }
            }

            return true;
        }

        // Regular crop for expanded states - fit around children
        // Get children of this state
        let children = self.get_children_of_node(state_id);
        if children.is_empty() {
            return false;
        }

        // Calculate bounding box of all children
        let mut min_x = f32::MAX;
        let mut min_y = f32::MAX;
        let mut max_x = f32::MIN;
        let mut max_y = f32::MIN;

        for child_id in &children {
            if let Some(node) = self.find_node(*child_id) {
                let bounds = node.bounds();
                min_x = min_x.min(bounds.x1);
                min_y = min_y.min(bounds.y1);
                max_x = max_x.max(bounds.x2);
                max_y = max_y.max(bounds.y2);
            }
        }

        if min_x == f32::MAX {
            return false;
        }

        // Calculate new bounds for parent (with margin and header)
        let new_bounds = Rect::new(
            min_x - MARGIN,
            min_y - HEADER_HEIGHT - MARGIN,
            max_x + MARGIN,
            max_y + MARGIN,
        );

        // Get current bounds
        let current_bounds = match self.find_node(state_id) {
            Some(n) => n.bounds().clone(),
            None => return false,
        };

        // Check if cropping would change anything
        let would_crop = new_bounds.x1 > current_bounds.x1
            || new_bounds.y1 > current_bounds.y1
            || new_bounds.x2 < current_bounds.x2
            || new_bounds.y2 < current_bounds.y2;

        if !would_crop {
            return false;
        }

        // Apply the crop
        if let Some(node) = self.find_node_mut(state_id) {
            *node.bounds_mut() = new_bounds;
            if let Some(state) = node.as_state_mut() {
                state.recalculate_regions();
            }
        }

        true
    }

    /// Check if a state should be cropped (has children or is a collapsed sub-statemachine)
    fn should_crop_state(&self, state_id: NodeId) -> bool {
        // Has children
        if !self.get_children_of_node(state_id).is_empty() {
            return true;
        }
        // Is a collapsed sub-statemachine
        self.find_node(state_id)
            .and_then(|n| n.as_state())
            .map(|s| s.has_substatemachine() && !s.show_expanded)
            .unwrap_or(false)
    }

    /// Crop all selected parent states, or all parent states if none selected
    pub fn crop_selected_or_all(&mut self) {
        let selected = self.selected_nodes();

        if selected.is_empty() {
            // Crop all parent states (states with children or collapsed sub-statemachines)
            let state_ids: Vec<NodeId> = self.nodes.iter()
                .filter_map(|n| n.as_state().map(|s| s.id))
                .collect();

            for state_id in state_ids {
                if self.should_crop_state(state_id) {
                    self.crop_state(state_id);
                }
            }
        } else {
            // Crop selected parent states
            for id in selected {
                if self.find_node(id).and_then(|n| n.as_state()).is_some() {
                    self.crop_state(id);
                }
            }
        }

        self.recalculate_connections();
    }

    /// Crop all parent states that have children or collapsed sub-statemachines
    pub fn crop_all_parents(&mut self) {
        let state_ids: Vec<NodeId> = self.nodes.iter()
            .filter_map(|n| n.as_state().map(|s| s.id))
            .collect();

        for state_id in state_ids {
            if self.should_crop_state(state_id) {
                self.crop_state(state_id);
            }
        }
    }

    /// Find the innermost state containing a point (by state bounds, not regions)
    /// Returns the state's node ID, or None if only the root state contains it
    /// `exclude_id` can be used to exclude a specific node (e.g., the node being dragged)
    pub fn find_state_at_point_excluding(&self, x: f32, y: f32, exclude_id: Option<NodeId>) -> Option<NodeId> {
        let point = Point::new(x, y);
        let mut best_match: Option<(NodeId, f32)> = None; // (state_id, area)

        for node in &self.nodes {
            if let Node::State(state) = node {
                // Skip the excluded node
                if Some(state.id) == exclude_id {
                    continue;
                }
                if state.bounds.contains_point(point) {
                    let area = state.bounds.width() * state.bounds.height();
                    if best_match.is_none() || area < best_match.unwrap().1 {
                        best_match = Some((state.id, area));
                    }
                }
            }
        }

        best_match.map(|(id, _)| id)
    }

    /// Find the innermost state containing a point (by state bounds, not regions)
    /// Returns the state's node ID, or None if only the root state contains it
    pub fn find_state_at_point(&self, x: f32, y: f32) -> Option<NodeId> {
        self.find_state_at_point_excluding(x, y, None)
    }

    /// Re-parent a node to the appropriate region based on its current position
    pub fn update_node_region(&mut self, node_id: NodeId) {
        // Get the node's center position, area, and current parent region
        let (center, node_area, current_region_id) = match self.find_node(node_id) {
            Some(n) => {
                let bounds = n.bounds();
                (bounds.center(), bounds.width() * bounds.height(), n.parent_region_id())
            }
            None => return,
        };

        // If node is currently a child of a collapsed sub-statemachine, don't reassign it.
        // This preserves children when the parent state is resized.
        if let Some(region_id) = current_region_id {
            if let Some(parent_state) = self.find_region_parent_state(region_id) {
                if parent_state.has_substatemachine() && !parent_state.show_expanded {
                    // Node is inside a collapsed sub-statemachine - keep it there
                    return;
                }
            }
        }

        // First, check if we're inside a state that has no regions
        // If so, create a default region for it
        // IMPORTANT: Exclude the node being moved from the search
        // Also check that the potential parent is LARGER than the node being moved
        // (to prevent circular parent-child relationships)
        if let Some(state_id) = self.find_state_at_point_excluding(center.x, center.y, Some(node_id)) {
            // Check if this state is larger than our node (only larger states can be parents)
            let parent_area = self.find_node(state_id)
                .map(|n| {
                    let b = n.bounds();
                    b.width() * b.height()
                })
                .unwrap_or(0.0);

            if parent_area > node_area {
                // Check if this state has any regions AND is not a non-expanded sub-statemachine
                let (needs_region, is_collapsed_substatemachine) = self.find_node(state_id)
                    .and_then(|n| n.as_state())
                    .map(|s| (s.regions.is_empty(), s.has_substatemachine() && !s.show_expanded))
                    .unwrap_or((false, false));

                // Don't create a region in a non-expanded sub-statemachine
                if needs_region && !is_collapsed_substatemachine {
                    // Create a default region for this state
                    if let Some(Node::State(state)) = self.find_node_mut(state_id) {
                        state.add_region("default");
                    }
                }
            }
        }

        // Now find which region should contain this node
        // We need to find a region that belongs to a state LARGER than this node
        if let Some(region_id) = self.find_region_at_point_for_node(center.x, center.y, node_id, node_area) {
            // Get current parent
            let current_parent = self.find_node(node_id)
                .and_then(|n| n.parent_region_id());

            // Only update if different
            if current_parent != Some(region_id) {
                self.assign_node_to_region(node_id, region_id);
            }
        }
    }

    /// Find the region at a point that can contain a node of the given area
    /// Only returns regions from states that are larger than the node
    fn find_region_at_point_for_node(&self, x: f32, y: f32, exclude_id: NodeId, node_area: f32) -> Option<Uuid> {
        // Check state nodes' regions first (inner states before outer)
        // We want the innermost region that contains the point AND belongs to a larger state
        let mut best_match: Option<(Uuid, f32)> = None; // (region_id, area)

        for node in &self.nodes {
            if let Node::State(state) = node {
                // Skip the excluded node's regions
                if state.id == exclude_id {
                    continue;
                }
                // Only consider states that are larger than the node being parented
                let state_area = state.bounds.width() * state.bounds.height();
                if state_area <= node_area {
                    continue;
                }
                // Skip states with non-expanded sub-statemachines
                // (don't allow dragging nodes into collapsed sub-statemachines)
                if state.has_substatemachine() && !state.show_expanded {
                    continue;
                }
                for region in &state.regions {
                    if region.contains_point(x, y) {
                        let area = region.bounds.width() * region.bounds.height();
                        if best_match.is_none() || area < best_match.as_ref().unwrap().1 {
                            best_match = Some((region.id, area));
                        }
                    }
                }
            }
        }

        // If found a region in a state node, return it
        if let Some((region_id, _area)) = best_match {
            return Some(region_id);
        }

        // Fall back to root state's regions
        for region in &self.root_state.regions {
            if region.contains_point(x, y) {
                return Some(region.id);
            }
        }

        // Default to root region if point is anywhere
        Some(self.root_region_id())
    }

    /// Re-evaluate all nodes' parent/child relationships based on current positions
    /// Call this after any drag operation to ensure all containment is correct
    pub fn update_all_node_regions(&mut self) {
        // FIRST: Recalculate all region bounds to ensure they match current state positions
        // This is critical because states may have been moved and regions need to update
        // Also clear any existing errors
        for node in &mut self.nodes {
            node.set_error(false);
            if let Node::State(state) = node {
                state.recalculate_regions();
            }
        }

        // Get all nodes sorted by area (largest first)
        // This ensures parent states are processed before their potential children
        let mut node_info: Vec<(NodeId, Rect)> = self.nodes.iter()
            .map(|n| (n.id(), n.bounds().clone()))
            .collect();

        // Sort by area descending (largest first)
        node_info.sort_by(|a, b| {
            let area_a = a.1.width() * a.1.height();
            let area_b = b.1.width() * b.1.height();
            area_b.partial_cmp(&area_a).unwrap_or(std::cmp::Ordering::Equal)
        });

        // Check for partial containment errors and update region assignments
        let mut error_nodes: Vec<NodeId> = Vec::new();

        for (node_id, bounds) in &node_info {
            // Check which larger nodes this node overlaps with
            for (other_id, other_bounds) in &node_info {
                if other_id == node_id {
                    continue;
                }

                // Count how many corners are contained
                let corners = [
                    Point::new(bounds.x1, bounds.y1),
                    Point::new(bounds.x2, bounds.y1),
                    Point::new(bounds.x1, bounds.y2),
                    Point::new(bounds.x2, bounds.y2),
                ];
                let contained_count = corners.iter().filter(|c| other_bounds.contains_point(**c)).count();

                // Partial containment (1-3 corners) is an error
                if contained_count > 0 && contained_count < 4 {
                    error_nodes.push(*node_id);
                }
            }

            self.update_node_region(*node_id);
        }

        // Mark nodes with errors
        for error_id in error_nodes {
            if let Some(node) = self.find_node_mut(error_id) {
                node.set_error(true);
            }
        }
    }

    /// Add a connection between two nodes
    pub fn add_connection(&mut self, source_id: NodeId, target_id: NodeId) -> Option<ConnectionId> {
        // Validate that both nodes exist
        let source = self.find_node(source_id)?;
        let target = self.find_node(target_id)?;

        // Check if source can be a connection source
        if let Node::Pseudo(p) = source {
            if !p.kind.can_be_source() {
                return None;
            }
        }

        // Check if target can be a connection target
        if let Node::Pseudo(p) = target {
            if !p.kind.can_be_target() {
                return None;
            }
        }

        let source_bounds = source.bounds().clone();
        let target_bounds = target.bounds().clone();

        // Calculate sides
        let (source_side, target_side) =
            Connection::calculate_sides(&source_bounds, &target_bounds, self.settings.stub_length);

        let mut conn = Connection::new(source_id, target_id);
        conn.source_side = source_side;
        conn.target_side = target_side;

        let id = conn.id;
        self.connections.push(conn);

        // Recalculate all connections (slots and segments) since adding a connection
        // may affect slot positions of other connections on the same node/side
        self.recalculate_connections();

        // Adjust for label overlap if needed (won't do anything for new connections without labels)
        self.adjust_for_label_overlap(id);

        Some(id)
    }

    /// Remove a connection by ID
    pub fn remove_connection(&mut self, id: ConnectionId) {
        self.connections.retain(|c| c.id != id);
        // Recalculate remaining connections since slot positions may change
        self.recalculate_connections();
    }

    /// Recalculate all connection segments (call after nodes move)
    pub fn recalculate_connections(&mut self) {
        let stub_len = self.settings.stub_length;

        // First pass: recalculate sides for all connections
        for conn in &mut self.connections {
            let source_bounds = self
                .nodes
                .iter()
                .find(|n| n.id() == conn.source_id)
                .map(|n| n.bounds().clone());
            let target_bounds = self
                .nodes
                .iter()
                .find(|n| n.id() == conn.target_id)
                .map(|n| n.bounds().clone());

            if let (Some(sb), Some(tb)) = (source_bounds, target_bounds) {
                // Recalculate sides
                let (source_side, target_side) = Connection::calculate_sides(&sb, &tb, stub_len);
                conn.source_side = source_side;
                conn.target_side = target_side;
            }
        }

        // Second pass: calculate slot offsets to prevent overlap
        self.recalculate_connection_slots();

        // Collect all node bounds for obstacle avoidance
        let all_bounds: Vec<(NodeId, Rect)> = self.nodes
            .iter()
            .map(|n| (n.id(), n.bounds().clone()))
            .collect();

        // Third pass: recalculate segments with updated offsets
        for conn in &mut self.connections {
            let source_bounds = all_bounds.iter()
                .find(|(id, _)| *id == conn.source_id)
                .map(|(_, b)| b.clone());
            let target_bounds = all_bounds.iter()
                .find(|(id, _)| *id == conn.target_id)
                .map(|(_, b)| b.clone());

            if let (Some(sb), Some(tb)) = (source_bounds, target_bounds) {
                conn.calculate_segments(&sb, &tb, stub_len);
            }
        }
    }

    /// Calculate slot offsets for all connections to prevent overlap
    /// Prioritizes aligned connections (where nodes are vertically/horizontally aligned)
    /// to get the center slot for straight lines. Non-aligned connections are distributed around.
    fn recalculate_connection_slots(&mut self) {
        use crate::node::Side;

        const SLOT_SPACING: f32 = 15.0;
        const ALIGNMENT_TOLERANCE: f32 = 20.0; // How close centers need to be considered "aligned"

        // Collect node centers for sorting and alignment checks
        let node_centers: std::collections::HashMap<NodeId, (f32, f32)> = self.nodes
            .iter()
            .map(|n| (n.id(), (n.bounds().center().x, n.bounds().center().y)))
            .collect();

        // Collect all node IDs
        let node_ids: Vec<NodeId> = self.nodes.iter().map(|n| n.id()).collect();

        // For each node, calculate offsets for connections on each side
        for node_id in node_ids {
            let this_center = match node_centers.get(&node_id) {
                Some(c) => *c,
                None => continue,
            };

            for side in [Side::Top, Side::Bottom, Side::Left, Side::Right] {
                // Collect all connections on this node/side with position and alignment info
                // (index, other_node_position, is_source, is_aligned)
                let mut all_conns: Vec<(usize, f32, bool, bool)> = Vec::new();

                // Source connections (this node is source)
                for (i, c) in self.connections.iter().enumerate() {
                    if c.source_id == node_id && c.source_side == side {
                        if let Some(&(other_x, other_y)) = node_centers.get(&c.target_id) {
                            let (pos, is_aligned) = match side {
                                Side::Top | Side::Bottom => {
                                    // For top/bottom sides, check if x-centers are aligned
                                    let is_aligned = (other_x - this_center.0).abs() < ALIGNMENT_TOLERANCE;
                                    (other_x, is_aligned)
                                }
                                Side::Left | Side::Right => {
                                    // For left/right sides, check if y-centers are aligned
                                    let is_aligned = (other_y - this_center.1).abs() < ALIGNMENT_TOLERANCE;
                                    (other_y, is_aligned)
                                }
                                Side::None => (0.0, false),
                            };
                            all_conns.push((i, pos, true, is_aligned));
                        }
                    }
                }

                // Target connections (this node is target)
                for (i, c) in self.connections.iter().enumerate() {
                    if c.target_id == node_id && c.target_side == side {
                        if let Some(&(other_x, other_y)) = node_centers.get(&c.source_id) {
                            let (pos, is_aligned) = match side {
                                Side::Top | Side::Bottom => {
                                    let is_aligned = (other_x - this_center.0).abs() < ALIGNMENT_TOLERANCE;
                                    (other_x, is_aligned)
                                }
                                Side::Left | Side::Right => {
                                    let is_aligned = (other_y - this_center.1).abs() < ALIGNMENT_TOLERANCE;
                                    (other_y, is_aligned)
                                }
                                Side::None => (0.0, false),
                            };
                            all_conns.push((i, pos, false, is_aligned));
                        }
                    }
                }

                let total = all_conns.len();
                if total == 0 {
                    continue;
                }

                // Separate aligned and non-aligned connections
                let mut aligned: Vec<_> = all_conns.iter().filter(|(_, _, _, a)| *a).cloned().collect();
                let mut non_aligned: Vec<_> = all_conns.iter().filter(|(_, _, _, a)| !*a).cloned().collect();

                // Sort non-aligned by position
                non_aligned.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));

                let num_aligned = aligned.len();
                let num_non_aligned = non_aligned.len();

                // Assign slots: aligned get center, non-aligned distributed around
                if num_aligned > 0 {
                    // Distribute aligned connections in center region
                    let aligned_width = (num_aligned as f32 - 1.0) * SLOT_SPACING;
                    let aligned_start = -aligned_width / 2.0;

                    // Sort aligned by position too for consistency
                    aligned.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));

                    for (slot, (idx, _, is_source, _)) in aligned.iter().enumerate() {
                        let offset = if num_aligned == 1 {
                            0.0
                        } else {
                            aligned_start + (slot as f32) * SLOT_SPACING
                        };

                        if *is_source {
                            self.connections[*idx].source_offset = offset;
                        } else {
                            self.connections[*idx].target_offset = offset;
                        }
                    }
                }

                // Distribute non-aligned around the aligned ones
                if num_non_aligned > 0 {
                    // Calculate the edge of the aligned region
                    let aligned_edge = if num_aligned > 0 {
                        ((num_aligned as f32 - 1.0) * SLOT_SPACING) / 2.0 + SLOT_SPACING
                    } else {
                        // No aligned connections - center the non-aligned ones
                        let width = (num_non_aligned as f32 - 1.0) * SLOT_SPACING;
                        let start = -width / 2.0;
                        for (slot, (idx, _, is_source, _)) in non_aligned.iter().enumerate() {
                            let offset = if num_non_aligned == 1 {
                                0.0
                            } else {
                                start + (slot as f32) * SLOT_SPACING
                            };
                            if *is_source {
                                self.connections[*idx].source_offset = offset;
                            } else {
                                self.connections[*idx].target_offset = offset;
                            }
                        }
                        continue;
                    };

                    // Split non-aligned: those with position < center go left, others go right
                    let center_ref = match side {
                        Side::Top | Side::Bottom => this_center.0,
                        Side::Left | Side::Right => this_center.1,
                        Side::None => 0.0,
                    };

                    let left: Vec<_> = non_aligned.iter().filter(|(_, pos, _, _)| *pos < center_ref).cloned().collect();
                    let right: Vec<_> = non_aligned.iter().filter(|(_, pos, _, _)| *pos >= center_ref).cloned().collect();

                    // Assign left connections (negative offsets, going outward from aligned)
                    for (i, (idx, _, is_source, _)) in left.iter().rev().enumerate() {
                        let offset = -aligned_edge - (i as f32) * SLOT_SPACING;
                        if *is_source {
                            self.connections[*idx].source_offset = offset;
                        } else {
                            self.connections[*idx].target_offset = offset;
                        }
                    }

                    // Assign right connections (positive offsets, going outward from aligned)
                    for (i, (idx, _, is_source, _)) in right.iter().enumerate() {
                        let offset = aligned_edge + (i as f32) * SLOT_SPACING;
                        if *is_source {
                            self.connections[*idx].source_offset = offset;
                        } else {
                            self.connections[*idx].target_offset = offset;
                        }
                    }
                }
            }
        }
    }

    /// Clear all selections
    pub fn clear_selection(&mut self) {
        // State machine elements
        for node in &mut self.nodes {
            node.set_focus(false);
        }
        for conn in &mut self.connections {
            conn.selected = false;
            conn.label_selected = false;
        }
        // Sequence elements
        for lifeline in &mut self.lifelines {
            lifeline.has_focus = false;
        }
        // Use case elements
        for actor in &mut self.actors {
            actor.has_focus = false;
        }
        for uc in &mut self.use_cases {
            uc.has_focus = false;
        }
        for sb in &mut self.system_boundaries {
            sb.has_focus = false;
        }
        // Activity elements
        for action in &mut self.actions {
            action.has_focus = false;
        }
        for swimlane in &mut self.swimlanes {
            swimlane.has_focus = false;
        }
        for obj in &mut self.object_nodes {
            obj.has_focus = false;
        }
        self.selection_order.clear();
        self.explicit_selection_order = false;
    }

    /// Select a single node (not explicit ordering - single click selection)
    pub fn select_node(&mut self, id: NodeId) {
        self.clear_selection();
        if let Some(node) = self.find_node_mut(id) {
            node.set_focus(true);
            self.selection_order.push(id);
        }
        self.explicit_selection_order = false;
    }

    /// Select multiple nodes by their IDs (not explicit ordering - marquee/batch)
    pub fn select_nodes(&mut self, ids: &[NodeId]) {
        self.clear_selection();
        for id in ids {
            if let Some(node) = self.find_node_mut(*id) {
                node.set_focus(true);
                self.selection_order.push(*id);
            }
        }
        self.explicit_selection_order = false;
    }

    /// Select all nodes within a rectangle (state machine only)
    pub fn select_nodes_in_rect(&mut self, rect: &Rect) {
        let ids = self.find_nodes_in_rect(rect);
        self.select_nodes(&ids);
    }

    /// Select all elements that are fully contained in a rectangle (for marquee selection)
    /// All four corners must be inside the rectangle
    /// Works for all diagram types
    pub fn select_elements_in_rect(&mut self, rect: &Rect) {
        self.clear_selection();

        match self.diagram_type {
            DiagramType::StateMachine => {
                // Select nodes fully contained in rect
                let mut ids = Vec::new();
                for node in &self.nodes {
                    if rect.contains_rect(node.bounds()) {
                        ids.push(node.id());
                    }
                }
                self.select_nodes(&ids);
            }
            DiagramType::Sequence => {
                // Select lifelines fully contained
                for lifeline in &mut self.lifelines {
                    if rect.contains_rect(&lifeline.head_bounds()) {
                        lifeline.has_focus = true;
                        self.selection_order.push(lifeline.id);
                    }
                }
            }
            DiagramType::UseCase => {
                // Select actors fully contained
                for actor in &mut self.actors {
                    if rect.contains_rect(&actor.bounds()) {
                        actor.has_focus = true;
                        self.selection_order.push(actor.id);
                    }
                }
                // Select use cases fully contained
                for uc in &mut self.use_cases {
                    if rect.contains_rect(&uc.bounds) {
                        uc.has_focus = true;
                        self.selection_order.push(uc.id);
                    }
                }
                // Select system boundaries fully contained
                for sb in &mut self.system_boundaries {
                    if rect.contains_rect(&sb.bounds) {
                        sb.has_focus = true;
                        self.selection_order.push(sb.id);
                    }
                }
            }
            DiagramType::Activity => {
                // Select actions fully contained
                for action in &mut self.actions {
                    if rect.contains_rect(&action.bounds) {
                        action.has_focus = true;
                        self.selection_order.push(action.id);
                    }
                }
                // Select swimlanes fully contained
                for swimlane in &mut self.swimlanes {
                    if rect.contains_rect(&swimlane.bounds) {
                        swimlane.has_focus = true;
                        self.selection_order.push(swimlane.id);
                    }
                }
                // Select object nodes fully contained
                for obj in &mut self.object_nodes {
                    if rect.contains_rect(&obj.bounds) {
                        obj.has_focus = true;
                        self.selection_order.push(obj.id);
                    }
                }
            }
        }
        // Marquee selection is not explicit ordering
        self.explicit_selection_order = false;
    }

    /// Select all elements that are fully contained in a polygon (for lasso selection)
    /// All four corners must be inside the polygon
    pub fn select_elements_in_polygon(&mut self, polygon: &[Point]) {
        use crate::geometry::point_in_polygon;

        // Helper: check if all four corners of bounds are in polygon
        let all_corners_in_polygon = |bounds: &Rect| -> bool {
            let corners = [
                Point::new(bounds.x1, bounds.y1),
                Point::new(bounds.x2, bounds.y1),
                Point::new(bounds.x1, bounds.y2),
                Point::new(bounds.x2, bounds.y2),
            ];
            corners.iter().all(|c| point_in_polygon(*c, polygon))
        };

        self.clear_selection();

        match self.diagram_type {
            DiagramType::StateMachine => {
                // Select nodes fully contained in polygon
                let mut ids = Vec::new();
                for node in &self.nodes {
                    if all_corners_in_polygon(node.bounds()) {
                        ids.push(node.id());
                    }
                }
                self.select_nodes(&ids);
            }
            DiagramType::Sequence => {
                // Select lifelines fully contained
                for lifeline in &mut self.lifelines {
                    if all_corners_in_polygon(&lifeline.head_bounds()) {
                        lifeline.has_focus = true;
                        self.selection_order.push(lifeline.id);
                    }
                }
            }
            DiagramType::UseCase => {
                // Select actors fully contained
                for actor in &mut self.actors {
                    if all_corners_in_polygon(&actor.bounds()) {
                        actor.has_focus = true;
                        self.selection_order.push(actor.id);
                    }
                }
                // Select use cases fully contained
                for uc in &mut self.use_cases {
                    if all_corners_in_polygon(&uc.bounds) {
                        uc.has_focus = true;
                        self.selection_order.push(uc.id);
                    }
                }
                // Select system boundaries fully contained
                for sb in &mut self.system_boundaries {
                    if all_corners_in_polygon(&sb.bounds) {
                        sb.has_focus = true;
                        self.selection_order.push(sb.id);
                    }
                }
            }
            DiagramType::Activity => {
                // Select actions fully contained
                for action in &mut self.actions {
                    if all_corners_in_polygon(&action.bounds) {
                        action.has_focus = true;
                        self.selection_order.push(action.id);
                    }
                }
                // Select swimlanes fully contained
                for swimlane in &mut self.swimlanes {
                    if all_corners_in_polygon(&swimlane.bounds) {
                        swimlane.has_focus = true;
                        self.selection_order.push(swimlane.id);
                    }
                }
                // Select object nodes fully contained
                for obj in &mut self.object_nodes {
                    if all_corners_in_polygon(&obj.bounds) {
                        obj.has_focus = true;
                        self.selection_order.push(obj.id);
                    }
                }
            }
        }
        // Lasso selection is not explicit ordering
        self.explicit_selection_order = false;
    }

    /// Toggle a node's selection (add if not selected, remove if selected)
    /// When adding via Ctrl+Click, marks as explicit ordering
    pub fn toggle_node_selection(&mut self, id: NodeId) {
        if let Some(node) = self.find_node_mut(id) {
            let currently_selected = node.has_focus();
            if currently_selected {
                // Remove from selection
                node.set_focus(false);
                self.selection_order.retain(|&x| x != id);
            } else {
                // Add to selection (at end of order) - explicit ordering
                node.set_focus(true);
                self.selection_order.push(id);
                self.explicit_selection_order = true;
            }
        }
    }

    /// Add a node to the current selection without clearing existing selection
    /// Marks as explicit ordering (Ctrl+Click behavior)
    pub fn add_to_selection(&mut self, id: NodeId) {
        if let Some(node) = self.find_node_mut(id) {
            if !node.has_focus() {
                node.set_focus(true);
                self.selection_order.push(id);
                self.explicit_selection_order = true;
            }
        }
    }

    /// Select a single connection
    pub fn select_connection(&mut self, id: ConnectionId) {
        self.clear_selection();
        if let Some(conn) = self.find_connection_mut(id) {
            conn.selected = true;
        }
    }

    /// Get all selected node IDs (unordered)
    pub fn selected_nodes(&self) -> Vec<NodeId> {
        self.nodes
            .iter()
            .filter(|n| n.has_focus())
            .map(|n| n.id())
            .collect()
    }

    /// Get selected node IDs in the order they were selected
    pub fn selected_nodes_in_order(&self) -> Vec<NodeId> {
        // Filter to only include nodes that are still selected (in case of desync)
        self.selection_order
            .iter()
            .filter(|id| self.find_node(**id).map(|n| n.has_focus()).unwrap_or(false))
            .copied()
            .collect()
    }

    /// Get selected nodes ordered by their connections.
    ///
    /// Orders nodes based on connection flow: if A→B→C, returns [A, B, C].
    /// Handles bidirectional connections (A↔B↔C becomes [A, B, C]).
    /// Starts with nodes that have no incoming connections from other selected nodes.
    /// Falls back to position-based ordering if no connections exist.
    pub fn selected_nodes_by_connection_order(&self) -> Vec<NodeId> {
        use std::collections::{HashMap, HashSet, VecDeque};

        let selected: HashSet<NodeId> = self.selected_nodes().into_iter().collect();
        if selected.len() < 2 {
            return selected.into_iter().collect();
        }

        // Build adjacency lists for connections between selected nodes only
        // outgoing[a] = set of nodes that a connects TO
        // incoming[a] = set of nodes that connect TO a
        let mut outgoing: HashMap<NodeId, HashSet<NodeId>> = HashMap::new();
        let mut incoming: HashMap<NodeId, HashSet<NodeId>> = HashMap::new();

        for node_id in &selected {
            outgoing.insert(*node_id, HashSet::new());
            incoming.insert(*node_id, HashSet::new());
        }

        for conn in &self.connections {
            if selected.contains(&conn.source_id) && selected.contains(&conn.target_id) {
                outgoing.get_mut(&conn.source_id).unwrap().insert(conn.target_id);
                incoming.get_mut(&conn.target_id).unwrap().insert(conn.source_id);
            }
        }

        // Find starting nodes: nodes with no incoming connections from selected nodes
        let mut starting_nodes: Vec<NodeId> = selected
            .iter()
            .filter(|id| incoming.get(id).map(|s| s.is_empty()).unwrap_or(true))
            .copied()
            .collect();

        // If all nodes have incoming (cycle), pick by position (top-left first)
        if starting_nodes.is_empty() {
            starting_nodes = selected.iter().copied().collect();
            starting_nodes.sort_by(|a, b| {
                let a_bounds = self.find_node(*a).map(|n| n.bounds());
                let b_bounds = self.find_node(*b).map(|n| n.bounds());
                match (a_bounds, b_bounds) {
                    (Some(a), Some(b)) => {
                        a.y1.partial_cmp(&b.y1)
                            .unwrap_or(std::cmp::Ordering::Equal)
                            .then(a.x1.partial_cmp(&b.x1).unwrap_or(std::cmp::Ordering::Equal))
                    }
                    _ => std::cmp::Ordering::Equal,
                }
            });
            // Just take the first one as starting point
            starting_nodes.truncate(1);
        } else {
            // Sort starting nodes by position (top-left first)
            starting_nodes.sort_by(|a, b| {
                let a_bounds = self.find_node(*a).map(|n| n.bounds());
                let b_bounds = self.find_node(*b).map(|n| n.bounds());
                match (a_bounds, b_bounds) {
                    (Some(a), Some(b)) => {
                        a.y1.partial_cmp(&b.y1)
                            .unwrap_or(std::cmp::Ordering::Equal)
                            .then(a.x1.partial_cmp(&b.x1).unwrap_or(std::cmp::Ordering::Equal))
                    }
                    _ => std::cmp::Ordering::Equal,
                }
            });
        }

        // BFS from starting nodes, following outgoing connections
        let mut result: Vec<NodeId> = Vec::new();
        let mut visited: HashSet<NodeId> = HashSet::new();
        let mut queue: VecDeque<NodeId> = VecDeque::new();

        for start in &starting_nodes {
            if !visited.contains(start) {
                queue.push_back(*start);
                visited.insert(*start);
            }
        }

        while let Some(current) = queue.pop_front() {
            result.push(current);

            // Get outgoing connections, sorted by target position
            let mut targets: Vec<NodeId> = outgoing
                .get(&current)
                .map(|s| s.iter().copied().collect())
                .unwrap_or_default();

            targets.sort_by(|a, b| {
                let a_bounds = self.find_node(*a).map(|n| n.bounds());
                let b_bounds = self.find_node(*b).map(|n| n.bounds());
                match (a_bounds, b_bounds) {
                    (Some(a), Some(b)) => {
                        a.y1.partial_cmp(&b.y1)
                            .unwrap_or(std::cmp::Ordering::Equal)
                            .then(a.x1.partial_cmp(&b.x1).unwrap_or(std::cmp::Ordering::Equal))
                    }
                    _ => std::cmp::Ordering::Equal,
                }
            });

            for target in targets {
                if !visited.contains(&target) {
                    visited.insert(target);
                    queue.push_back(target);
                }
            }

            // Also check for bidirectional - if we haven't visited nodes that connect TO us,
            // add them (handles A↔B where we came from A and need to continue to nodes B connects to)
            let mut sources: Vec<NodeId> = incoming
                .get(&current)
                .map(|s| s.iter().copied().collect())
                .unwrap_or_default();

            sources.sort_by(|a, b| {
                let a_bounds = self.find_node(*a).map(|n| n.bounds());
                let b_bounds = self.find_node(*b).map(|n| n.bounds());
                match (a_bounds, b_bounds) {
                    (Some(a), Some(b)) => {
                        a.y1.partial_cmp(&b.y1)
                            .unwrap_or(std::cmp::Ordering::Equal)
                            .then(a.x1.partial_cmp(&b.x1).unwrap_or(std::cmp::Ordering::Equal))
                    }
                    _ => std::cmp::Ordering::Equal,
                }
            });

            for source in sources {
                if !visited.contains(&source) {
                    visited.insert(source);
                    queue.push_back(source);
                }
            }
        }

        // Add any unvisited nodes (disconnected from the graph)
        for node_id in &selected {
            if !visited.contains(node_id) {
                result.push(*node_id);
            }
        }

        result
    }

    /// Returns true if selection was made via explicit Ctrl+Click ordering
    /// Returns false if selection was via marquee/lasso (should use position order)
    pub fn has_explicit_selection_order(&self) -> bool {
        self.explicit_selection_order
    }

    /// Get selected connection ID (if any)
    pub fn selected_connection(&self) -> Option<ConnectionId> {
        self.connections.iter().find(|c| c.selected).map(|c| c.id)
    }

    /// Push current state to undo stack
    pub fn push_undo(&mut self) {
        if let Ok(snapshot) = serde_json::to_string(self) {
            self.undo_stack.push(snapshot);
            if self.undo_stack.len() > self.max_undo_levels {
                self.undo_stack.remove(0);
            }
            self.redo_stack.clear();
        }
    }

    /// Undo the last change
    pub fn undo(&mut self) -> bool {
        if let Some(snapshot) = self.undo_stack.pop() {
            // Save current state to redo stack
            if let Ok(current) = serde_json::to_string(self) {
                self.redo_stack.push(current);
            }

            // Restore from snapshot
            if let Ok(restored) = serde_json::from_str::<Diagram>(&snapshot) {
                self.restore_from(&restored);
                return true;
            }
        }
        false
    }

    /// Redo a previously undone change
    pub fn redo(&mut self) -> bool {
        if let Some(snapshot) = self.redo_stack.pop() {
            // Save current state to undo stack
            if let Ok(current) = serde_json::to_string(self) {
                self.undo_stack.push(current);
            }

            // Restore from snapshot
            if let Ok(restored) = serde_json::from_str::<Diagram>(&snapshot) {
                self.restore_from(&restored);
                return true;
            }
        }
        false
    }

    /// Restore all diagram state from another diagram
    fn restore_from(&mut self, other: &Diagram) {
        self.title = other.title.clone();
        self.title_style = other.title_style;
        self.diagram_type = other.diagram_type;
        self.settings = other.settings.clone();
        self.root_state = other.root_state.clone();
        self.nodes = other.nodes.clone();
        self.connections = other.connections.clone();
        // Sequence
        self.lifelines = other.lifelines.clone();
        self.messages = other.messages.clone();
        self.activations = other.activations.clone();
        self.fragments = other.fragments.clone();
        // Use case
        self.actors = other.actors.clone();
        self.use_cases = other.use_cases.clone();
        self.system_boundaries = other.system_boundaries.clone();
        self.uc_relationships = other.uc_relationships.clone();
        // Activity
        self.actions = other.actions.clone();
        self.swimlanes = other.swimlanes.clone();
        self.partitions = other.partitions.clone();
        self.object_nodes = other.object_nodes.clone();
        self.control_flows = other.control_flows.clone();
        // Recalculate connections
        self.recalculate_connections();
    }

    /// Check if undo is available
    pub fn can_undo(&self) -> bool {
        !self.undo_stack.is_empty()
    }

    /// Check if redo is available
    pub fn can_redo(&self) -> bool {
        !self.redo_stack.is_empty()
    }

    /// Get the number of items in the undo stack (for debugging)
    pub fn undo_stack_len(&self) -> usize {
        self.undo_stack.len()
    }

    /// Delete all selected nodes and connections
    pub fn delete_selected(&mut self) {
        let selected_nodes: Vec<NodeId> = self.selected_nodes();
        let selected_conn = self.selected_connection();

        for id in selected_nodes {
            self.remove_node(id);
        }

        if let Some(id) = selected_conn {
            self.remove_connection(id);
        }
    }

    // ===== Sequence Diagram Methods =====

    /// Add a lifeline to the sequence diagram (centered on x position)
    pub fn add_lifeline(&mut self, name: &str, x: f32, y: f32) -> Uuid {
        // Lifeline x is already centered, y is top of head
        // Adjust y so cursor is at center of head
        let lifeline = Lifeline::new(name, x, y - 20.0);
        let id = lifeline.id;
        self.lifelines.push(lifeline);
        id
    }

    /// Add a message between two lifelines
    pub fn add_message(&mut self, source_id: Uuid, target_id: Uuid, label: &str, y: f32) -> Uuid {
        let message = Message::new(source_id, target_id, label, y);
        let id = message.id;
        self.messages.push(message);
        id
    }

    /// Find a lifeline at a given point
    pub fn find_lifeline_at(&self, pos: Point) -> Option<Uuid> {
        self.lifelines.iter()
            .find(|l| l.contains_point(pos))
            .map(|l| l.id)
    }

    /// Find a lifeline by ID
    pub fn find_lifeline(&self, id: Uuid) -> Option<&Lifeline> {
        self.lifelines.iter().find(|l| l.id == id)
    }

    /// Find a mutable lifeline by ID
    pub fn find_lifeline_mut(&mut self, id: Uuid) -> Option<&mut Lifeline> {
        self.lifelines.iter_mut().find(|l| l.id == id)
    }

    // ===== Use Case Diagram Methods =====

    /// Add an actor to the use case diagram (centered on position)
    pub fn add_actor(&mut self, name: &str, x: f32, y: f32) -> Uuid {
        // Actor: x is center, y is top, height is ~70
        // Center vertically on cursor
        let actor = Actor::new(name, x, y - 35.0);
        let id = actor.id;
        self.actors.push(actor);
        id
    }

    /// Add a use case to the diagram (centered on position)
    pub fn add_use_case(&mut self, name: &str, x: f32, y: f32) -> Uuid {
        // UseCase: x, y is top-left, default size 120x60
        // Center on cursor position
        let use_case = UseCase::new(name, x - 60.0, y - 30.0);
        let id = use_case.id;
        self.use_cases.push(use_case);
        id
    }

    /// Add a system boundary (centered on position)
    pub fn add_system_boundary(&mut self, name: &str, x: f32, y: f32, w: f32, h: f32) -> Uuid {
        // Center on cursor position
        let boundary = SystemBoundary::new(name, x - w / 2.0, y - h / 2.0, w, h);
        let id = boundary.id;
        self.system_boundaries.push(boundary);
        id
    }

    /// Find an actor at a given point
    pub fn find_actor_at(&self, pos: Point) -> Option<Uuid> {
        self.actors.iter()
            .find(|a| a.contains_point(pos))
            .map(|a| a.id)
    }

    /// Find a use case at a given point
    pub fn find_use_case_at(&self, pos: Point) -> Option<Uuid> {
        self.use_cases.iter()
            .find(|u| u.contains_point(pos))
            .map(|u| u.id)
    }

    /// Find a system boundary at a given point
    pub fn find_system_boundary_at(&self, pos: Point) -> Option<Uuid> {
        self.system_boundaries.iter()
            .find(|s| s.contains_point(pos))
            .map(|s| s.id)
    }

    // ===== Activity Diagram Methods =====

    /// Add an action to the activity diagram (centered on position)
    pub fn add_action(&mut self, name: &str, x: f32, y: f32) -> Uuid {
        // Action: x, y is top-left, default size 100x50
        // Center on cursor position
        let action = Action::new(name, x - 50.0, y - 25.0);
        let id = action.id;
        self.actions.push(action);
        id
    }

    /// Add a swimlane to the activity diagram (centered on position)
    pub fn add_swimlane(&mut self, name: &str, x: f32, y: f32, w: f32, h: f32) -> Uuid {
        // Center on cursor position
        let swimlane = Swimlane::new(name, x - w / 2.0, y - h / 2.0, w, h);
        let id = swimlane.id;
        self.swimlanes.push(swimlane);
        id
    }

    /// Add a control flow between two activity nodes
    pub fn add_control_flow(&mut self, source_id: Uuid, target_id: Uuid) -> Uuid {
        let flow = ControlFlow::new(source_id, target_id);
        let id = flow.id;
        self.control_flows.push(flow);
        id
    }

    /// Add a decision/merge node (centered on position)
    pub fn add_decision_node(&mut self, x: f32, y: f32) -> Uuid {
        use crate::activity::ActionKind;
        let mut action = Action::new("", x - 15.0, y - 15.0);
        action.kind = ActionKind::Action; // We'll use a pseudo-state for this
        action.bounds = crate::geometry::Rect::from_pos_size(x - 15.0, y - 15.0, 30.0, 30.0);
        let id = action.id;
        self.actions.push(action);
        id
    }

    /// Add a send signal action (centered on position)
    pub fn add_send_signal(&mut self, name: &str, x: f32, y: f32) -> Uuid {
        use crate::activity::ActionKind;
        // Default size is 100x40
        let mut action = Action::new(name, x - 50.0, y - 20.0);
        action.kind = ActionKind::SendSignal;
        action.bounds = crate::geometry::Rect::from_pos_size(x - 50.0, y - 20.0, 100.0, 40.0);
        let id = action.id;
        self.actions.push(action);
        id
    }

    /// Add an accept event action (centered on position)
    pub fn add_accept_event(&mut self, name: &str, x: f32, y: f32) -> Uuid {
        use crate::activity::ActionKind;
        // Default size is 100x40
        let mut action = Action::new(name, x - 50.0, y - 20.0);
        action.kind = ActionKind::AcceptEvent;
        action.bounds = crate::geometry::Rect::from_pos_size(x - 50.0, y - 20.0, 100.0, 40.0);
        let id = action.id;
        self.actions.push(action);
        id
    }

    /// Add a time event action (centered on position)
    pub fn add_time_event(&mut self, name: &str, x: f32, y: f32) -> Uuid {
        use crate::activity::ActionKind;
        // Default size is 30x40
        let mut action = Action::new(name, x - 15.0, y - 20.0);
        action.kind = ActionKind::AcceptTimeEvent;
        action.bounds = crate::geometry::Rect::from_pos_size(x - 15.0, y - 20.0, 30.0, 40.0);
        let id = action.id;
        self.actions.push(action);
        id
    }

    /// Add an object node (centered on position)
    pub fn add_object_node(&mut self, name: &str, x: f32, y: f32) -> Uuid {
        use crate::activity::{ObjectNode, ObjectNodeKind};
        // Default size is 80x40
        let node = ObjectNode::new(name, ObjectNodeKind::CentralBuffer, x - 40.0, y - 20.0);
        let id = node.id;
        self.object_nodes.push(node);
        id
    }

    /// Add a data store (centered on position)
    pub fn add_data_store(&mut self, name: &str, x: f32, y: f32) -> Uuid {
        use crate::activity::ObjectNode;
        // Default size is 80x40
        let node = ObjectNode::new_data_store(name, x - 40.0, y - 20.0);
        let id = node.id;
        self.object_nodes.push(node);
        id
    }

    /// Add a combined fragment to a sequence diagram (centered on position)
    pub fn add_combined_fragment(&mut self, x: f32, y: f32, w: f32, h: f32) -> Uuid {
        use crate::sequence::{CombinedFragment, FragmentKind};
        // Center on cursor position
        let fragment = CombinedFragment::new(FragmentKind::Alt, x - w / 2.0, y - h / 2.0, w, h);
        let id = fragment.id;
        self.fragments.push(fragment);
        id
    }

    /// Find an action at a given point
    pub fn find_action_at(&self, pos: Point) -> Option<Uuid> {
        self.actions.iter()
            .find(|a| a.contains_point(pos))
            .map(|a| a.id)
    }

    /// Find a swimlane at a given point
    pub fn find_swimlane_at(&self, pos: Point) -> Option<Uuid> {
        self.swimlanes.iter()
            .find(|s| s.contains_point(pos))
            .map(|s| s.id)
    }

    /// Find an object node at a given point
    pub fn find_object_node_at(&self, pos: Point) -> Option<Uuid> {
        self.object_nodes.iter()
            .find(|o| o.contains_point(pos))
            .map(|o| o.id)
    }

    /// Find an action by ID
    pub fn find_action(&self, id: Uuid) -> Option<&Action> {
        self.actions.iter().find(|a| a.id == id)
    }

    /// Find a mutable action by ID
    pub fn find_action_mut(&mut self, id: Uuid) -> Option<&mut Action> {
        self.actions.iter_mut().find(|a| a.id == id)
    }
}

impl Default for Diagram {
    fn default() -> Self {
        Self::new("Untitled")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_remove_nodes() {
        let mut diagram = Diagram::new("Test");

        let id1 = diagram.add_state("State1", 100.0, 100.0);
        let id2 = diagram.add_state("State2", 200.0, 200.0);

        assert_eq!(diagram.nodes().len(), 2);

        diagram.remove_node(id1);
        assert_eq!(diagram.nodes().len(), 1);
        assert!(diagram.find_node(id1).is_none());
        assert!(diagram.find_node(id2).is_some());
    }

    #[test]
    fn test_connections() {
        let mut diagram = Diagram::new("Test");

        let id1 = diagram.add_state("State1", 100.0, 100.0);
        let id2 = diagram.add_state("State2", 100.0, 300.0);

        let conn_id = diagram.add_connection(id1, id2).unwrap();
        assert_eq!(diagram.connections().len(), 1);

        // Removing a node should remove its connections
        diagram.remove_node(id1);
        assert_eq!(diagram.connections().len(), 0);
        assert!(diagram.find_connection(conn_id).is_none());
    }

    #[test]
    fn test_undo_redo() {
        let mut diagram = Diagram::new("Test");

        let id1 = diagram.add_state("State1", 100.0, 100.0);
        diagram.push_undo();

        diagram.add_state("State2", 200.0, 200.0);
        assert_eq!(diagram.nodes().len(), 2);

        diagram.undo();
        assert_eq!(diagram.nodes().len(), 1);

        diagram.redo();
        assert_eq!(diagram.nodes().len(), 2);
    }
}
