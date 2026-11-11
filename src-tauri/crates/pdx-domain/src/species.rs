use serde::Serialize;

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
    /// Base stat total.
    pub bst: u16,
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
    use super::{BaseStats, NationalDexNumber};

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
}
