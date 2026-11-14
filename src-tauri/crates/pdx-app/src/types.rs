use pdx_domain::TypeMatrix;

/// Persistence port for the type effectiveness chart.
pub trait TypeRepository {
    /// Returns the chart for one generation, with localized type names.
    fn type_matrix(&self, generation: u8, lang: &str) -> Result<TypeMatrix, String>;
}

/// Latest generation supported by the dataset.
pub const LATEST_TYPE_GENERATION: u8 = 9;

/// Loads the type chart, defaulting to the latest generation.
///
/// Out-of-range generations are clamped so a stale UI cannot break the page.
pub fn type_matrix(
    repository: &dyn TypeRepository,
    generation: Option<u8>,
    lang: &str,
) -> Result<TypeMatrix, String> {
    let generation = generation
        .filter(|value| (1..=LATEST_TYPE_GENERATION).contains(value))
        .unwrap_or(LATEST_TYPE_GENERATION);

    repository.type_matrix(generation, super::species::normalize_language(lang))
}

#[cfg(test)]
mod tests {
    use std::cell::Cell;

    use pdx_domain::{TypeMatrix, TypeRef};

    use super::{LATEST_TYPE_GENERATION, TypeRepository, type_matrix};

    #[derive(Default)]
    struct FakeTypes {
        requested: Cell<Option<u8>>,
    }

    impl TypeRepository for FakeTypes {
        fn type_matrix(&self, generation: u8, lang: &str) -> Result<TypeMatrix, String> {
            self.requested.set(Some(generation));
            Ok(TypeMatrix {
                generation,
                generations: (1..=LATEST_TYPE_GENERATION).rev().collect(),
                types: vec![TypeRef {
                    slug: "fire".to_owned(),
                    name: if lang == "fr" { "Feu" } else { "Fire" }.to_owned(),
                    introduced_in: 1,
                }],
                multipliers: vec![vec![50]],
            })
        }
    }

    #[test]
    fn defaults_to_the_latest_generation() {
        let repository = FakeTypes::default();

        let matrix = type_matrix(&repository, None, "fr").expect("matrix should load");
        assert_eq!(matrix.generation, LATEST_TYPE_GENERATION);
        assert_eq!(matrix.types[0].name, "Feu");
        assert_eq!(matrix.multiplier("fire", "fire"), Some(50));
    }

    #[test]
    fn clamps_unknown_generations() {
        let repository = FakeTypes::default();

        assert_eq!(
            type_matrix(&repository, Some(0), "en")
                .expect("matrix should load")
                .generation,
            LATEST_TYPE_GENERATION
        );
        assert_eq!(
            type_matrix(&repository, Some(42), "en")
                .expect("matrix should load")
                .generation,
            LATEST_TYPE_GENERATION
        );
        assert_eq!(
            type_matrix(&repository, Some(9), "en")
                .expect("matrix should load")
                .generation,
            9
        );
    }
}
