//! Reference database adapters (SQLite, FTS5).

mod abilities;
mod dataset;
mod moves;
mod repository;
mod search;
mod types;

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

    #[test]
    fn pokemon_detail_falls_back_to_the_default_form_in_the_fixture() {
        let repository = repository();

        let detail = repository
            .pokemon_detail(6, Some(9_999), "fr", Some(9))
            .expect("detail should succeed")
            .expect("charizard should exist");

        assert_eq!(detail.name, "Dracaufeu");
        assert_eq!(detail.form.id, detail.species_id);
        assert!(detail.form.is_default);
        assert_eq!(detail.form.types, vec!["fire", "flying"]);
        assert_eq!(detail.form.stats.attack, 84);
        assert_eq!(detail.forms.len(), 1);
        assert!(detail.abilities.is_empty());
        assert!(detail.egg_groups.is_empty());
        assert!(detail.flavor_texts.is_empty());
        assert!(detail.evolutions.is_empty());
        assert_eq!(detail.effectiveness_generation_id, 9);
    }

    #[test]
    fn pokemon_detail_returns_none_for_unknown_species() {
        let repository = repository();

        assert!(
            repository
                .pokemon_detail(9_999, None, "en", None)
                .expect("detail should succeed")
                .is_none()
        );
    }

    #[test]
    fn generated_database_exposes_a_complete_detail_payload() {
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

        let detail = repository
            .pokemon_detail(6, None, "fr", Some(6))
            .expect("detail should succeed")
            .expect("charizard should exist");

        assert_eq!(detail.name, "Dracaufeu");
        assert_eq!(detail.national_dex_number, 6);
        assert!(detail.genus.is_some());
        assert_eq!(detail.form.types, vec!["fire", "flying"]);
        assert!(detail.forms.len() > 1);
        assert!(
            detail
                .forms
                .iter()
                .any(|form| form.slug == "charizard-mega-x" && form.is_mega)
        );

        assert!(!detail.abilities.is_empty());
        assert!(!detail.egg_groups.is_empty());
        assert!(!detail.flavor_texts.is_empty());
        assert!(
            detail
                .flavor_texts
                .iter()
                .all(|entry| !entry.text.is_empty() && entry.generation_id >= 1)
        );

        assert_eq!(detail.evolution_chain_id, Some(2));
        assert!(
            detail
                .evolutions
                .iter()
                .any(|step| step.to_national_dex_number == 6)
        );
        assert!(
            detail
                .evolutions
                .iter()
                .any(|step| step.from_species_id == Some(4))
        );

        let multiplier = |slug: &str| {
            detail
                .effectiveness
                .iter()
                .find(|entry| entry.attacking_type == slug)
                .map(|entry| entry.multiplier_percent)
                .unwrap_or(100)
        };
        assert_eq!(multiplier("rock"), 400);
        assert_eq!(multiplier("electric"), 200);
        assert_eq!(multiplier("ground"), 0);
        assert_eq!(multiplier("grass"), 25);

        let mega = detail
            .forms
            .iter()
            .find(|form| form.slug == "charizard-mega-x")
            .expect("mega charizard x should exist");
        let mega_detail = repository
            .pokemon_detail(6, Some(mega.id), "fr", Some(6))
            .expect("detail should succeed")
            .expect("mega charizard should exist");
        assert_eq!(mega_detail.form.id, mega.id);
        assert_eq!(mega_detail.form.types, vec!["fire", "dragon"]);
        assert_ne!(mega_detail.form.stats.attack, detail.form.stats.attack);
    }
}

#[cfg(test)]
mod catalog_tests {
    use std::path::PathBuf;

    use pdx_app::{AbilityRepository, MoveRepository, SearchRepository};
    use pdx_domain::{AbilityListQuery, MoveListQuery, MoveSortCriterion, MoveSortField, SortDirection};

    use super::SqliteSpeciesRepository;

    fn generated_repository() -> Option<SqliteSpeciesRepository> {
        let path = generated_database_path()?;
        Some(SqliteSpeciesRepository::open(&path).expect("generated database should open"))
    }

    fn generated_database_path() -> Option<PathBuf> {
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("..")
            .join("..")
            .join("resources")
            .join("reference.db");
        path.is_file().then_some(path)
    }

