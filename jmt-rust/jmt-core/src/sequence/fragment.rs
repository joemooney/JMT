//! Combined Fragment - represents alt, opt, loop, etc. in sequence diagrams

use serde::{Deserialize, Serialize};
use uuid::Uuid;
use crate::geometry::Rect;

/// Unique identifier for a combined fragment
pub type FragmentId = Uuid;

/// The kind of combined fragment
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
pub enum FragmentKind {
    /// Alternative (if-else)
    #[default]
    Alt,
    /// Optional
    Opt,
    /// Loop
    Loop,
    /// Parallel (concurrent execution)
    Par,
    /// Critical region
    Critical,
    /// Negative (invalid trace)
    Neg,
    /// Assertion
    Assert,
    /// Strict sequencing
    Strict,
    /// Weak sequencing
    Seq,
    /// Ignore
    Ignore,
    /// Consider
    Consider,
    /// Break
    Break,
    /// Reference to another interaction
    Ref,
}

impl FragmentKind {
    /// Returns the display name (keyword shown in the pentagon)
    pub fn display_name(&self) -> &'static str {
        match self {
            FragmentKind::Alt => "alt",
            FragmentKind::Opt => "opt",
            FragmentKind::Loop => "loop",
            FragmentKind::Par => "par",
            FragmentKind::Critical => "critical",
            FragmentKind::Neg => "neg",
            FragmentKind::Assert => "assert",
            FragmentKind::Strict => "strict",
            FragmentKind::Seq => "seq",
            FragmentKind::Ignore => "ignore",
            FragmentKind::Consider => "consider",
            FragmentKind::Break => "break",
            FragmentKind::Ref => "ref",
        }
    }

    /// Returns all fragment kinds
    pub fn all() -> &'static [FragmentKind] {
        &[
            FragmentKind::Alt,
            FragmentKind::Opt,
            FragmentKind::Loop,
            FragmentKind::Par,
            FragmentKind::Critical,
            FragmentKind::Neg,
            FragmentKind::Assert,
            FragmentKind::Break,
            FragmentKind::Ref,
        ]
    }
}

/// An operand within a combined fragment (separated by dashed lines)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InteractionOperand {
    /// Unique identifier
    pub id: Uuid,
    /// Guard condition (e.g., "[x > 0]", "[else]")
    pub guard: Option<String>,
    /// Y position of the top of this operand
    pub start_y: f32,
    /// Y position of the bottom of this operand
    pub end_y: f32,
}

impl InteractionOperand {
    /// Create a new operand
    pub fn new(guard: Option<&str>, start_y: f32, end_y: f32) -> Self {
        Self {
            id: Uuid::new_v4(),
            guard: guard.map(|s| s.to_string()),
            start_y,
            end_y,
        }
    }

    /// Get the height of this operand
    pub fn height(&self) -> f32 {
        self.end_y - self.start_y
    }
}

/// A combined fragment in a sequence diagram
/// Groups a set of messages with a specific semantic (alt, opt, loop, etc.)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CombinedFragment {
    /// Unique identifier
    pub id: FragmentId,
    /// The kind of fragment
    pub kind: FragmentKind,
    /// Bounding rectangle
    pub bounds: Rect,
    /// Operands within this fragment (for alt, par, etc.)
    pub operands: Vec<InteractionOperand>,
    /// Optional label for loop bounds (e.g., "1, 5" for loop(1,5))
    pub loop_bounds: Option<String>,
    /// Whether this fragment is currently selected
    #[serde(skip)]
    pub selected: bool,
}

impl CombinedFragment {
    /// Create a new combined fragment
    pub fn new(kind: FragmentKind, x: f32, y: f32, width: f32, height: f32) -> Self {
        let bounds = Rect::from_pos_size(x, y, width, height);
        let operand = InteractionOperand::new(None, y + 20.0, y + height);

        Self {
            id: Uuid::new_v4(),
            kind,
            bounds,
            operands: vec![operand],
            loop_bounds: None,
            selected: false,
        }
    }

    /// Create an alt fragment with two operands (if/else)
    pub fn new_alt(x: f32, y: f32, width: f32, height: f32) -> Self {
        let bounds = Rect::from_pos_size(x, y, width, height);
        let mid_y = y + height / 2.0;
        let operands = vec![
            InteractionOperand::new(Some("[condition]"), y + 20.0, mid_y),
            InteractionOperand::new(Some("[else]"), mid_y, y + height),
        ];

        Self {
            id: Uuid::new_v4(),
            kind: FragmentKind::Alt,
            bounds,
            operands,
            loop_bounds: None,
            selected: false,
        }
    }

    /// Add an operand to this fragment
    pub fn add_operand(&mut self, guard: Option<&str>) {
        if let Some(last) = self.operands.last() {
            let new_start = last.end_y;
            let new_end = self.bounds.y2;
            // Adjust the previous operand's end
            if let Some(last_mut) = self.operands.last_mut() {
                last_mut.end_y = new_start;
            }
            self.operands.push(InteractionOperand::new(guard, new_start, new_end));
        }
    }

    /// Translate the fragment by an offset
    pub fn translate(&mut self, dx: f32, dy: f32) {
        self.bounds = self.bounds.translate(dx, dy);
        for operand in &mut self.operands {
            operand.start_y += dy;
            operand.end_y += dy;
        }
    }

    /// Check if a point is inside the fragment
    pub fn contains_point(&self, p: crate::geometry::Point) -> bool {
        self.bounds.contains_point(p)
    }
}

impl Default for CombinedFragment {
    fn default() -> Self {
        Self::new(FragmentKind::Alt, 100.0, 100.0, 200.0, 150.0)
    }
}
