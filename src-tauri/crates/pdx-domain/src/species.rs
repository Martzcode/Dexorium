use serde::{Deserialize, Serialize};

/// Stable national Pokédex identifier.
///
/// The number is validated at construction time so the rest of the codebase
/// can rely on it being inside the supported range. This is the stable key used
/// across datasets and inside user data, as described in the specifications.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct NationalDexNumber(u16);

impl NationalDexNumber {
    /// First supported national Pokédex number.
    pub const MIN: u16 = 1;
    /// Last supported national Pokédex number (Pokémon Scarlet/Violet era).
    pub const MAX: u16 = 1_025;

    /// Creates a number if it falls inside the supported range.
    #[must_use]
    pub const fn new(value: u16) -> Option<Self> {
        if value >= Self::MIN && value <= Self::MAX {
            Some(Self(value))
        } else {
            None
        }
    }

    /// Returns the underlying number.
    #[must_use]
    pub const fn get(self) -> u16 {
        self.0
    }
}

/// Compact species row used by the Pokédex list and search results.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SpeciesSummary {
    /// Internal dataset identifier.
    pub id: i64,
    /// National Pokédex number.
    pub national_dex_number: u16,
    /// Stable inter-dataset slug.
    pub slug: String,
    /// Name in the requested language, falling back to the slug.
    pub name: String,
    /// Type slugs in slot order.
    pub types: Vec<String>,
    /// Base stats of the default form.
    pub stats: BaseStats,
    /// Base stat total.
    pub bst: u16,
}

/// How the requested type filter must match the types of a form.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum TypeMatchMode {
    /// The form owns at least one of the requested types.
    #[default]
    Any,
    /// The form owns exactly the requested set of types.
    Exact,
    /// The form owns a single type, and it is one of the requested types.
    Mono,
}

/// Base stat used by a stat range filter or by a sort criterion.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum SpeciesStat {
    Hp,
    Attack,
    Defense,
    SpAtk,
    SpDef,
    Speed,
    Bst,
}

/// Inclusive bounds applied to one base stat.
///
/// Both bounds are optional so callers can express "at least 100" or "at most
/// 45" without inventing sentinel values.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StatRange {
    pub stat: SpeciesStat,
    #[serde(default)]
    pub min: Option<u16>,
    #[serde(default)]
    pub max: Option<u16>,
}

/// Species flags exposed as filters.
///
/// Selected flags are combined with a logical OR: requesting `legendary` and
/// `mythical` returns species that are either one or the other.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct SpeciesFlags {
    pub legendary: bool,
    pub mythical: bool,
    pub baby: bool,
    pub ultra_beast: bool,
}

impl SpeciesFlags {
    /// Returns `true` when at least one flag is selected.
    #[must_use]
    pub const fn any(self) -> bool {
        self.legendary || self.mythical || self.baby || self.ultra_beast
    }
}

/// Sortable species column.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum SpeciesSortField {
    DexNumber,
    Name,
    Bst,
    Hp,
    Attack,
    Defense,
    SpAtk,
    SpDef,
    Speed,
}

/// Sort direction.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum SortDirection {
    #[default]
    Asc,
    Desc,
}

/// One entry of the multi-column sort.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SortCriterion {
    pub field: SpeciesSortField,
    #[serde(default)]
    pub direction: SortDirection,
}

/// Filtered, sorted and paginated list query.
///
/// The struct is the domain-facing counterpart of the `list_pokemon` IPC
/// payload. The application layer sanitizes it (trimming, de-duplicating,
/// clamping) before it reaches a repository.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct SpeciesListQuery {
    /// Free-text term matched against names, slug and national number.
    pub search: Option<String>,
    /// Type slugs to match. Empty means "no type filter".
    pub types: Vec<String>,
    pub type_match: TypeMatchMode,
    /// Generation identifiers to include. Empty means "all generations".
    pub generations: Vec<u16>,
    pub stat_ranges: Vec<StatRange>,
    pub flags: SpeciesFlags,
    /// When set, only species whose id is listed are returned. The Tauri
    /// command fills it from the user database when the "favorites only"
    /// filter is active; an empty list means "no species".
    pub favorite_ids: Option<Vec<i64>>,
    /// Restricts the page to the user's favourite species.
    ///
    /// The command resolves the identifiers from the user database before the
    /// query reaches the repository, so the reference database stays read-only.
    pub favorites_only: bool,
    /// Requested sort order. Empty means "national dex number ascending".
    pub sort: Vec<SortCriterion>,
    /// Zero-based index of the first item.
    pub offset: u32,
    /// Maximum number of items. `0` lets the application layer pick a default.
    pub limit: u32,
    /// Requested language (`fr` or `en`).
    pub lang: String,
}

