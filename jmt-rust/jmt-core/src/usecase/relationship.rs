//! Relationships in use case diagrams

use serde::{Deserialize, Serialize};
use uuid::Uuid;
use crate::geometry::Point;

/// Unique identifier for a relationship
pub type RelationshipId = Uuid;

/// The kind of relationship in a use case diagram
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
pub enum RelationshipKind {
    /// Association between actor and use case (solid line)
    #[default]
    Association,
    /// Include relationship (dashed arrow with <<include>>)
    Include,
    /// Extend relationship (dashed arrow with <<extend>>)
    Extend,
    /// Generalization (solid line with hollow triangle)
    Generalization,
}

impl RelationshipKind {
    /// Returns the display name
    pub fn display_name(&self) -> &'static str {
        match self {
            RelationshipKind::Association => "Association",
            RelationshipKind::Include => "Include",
            RelationshipKind::Extend => "Extend",
            RelationshipKind::Generalization => "Generalization",
        }
    }

    /// Returns the stereotype label (if any)
    pub fn stereotype(&self) -> Option<&'static str> {
        match self {
            RelationshipKind::Include => Some("<<include>>"),
            RelationshipKind::Extend => Some("<<extend>>"),
            _ => None,
        }
    }

    /// Returns true if this relationship uses a dashed line
    pub fn is_dashed(&self) -> bool {
        matches!(self, RelationshipKind::Include | RelationshipKind::Extend)
    }

    /// Returns true if this relationship has an arrowhead
    pub fn has_arrow(&self) -> bool {
        !matches!(self, RelationshipKind::Association)
    }
}

/// The type of element involved in a relationship
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum UseCaseElementKind {
    Actor,
    UseCase,
}

/// A relationship in a use case diagram
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UseCaseRelationship {
    /// Unique identifier
    pub id: RelationshipId,
    /// Source element ID
    pub source_id: Uuid,
    /// Source element kind
    pub source_kind: UseCaseElementKind,
    /// Target element ID
    pub target_id: Uuid,
    /// Target element kind
    pub target_kind: UseCaseElementKind,
    /// The kind of relationship
    pub kind: RelationshipKind,
    /// Optional condition (for extend relationships)
    pub condition: Option<String>,
    /// Optional extension point reference (for extend relationships)
    pub extension_point: Option<String>,
    /// Whether this relationship is currently selected
    #[serde(skip)]
    pub selected: bool,
}

impl UseCaseRelationship {
    /// Create an association between an actor and a use case
    pub fn new_association(actor_id: Uuid, use_case_id: Uuid) -> Self {
        Self {
            id: Uuid::new_v4(),
            source_id: actor_id,
            source_kind: UseCaseElementKind::Actor,
            target_id: use_case_id,
            target_kind: UseCaseElementKind::UseCase,
            kind: RelationshipKind::Association,
            condition: None,
            extension_point: None,
            selected: false,
        }
    }

    /// Create an include relationship between use cases
    pub fn new_include(base_id: Uuid, included_id: Uuid) -> Self {
        Self {
            id: Uuid::new_v4(),
            source_id: base_id,
            source_kind: UseCaseElementKind::UseCase,
            target_id: included_id,
            target_kind: UseCaseElementKind::UseCase,
            kind: RelationshipKind::Include,
            condition: None,
            extension_point: None,
            selected: false,
        }
    }

    /// Create an extend relationship between use cases
    pub fn new_extend(extending_id: Uuid, base_id: Uuid, condition: Option<&str>) -> Self {
        Self {
            id: Uuid::new_v4(),
            source_id: extending_id,
            source_kind: UseCaseElementKind::UseCase,
            target_id: base_id,
            target_kind: UseCaseElementKind::UseCase,
            kind: RelationshipKind::Extend,
            condition: condition.map(|s| s.to_string()),
            extension_point: None,
            selected: false,
        }
    }

    /// Create a generalization relationship
    pub fn new_generalization(child_id: Uuid, child_kind: UseCaseElementKind, parent_id: Uuid, parent_kind: UseCaseElementKind) -> Self {
        Self {
            id: Uuid::new_v4(),
            source_id: child_id,
            source_kind: child_kind,
            target_id: parent_id,
            target_kind: parent_kind,
            kind: RelationshipKind::Generalization,
            condition: None,
            extension_point: None,
            selected: false,
        }
    }

    /// Check if a point is near this relationship line
    pub fn is_near_point(&self, p: Point, source_point: Point, target_point: Point, tolerance: f32) -> bool {
        // Calculate distance from point to line segment
        let dx = target_point.x - source_point.x;
        let dy = target_point.y - source_point.y;
        let length_sq = dx * dx + dy * dy;

        if length_sq == 0.0 {
            return p.distance_to(source_point) <= tolerance;
        }

        // Project point onto line
        let t = ((p.x - source_point.x) * dx + (p.y - source_point.y) * dy) / length_sq;
        let t = t.clamp(0.0, 1.0);

        let proj = Point::new(
            source_point.x + t * dx,
            source_point.y + t * dy,
        );

        p.distance_to(proj) <= tolerance
    }
}

impl Default for UseCaseRelationship {
    fn default() -> Self {
        Self {
            id: Uuid::new_v4(),
            source_id: Uuid::nil(),
            source_kind: UseCaseElementKind::Actor,
            target_id: Uuid::nil(),
            target_kind: UseCaseElementKind::UseCase,
            kind: RelationshipKind::Association,
            condition: None,
            extension_point: None,
            selected: false,
        }
    }
}
