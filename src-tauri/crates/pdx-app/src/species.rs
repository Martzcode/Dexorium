use pdx_domain::{SpeciesDetail, SpeciesSummary};

/// Persistence port consumed by Pokédex use cases.
///
/// Implementations live in `pdx-infra`; the application layer never knows about
/// SQLite. Errors are converted to user-facing messages at the Tauri boundary.
pub trait SpeciesRepository {
    fn search_species(
        &self,
        query: &str,
        lang: &str,
        limit: usize,
    ) -> Result<Vec<SpeciesSummary>, String>;

    fn species_detail(&self, id: i64, lang: &str) -> Result<Option<SpeciesDetail>, String>;
}

/// Default maximum number of results returned by the list endpoint.
///
/// The MVP reference dataset holds ~1,025 species; the whole national dex is
/// returned in one call. Virtual scrolling replaces this cap in a later sprint.
pub const DEFAULT_SEARCH_LIMIT: usize = 1300;

/// Upper bound accepted from the IPC boundary.
pub const MAX_SEARCH_LIMIT: usize = 2000;

/// Searches species by localized name, English name, slug or national number.
pub fn search_species(
    repository: &dyn SpeciesRepository,
    query: &str,
    lang: &str,
    limit: Option<usize>,
) -> Result<Vec<SpeciesSummary>, String> {
    let query = sanitize_query(query);
    let lang = normalize_language(lang);
    let limit = limit
        .unwrap_or(DEFAULT_SEARCH_LIMIT)
        .clamp(1, MAX_SEARCH_LIMIT);

    repository.search_species(&query, lang, limit)
}

/// Loads the detail payload for one species.
pub fn species_detail(
    repository: &dyn SpeciesRepository,
    id: i64,
    lang: &str,
) -> Result<Option<SpeciesDetail>, String> {
    if id <= 0 {
        return Ok(None);
    }

    repository.species_detail(id, normalize_language(lang))
}

fn sanitize_query(query: &str) -> String {
    query.trim().chars().take(80).collect()
}

fn normalize_language(lang: &str) -> &str {
    match lang {
        "fr" => "fr",
        _ => "en",
    }
}

#[cfg(test)]
mod tests {
    use pdx_domain::{BaseStats, SpeciesDetail, SpeciesSummary};

    use super::{SpeciesRepository, search_species, species_detail};

    struct FakeRepository;

    impl SpeciesRepository for FakeRepository {
        fn search_species(
            &self,
            query: &str,
            lang: &str,
            limit: usize,
        ) -> Result<Vec<SpeciesSummary>, String> {
            assert_eq!(lang, "fr");
            assert_eq!(query, "dracaufeu");
            assert_eq!(limit, 2000);

            Ok(vec![SpeciesSummary {
                id: 6,
                national_dex_number: 6,
                slug: "charizard".to_owned(),
                name: "Dracaufeu".to_owned(),
                types: vec!["fire".to_owned(), "flying".to_owned()],
                bst: 534,
            }])
        }

        fn species_detail(&self, id: i64, lang: &str) -> Result<Option<SpeciesDetail>, String> {
            assert_eq!(id, 6);
            assert_eq!(lang, "fr");

            Ok(Some(SpeciesDetail {
                id,
                national_dex_number: 6,
                slug: "charizard".to_owned(),
                name: "Dracaufeu".to_owned(),
                types: vec!["fire".to_owned(), "flying".to_owned()],
                stats: BaseStats {
                    hp: 78,
                    attack: 84,
                    defense: 78,
                    sp_atk: 109,
                    sp_def: 85,
                    speed: 100,
                },
                bst: 534,
                height_dm: 17,
                weight_hg: 905,
                capture_rate: 45,
                base_happiness: Some(50),
                is_legendary: false,
                is_mythical: false,
            }))
        }
    }

    #[test]
    fn trims_query_and_clamps_limit() {
        let results = search_species(&FakeRepository, "  dracaufeu  ", "fr", Some(10_000)).unwrap();

        assert_eq!(results.len(), 1);
        assert_eq!(results[0].name, "Dracaufeu");
    }

    #[test]
    fn falls_back_to_english_for_unknown_languages() {
        struct EnglishRepository;

        impl SpeciesRepository for EnglishRepository {
            fn search_species(
                &self,
                _query: &str,
                lang: &str,
                _limit: usize,
            ) -> Result<Vec<SpeciesSummary>, String> {
                assert_eq!(lang, "en");
                Ok(Vec::new())
            }

            fn species_detail(
                &self,
                _id: i64,
                _lang: &str,
            ) -> Result<Option<SpeciesDetail>, String> {
                Ok(None)
            }
        }

        assert!(
            search_species(&EnglishRepository, "", "de", None)
                .unwrap()
                .is_empty()
        );
    }

    #[test]
    fn rejects_non_positive_ids() {
        assert!(species_detail(&FakeRepository, 0, "fr").unwrap().is_none());
    }
}