impl Default for SpeciesListQuery {
    fn default() -> Self {
        Self {
            search: None,
            types: Vec::new(),
            type_match: TypeMatchMode::Any,
            generations: Vec::new(),
            stat_ranges: Vec::new(),
            flags: SpeciesFlags::default(),
            favorite_ids: None,
            favorites_only: false,
            sort: Vec::new(),
            offset: 0,
            limit: 0,
            lang: String::new(),
        }
    }
}

/// One page of species plus the total number of matches.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SpeciesPage {
    pub items: Vec<SpeciesSummary>,
    /// Total number of species matching the filters, before pagination.
    pub total: u32,
    /// Offset echoed back to the caller.
    pub offset: u32,
    /// Page size echoed back to the caller.
    pub limit: u32,
}

/// Base stats for a playable form.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BaseStats {
    pub hp: u16,
    pub attack: u16,
    pub defense: u16,
    pub sp_atk: u16,
    pub sp_def: u16,
    pub speed: u16,
}

impl BaseStats {
    /// Sum of all six base stats.
    #[must_use]
    pub const fn bst(self) -> u16 {
        self.hp + self.attack + self.defense + self.sp_atk + self.sp_def + self.speed
    }
}

/// Full species payload for the detail page.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SpeciesDetail {
    /// Internal dataset identifier.
    pub id: i64,
    /// National Pokédex number.
    pub national_dex_number: u16,
    /// Stable inter-dataset slug.
    pub slug: String,
    /// Name in the requested language, falling back to the slug.
    pub name: String,
    /// Type slugs in slot order.
    pub types: Vec<String>,
    /// Base stats of the default form.
    pub stats: BaseStats,
    /// Base stat total.
    pub bst: u16,
    /// Height in decimetres.
    pub height_dm: u16,
    /// Weight in hectograms.
    pub weight_hg: u16,
    /// Capture rate from the games.
    pub capture_rate: u16,
    /// Base happiness when available.
    pub base_happiness: Option<u16>,
    pub is_legendary: bool,
    pub is_mythical: bool,
}

#[cfg(test)]
mod tests {
    use super::{
        BaseStats, NationalDexNumber, SortCriterion, SortDirection, SpeciesFlags, SpeciesListQuery,
        SpeciesSortField, TypeMatchMode,
    };

    #[test]
    fn accepts_supported_range() {
        assert_eq!(
            NationalDexNumber::new(1).map(NationalDexNumber::get),
            Some(1)
        );
        assert_eq!(
            NationalDexNumber::new(NationalDexNumber::MAX).map(NationalDexNumber::get),
            Some(NationalDexNumber::MAX)
        );
    }

    #[test]
    fn rejects_out_of_range_values() {
        assert!(NationalDexNumber::new(0).is_none());
        assert!(NationalDexNumber::new(NationalDexNumber::MAX + 1).is_none());
    }

    #[test]
    fn computes_base_stat_total() {
        let stats = BaseStats {
            hp: 45,
            attack: 49,
            defense: 49,
            sp_atk: 65,
            sp_def: 65,
            speed: 45,
        };

        assert_eq!(stats.bst(), 318);
    }

    #[test]
    fn list_query_deserializes_with_defaults() {
        let query: SpeciesListQuery = serde_json::from_str(
            r#"{
                "search": "dracaufeu",
                "types": ["fire", "flying"],
                "sort": [{ "field": "bst", "direction": "desc" }],
                "lang": "fr"
            }"#,
        )
        .expect("a partial query should deserialize");

        assert_eq!(query.search.as_deref(), Some("dracaufeu"));
        assert_eq!(query.types, vec!["fire", "flying"]);
        assert_eq!(query.type_match, TypeMatchMode::Any);
        assert!(query.generations.is_empty());
        assert_eq!(query.offset, 0);
        assert_eq!(query.limit, 0);
        assert_eq!(
            query.sort,
            vec![SortCriterion {
                field: SpeciesSortField::Bst,
                direction: SortDirection::Desc,
            }]
        );
    }

    #[test]
    fn flags_serialize_in_camel_case() {
        let flags = SpeciesFlags {
            ultra_beast: true,
            ..SpeciesFlags::default()
        };

        assert_eq!(
            serde_json::to_string(&flags).expect("flags should serialize"),
            r#"{"legendary":false,"mythical":false,"baby":false,"ultraBeast":true}"#
        );
        assert!(flags.any());
    }

    #[test]
    fn type_match_mode_serializes_as_camel_case() {
        assert_eq!(
            serde_json::to_string(&TypeMatchMode::Exact).expect("mode should serialize"),
            r#""exact""#
        );
    }
}
