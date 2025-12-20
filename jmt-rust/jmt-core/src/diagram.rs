//! Diagram - the top-level container for UML diagrams

use serde::{Deserialize, Serialize};
use uuid::Uuid;
use crate::geometry::{Point, Rect};
use crate::node::{Node, NodeId, NodeType, State, PseudoState, PseudoStateKind};
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

    /// Find all nodes within a rectangle
    pub fn find_nodes_in_rect(&self, rect: &Rect) -> Vec<NodeId> {
        self.nodes
            .iter()
            .filter(|n| rect.overlaps(n.bounds()))
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
        id
    }

    /// Add a new pseudo-state at the given position (centered on the position)
    pub fn add_pseudo_state(&mut self, kind: PseudoStateKind, x: f32, y: f32) -> NodeId {
        let (width, height) = kind.default_size();
        // Center on the given position
        let pseudo = PseudoState::new(kind, x - width / 2.0, y - height / 2.0);
        let id = pseudo.id;
        self.nodes.push(Node::Pseudo(pseudo));
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
        // Remove any connections involving this node
        self.connections.retain(|c| c.source_id != id && c.target_id != id);

        // Remove the node
        self.nodes.retain(|n| n.id() != id);
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

    /// Calculate slot offsets for all connections to prevent incoming/outgoing overlap
    fn recalculate_connection_slots(&mut self) {
        use crate::node::Side;

        const SLOT_SPACING: f32 = 15.0;

        // Collect all node IDs
        let node_ids: Vec<NodeId> = self.nodes.iter().map(|n| n.id()).collect();

        // For each node, calculate offsets for connections on each side
        for node_id in node_ids {
            for side in [Side::Top, Side::Bottom, Side::Left, Side::Right] {
                // Find outgoing connections (this node is source, connection uses this side)
                let outgoing_indices: Vec<usize> = self.connections
                    .iter()
                    .enumerate()
                    .filter(|(_, c)| c.source_id == node_id && c.source_side == side)
                    .map(|(i, _)| i)
                    .collect();

                // Find incoming connections (this node is target, connection uses this side)
                let incoming_indices: Vec<usize> = self.connections
                    .iter()
                    .enumerate()
                    .filter(|(_, c)| c.target_id == node_id && c.target_side == side)
                    .map(|(i, _)| i)
                    .collect();

                let num_outgoing = outgoing_indices.len();
                let num_incoming = incoming_indices.len();

                // Assign offsets: outgoing on negative side, incoming on positive side
                // Center each group around their respective side
                for (slot, &idx) in outgoing_indices.iter().enumerate() {
                    let offset = if num_outgoing == 1 && num_incoming == 0 {
                        0.0 // Single connection, center it
                    } else {
                        // Offset to negative side
                        let group_width = (num_outgoing as f32 - 1.0) * SLOT_SPACING;
                        -SLOT_SPACING / 2.0 - group_width / 2.0 + (slot as f32) * SLOT_SPACING
                    };
                    self.connections[idx].source_offset = offset;
                }

                for (slot, &idx) in incoming_indices.iter().enumerate() {
                    let offset = if num_incoming == 1 && num_outgoing == 0 {
                        0.0 // Single connection, center it
                    } else {
                        // Offset to positive side
                        let group_width = (num_incoming as f32 - 1.0) * SLOT_SPACING;
                        SLOT_SPACING / 2.0 - group_width / 2.0 + (slot as f32) * SLOT_SPACING
                    };
                    self.connections[idx].target_offset = offset;
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
    }

    /// Select a single node
    pub fn select_node(&mut self, id: NodeId) {
        self.clear_selection();
        if let Some(node) = self.find_node_mut(id) {
            node.set_focus(true);
            self.selection_order.push(id);
        }
    }

    /// Select multiple nodes by their IDs (preserves order of ids slice)
    pub fn select_nodes(&mut self, ids: &[NodeId]) {
        self.clear_selection();
        for id in ids {
            if let Some(node) = self.find_node_mut(*id) {
                node.set_focus(true);
                self.selection_order.push(*id);
            }
        }
    }

    /// Select all nodes within a rectangle
    pub fn select_nodes_in_rect(&mut self, rect: &Rect) {
        let ids = self.find_nodes_in_rect(rect);
        self.select_nodes(&ids);
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
    }

    /// Toggle a node's selection (add if not selected, remove if selected)
    pub fn toggle_node_selection(&mut self, id: NodeId) {
        if let Some(node) = self.find_node_mut(id) {
            let currently_selected = node.has_focus();
            if currently_selected {
                // Remove from selection
                node.set_focus(false);
                self.selection_order.retain(|&x| x != id);
            } else {
                // Add to selection (at end of order)
                node.set_focus(true);
                self.selection_order.push(id);
            }
        }
    }

    /// Add a node to the current selection without clearing existing selection
    pub fn add_to_selection(&mut self, id: NodeId) {
        if let Some(node) = self.find_node_mut(id) {
            if !node.has_focus() {
                node.set_focus(true);
                self.selection_order.push(id);
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
