//! Reference database adapters (SQLite, FTS5).

mod repository;

pub use repository::{REFERENCE_SCHEMA, SqliteSpeciesRepository};

#[cfg(test)]
mod tests {
    use pdx_app::SpeciesRepository;
    use pdx_domain::{
        SortCriterion, SortDirection, SpeciesFlags, SpeciesListQuery, SpeciesSortField,
        SpeciesStat, StatRange, TypeMatchMode,
    };

    use super::SqliteSpeciesRepository;

    fn repository() -> SqliteSpeciesRepository {
        SqliteSpeciesRepository::from_fixture().expect("fixture database should build")
    }

    fn list_query() -> SpeciesListQuery {
        SpeciesListQuery {
            lang: "en".to_owned(),
            limit: 100,
            ..SpeciesListQuery::default()
        }
    }

    #[test]
    fn empty_query_lists_every_species_in_dex_order() {
        let repository = repository();

        let results = repository
            .search_species("", "en", 100)
            .expect("search should succeed");

        assert_eq!(results.len(), 31);
        assert_eq!(results[0].national_dex_number, 1);
        assert_eq!(results[31 - 1].national_dex_number, 448);
    }

    #[test]
    fn respects_the_limit() {
        let repository = repository();

        let results = repository
            .search_species("", "en", 5)
            .expect("search should succeed");

        assert_eq!(results.len(), 5);
    }

    #[test]
    fn prefix_search_matches_a_localized_name() {
        let repository = repository();

        let results = repository
            .search_species("pika", "fr", 10)
            .expect("search should succeed");

        assert!(results.iter().any(|species| species.name == "Pikachu"));
    }

    #[test]
    fn search_ignores_accents() {
        let repository = repository();

        let results = repository
            .search_species("evoli", "fr", 10)
            .expect("search should succeed");

        assert!(results.iter().any(|species| species.name == "Évoli"));
    }

    #[test]
    fn search_matches_english_names_in_french_mode() {
        let repository = repository();

        let results = repository
            .search_species("charizard", "fr", 10)
            .expect("search should succeed");

        assert!(results.iter().any(|species| species.name == "Dracaufeu"));
    }

    #[test]
    fn search_matches_national_numbers() {
        let repository = repository();

        let results = repository
            .search_species("448", "en", 10)
            .expect("search should succeed");

        assert_eq!(results.len(), 1);
        assert_eq!(results[0].name, "Lucario");
    }

    #[test]
    fn search_ignores_fts_operators() {
        let repository = repository();

        let results = repository
            .search_species("NOT OR NEAR", "en", 10)
            .expect("search should not fail on operator keywords");

        assert!(results.is_empty());
    }

    #[test]
    fn detail_returns_stats_and_types() {
        let repository = repository();

        let detail = repository
            .species_detail(6, "fr")
            .expect("detail should succeed")
            .expect("charizard should exist");

        assert_eq!(detail.name, "Dracaufeu");
        assert_eq!(detail.types, vec!["fire", "flying"]);
        assert_eq!(detail.stats.attack, 84);
        assert_eq!(detail.bst, 534);
        assert_eq!(detail.height_dm, 17);
        assert_eq!(detail.weight_hg, 905);
        assert!(!detail.is_legendary);
    }

    #[test]
    fn detail_returns_none_for_unknown_ids() {
        let repository = repository();

        assert!(
            repository
                .species_detail(9_999, "en")
                .expect("detail should succeed")
                .is_none()
        );
    }

    #[test]
    fn list_paginates_and_reports_the_total() {
        let repository = repository();

        let first = repository
            .list_species(&SpeciesListQuery {
                limit: 10,
                ..list_query()
            })
            .expect("list should succeed");
        assert_eq!(first.total, 31);
        assert_eq!(first.items.len(), 10);
        assert_eq!(first.items[0].national_dex_number, 1);

        let last = repository
            .list_species(&SpeciesListQuery {
                offset: 30,
                limit: 10,
                ..list_query()
            })
            .expect("list should succeed");
        assert_eq!(last.total, 31);
        assert_eq!(last.items.len(), 1);
        assert_eq!(last.items[0].national_dex_number, 448);
    }

