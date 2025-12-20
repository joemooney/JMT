//! Diagram - the top-level container for a state machine diagram

use serde::{Deserialize, Serialize};
use uuid::Uuid;
use crate::geometry::{Point, Rect};
use crate::node::{Node, NodeId, NodeType, State, PseudoState, PseudoStateKind};
use crate::connection::{Connection, ConnectionId};
use crate::settings::DiagramSettings;

/// A complete state machine diagram
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Diagram {
    /// Unique identifier
    pub id: Uuid,
    /// Diagram settings
    pub settings: DiagramSettings,
    /// Root state (contains all other nodes)
    pub root_state: State,
    /// All nodes in the diagram (indexed by ID for fast lookup)
    nodes: Vec<Node>,
    /// All connections in the diagram
    connections: Vec<Connection>,
    /// Undo history (serialized diagram snapshots)
    #[serde(skip)]
    undo_stack: Vec<String>,
    /// Redo history
    #[serde(skip)]
    redo_stack: Vec<String>,
    /// Maximum undo levels
    #[serde(skip)]
    max_undo_levels: usize,
}

impl Diagram {
    /// Create a new empty diagram
    pub fn new(name: &str) -> Self {
        let mut root_state = State::new("Root", 50.0, 50.0, 700.0, 500.0);
        root_state.add_region("default");

        Self {
            id: Uuid::new_v4(),
            settings: DiagramSettings::new(name),
            root_state,
            nodes: Vec::new(),
            connections: Vec::new(),
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
            max_undo_levels: 50,
        }
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

    /// Add a new state at the given position
    pub fn add_state(&mut self, name: &str, x: f32, y: f32) -> NodeId {
        let state = State::new(
            name,
            x,
            y,
            self.settings.default_state_width,
            self.settings.default_state_height,
        );
        let id = state.id;
        self.nodes.push(Node::State(state));
        id
    }

    /// Add a new pseudo-state at the given position
    pub fn add_pseudo_state(&mut self, kind: PseudoStateKind, x: f32, y: f32) -> NodeId {
        let pseudo = PseudoState::new(kind, x, y);
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
        conn.calculate_segments(&source_bounds, &target_bounds, self.settings.stub_length);

        let id = conn.id;
        self.connections.push(conn);
        Some(id)
    }

    /// Remove a connection by ID
    pub fn remove_connection(&mut self, id: ConnectionId) {
        self.connections.retain(|c| c.id != id);
    }

    /// Recalculate all connection segments (call after nodes move)
    pub fn recalculate_connections(&mut self) {
        let stub_len = self.settings.stub_length;

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
                conn.calculate_segments(&sb, &tb, stub_len);
            }
        }
    }

    /// Clear all selections
    pub fn clear_selection(&mut self) {
        for node in &mut self.nodes {
            node.set_focus(false);
        }
        for conn in &mut self.connections {
            conn.selected = false;
        }
    }

    /// Select a single node
    pub fn select_node(&mut self, id: NodeId) {
        self.clear_selection();
        if let Some(node) = self.find_node_mut(id) {
            node.set_focus(true);
        }
    }

    /// Select multiple nodes by their IDs
    pub fn select_nodes(&mut self, ids: &[NodeId]) {
        self.clear_selection();
        for id in ids {
            if let Some(node) = self.find_node_mut(*id) {
                node.set_focus(true);
            }
        }
    }

    /// Select all nodes within a rectangle
    pub fn select_nodes_in_rect(&mut self, rect: &Rect) {
        let ids = self.find_nodes_in_rect(rect);
        self.select_nodes(&ids);
    }

    /// Select a single connection
    pub fn select_connection(&mut self, id: ConnectionId) {
        self.clear_selection();
        if let Some(conn) = self.find_connection_mut(id) {
            conn.selected = true;
        }
    }

    /// Get all selected node IDs
    pub fn selected_nodes(&self) -> Vec<NodeId> {
        self.nodes
            .iter()
            .filter(|n| n.has_focus())
            .map(|n| n.id())
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
                self.settings = restored.settings;
                self.root_state = restored.root_state;
                self.nodes = restored.nodes;
                self.connections = restored.connections;
                // Recalculate connection segments (they're not serialized)
                self.recalculate_connections();
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
                self.settings = restored.settings;
                self.root_state = restored.root_state;
                self.nodes = restored.nodes;
                self.connections = restored.connections;
                // Recalculate connection segments (they're not serialized)
                self.recalculate_connections();
                return true;
            }
        }
        false
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
