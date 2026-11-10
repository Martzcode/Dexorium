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

#[cfg(test)]
mod tests {
    use super::NationalDexNumber;

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
}
