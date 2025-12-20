//! Diagram types supported by JMT

use serde::{Deserialize, Serialize};

/// The type of UML diagram
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
pub enum DiagramType {
    /// State machine diagram (default)
    #[default]
    StateMachine,
    /// Sequence diagram
    Sequence,
    /// Use case diagram
    UseCase,
    /// Activity diagram
    Activity,
}

impl DiagramType {
    /// Returns the display name for this diagram type
    pub fn display_name(&self) -> &'static str {
        match self {
            DiagramType::StateMachine => "State Machine",
            DiagramType::Sequence => "Sequence",
            DiagramType::UseCase => "Use Case",
            DiagramType::Activity => "Activity",
        }
    }

    /// Returns all available diagram types
    pub fn all() -> &'static [DiagramType] {
        &[
            DiagramType::StateMachine,
            DiagramType::Sequence,
            DiagramType::UseCase,
            DiagramType::Activity,
        ]
    }

    /// Returns an icon character for this diagram type
    pub fn icon(&self) -> &'static str {
        match self {
            DiagramType::StateMachine => "⬡",  // hexagon/state
            DiagramType::Sequence => "⎸⎹",     // vertical lines
            DiagramType::UseCase => "○",       // ellipse
            DiagramType::Activity => "▷",      // activity arrow
        }
    }
}
