//! Edit modes for the diagram editor

use serde::{Deserialize, Serialize};

/// The current editing mode of the canvas
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum EditMode {
    /// Default mode - select and move nodes
    #[default]
    Arrow,
    /// Rectangle selection mode
    Select,
    /// Adding a new state
    AddState,
    /// Adding an initial pseudo-state
    AddInitial,
    /// Adding a final pseudo-state
    AddFinal,
    /// Adding a choice pseudo-state
    AddChoice,
    /// Adding a junction pseudo-state
    AddJunction,
    /// Adding a fork pseudo-state
    AddFork,
    /// Adding a join pseudo-state
    AddJoin,
    /// Creating a connection between nodes
    Connect,
    /// Moving a selected node
    Move,
    /// Resizing a node from a corner
    Resize,
    /// Moving a region separator
    MoveRegionSeparator,
}

impl EditMode {
    /// Returns true if this mode is for adding a new node
    pub fn is_add_node(&self) -> bool {
        matches!(
            self,
            EditMode::AddState
                | EditMode::AddInitial
                | EditMode::AddFinal
                | EditMode::AddChoice
                | EditMode::AddJunction
                | EditMode::AddFork
                | EditMode::AddJoin
        )
    }

    /// Returns the display name for this mode
    pub fn display_name(&self) -> &'static str {
        match self {
            EditMode::Arrow => "Select",
            EditMode::Select => "Rectangle Select",
            EditMode::AddState => "Add State",
            EditMode::AddInitial => "Add Initial",
            EditMode::AddFinal => "Add Final",
            EditMode::AddChoice => "Add Choice",
            EditMode::AddJunction => "Add Junction",
            EditMode::AddFork => "Add Fork",
            EditMode::AddJoin => "Add Join",
            EditMode::Connect => "Connect",
            EditMode::Move => "Moving",
            EditMode::Resize => "Resizing",
            EditMode::MoveRegionSeparator => "Moving Separator",
        }
    }
}
