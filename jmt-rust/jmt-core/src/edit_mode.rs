//! Edit modes for the diagram editor

use serde::{Deserialize, Serialize};
use crate::DiagramType;

/// The current editing mode of the canvas
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum EditMode {
    /// Default mode - select and move nodes
    #[default]
    Arrow,
    /// Rectangle selection mode
    Select,
    /// Lasso (freeform) selection mode
    Lasso,

    // === State Machine Diagram modes ===
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
    /// Adding a region to a composite state
    AddRegion,

    // === Sequence Diagram modes ===
    /// Adding a lifeline
    AddLifeline,
    /// Adding a message between lifelines
    AddMessage,
    /// Adding a synchronous message
    AddSyncMessage,
    /// Adding an asynchronous message
    AddAsyncMessage,
    /// Adding a return message
    AddReturnMessage,
    /// Adding a self message
    AddSelfMessage,
    /// Adding an activation box
    AddActivation,
    /// Adding a combined fragment (alt, opt, loop, etc.)
    AddFragment,

    // === Use Case Diagram modes ===
    /// Adding an actor
    AddActor,
    /// Adding a use case
    AddUseCase,
    /// Adding a system boundary
    AddSystemBoundary,
    /// Adding an association
    AddAssociation,
    /// Adding an include relationship
    AddInclude,
    /// Adding an extend relationship
    AddExtend,
    /// Adding a generalization
    AddGeneralization,

    // === Activity Diagram modes ===
    /// Adding an action
    AddAction,
    /// Adding a decision/merge node (diamond)
    AddDecision,
    /// Adding a send signal action
    AddSendSignal,
    /// Adding an accept event action
    AddAcceptEvent,
    /// Adding an accept time event action
    AddTimeEvent,
    /// Adding a swimlane/partition
    AddSwimlane,
    /// Adding an object node
    AddObjectNode,
    /// Adding a data store
    AddDataStore,

    // === Common modes ===
    /// Creating a connection/transition/flow between nodes
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
            // State machine
            EditMode::AddState
                | EditMode::AddInitial
                | EditMode::AddFinal
                | EditMode::AddChoice
                | EditMode::AddJunction
                | EditMode::AddFork
                | EditMode::AddJoin
            // Sequence
                | EditMode::AddLifeline
                | EditMode::AddFragment
            // Use case
                | EditMode::AddActor
                | EditMode::AddUseCase
                | EditMode::AddSystemBoundary
            // Activity
                | EditMode::AddAction
                | EditMode::AddDecision
                | EditMode::AddSendSignal
                | EditMode::AddAcceptEvent
                | EditMode::AddTimeEvent
                | EditMode::AddSwimlane
                | EditMode::AddObjectNode
                | EditMode::AddDataStore
        )
    }

    /// Returns true if this mode is for adding a connection/relationship
    pub fn is_add_connection(&self) -> bool {
        matches!(
            self,
            EditMode::Connect
                | EditMode::AddMessage
                | EditMode::AddSyncMessage
                | EditMode::AddAsyncMessage
                | EditMode::AddReturnMessage
                | EditMode::AddSelfMessage
                | EditMode::AddAssociation
                | EditMode::AddInclude
                | EditMode::AddExtend
                | EditMode::AddGeneralization
        )
    }

    /// Returns the display name for this mode
    pub fn display_name(&self) -> &'static str {
        match self {
            EditMode::Arrow => "Select",
            EditMode::Select => "Rectangle Select",
            EditMode::Lasso => "Lasso Select",
            // State machine
            EditMode::AddState => "Add State",
            EditMode::AddInitial => "Add Initial",
            EditMode::AddFinal => "Add Final",
            EditMode::AddChoice => "Add Choice",
            EditMode::AddJunction => "Add Junction",
            EditMode::AddFork => "Add Fork",
            EditMode::AddJoin => "Add Join",
            EditMode::AddRegion => "Add Region",
            // Sequence
            EditMode::AddLifeline => "Add Lifeline",
            EditMode::AddMessage => "Add Message",
            EditMode::AddSyncMessage => "Add Sync Message",
            EditMode::AddAsyncMessage => "Add Async Message",
            EditMode::AddReturnMessage => "Add Return Message",
            EditMode::AddSelfMessage => "Add Self Message",
            EditMode::AddActivation => "Add Activation",
            EditMode::AddFragment => "Add Fragment",
            // Use case
            EditMode::AddActor => "Add Actor",
            EditMode::AddUseCase => "Add Use Case",
            EditMode::AddSystemBoundary => "Add System Boundary",
            EditMode::AddAssociation => "Add Association",
            EditMode::AddInclude => "Add Include",
            EditMode::AddExtend => "Add Extend",
            EditMode::AddGeneralization => "Add Generalization",
            // Activity
            EditMode::AddAction => "Add Action",
            EditMode::AddDecision => "Add Decision",
            EditMode::AddSendSignal => "Add Send Signal",
            EditMode::AddAcceptEvent => "Add Accept Event",
            EditMode::AddTimeEvent => "Add Time Event",
            EditMode::AddSwimlane => "Add Swimlane",
            EditMode::AddObjectNode => "Add Object Node",
            EditMode::AddDataStore => "Add Data Store",
            // Common
            EditMode::Connect => "Connect",
            EditMode::Move => "Moving",
            EditMode::Resize => "Resizing",
            EditMode::MoveRegionSeparator => "Moving Separator",
        }
    }

    /// Returns edit modes available for a given diagram type
    pub fn modes_for_diagram_type(diagram_type: DiagramType) -> Vec<EditMode> {
        let mut modes = vec![EditMode::Arrow, EditMode::Lasso];

        match diagram_type {
            DiagramType::StateMachine => {
                modes.extend([
                    EditMode::AddState,
                    EditMode::AddInitial,
                    EditMode::AddFinal,
                    EditMode::AddChoice,
                    EditMode::AddJunction,
                    EditMode::AddFork,
                    EditMode::AddJoin,
                    EditMode::Connect,
                ]);
            }
            DiagramType::Sequence => {
                modes.extend([
                    EditMode::AddLifeline,
                    EditMode::AddSyncMessage,
                    EditMode::AddAsyncMessage,
                    EditMode::AddReturnMessage,
                    EditMode::AddSelfMessage,
                    EditMode::AddActivation,
                    EditMode::AddFragment,
                ]);
            }
            DiagramType::UseCase => {
                modes.extend([
                    EditMode::AddActor,
                    EditMode::AddUseCase,
                    EditMode::AddSystemBoundary,
                    EditMode::AddAssociation,
                    EditMode::AddInclude,
                    EditMode::AddExtend,
                    EditMode::AddGeneralization,
                ]);
            }
            DiagramType::Activity => {
                modes.extend([
                    EditMode::AddAction,
                    EditMode::AddInitial,
                    EditMode::AddFinal,
                    EditMode::AddDecision,
                    EditMode::AddFork,
                    EditMode::AddJoin,
                    EditMode::AddSendSignal,
                    EditMode::AddAcceptEvent,
                    EditMode::AddTimeEvent,
                    EditMode::AddSwimlane,
                    EditMode::AddObjectNode,
                    EditMode::AddDataStore,
                    EditMode::Connect,
                ]);
            }
        }

        modes
    }
}