    #[test]
    fn fixture_catalogues_are_queryable_but_empty() {
        let repository =
            SqliteSpeciesRepository::from_fixture().expect("fixture database should build");

        let moves = repository
            .list_moves(&MoveListQuery {
                lang: "en".to_owned(),
                limit: 50,
                ..MoveListQuery::default()
            })
            .expect("move catalogue should succeed");
        assert_eq!(moves.total, 0);
        assert!(moves.items.is_empty());

        let abilities = repository
            .list_abilities(&AbilityListQuery {
                lang: "en".to_owned(),
                limit: 50,
                ..AbilityListQuery::default()
            })
            .expect("ability catalogue should succeed");
        assert_eq!(abilities.total, 0);

        let learnset = repository
            .learnset(6, "en", None)
            .expect("learnset should succeed");
        assert!(learnset.entries.is_empty());
        assert!(learnset.version_group.is_none());

        assert!(
            repository
                .search_global("pikachu", "en", 10)
                .expect("search should succeed")
                .hits
                .iter()
                .all(|hit| hit.entity_type.as_str() != "move")
        );
    }

    #[test]
    fn generated_database_filters_and_sorts_moves() {
        let Some(repository) = generated_repository() else {
            return;
        };

        let all = repository
            .list_moves(&MoveListQuery {
                lang: "fr".to_owned(),
                limit: 20,
                ..MoveListQuery::default()
            })
            .expect("catalogue should succeed");
        assert!(all.total > 800, "expected the full move catalogue");
        assert_eq!(all.items.len(), 20);
        assert!(all.items.iter().all(|item| !item.name.is_empty()));

        let fire_specials = repository
            .list_moves(&MoveListQuery {
                lang: "fr".to_owned(),
                types: vec!["fire".to_owned()],
                damage_classes: vec!["special".to_owned()],
                sort: vec![MoveSortCriterion {
                    field: MoveSortField::Power,
                    direction: SortDirection::Desc,
                }],
                limit: 100,
                ..MoveListQuery::default()
            })
            .expect("filtered catalogue should succeed");
        assert!(fire_specials.total > 5);
        assert!(
            fire_specials
                .items
                .iter()
                .all(|item| item.type_slug == "fire" && item.damage_class == "special")
        );
        let powers: Vec<u16> = fire_specials
            .items
            .iter()
            .filter_map(|item| item.power)
            .collect();
        assert!(powers.windows(2).all(|pair| pair[0] >= pair[1]));

        let highest = fire_specials
            .items
            .first()
            .expect("at least one fire special move");
        assert_eq!(highest.power, Some(150));
        assert!(
            fire_specials
                .items
                .iter()
                .any(|item| item.slug == "blast-burn")
        );
    }

    #[test]
    fn generated_database_returns_flamethrower_detail_with_learners() {
        let Some(repository) = generated_repository() else {
            return;
        };

        let listed = repository
            .list_moves(&MoveListQuery {
                search: Some("flamme".to_owned()),
                lang: "fr".to_owned(),
                limit: 50,
                ..MoveListQuery::default()
            })
            .expect("search should succeed");
        let flamethrower = listed
            .items
            .iter()
            .find(|item| item.slug == "flamethrower")
            .expect("flamethrower should match the localized search");
        assert_eq!(flamethrower.name, "Lance-Flammes");

        let detail = repository
            .move_detail(flamethrower.id, "fr")
            .expect("detail should succeed")
            .expect("flamethrower should exist");
        assert_eq!(detail.type_slug, "fire");
        assert_eq!(detail.damage_class, "special");
        assert_eq!(detail.power, Some(90));
        assert_eq!(detail.pp, Some(15));
        assert!(detail.effect_short.is_some() || detail.description.is_some());
        assert!(detail.learner_version_group.is_some());
        assert!(!detail.learners.is_empty());
        assert!(
            detail
                .learners
                .iter()
                .any(|learner| learner.national_dex_number == 6)
        );
        assert!(
            detail
                .learners
                .iter()
                .all(|learner| !learner.methods.is_empty())
        );

        assert!(
            repository
                .move_detail(999_999, "fr")
                .expect("detail should succeed")
                .is_none()
        );
    }

