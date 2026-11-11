//! Reference database adapters (SQLite, FTS5).

mod repository;

pub use repository::SqliteSpeciesRepository;

#[cfg(test)]
mod tests {
    use pdx_app::SpeciesRepository;

    use super::SqliteSpeciesRepository;

    fn repository() -> SqliteSpeciesRepository {
        SqliteSpeciesRepository::from_fixture().expect("fixture database should build")
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
}
