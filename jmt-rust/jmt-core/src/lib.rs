//! JMT Core - Data types and domain logic for UML diagram editors

pub mod geometry;
pub mod node;
pub mod connection;
pub mod diagram;
pub mod diagram_type;
pub mod edit_mode;
pub mod settings;

// Sequence diagram specific
pub mod sequence;
// Use case diagram specific
pub mod usecase;
// Activity diagram specific
pub mod activity;

pub use geometry::{Point, Rect, Color};
pub use node::{NodeType, Side, Corner, Node, NodeId};
pub use connection::{Connection, ConnectionId, LineSegment};
pub use diagram::Diagram;
pub use diagram_type::DiagramType;
pub use edit_mode::EditMode;
pub use settings::DiagramSettings;
