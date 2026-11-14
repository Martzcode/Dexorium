use serde::{Deserialize, Serialize};

/// One of the eighteen battle types.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TypeRef {
    /// Stable PokeAPI slug, also used as the CSS token in the UI.
    pub slug: String,
    /// Localized display name.
    pub name: String,
    /// Generation that introduced the type.
    pub introduced_in: u8,
}

/// Type effectiveness chart for one generation.
///
/// `multipliers` is row-major: `multipliers[attacker][defender]` holds the
/// damage multiplier as an integer percentage (`0`, `25`, `50`, `100`, `200`
/// or `400`). Rows and columns follow the order of `types`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TypeMatrix {
    /// Generation the chart was computed for.
    pub generation: u8,
    /// Generations available in the dataset, newest first.
    pub generations: Vec<u8>,
    /// Types in display order (the eighteen types supported by the MVP).
    pub types: Vec<TypeRef>,
    /// Effectiveness values expressed as percentages.
    pub multipliers: Vec<Vec<u16>>,
}

impl TypeMatrix {
    /// Returns the multiplier for an attacking/defending pair, or `None` when
    /// one of the slugs is unknown.
    #[must_use]
    pub fn multiplier(&self, attacking: &str, defending: &str) -> Option<u16> {
        let row = self.types.iter().position(|item| item.slug == attacking)?;
        let column = self.types.iter().position(|item| item.slug == defending)?;
        self.multipliers
            .get(row)
            .and_then(|values| values.get(column))
            .copied()
    }
}
