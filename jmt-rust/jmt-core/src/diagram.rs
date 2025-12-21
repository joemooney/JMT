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

/// A complete UML diagram (state machine, sequence, use case, or activity)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Diagram {
    /// Unique identifier
    pub id: Uuid,
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
        // Check in reverse order (top-most first)
        self.nodes.iter().rev().find(|n| n.contains_point(pos)).map(|n| n.id())
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
        let region_id = self.find_region_at_point(x, y)
            .unwrap_or_else(|| self.root_region_id());
        self.assign_node_to_region(id, region_id);

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
        let region_id = self.find_region_at_point(x, y)
            .unwrap_or_else(|| self.root_region_id());
        self.assign_node_to_region(id, region_id);

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
    pub fn find_region_at_point(&self, x: f32, y: f32) -> Option<Uuid> {
        // Check state nodes' regions first (inner states before outer)
        // We want the innermost region that contains the point
        let mut best_match: Option<(Uuid, f32)> = None; // (region_id, area)

        for node in &self.nodes {
            if let Node::State(state) = node {
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

    /// Find the innermost state containing a point (by state bounds, not regions)
    /// Returns the state's node ID, or None if only the root state contains it
    pub fn find_state_at_point(&self, x: f32, y: f32) -> Option<NodeId> {
        let point = Point::new(x, y);
        let mut best_match: Option<(NodeId, f32)> = None; // (state_id, area)

        for node in &self.nodes {
            if let Node::State(state) = node {
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

    /// Re-parent a node to the appropriate region based on its current position
    pub fn update_node_region(&mut self, node_id: NodeId) {
        // Get the node's center position
        let center = self.find_node(node_id)
            .map(|n| n.bounds().center());

        if let Some(pos) = center {
            // First, check if we're inside a state that has no regions
            // If so, create a default region for it
            if let Some(state_id) = self.find_state_at_point(pos.x, pos.y) {
                // Don't assign a node to itself
                if state_id != node_id {
                    // Check if this state has any regions
                    let needs_region = self.find_node(state_id)
                        .and_then(|n| n.as_state())
                        .map(|s| s.regions.is_empty())
                        .unwrap_or(false);

                    if needs_region {
                        // Create a default region for this state
                        if let Some(Node::State(state)) = self.find_node_mut(state_id) {
                            state.add_region("default");
                        }
                    }
                }
            }

            // Now find which region should contain this node
            if let Some(region_id) = self.find_region_at_point(pos.x, pos.y) {
                // Get current parent
                let current_parent = self.find_node(node_id)
                    .and_then(|n| n.parent_region_id());

                // Only update if different
                if current_parent != Some(region_id) {
                    self.assign_node_to_region(node_id, region_id);
                }
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

        // Third pass: recalculate segments with updated offsets
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

    /// Select all elements whose center is inside a rectangle (for marquee selection)
    /// Works for all diagram types
    pub fn select_elements_in_rect(&mut self, rect: &Rect) {
        self.clear_selection();

        match self.diagram_type {
            DiagramType::StateMachine => {
                // Select nodes whose center is in the rect
                let mut ids = Vec::new();
                for node in &self.nodes {
                    let center = node.bounds().center();
                    if rect.contains_point(center) {
                        ids.push(node.id());
                    }
                }
                self.select_nodes(&ids);
            }
            DiagramType::Sequence => {
                // Select lifelines
                for lifeline in &mut self.lifelines {
                    let center = lifeline.head_bounds().center();
                    if rect.contains_point(center) {
                        lifeline.has_focus = true;
                        self.selection_order.push(lifeline.id);
                    }
                }
            }
            DiagramType::UseCase => {
                // Select actors
                for actor in &mut self.actors {
                    let center = actor.center();
                    if rect.contains_point(center) {
                        actor.has_focus = true;
                        self.selection_order.push(actor.id);
                    }
                }
                // Select use cases
                for uc in &mut self.use_cases {
                    let center = uc.center();
                    if rect.contains_point(center) {
                        uc.has_focus = true;
                        self.selection_order.push(uc.id);
                    }
                }
                // Select system boundaries
                for sb in &mut self.system_boundaries {
                    let center = sb.bounds.center();
                    if rect.contains_point(center) {
                        sb.has_focus = true;
                        self.selection_order.push(sb.id);
                    }
                }
            }
            DiagramType::Activity => {
                // Select actions
                for action in &mut self.actions {
                    let center = action.center();
                    if rect.contains_point(center) {
                        action.has_focus = true;
                        self.selection_order.push(action.id);
                    }
                }
                // Select swimlanes
                for swimlane in &mut self.swimlanes {
                    let center = swimlane.bounds.center();
                    if rect.contains_point(center) {
                        swimlane.has_focus = true;
                        self.selection_order.push(swimlane.id);
                    }
                }
                // Select object nodes
                for obj in &mut self.object_nodes {
                    let center = obj.bounds.center();
                    if rect.contains_point(center) {
                        obj.has_focus = true;
                        self.selection_order.push(obj.id);
                    }
                }
            }
        }
        // Marquee selection is not explicit ordering
        self.explicit_selection_order = false;
    }

    /// Select all elements whose center is inside a polygon (for lasso selection)
    pub fn select_elements_in_polygon(&mut self, polygon: &[Point]) {
        use crate::geometry::point_in_polygon;

        self.clear_selection();

        match self.diagram_type {
            DiagramType::StateMachine => {
                // Select nodes whose center is in the polygon
                let mut ids = Vec::new();
                for node in &self.nodes {
                    let center = node.bounds().center();
                    if point_in_polygon(center, polygon) {
                        ids.push(node.id());
                    }
                }
                self.select_nodes(&ids);
            }
            DiagramType::Sequence => {
                // Select lifelines
                for lifeline in &mut self.lifelines {
                    let center = lifeline.head_bounds().center();
                    if point_in_polygon(center, polygon) {
                        lifeline.has_focus = true;
                        self.selection_order.push(lifeline.id);
                    }
                }
            }
            DiagramType::UseCase => {
                // Select actors
                for actor in &mut self.actors {
                    let center = actor.center();
                    if point_in_polygon(center, polygon) {
                        actor.has_focus = true;
                        self.selection_order.push(actor.id);
                    }
                }
                // Select use cases
                for uc in &mut self.use_cases {
                    let center = uc.center();
                    if point_in_polygon(center, polygon) {
                        uc.has_focus = true;
                        self.selection_order.push(uc.id);
                    }
                }
                // Select system boundaries
                for sb in &mut self.system_boundaries {
                    let center = sb.bounds.center();
                    if point_in_polygon(center, polygon) {
                        sb.has_focus = true;
                        self.selection_order.push(sb.id);
                    }
                }
            }
            DiagramType::Activity => {
                // Select actions
                for action in &mut self.actions {
                    let center = action.center();
                    if point_in_polygon(center, polygon) {
                        action.has_focus = true;
                        self.selection_order.push(action.id);
                    }
                }
                // Select swimlanes
                for swimlane in &mut self.swimlanes {
                    let center = swimlane.bounds.center();
                    if point_in_polygon(center, polygon) {
                        swimlane.has_focus = true;
                        self.selection_order.push(swimlane.id);
                    }
                }
                // Select object nodes
                for obj in &mut self.object_nodes {
                    let center = obj.bounds.center();
                    if point_in_polygon(center, polygon) {
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