    #[test]
    fn list_filters_by_generation() {
        let repository = repository();

        let page = repository
            .list_species(&SpeciesListQuery {
                generations: vec![4],
                ..list_query()
            })
            .expect("list should succeed");

        assert!(page.total >= 2);
        assert!(page.items.iter().any(|species| species.name == "Lucario"));
        assert!(page.items.iter().any(|species| species.name == "Garchomp"));
    }

    #[test]
    fn type_filter_any_matches_at_least_one_type() {
        let repository = repository();

        let page = repository
            .list_species(&SpeciesListQuery {
                types: vec!["fire".to_owned()],
                type_match: TypeMatchMode::Any,
                ..list_query()
            })
            .expect("list should succeed");

        assert!(
            page.items
                .iter()
                .all(|species| species.types.contains(&"fire".to_owned()))
        );
        assert!(page.items.iter().any(|species| species.name == "Charizard"));
        assert!(
            page.items
                .iter()
                .any(|species| species.name == "Charmander")
        );
    }

    #[test]
    fn type_filter_exact_matches_the_full_type_set() {
        let repository = repository();

        let page = repository
            .list_species(&SpeciesListQuery {
                types: vec!["fire".to_owned(), "flying".to_owned()],
                type_match: TypeMatchMode::Exact,
                ..list_query()
            })
            .expect("list should succeed");

        assert_eq!(page.total, 1);
        assert_eq!(page.items[0].name, "Charizard");
    }

    #[test]
    fn type_filter_mono_excludes_dual_types() {
        let repository = repository();

        let page = repository
            .list_species(&SpeciesListQuery {
                types: vec!["fire".to_owned()],
                type_match: TypeMatchMode::Mono,
                ..list_query()
            })
            .expect("list should succeed");

        assert!(page.items.iter().all(|species| species.types.len() == 1));
        assert!(
            page.items
                .iter()
                .any(|species| species.name == "Charmander")
        );
        assert!(!page.items.iter().any(|species| species.name == "Charizard"));
    }

    #[test]
    fn stat_range_filters_on_bst() {
        let repository = repository();

        let page = repository
            .list_species(&SpeciesListQuery {
                stat_ranges: vec![StatRange {
                    stat: SpeciesStat::Bst,
                    min: Some(600),
                    max: None,
                }],
                ..list_query()
            })
            .expect("list should succeed");

        assert!(page.total >= 1);
        assert!(page.items.iter().all(|species| species.bst >= 600));
    }

    #[test]
    fn flag_filter_returns_legendaries_and_mythicals() {
        let repository = repository();

        let legendaries = repository
            .list_species(&SpeciesListQuery {
                flags: SpeciesFlags {
                    legendary: true,
                    ..SpeciesFlags::default()
                },
                ..list_query()
            })
            .expect("list should succeed");
        assert_eq!(legendaries.total, 1);
        assert_eq!(legendaries.items[0].name, "Mewtwo");

        let mythicals = repository
            .list_species(&SpeciesListQuery {
                flags: SpeciesFlags {
                    mythical: true,
                    ..SpeciesFlags::default()
                },
                ..list_query()
            })
            .expect("list should succeed");
        assert_eq!(mythicals.total, 1);
        assert_eq!(mythicals.items[0].name, "Mew");

        let either = repository
            .list_species(&SpeciesListQuery {
                flags: SpeciesFlags {
                    legendary: true,
                    mythical: true,
                    ..SpeciesFlags::default()
                },
                ..list_query()
            })
            .expect("list should succeed");
        assert_eq!(either.total, 2);
    }

    #[test]
    fn sorts_by_bst_descending() {
        let repository = repository();

        let page = repository
            .list_species(&SpeciesListQuery {
                sort: vec![SortCriterion {
                    field: SpeciesSortField::Bst,
                    direction: SortDirection::Desc,
                }],
                limit: 5,
                ..list_query()
            })
            .expect("list should succeed");

        assert_eq!(page.items[0].name, "Mewtwo");
        assert!(page.items.windows(2).all(|pair| pair[0].bst >= pair[1].bst));
    }

    #[test]
    fn sorts_by_localized_name_ascending() {
        let repository = repository();

        let page = repository
            .list_species(&SpeciesListQuery {
                sort: vec![SortCriterion {
                    field: SpeciesSortField::Name,
                    direction: SortDirection::Asc,
                }],
                ..list_query()
            })
            .expect("list should succeed");

        assert_eq!(page.items[0].name, "Blastoise");
    }