    #[test]
    fn generated_database_exposes_charizard_learnset_per_version_group() {
        let Some(repository) = generated_repository() else {
            return;
        };

        let detail = pdx_app::pokemon_detail(&repository, 6, None, "fr", Some(9))
            .expect("detail should succeed")
            .expect("charizard should exist");
        let form_id = detail.form.id;

        let learnset = repository
            .learnset(form_id, "fr", None)
            .expect("learnset should succeed");
        assert!(learnset.version_group.is_some());
        assert!(learnset.version_groups.len() > 5);
        assert!(learnset.total > 50);
        assert!(
            learnset
                .entries
                .iter()
                .any(|entry| entry.slug == "flamethrower")
        );
        assert!(
            learnset
                .version_groups
                .windows(2)
                .all(|pair| pair[0].order_index >= pair[1].order_index)
        );

        let selected = learnset
            .version_group
            .as_ref()
            .expect("a version group should be selected");
        assert_eq!(selected.slug, learnset.version_groups[0].slug);

        let older = &learnset.version_groups[learnset.version_groups.len() - 1];
        let old_learnset = repository
            .learnset(form_id, "fr", Some(&older.slug))
            .expect("learnset should succeed");
        assert_eq!(
            old_learnset
                .version_group
                .as_ref()
                .expect("a version group should be selected")
                .slug,
            older.slug
        );

        let unknown = repository
            .learnset(form_id, "fr", Some("not-a-version-group"))
            .expect("learnset should succeed");
        assert_eq!(
            unknown
                .version_group
                .as_ref()
                .expect("fallback version group")
                .slug,
            learnset.version_groups[0].slug
        );

        let empty = repository
            .learnset(999_999, "fr", None)
            .expect("learnset should succeed");
        assert!(empty.entries.is_empty());
        assert!(empty.version_groups.is_empty());
    }

    #[test]
    fn generated_database_lists_abilities_and_their_holders() {
        let Some(repository) = generated_repository() else {
            return;
        };

        let page = repository
            .list_abilities(&AbilityListQuery {
                lang: "fr".to_owned(),
                limit: 500,
                ..AbilityListQuery::default()
            })
            .expect("catalogue should succeed");
        assert!(page.total > 300);
        let names: Vec<&str> = page.items.iter().map(|item| item.name.as_str()).collect();
        let mut sorted = names.clone();
        sorted.sort_by_key(|name| name.to_lowercase());
        assert_eq!(names, sorted, "abilities are sorted by localized name");

        let blaze = page
            .items
            .iter()
            .find(|item| item.slug == "blaze")
            .expect("blaze should exist");
        assert!(blaze.holder_count >= 15);

        let detail = repository
            .ability_detail(blaze.id, "fr")
            .expect("detail should succeed")
            .expect("blaze should exist");
        assert_eq!(detail.name, "Brasier");
        assert!(detail.effect_short.is_some());
        assert!(!detail.holders.is_empty());
        assert!(
            detail
                .holders
                .iter()
                .any(|holder| holder.national_dex_number == 6)
        );
        assert!(detail.holders.windows(2).all(|pair| {
            pair[0].national_dex_number <= pair[1].national_dex_number
        }));

        let filtered = repository
            .list_abilities(&AbilityListQuery {
                search: Some("brasier".to_owned()),
                lang: "fr".to_owned(),
                limit: 20,
                ..AbilityListQuery::default()
            })
            .expect("search should succeed");
        assert!(
            filtered
                .items
                .iter()
                .any(|item| item.slug == "blaze")
        );
    }

    #[test]
    fn generated_database_searches_across_entities_and_tolerates_typos() {
        let Some(repository) = generated_repository() else {
            return;
        };

        let prefix = repository
            .search_global("pikach", "fr", 10)
            .expect("search should succeed");
        let pikachu = prefix
            .hits
            .iter()
            .find(|hit| hit.entity_type.as_str() == "species")
            .expect("pikachu should be found");
        assert_eq!(pikachu.slug, "pikachu");
        assert!(pikachu.subtitle.contains("#0025"));

        let typo = repository
            .search_global("pikatchu", "fr", 10)
            .expect("search should succeed");
        assert!(
            typo.hits
                .iter()
                .any(|hit| hit.entity_type.as_str() == "species" && hit.slug == "pikachu")
        );

        let flamethrower = repository
            .search_global("lance-flamme", "fr", 10)
            .expect("search should succeed");
        assert!(
            flamethrower
                .hits
                .iter()
                .any(|hit| hit.entity_type.as_str() == "move" && hit.slug == "flamethrower")
        );

        let blaze = repository
            .search_global("brasier", "fr", 10)
            .expect("search should succeed");
        assert!(
            blaze
                .hits
                .iter()
                .any(|hit| hit.entity_type.as_str() == "ability" && hit.slug == "blaze")
        );

        assert!(
            repository
                .search_global("   ", "fr", 10)
                .expect("search should succeed")
                .hits
                .is_empty()
        );
    }

    #[test]
    fn generated_database_supports_damage_class_filters() {
        let Some(repository) = generated_repository() else {
            return;
        };

        let status = repository
            .list_moves(&MoveListQuery {
                lang: "en".to_owned(),
                damage_classes: vec!["status".to_owned()],
                min_power: Some(10),
                limit: 50,
                ..MoveListQuery::default()
            })
            .expect("catalogue should succeed");
        assert_eq!(status.total, 0, "status moves never have power");
    }
}
