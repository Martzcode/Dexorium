use serde::{Deserialize, Serialize};

/// Entity kinds covered by the federated search.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum SearchEntityType {
    Species,
    Move,
    Ability,
}

impl SearchEntityType {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Species => "species",
            Self::Move => "move",
            Self::Ability => "ability",
        }
    }
}

/// One federated search hit with its display context.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SearchHit {
    pub entity_type: SearchEntityType,
    pub id: i64,
    pub slug: String,
    pub name: String,
    /// Short context line: dex number and types, move class, ability holder count.
    pub subtitle: String,
    /// Higher is better; exact and prefix matches rank above fuzzy matches.
    pub score: u32,
}

/// Federated search response.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GlobalSearchResult {
    pub query: String,
    pub hits: Vec<SearchHit>,
}

fn default_lang() -> String {
    "en".to_owned()
}

/// Search request shared by the command palette and the global search page.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GlobalSearchQuery {
    #[serde(default)]
    pub query: String,
    #[serde(default)]
    pub limit: u32,
    #[serde(default = "default_lang")]
    pub lang: String,
}

impl Default for GlobalSearchQuery {
    fn default() -> Self {
        Self {
            query: String::new(),
            limit: 0,
            lang: default_lang(),
        }
    }
}