    #[test]
    fn combines_search_with_filters() {
        let repository = repository();

        let page = repository
            .list_species(&SpeciesListQuery {
                search: Some("char".to_owned()),
                generations: vec![1],
                ..list_query()
            })
            .expect("list should succeed");

        assert_eq!(page.total, 3);
        assert!(
            page.items
                .iter()
                .all(|species| species.name.starts_with("Char"))
        );
    }

    #[test]
    fn returns_an_empty_page_when_the_search_has_no_token() {
        let repository = repository();

        let page = repository
            .list_species(&SpeciesListQuery {
                search: Some("!!!".to_owned()),
                ..list_query()
            })
            .expect("list should succeed");

        assert_eq!(page.total, 0);
        assert!(page.items.is_empty());
    }

    /// Smoke test for the generated dataset. It only runs when
    /// `src-tauri/resources/reference.db` exists, so CI (which does not build
    /// the dataset) still passes.
    #[test]
    fn opens_the_generated_reference_database_when_present() {
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("..")
            .join("..")
            .join("resources")
            .join("reference.db");
        if !path.is_file() {
            return;
        }

        let repository =
            SqliteSpeciesRepository::open(&path).expect("generated database should open");

        let results = repository
            .search_species("", "fr", 2_000)
            .expect("search should succeed");
        assert_eq!(results.len(), 1_025);
        assert_eq!(results[0].national_dex_number, 1);

        let detail = repository
            .species_detail(6, "fr")
            .expect("detail should succeed")
            .expect("charizard should exist");
        assert_eq!(detail.name, "Dracaufeu");
        assert_eq!(detail.bst, 534);
        assert_eq!(detail.types, vec!["fire".to_owned(), "flying".to_owned()]);

        assert!(
            repository
                .search_species("dracaufeu", "fr", 10)
                .expect("search should succeed")
                .iter()
                .any(|species| species.national_dex_number == 6)
        );
        assert!(
            repository
                .search_species("evoli", "fr", 10)
                .expect("search should succeed")
                .iter()
                .any(|species| species.national_dex_number == 133)
        );

        let first_page = repository
            .list_species(&SpeciesListQuery {
                lang: "fr".to_owned(),
                limit: 20,
                ..SpeciesListQuery::default()
            })
            .expect("list should succeed");
        assert_eq!(first_page.total, 1_025);
        assert_eq!(first_page.items.len(), 20);
        assert_eq!(first_page.items[0].national_dex_number, 1);

        let kanto = repository
            .list_species(&SpeciesListQuery {
                lang: "fr".to_owned(),
                generations: vec![1],
                limit: 500,
                ..SpeciesListQuery::default()
            })
            .expect("list should succeed");
        assert_eq!(kanto.total, 151);

        let fire = repository
            .list_species(&SpeciesListQuery {
                lang: "fr".to_owned(),
                types: vec!["fire".to_owned()],
                limit: 500,
                ..SpeciesListQuery::default()
            })
            .expect("list should succeed");
        assert!(fire.total > 0);
        assert!(
            fire.items
                .iter()
                .all(|species| species.types.contains(&"fire".to_owned()))
        );

        let fire_flying = repository
            .list_species(&SpeciesListQuery {
                lang: "fr".to_owned(),
                types: vec!["fire".to_owned(), "flying".to_owned()],
                type_match: TypeMatchMode::Exact,
                limit: 500,
                ..SpeciesListQuery::default()
            })
            .expect("list should succeed");
        assert!(fire_flying.total >= 1);
        assert!(
            fire_flying
                .items
                .iter()
                .all(|species| species.types.len() == 2)
        );
        assert!(
            fire_flying
                .items
                .iter()
                .any(|species| species.national_dex_number == 6)
        );

        let strongest = repository
            .list_species(&SpeciesListQuery {
                lang: "fr".to_owned(),
                sort: vec![SortCriterion {
                    field: SpeciesSortField::Bst,
                    direction: SortDirection::Desc,
                }],
                limit: 5,
                ..SpeciesListQuery::default()
            })
            .expect("list should succeed");
        assert_eq!(strongest.items.len(), 5);
        assert!(
            strongest
                .items
                .windows(2)
                .all(|pair| pair[0].bst >= pair[1].bst)
        );
    }
}
