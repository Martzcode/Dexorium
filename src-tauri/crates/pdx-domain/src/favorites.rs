use serde::{Deserialize, Serialize};

/// Kinds of entities that can be starred in the collection.
///
/// The MVP only tracks favourite species. The enum keeps the persisted shape
/// (`entity_type`, `entity_id`) forward compatible with the collections
/// described in the specifications.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FavoriteKind {
    Species,
}

impl FavoriteKind {
    /// String stored in `favorite.entity_type`.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Species => "species",
        }
    }

    /// Parses a persisted entity type.
    #[must_use]
    pub fn parse(value: &str) -> Option<Self> {
        match value {
            "species" => Some(Self::Species),
            _ => None,
        }
    }
}

/// One starred entity.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FavoriteEntry {
    pub kind: FavoriteKind,
    pub entity_id: i64,
    /// ISO 8601 timestamp recorded when the entity was starred.
    pub created_at: String,
}

/// Full favourite list, ordered from the most recent to the oldest.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FavoriteCollection {
    pub items: Vec<FavoriteEntry>,
    pub total: usize,
}

/// Answer to the "is this entity starred?" query.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FavoriteStatus {
    pub kind: FavoriteKind,
    pub entity_id: i64,
    pub favorite: bool,
}
