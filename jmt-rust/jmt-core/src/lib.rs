//! JMT Core - Data types and domain logic for the state machine editor

pub mod geometry;
pub mod node;
pub mod connection;
pub mod diagram;
pub mod edit_mode;
pub mod settings;

pub use geometry::{Point, Rect, Color};
pub use node::{NodeType, Side, Corner, Node, NodeId};
pub use connection::{Connection, ConnectionId, LineSegment};
pub use diagram::Diagram;
pub use edit_mode::EditMode;
pub use settings::DiagramSettings;
