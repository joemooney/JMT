//! Sequence diagram elements

mod lifeline;
mod message;
mod activation;
mod fragment;

pub use lifeline::Lifeline;
pub use message::{Message, MessageKind};
pub use activation::Activation;
pub use fragment::{CombinedFragment, FragmentKind, InteractionOperand};
