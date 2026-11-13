use pdx_domain::{SpeciesDetail, SpeciesListQuery, SpeciesPage, SpeciesSummary};

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

    /// Returns one filtered, sorted and paginated page of species.
    fn list_species(&self, query: &SpeciesListQuery) -> Result<SpeciesPage, String>;

    fn species_detail(&self, id: i64, lang: &str) -> Result<Option<SpeciesDetail>, String>;
}

/// Default maximum number of results returned by the list endpoint.
///
/// The MVP reference dataset holds ~1,025 species; the whole national dex is
/// returned in one call. Virtual scrolling replaces this cap in a later sprint.
pub const DEFAULT_SEARCH_LIMIT: usize = 1300;

/// Upper bound accepted from the IPC boundary.
pub const MAX_SEARCH_LIMIT: usize = 2000;

/// Page size used when the caller does not provide one.
pub const DEFAULT_PAGE_SIZE: u32 = 60;

/// Upper bound accepted for a single page.
pub const MAX_PAGE_SIZE: u32 = 500;

const MAX_FILTER_TYPES: usize = 18;
const MAX_FILTER_GENERATIONS: usize = 9;
const MAX_STAT_RANGES: usize = 16;
const MAX_SORT_CRITERIA: usize = 3;

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

/// Lists species with filters, multi-column sort and pagination.
///
/// The query is sanitized so every implementation can trust it: the search term
/// is trimmed and bounded, slugs are lowercased and de-duplicated, lists are
/// capped, and the page size falls back to [`DEFAULT_PAGE_SIZE`].
pub fn list_species(
    repository: &dyn SpeciesRepository,
    query: &SpeciesListQuery,
) -> Result<SpeciesPage, String> {
    let mut query = query.clone();

    query.search = query
        .search
        .as_deref()
        .map(sanitize_query)
        .filter(|value| !value.is_empty());
    query.lang = normalize_language(&query.lang).to_owned();
    query.types = sanitize_slugs(query.types, MAX_FILTER_TYPES);
    query.generations = sanitize_generations(query.generations);
    query.stat_ranges.truncate(MAX_STAT_RANGES);
    query.sort.truncate(MAX_SORT_CRITERIA);
    query.limit = if query.limit == 0 {
        DEFAULT_PAGE_SIZE
    } else {
        query.limit.min(MAX_PAGE_SIZE)
    };

    repository.list_species(&query)
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

fn sanitize_slugs(values: Vec<String>, max: usize) -> Vec<String> {
    let mut sanitized: Vec<String> = Vec::with_capacity(values.len().min(max));

    for value in values {
        let slug = value.trim().to_lowercase();
        if slug.is_empty() || slug.len() > 40 || sanitized.contains(&slug) {
            continue;
        }

        sanitized.push(slug);
        if sanitized.len() == max {
            break;
        }
    }

    sanitized
}

fn sanitize_generations(values: Vec<u16>) -> Vec<u16> {
    let mut sanitized: Vec<u16> = Vec::with_capacity(values.len().min(MAX_FILTER_GENERATIONS));

    for value in values {
        if value == 0 || sanitized.contains(&value) {
            continue;
        }

        sanitized.push(value);
        if sanitized.len() == MAX_FILTER_GENERATIONS {
            break;
        }
    }

    sanitized
}

fn normalize_language(lang: &str) -> &str {
    match lang {
        "fr" => "fr",
        _ => "en",
    }
}

#[cfg(test)]
mod tests {
    use std::cell::RefCell;

    use pdx_domain::{
        BaseStats, SpeciesDetail, SpeciesListQuery, SpeciesPage, SpeciesSummary, TypeMatchMode,
    };

    use super::{
        DEFAULT_PAGE_SIZE, MAX_PAGE_SIZE, SpeciesRepository, list_species, search_species,
        species_detail,
    };

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

            Ok(vec![summary()])
        }

        fn list_species(&self, query: &SpeciesListQuery) -> Result<SpeciesPage, String> {
            Ok(SpeciesPage {
                items: vec![summary()],
                total: 1,
                offset: query.offset,
                limit: query.limit,
            })
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

            fn list_species(&self, query: &SpeciesListQuery) -> Result<SpeciesPage, String> {
                assert_eq!(query.lang, "en");
                Ok(SpeciesPage {
                    items: Vec::new(),
                    total: 0,
                    offset: query.offset,
                    limit: query.limit,
                })
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

        let page = list_species(&EnglishRepository, &SpeciesListQuery::default()).unwrap();
        assert_eq!(page.limit, DEFAULT_PAGE_SIZE);
    }

    #[test]
    fn rejects_non_positive_ids() {
        assert!(species_detail(&FakeRepository, 0, "fr").unwrap().is_none());
    }

    #[test]
    fn sanitizes_and_clamps_a_list_query() {
        struct CapturingRepository(RefCell<Option<SpeciesListQuery>>);

        impl SpeciesRepository for CapturingRepository {
            fn search_species(
                &self,
                _query: &str,
                _lang: &str,
                _limit: usize,
            ) -> Result<Vec<SpeciesSummary>, String> {
                Ok(Vec::new())
            }

            fn list_species(&self, query: &SpeciesListQuery) -> Result<SpeciesPage, String> {
                *self.0.borrow_mut() = Some(query.clone());
                Ok(SpeciesPage {
                    items: Vec::new(),
                    total: 0,
                    offset: query.offset,
                    limit: query.limit,
                })
            }

            fn species_detail(
                &self,
                _id: i64,
                _lang: &str,
            ) -> Result<Option<SpeciesDetail>, String> {
                Ok(None)
            }
        }

        let repository = CapturingRepository(RefCell::new(None));
        let query = SpeciesListQuery {
            search: Some("   ".to_owned()),
            types: vec![
                " Fire ".to_owned(),
                "fire".to_owned(),
                "".to_owned(),
                "WATER".to_owned(),
            ],
            type_match: TypeMatchMode::Exact,
            generations: vec![1, 1, 0, 9],
            sort: Vec::new(),
            offset: 10,
            limit: 10_000,
            lang: "de".to_owned(),
            ..SpeciesListQuery::default()
        };

        let page = list_species(&repository, &query).unwrap();

        assert_eq!(page.limit, MAX_PAGE_SIZE);
        let captured = repository.0.borrow();
        let captured = captured.as_ref().expect("repository should be called");
        assert!(captured.search.is_none());
        assert_eq!(captured.lang, "en");
        assert_eq!(captured.types, vec!["fire", "water"]);
        assert_eq!(captured.generations, vec![1, 9]);
        assert_eq!(captured.offset, 10);
        assert_eq!(captured.limit, MAX_PAGE_SIZE);
    }

    fn summary() -> SpeciesSummary {
        SpeciesSummary {
            id: 6,
            national_dex_number: 6,
            slug: "charizard".to_owned(),
            name: "Dracaufeu".to_owned(),
            types: vec!["fire".to_owned(), "flying".to_owned()],
            bst: 534,
        }
    }
}
