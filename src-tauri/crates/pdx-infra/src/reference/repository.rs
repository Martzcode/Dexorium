//! SQLite implementation of the species repository.
//!
//! The first vertical slice seeds an in-memory database from the shared JSON
//! fixture. The schema already matches the documented reference schema so the
//! future `pdx-dataset` pipeline can replace the seeding step without touching
//! the query code.

use std::collections::HashMap;
use std::path::Path;

use pdx_app::SpeciesRepository;
use pdx_domain::{
    AbilityRef, BaseStats, EvolutionStepView, FlavorTextEntry, FormSummary, PokemonDetail,
    SortCriterion, SortDirection, SpeciesDetail, SpeciesListQuery, SpeciesPage, SpeciesSortField,
    SpeciesStat, SpeciesSummary, TypeEffectiveness, TypeMatchMode,
};
use rusqlite::types::Value;
use rusqlite::{Connection, OpenFlags, OptionalExtension, Row, params, params_from_iter};
use serde::Deserialize;

const FIXTURE_JSON: &str = include_str!("../../fixtures/species.json");

/// Canonical reference schema shared with the `pdx-dataset` builder.
pub const REFERENCE_SCHEMA: &str = include_str!("schema.sql");

/// Supported generations, in national order.
const GENERATIONS: &[(&str, &str)] = &[
    ("generation-i", "kanto"),
    ("generation-ii", "johto"),
    ("generation-iii", "hoenn"),
    ("generation-iv", "sinnoh"),
    ("generation-v", "unova"),
    ("generation-vi", "kalos"),
    ("generation-vii", "alola"),
    ("generation-viii", "galar"),
    ("generation-ix", "paldea"),
];

/// All types with the generation that introduced them.
const TYPES: &[(&str, i64)] = &[
    ("normal", 1),
    ("fire", 1),
    ("water", 1),
    ("electric", 1),
    ("grass", 1),
    ("ice", 1),
    ("fighting", 1),
    ("poison", 1),
    ("ground", 1),
    ("flying", 1),
    ("psychic", 1),
    ("bug", 1),
    ("rock", 1),
    ("ghost", 1),
    ("dragon", 1),
    ("dark", 2),
    ("steel", 2),
    ("fairy", 6),
];

const LIST_FROM: &str = r"
FROM species s
JOIN pokemon_form f ON f.species_id = s.id AND f.is_default = 1
JOIN form_stat_summary fs ON fs.form_id = f.id
LEFT JOIN translation t
       ON t.entity_type = 'species'
      AND t.entity_id = s.id
      AND t.lang = ?
      AND t.field = 'name'
LEFT JOIN translation t_en
       ON t_en.entity_type = 'species'
      AND t_en.entity_id = s.id
      AND t_en.lang = 'en'
      AND t_en.field = 'name'
";

const LIST_COLUMNS: &str = r"
SELECT s.id,
       s.national_dex_number,
       s.slug,
       COALESCE(t.value, t_en.value, s.slug) AS name,
       fs.hp,
       fs.attack,
       fs.defense,
       fs.sp_atk,
       fs.sp_def,
       fs.speed,
       fs.bst,
       f.id AS form_id
";

const DETAIL_SQL: &str = r"
SELECT s.id,
       s.national_dex_number,
       s.slug,
       COALESCE(t.value, t_en.value, s.slug) AS name,
       fs.hp,
       fs.attack,
       fs.defense,
       fs.sp_atk,
       fs.sp_def,
       fs.speed,
       fs.bst,
       f.id AS form_id,
       f.height_dm,
       f.weight_hg,
       s.capture_rate,
       s.base_happiness,
       s.is_legendary,
       s.is_mythical
FROM species s
JOIN pokemon_form f ON f.species_id = s.id AND f.is_default = 1
JOIN form_stat_summary fs ON fs.form_id = f.id
LEFT JOIN translation t
       ON t.entity_type = 'species'
      AND t.entity_id = s.id
      AND t.lang = ?2
      AND t.field = 'name'
LEFT JOIN translation t_en
       ON t_en.entity_type = 'species'
      AND t_en.entity_id = s.id
      AND t_en.lang = 'en'
      AND t_en.field = 'name'
WHERE s.id = ?1
";

type SummaryRow = (i64, u16, String, String, BaseStats, u16, i64);

/// Species repository backed by a SQLite reference database.
pub struct SqliteSpeciesRepository {
    connection: Connection,
}

impl SqliteSpeciesRepository {
    /// Builds an in-memory database seeded from the shared fixture.
    ///
    /// # Errors
    ///
    /// Returns a message when the fixture is invalid or SQLite rejects the
    /// schema or seed data.
    pub fn from_fixture() -> Result<Self, String> {
        let species: Vec<FixtureSpecies> = serde_json::from_str(FIXTURE_JSON)
            .map_err(|error| format!("invalid species fixture: {error}"))?;

        let mut connection = Connection::open_in_memory().map_err(db_error)?;
        connection
            .execute_batch(REFERENCE_SCHEMA)
            .map_err(db_error)?;
        seed(&mut connection, &species)?;

        Ok(Self { connection })
    }

    /// Opens a generated reference database in read-only mode.
    ///
    /// # Errors
    ///
    /// Returns a message when the file cannot be opened. The caller is
    /// responsible for falling back to the bundled fixture during development.
    pub fn open(path: &Path) -> Result<Self, String> {
        let connection = Connection::open_with_flags(
            path,
            OpenFlags::SQLITE_OPEN_READ_ONLY | OpenFlags::SQLITE_OPEN_NO_MUTEX,
        )
        .map_err(db_error)?;

        Ok(Self { connection })
    }

    fn query_page(&self, query: &SpeciesListQuery) -> Result<SpeciesPage, String> {
        let Some(sql) = build_list_sql(query) else {
            return Ok(SpeciesPage {
                items: Vec::new(),
                total: 0,
                offset: query.offset,
                limit: query.limit,
            });
        };

        let count_sql = format!("SELECT COUNT(*) {LIST_FROM}{}", sql.where_clause);
        let total: i64 = self
            .connection
            .query_row(&count_sql, params_from_iter(sql.params.iter()), |row| {
                row.get(0)
            })
            .map_err(db_error)?;

        let mut page_params = sql.params;
        page_params.push(Value::Integer(i64::from(query.limit)));
        page_params.push(Value::Integer(i64::from(query.offset)));

        let page_sql = format!(
            "{LIST_COLUMNS}{LIST_FROM}{}{} LIMIT ? OFFSET ?",
            sql.where_clause, sql.order_clause
        );
        let mut statement = self.connection.prepare(&page_sql).map_err(db_error)?;
        let rows = statement
            .query_map(params_from_iter(page_params.iter()), map_summary_row)
            .map_err(db_error)?
            .collect::<rusqlite::Result<Vec<_>>>()
            .map_err(db_error)?;

        let items = rows
            .into_iter()
            .map(
                |(id, national_dex_number, slug, name, stats, bst, form_id)| {
                    Ok(SpeciesSummary {
                        id,
                        national_dex_number,
                        slug,
                        name,
                        types: load_types(&self.connection, form_id)?,
                        stats,
                        bst,
                    })
                },
            )
            .collect::<Result<Vec<_>, String>>()?;

        Ok(SpeciesPage {
            items,
            total: u32::try_from(total).unwrap_or(u32::MAX),
            offset: query.offset,
            limit: query.limit,
        })
    }
}

impl SpeciesRepository for SqliteSpeciesRepository {
    fn search_species(
        &self,
        query: &str,
        lang: &str,
        limit: usize,
    ) -> Result<Vec<SpeciesSummary>, String> {
        let trimmed = query.trim();
        let list_query = SpeciesListQuery {
            search: (!trimmed.is_empty()).then(|| trimmed.to_owned()),
            lang: lang.to_owned(),
            limit: u32::try_from(limit).unwrap_or(u32::MAX),
            ..SpeciesListQuery::default()
        };

        Ok(self.query_page(&list_query)?.items)
    }

    fn list_species(&self, query: &SpeciesListQuery) -> Result<SpeciesPage, String> {
        self.query_page(query)
    }

    fn species_detail(&self, id: i64, lang: &str) -> Result<Option<SpeciesDetail>, String> {
        let mut statement = self.connection.prepare(DETAIL_SQL).map_err(db_error)?;
        let row = statement
            .query_row(params![id, lang], map_detail_row)
            .optional()
            .map_err(db_error)?;

        let Some((
            id,
            national_dex_number,
            slug,
            name,
            stats,
            bst,
            form_id,
            height_dm,
            weight_hg,
            capture_rate,
            base_happiness,
            is_legendary,
            is_mythical,
        )) = row
        else {
            return Ok(None);
        };

        Ok(Some(SpeciesDetail {
            id,
            national_dex_number,
            slug,
            name,
            types: load_types(&self.connection, form_id)?,
            stats,
            bst,
            height_dm,
            weight_hg,
            capture_rate,
            base_happiness,
            is_legendary,
            is_mythical,
        }))
    }

    fn pokemon_detail(
        &self,
        species_id: i64,
        form_id: Option<i64>,
        lang: &str,
        generation: Option<u16>,
    ) -> Result<Option<PokemonDetail>, String> {
        let Some(identity) = load_species_identity(&self.connection, species_id, lang)? else {
            return Ok(None);
        };

        let forms = load_forms(&self.connection, species_id, lang)?;
        let Some(form) = select_form(&forms, form_id).cloned() else {
            return Ok(None);
        };

        let abilities = load_abilities(&self.connection, form.id, lang)?;
        let egg_groups = load_egg_groups(&self.connection, species_id, lang)?;
        let flavor_texts = load_flavor_texts(&self.connection, species_id, lang)?;
        let evolutions = match identity.evolution_chain_id {
            Some(chain_id) => load_evolutions(&self.connection, chain_id, lang)?,
            None => Vec::new(),
        };
        let effectiveness_generation_id = generation.unwrap_or(identity.generation_id);
        let effectiveness =
            load_effectiveness(&self.connection, &form.types, effectiveness_generation_id)?;

        Ok(Some(PokemonDetail {
            species_id: identity.species_id,
            national_dex_number: identity.national_dex_number,
            slug: identity.slug,
            name: identity.name,
            genus: identity.genus,
            generation_id: identity.generation_id,
            is_legendary: identity.is_legendary,
            is_mythical: identity.is_mythical,
            is_baby: identity.is_baby,
            is_ultra_beast: identity.is_ultra_beast,
            capture_rate: identity.capture_rate,
            base_happiness: identity.base_happiness,
            hatch_counter: identity.hatch_counter,
            gender_rate: identity.gender_rate,
            growth_rate: identity.growth_rate,
            color: identity.color,
            shape: identity.shape,
            habitat: identity.habitat,
            egg_groups,
            form,
            forms,
            abilities,
            evolution_chain_id: identity.evolution_chain_id,
            evolutions,
            flavor_texts,
            effectiveness,
            effectiveness_generation_id,
        }))
    }
}

/// Species-level columns shared by every form of the requested species.
struct SpeciesIdentity {
    species_id: i64,
    national_dex_number: u16,
    slug: String,
    name: String,
    genus: Option<String>,
    generation_id: u16,
    is_legendary: bool,
    is_mythical: bool,
    is_baby: bool,
    is_ultra_beast: bool,
    capture_rate: u16,
    base_happiness: Option<u16>,
    hatch_counter: Option<u16>,
    gender_rate: i16,
    growth_rate: String,
    color: Option<String>,
    shape: Option<String>,
    habitat: Option<String>,
    evolution_chain_id: Option<i64>,
}

const IDENTITY_SQL: &str = r"
SELECT s.id,
       s.national_dex_number,
       s.slug,
       COALESCE(t.value, t_en.value, s.slug) AS name,
       COALESCE(tg.value, tg_en.value) AS genus,
       s.generation_id,
       s.is_legendary,
       s.is_mythical,
       s.is_baby,
       s.is_ultra_beast,
       s.capture_rate,
       s.base_happiness,
       s.hatch_counter,
       s.gender_rate,
       s.growth_rate_slug,
       s.color_slug,
       s.shape_slug,
       s.habitat_slug,
       s.evolution_chain_id
FROM species s
LEFT JOIN translation t
       ON t.entity_type = 'species'
      AND t.entity_id = s.id
      AND t.lang = ?2
      AND t.field = 'name'
LEFT JOIN translation t_en
       ON t_en.entity_type = 'species'
      AND t_en.entity_id = s.id
      AND t_en.lang = 'en'
      AND t_en.field = 'name'
LEFT JOIN translation tg
       ON tg.entity_type = 'species'
      AND tg.entity_id = s.id
      AND tg.lang = ?2
      AND tg.field = 'genus'
LEFT JOIN translation tg_en
       ON tg_en.entity_type = 'species'
      AND tg_en.entity_id = s.id
      AND tg_en.lang = 'en'
      AND tg_en.field = 'genus'
WHERE s.id = ?1
";

fn load_species_identity(
    connection: &Connection,
    species_id: i64,
    lang: &str,
) -> Result<Option<SpeciesIdentity>, String> {
    let mut statement = connection.prepare(IDENTITY_SQL).map_err(db_error)?;
    let identity = statement
        .query_row(params![species_id, lang], |row| {
            Ok(SpeciesIdentity {
                species_id: row.get(0)?,
                national_dex_number: get_u16(row, 1)?,
                slug: row.get(2)?,
                name: row.get(3)?,
                genus: row.get::<_, Option<String>>(4)?,
                generation_id: get_u16(row, 5)?,
                is_legendary: row.get::<_, i64>(6)? != 0,
                is_mythical: row.get::<_, i64>(7)? != 0,
                is_baby: row.get::<_, i64>(8)? != 0,
                is_ultra_beast: row.get::<_, i64>(9)? != 0,
                capture_rate: get_u16(row, 10)?,
                base_happiness: get_optional_u16(row, 11)?,
                hatch_counter: get_optional_u16(row, 12)?,
                gender_rate: i16::try_from(row.get::<_, i64>(13)?).unwrap_or(0),
                growth_rate: row.get(14)?,
                color: row.get::<_, Option<String>>(15)?,
                shape: row.get::<_, Option<String>>(16)?,
                habitat: row.get::<_, Option<String>>(17)?,
                evolution_chain_id: row.get::<_, Option<i64>>(18)?,
            })
        })
        .optional()
        .map_err(db_error)?;

    Ok(identity)
}

const FORMS_SQL: &str = r"
SELECT f.id,
       f.slug,
       COALESCE(tf.value, tf_en.value, ts.value, ts_en.value, f.slug) AS name,
       f.is_default,
       f.is_mega,
       f.is_gigantamax,
       f.is_regional,
       f.region_slug,
       f.introduced_generation_id,
       f.height_dm,
       f.weight_hg,
       fs.hp,
       fs.attack,
       fs.defense,
       fs.sp_atk,
       fs.sp_def,
       fs.speed,
       fs.bst
FROM pokemon_form f
JOIN species s ON s.id = f.species_id
JOIN form_stat_summary fs ON fs.form_id = f.id
LEFT JOIN translation tf
       ON tf.entity_type = 'form'
      AND tf.entity_id = f.id
      AND tf.lang = ?2
      AND tf.field = 'name'
LEFT JOIN translation tf_en
       ON tf_en.entity_type = 'form'
      AND tf_en.entity_id = f.id
      AND tf_en.lang = 'en'
      AND tf_en.field = 'name'
LEFT JOIN translation ts
       ON ts.entity_type = 'species'
      AND ts.entity_id = s.id
      AND ts.lang = ?2
      AND ts.field = 'name'
LEFT JOIN translation ts_en
       ON ts_en.entity_type = 'species'
      AND ts_en.entity_id = s.id
      AND ts_en.lang = 'en'
      AND ts_en.field = 'name'
WHERE f.species_id = ?1
ORDER BY f.is_default DESC, f.form_order, f.id
";

fn load_forms(
    connection: &Connection,
    species_id: i64,
    lang: &str,
) -> Result<Vec<FormSummary>, String> {
    let mut statement = connection.prepare(FORMS_SQL).map_err(db_error)?;
    let rows = statement
        .query_map(params![species_id, lang], |row| {
            Ok(FormSummary {
                id: row.get(0)?,
                slug: row.get(1)?,
                name: row.get(2)?,
                is_default: row.get::<_, i64>(3)? != 0,
                is_mega: row.get::<_, i64>(4)? != 0,
                is_gigantamax: row.get::<_, i64>(5)? != 0,
                is_regional: row.get::<_, i64>(6)? != 0,
                region: row.get::<_, Option<String>>(7)?,
                generation_id: get_u16(row, 8)?,
                types: Vec::new(),
                stats: BaseStats {
                    hp: get_u16(row, 11)?,
                    attack: get_u16(row, 12)?,
                    defense: get_u16(row, 13)?,
                    sp_atk: get_u16(row, 14)?,
                    sp_def: get_u16(row, 15)?,
                    speed: get_u16(row, 16)?,
                },
                bst: get_u16(row, 17)?,
                height_dm: get_u16(row, 9)?,
                weight_hg: get_u16(row, 10)?,
            })
        })
        .map_err(db_error)?
        .collect::<rusqlite::Result<Vec<_>>>()
        .map_err(db_error)?;

    let mut forms = Vec::with_capacity(rows.len());
    for mut form in rows {
        form.types = load_types(connection, form.id)?;
        forms.push(form);
    }

    Ok(forms)
}

fn select_form(forms: &[FormSummary], form_id: Option<i64>) -> Option<&FormSummary> {
    form_id
        .and_then(|id| forms.iter().find(|form| form.id == id))
        .or_else(|| forms.iter().find(|form| form.is_default))
        .or_else(|| forms.first())
}

const ABILITIES_SQL: &str = r"
SELECT a.id,
       a.slug,
       COALESCE(t.value, t_en.value, a.slug) AS name,
       fa.slot,
       fa.is_hidden
FROM form_ability fa
JOIN ability a ON a.id = fa.ability_id
LEFT JOIN translation t
       ON t.entity_type = 'ability'
      AND t.entity_id = a.id
      AND t.lang = ?2
      AND t.field = 'name'
LEFT JOIN translation t_en
       ON t_en.entity_type = 'ability'
      AND t_en.entity_id = a.id
      AND t_en.lang = 'en'
      AND t_en.field = 'name'
WHERE fa.form_id = ?1
ORDER BY fa.is_hidden, fa.slot
";

fn load_abilities(
    connection: &Connection,
    form_id: i64,
    lang: &str,
) -> Result<Vec<AbilityRef>, String> {
    let mut statement = connection.prepare(ABILITIES_SQL).map_err(db_error)?;

    statement
        .query_map(params![form_id, lang], |row| {
            Ok(AbilityRef {
                id: row.get(0)?,
                slug: row.get(1)?,
                name: row.get(2)?,
                slot: u8::try_from(row.get::<_, i64>(3)?).unwrap_or(1),
                is_hidden: row.get::<_, i64>(4)? != 0,
            })
        })
        .map_err(db_error)?
        .collect::<rusqlite::Result<Vec<_>>>()
        .map_err(db_error)
}

fn load_egg_groups(
    connection: &Connection,
    species_id: i64,
    lang: &str,
) -> Result<Vec<String>, String> {
    let mut statement = connection
        .prepare(
            "SELECT COALESCE(t.value, t_en.value, eg.slug)
               FROM species_egg_group seg
               JOIN egg_group eg ON eg.id = seg.egg_group_id
               LEFT JOIN translation t
                      ON t.entity_type = 'egg_group'
                     AND t.entity_id = eg.id
                     AND t.lang = ?2
                     AND t.field = 'name'
               LEFT JOIN translation t_en
                      ON t_en.entity_type = 'egg_group'
                     AND t_en.entity_id = eg.id
                     AND t_en.lang = 'en'
                     AND t_en.field = 'name'
              WHERE seg.species_id = ?1
              ORDER BY eg.id",
        )
        .map_err(db_error)?;

    statement
        .query_map(params![species_id, lang], |row| row.get::<_, String>(0))
        .map_err(db_error)?
        .collect::<rusqlite::Result<Vec<_>>>()
        .map_err(db_error)
}

fn load_flavor_texts(
    connection: &Connection,
    species_id: i64,
    lang: &str,
) -> Result<Vec<FlavorTextEntry>, String> {
    let mut statement = connection
        .prepare(
            "SELECT ft.version_id,
                    gv.slug,
                    vg.generation_id,
                    ft.text
               FROM flavor_text ft
               JOIN game_version gv ON gv.id = ft.version_id
               JOIN version_group vg ON vg.id = gv.version_group_id
              WHERE ft.species_id = ?1
                AND ft.lang = ?2
              ORDER BY vg.generation_id DESC, ft.version_id DESC",
        )
        .map_err(db_error)?;

    statement
        .query_map(params![species_id, lang], |row| {
            Ok(FlavorTextEntry {
                version_id: row.get(0)?,
                version_slug: row.get(1)?,
                generation_id: get_u16(row, 2)?,
                text: row.get(3)?,
            })
        })
        .map_err(db_error)?
        .collect::<rusqlite::Result<Vec<_>>>()
        .map_err(db_error)
}

const EVOLUTIONS_SQL: &str = r"
SELECT es.from_species_id,
       es.to_species_id,
       sto.slug,
       sto.national_dex_number,
       es.trigger_slug,
       es.min_level,
       es.trigger_item_slug,
       es.held_item_slug,
       es.time_of_day,
       es.location_slug,
       es.known_move_id,
       es.known_move_type_id,
       es.min_happiness,
       es.min_affection,
       es.min_beauty,
       es.gender,
       es.needs_overworld_rain,
       es.turn_upside_down,
       es.relative_physical_stats,
       es.condition_expression,
       es.extra_condition_json,
       es.trade_species_id,
       es.party_species_id,
       es.party_type_id
FROM evolution_step es
JOIN species sto ON sto.id = es.to_species_id
WHERE es.chain_id = ?1
ORDER BY es.id
";

fn load_evolutions(
    connection: &Connection,
    chain_id: i64,
    lang: &str,
) -> Result<Vec<EvolutionStepView>, String> {
    let mut statement = connection.prepare(EVOLUTIONS_SQL).map_err(db_error)?;
    let rows = statement
        .query_map([chain_id], |row| {
            Ok((
                row.get::<_, Option<i64>>(0)?,
                row.get::<_, i64>(1)?,
                row.get::<_, String>(2)?,
                get_u16(row, 3)?,
                row.get::<_, String>(4)?,
                get_optional_u16(row, 5)?,
                row.get::<_, Option<String>>(6)?,
                row.get::<_, Option<String>>(7)?,
                row.get::<_, Option<String>>(8)?,
                row.get::<_, Option<String>>(9)?,
                row.get::<_, Option<i64>>(10)?,
                row.get::<_, Option<i64>>(11)?,
                get_optional_u16(row, 12)?,
                get_optional_u16(row, 13)?,
                get_optional_u16(row, 14)?,
                row.get::<_, Option<String>>(15)?,
                row.get::<_, i64>(16)? != 0,
                row.get::<_, i64>(17)? != 0,
                row.get::<_, Option<i16>>(18)?,
                row.get::<_, Option<String>>(19)?,
                row.get::<_, Option<String>>(20)?,
                row.get::<_, Option<i64>>(21)?,
                row.get::<_, Option<i64>>(22)?,
                row.get::<_, Option<i64>>(23)?,
            ))
        })
        .map_err(db_error)?
        .collect::<rusqlite::Result<Vec<_>>>()
        .map_err(db_error)?;

    rows.into_iter()
        .map(
            |(
                from_species_id,
                to_species_id,
                to_slug,
                to_national_dex_number,
                trigger,
                min_level,
                trigger_item,
                held_item,
                time_of_day,
                location,
                known_move_id,
                known_move_type_id,
                min_happiness,
                min_affection,
                min_beauty,
                gender,
                needs_overworld_rain,
                turn_upside_down,
                relative_physical_stats,
                condition_expression,
                extra_condition_json,
                trade_species_id,
                party_species_id,
                party_type_id,
            )| {
                Ok(EvolutionStepView {
                    from_species_id,
                    from_name: match from_species_id {
                        Some(id) => species_name(connection, id, lang)?,
                        None => None,
                    },
                    to_species_id,
                    to_name: species_name(connection, to_species_id, lang)?.unwrap_or(to_slug),
                    to_national_dex_number,
                    trigger,
                    min_level,
                    trigger_item,
                    held_item,
                    time_of_day,
                    location,
                    known_move: match known_move_id {
                        Some(id) => move_name(connection, id, lang)?,
                        None => None,
                    },
                    known_move_type: match known_move_type_id {
                        Some(id) => type_name(connection, id, lang)?,
                        None => None,
                    },
                    min_happiness,
                    min_affection,
                    min_beauty,
                    gender,
                    needs_overworld_rain,
                    turn_upside_down,
                    relative_physical_stats,
                    condition_expression,
                    extra_condition_json,
                    trade_species: match trade_species_id {
                        Some(id) => species_name(connection, id, lang)?,
                        None => None,
                    },
                    party_species: match party_species_id {
                        Some(id) => species_name(connection, id, lang)?,
                        None => None,
                    },
                    party_type: match party_type_id {
                        Some(id) => type_name(connection, id, lang)?,
                        None => None,
                    },
                })
            },
        )
        .collect()
}

/// Multiplies the per-type defensive multipliers of every defending type.
///
/// A dual type can stack two weaknesses (400 %), two resistances (25 %) or a
/// weakness and a resistance (100 %), hence the product divided by 100.
fn load_effectiveness(
    connection: &Connection,
    defending_types: &[String],
    generation_id: u16,
) -> Result<Vec<TypeEffectiveness>, String> {
    if defending_types.is_empty() {
        return Ok(Vec::new());
    }

    let mut sql = String::from(
        "SELECT at.slug, te.multiplier_x100
           FROM type_efficacy te
           JOIN poke_type at ON at.id = te.attacking_type
           JOIN poke_type dt ON dt.id = te.defending_type
          WHERE te.generation_id = ?
            AND dt.slug IN ",
    );
    let mut params = vec![Value::Integer(i64::from(generation_id))];
    push_in_clause(&mut sql, &mut params, &dedupe_slugs(defending_types));
    sql.push_str(" ORDER BY at.id");

    let mut statement = connection.prepare(&sql).map_err(db_error)?;
    let rows = statement
        .query_map(params_from_iter(params.iter()), |row| {
            Ok((row.get::<_, String>(0)?, get_u16(row, 1)?))
        })
        .map_err(db_error)?
        .collect::<rusqlite::Result<Vec<_>>>()
        .map_err(db_error)?;

    let mut effectiveness: Vec<TypeEffectiveness> = Vec::with_capacity(rows.len());
    for (attacking_type, multiplier_percent) in rows {
        match effectiveness
            .iter_mut()
            .find(|entry| entry.attacking_type == attacking_type)
        {
            Some(entry) => {
                let product = u32::from(entry.multiplier_percent) * u32::from(multiplier_percent);
                entry.multiplier_percent = u16::try_from(product / 100).unwrap_or(u16::MAX);
            }
            None => effectiveness.push(TypeEffectiveness {
                attacking_type,
                multiplier_percent,
            }),
        }
    }

    Ok(effectiveness)
}

fn translation_value(
    connection: &Connection,
    entity_type: &str,
    entity_id: i64,
    lang: &str,
    field: &str,
) -> Result<Option<String>, String> {
    connection
        .query_row(
            "SELECT value
               FROM translation
              WHERE entity_type = ?1
                AND entity_id = ?2
                AND lang = ?3
                AND field = ?4",
            params![entity_type, entity_id, lang, field],
            |row| row.get(0),
        )
        .optional()
        .map_err(db_error)
}

fn species_name(
    connection: &Connection,
    species_id: i64,
    lang: &str,
) -> Result<Option<String>, String> {
    translation_value(connection, "species", species_id, lang, "name")
}

fn move_name(connection: &Connection, move_id: i64, lang: &str) -> Result<Option<String>, String> {
    if let Some(name) = translation_value(connection, "move", move_id, lang, "name")? {
        return Ok(Some(name));
    }

    connection
        .query_row("SELECT slug FROM move_ WHERE id = ?1", [move_id], |row| {
            row.get(0)
        })
        .optional()
        .map_err(db_error)
}

fn type_name(connection: &Connection, type_id: i64, lang: &str) -> Result<Option<String>, String> {
    if let Some(name) = translation_value(connection, "type", type_id, lang, "name")? {
        return Ok(Some(name));
    }

    connection
        .query_row(
            "SELECT slug FROM poke_type WHERE id = ?1",
            [type_id],
            |row| row.get(0),
        )
        .optional()
        .map_err(db_error)
}

struct ListSql {
    where_clause: String,
    order_clause: String,
    params: Vec<Value>,
}

/// Builds the shared `WHERE`/`ORDER BY` fragments of a list query.
///
/// Returns `None` when the search term cannot produce a valid FTS5 query: the
/// caller then answers with an empty page instead of silently dropping the
/// search filter. Every identifier interpolated into the SQL comes from a
/// closed set (`match` over domain enums), so user input only ever travels as a
/// bound parameter.
fn build_list_sql(query: &SpeciesListQuery) -> Option<ListSql> {
    let mut params = Vec::new();
    params.push(Value::Text(query.lang.clone()));

    let mut where_clause = String::from(" WHERE 1 = 1");

    if let Some(search) = query.search.as_deref() {
        let trimmed = search.trim();
        if !trimmed.is_empty() {
            let fts = fts_query(trimmed)?;
            where_clause.push_str(
                " AND s.id IN (
                    SELECT CAST(entity_id AS INTEGER)
                      FROM search_index
                     WHERE search_index MATCH ?
                       AND entity_type = 'species'
                  )",
            );
            params.push(Value::Text(fts));
        }
    }

    let types = dedupe_slugs(&query.types);
    if !types.is_empty() {
        match query.type_match {
            TypeMatchMode::Any => {
                where_clause.push_str(
                    " AND EXISTS (
                        SELECT 1
                          FROM form_type ft
                          JOIN poke_type pt ON pt.id = ft.type_id
                         WHERE ft.form_id = f.id
                           AND pt.slug IN ",
                );
                push_in_clause(&mut where_clause, &mut params, &types);
                where_clause.push(')');
            }
            TypeMatchMode::Exact => {
                where_clause.push_str(
                    " AND (SELECT COUNT(*) FROM form_type ft WHERE ft.form_id = f.id) = ?",
                );
                params.push(Value::Integer(
                    i64::try_from(types.len()).unwrap_or(i64::MAX),
                ));
                where_clause.push_str(
                    " AND (SELECT COUNT(*)
                             FROM form_type ft
                             JOIN poke_type pt ON pt.id = ft.type_id
                            WHERE ft.form_id = f.id
                              AND pt.slug IN ",
                );
                push_in_clause(&mut where_clause, &mut params, &types);
                where_clause.push_str(") = ?");
                params.push(Value::Integer(
                    i64::try_from(types.len()).unwrap_or(i64::MAX),
                ));
            }
            TypeMatchMode::Mono => {
                where_clause.push_str(
                    " AND (SELECT COUNT(*) FROM form_type ft WHERE ft.form_id = f.id) = 1
                      AND EXISTS (
                        SELECT 1
                          FROM form_type ft
                          JOIN poke_type pt ON pt.id = ft.type_id
                         WHERE ft.form_id = f.id
                           AND pt.slug IN ",
                );
                push_in_clause(&mut where_clause, &mut params, &types);
                where_clause.push(')');
            }
        }
    }

    let generations = dedupe_generations(&query.generations);
    if !generations.is_empty() {
        where_clause.push_str(" AND s.generation_id IN ");
        where_clause.push('(');
        for (index, generation) in generations.iter().enumerate() {
            if index > 0 {
                where_clause.push_str(", ");
            }
            where_clause.push('?');
            params.push(Value::Integer(i64::from(*generation)));
        }
        where_clause.push(')');
    }

    for range in &query.stat_ranges {
        let column = stat_column(range.stat);
        if let Some(min) = range.min {
            where_clause.push_str(" AND ");
            where_clause.push_str(column);
            where_clause.push_str(" >= ?");
            params.push(Value::Integer(i64::from(min)));
        }
        if let Some(max) = range.max {
            where_clause.push_str(" AND ");
            where_clause.push_str(column);
            where_clause.push_str(" <= ?");
            params.push(Value::Integer(i64::from(max)));
        }
    }

    if query.flags.any() {
        let mut clauses = Vec::new();
        if query.flags.legendary {
            clauses.push("s.is_legendary = 1");
        }
        if query.flags.mythical {
            clauses.push("s.is_mythical = 1");
        }
        if query.flags.baby {
            clauses.push("s.is_baby = 1");
        }
        if query.flags.ultra_beast {
            clauses.push("s.is_ultra_beast = 1");
        }

        where_clause.push_str(" AND (");
        where_clause.push_str(&clauses.join(" OR "));
        where_clause.push(')');
    }

    let order_clause = build_order_clause(&query.sort);

    Some(ListSql {
        where_clause,
        order_clause,
        params,
    })
}

fn build_order_clause(sort: &[SortCriterion]) -> String {
    let mut parts = Vec::with_capacity(sort.len() + 1);

    for criterion in sort {
        let expression = sort_expression(criterion.field);
        let direction = match criterion.direction {
            SortDirection::Asc => "ASC",
            SortDirection::Desc => "DESC",
        };
        parts.push(format!("{expression} {direction}"));
    }

    // A stable tie-breaker keeps pagination deterministic across pages.
    if !matches!(
        sort.last().map(|criterion| criterion.field),
        Some(SpeciesSortField::DexNumber)
    ) {
        parts.push("s.national_dex_number ASC".to_owned());
    }

    format!(" ORDER BY {}", parts.join(", "))
}

const fn sort_expression(field: SpeciesSortField) -> &'static str {
    match field {
        SpeciesSortField::DexNumber => "s.national_dex_number",
        SpeciesSortField::Name => "name COLLATE NOCASE",
        SpeciesSortField::Bst => "fs.bst",
        SpeciesSortField::Hp => "fs.hp",
        SpeciesSortField::Attack => "fs.attack",
        SpeciesSortField::Defense => "fs.defense",
        SpeciesSortField::SpAtk => "fs.sp_atk",
        SpeciesSortField::SpDef => "fs.sp_def",
        SpeciesSortField::Speed => "fs.speed",
    }
}

const fn stat_column(stat: SpeciesStat) -> &'static str {
    match stat {
        SpeciesStat::Hp => "fs.hp",
        SpeciesStat::Attack => "fs.attack",
        SpeciesStat::Defense => "fs.defense",
        SpeciesStat::SpAtk => "fs.sp_atk",
        SpeciesStat::SpDef => "fs.sp_def",
        SpeciesStat::Speed => "fs.speed",
        SpeciesStat::Bst => "fs.bst",
    }
}

fn push_in_clause(clause: &mut String, params: &mut Vec<Value>, values: &[String]) {
    clause.push('(');
    for (index, value) in values.iter().enumerate() {
        if index > 0 {
            clause.push_str(", ");
        }
        clause.push('?');
        params.push(Value::Text(value.clone()));
    }
    clause.push(')');
}

fn dedupe_slugs(values: &[String]) -> Vec<String> {
    let mut deduped: Vec<String> = Vec::with_capacity(values.len());

    for value in values {
        let slug = value.trim().to_lowercase();
        if !slug.is_empty() && !deduped.contains(&slug) {
            deduped.push(slug);
        }
    }

    deduped
}

fn dedupe_generations(values: &[u16]) -> Vec<u16> {
    let mut deduped: Vec<u16> = Vec::with_capacity(values.len());

    for value in values {
        if *value > 0 && !deduped.contains(value) {
            deduped.push(*value);
        }
    }

    deduped
}

fn seed(connection: &mut Connection, species: &[FixtureSpecies]) -> Result<(), String> {
    let transaction = connection.transaction().map_err(db_error)?;

    for (index, (slug, region)) in GENERATIONS.iter().enumerate() {
        transaction
            .execute(
                "INSERT INTO generation (id, slug, region_slug) VALUES (?1, ?2, ?3)",
                params![index as i64 + 1, slug, region],
            )
            .map_err(db_error)?;
    }

    let mut type_ids = HashMap::new();
    for (index, (slug, generation)) in TYPES.iter().enumerate() {
        let id = index as i64 + 1;
        transaction
            .execute(
                "INSERT INTO poke_type (id, slug, introduced_in) VALUES (?1, ?2, ?3)",
                params![id, slug, generation],
            )
            .map_err(db_error)?;
        type_ids.insert(*slug, id);
    }

    for entry in species {
        transaction
            .execute(
                "INSERT INTO species (
                    id, slug, national_dex_number, generation_id,
                    capture_rate, base_happiness, is_legendary, is_mythical
                 ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
                params![
                    entry.id,
                    entry.slug,
                    entry.national_dex_number,
                    entry.generation,
                    entry.capture_rate,
                    entry.base_happiness,
                    entry.is_legendary,
                    entry.is_mythical,
                ],
            )
            .map_err(db_error)?;

        // The fixture only contains default forms, so the form reuses the
        // species identifier. Future datasets keep their own form ids.
        transaction
            .execute(
                "INSERT INTO pokemon_form (
                    id, species_id, slug, form_order, is_default,
                    introduced_generation_id, height_dm, weight_hg
                 ) VALUES (?1, ?2, ?3, 1, 1, ?4, ?5, ?6)",
                params![
                    entry.id,
                    entry.id,
                    entry.slug,
                    entry.generation,
                    entry.height_dm,
                    entry.weight_hg,
                ],
            )
            .map_err(db_error)?;

        for (index, type_slug) in entry.types.iter().enumerate() {
            let type_id = type_ids.get(type_slug.as_str()).ok_or_else(|| {
                format!(
                    "unknown type `{type_slug}` in fixture species `{}`",
                    entry.slug
                )
            })?;

            transaction
                .execute(
                    "INSERT INTO form_type (form_id, slot, type_id) VALUES (?1, ?2, ?3)",
                    params![entry.id, index as i64 + 1, type_id],
                )
                .map_err(db_error)?;
        }

        let stats = &entry.stats;
        transaction
            .execute(
                "INSERT INTO form_stat_summary (
                    form_id, hp, attack, defense, sp_atk, sp_def, speed, bst
                 ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
                params![
                    entry.id,
                    stats.hp,
                    stats.attack,
                    stats.defense,
                    stats.sp_atk,
                    stats.sp_def,
                    stats.speed,
                    stats.bst(),
                ],
            )
            .map_err(db_error)?;

        for (lang, name) in [("fr", &entry.names.fr), ("en", &entry.names.en)] {
            transaction
                .execute(
                    "INSERT INTO translation (entity_type, entity_id, lang, field, value)
                     VALUES ('species', ?1, ?2, 'name', ?3)",
                    params![entry.id, lang, name],
                )
                .map_err(db_error)?;
        }

        let aliases = format!(
            "{} {} {} {}",
            entry.names.fr, entry.names.en, entry.slug, entry.national_dex_number
        );

        transaction
            .execute(
                "INSERT INTO search_index (entity_type, entity_id, name, name_normalized, aliases)
                 VALUES ('species', ?1, ?2, ?3, ?4)",
                params![
                    entry.id.to_string(),
                    entry.names.en,
                    normalize(&entry.names.en),
                    aliases,
                ],
            )
            .map_err(db_error)?;
    }

    let species_count: i64 = transaction
        .query_row("SELECT COUNT(*) FROM species", [], |row| row.get(0))
        .map_err(db_error)?;

    for (key, value) in [
        ("version", "0.1.0-fixture".to_owned()),
        ("built_at", "fixture".to_owned()),
        ("source_commit", "fixture".to_owned()),
        ("schema_version", "1".to_owned()),
        ("checksum", "fixture".to_owned()),
        (
            "row_counts_json",
            format!("{{\"species\":{species_count},\"forms\":{species_count}}}"),
        ),
    ] {
        transaction
            .execute(
                "INSERT INTO dataset_meta (key, value) VALUES (?1, ?2)",
                params![key, value],
            )
            .map_err(db_error)?;
    }

    transaction.commit().map_err(db_error)
}

fn load_types(connection: &Connection, form_id: i64) -> Result<Vec<String>, String> {
    let mut statement = connection
        .prepare(
            "SELECT pt.slug
               FROM form_type ft
               JOIN poke_type pt ON pt.id = ft.type_id
              WHERE ft.form_id = ?1
              ORDER BY ft.slot",
        )
        .map_err(db_error)?;

    statement
        .query_map([form_id], |row| row.get::<_, String>(0))
        .map_err(db_error)?
        .collect::<rusqlite::Result<Vec<_>>>()
        .map_err(db_error)
}

fn map_summary_row(row: &Row<'_>) -> rusqlite::Result<SummaryRow> {
    Ok((
        row.get(0)?,
        get_u16(row, 1)?,
        row.get(2)?,
        row.get(3)?,
        read_stats(row)?,
        get_u16(row, 10)?,
        row.get(11)?,
    ))
}

type DetailRow = (
    i64,
    u16,
    String,
    String,
    BaseStats,
    u16,
    i64,
    u16,
    u16,
    u16,
    Option<u16>,
    bool,
    bool,
);

fn map_detail_row(row: &Row<'_>) -> rusqlite::Result<DetailRow> {
    Ok((
        row.get(0)?,
        get_u16(row, 1)?,
        row.get(2)?,
        row.get(3)?,
        read_stats(row)?,
        get_u16(row, 10)?,
        row.get(11)?,
        get_u16(row, 12)?,
        get_u16(row, 13)?,
        get_u16(row, 14)?,
        get_optional_u16(row, 15)?,
        row.get::<_, i64>(16)? != 0,
        row.get::<_, i64>(17)? != 0,
    ))
}

fn read_stats(row: &Row<'_>) -> rusqlite::Result<BaseStats> {
    Ok(BaseStats {
        hp: get_u16(row, 4)?,
        attack: get_u16(row, 5)?,
        defense: get_u16(row, 6)?,
        sp_atk: get_u16(row, 7)?,
        sp_def: get_u16(row, 8)?,
        speed: get_u16(row, 9)?,
    })
}

fn get_u16(row: &Row<'_>, index: usize) -> rusqlite::Result<u16> {
    let value: i64 = row.get(index)?;
    u16::try_from(value).map_err(|_| rusqlite::Error::IntegralValueOutOfRange(index, value))
}

fn get_optional_u16(row: &Row<'_>, index: usize) -> rusqlite::Result<Option<u16>> {
    let value: Option<i64> = row.get(index)?;
    value
        .map(|value| {
            u16::try_from(value).map_err(|_| rusqlite::Error::IntegralValueOutOfRange(index, value))
        })
        .transpose()
}

/// Builds a safe FTS5 prefix query from user input.
///
/// Punctuation is dropped instead of escaped: user input never reaches the
/// FTS5 grammar verbatim, so operators such as `NOT` or `NEAR` cannot break the
/// query.
fn fts_query(query: &str) -> Option<String> {
    let normalized = normalize(query);
    let cleaned: String = normalized
        .chars()
        .map(|character| {
            if character.is_alphanumeric() {
                character
            } else {
                ' '
            }
        })
        .collect();

    let tokens: Vec<String> = cleaned
        .split_whitespace()
        .map(|token| format!("{token}*"))
        .collect();

    if tokens.is_empty() {
        None
    } else {
        Some(tokens.join(" "))
    }
}

/// Folds accented characters and lowercases the result.
///
/// FTS5 already removes diacritics, but the same normalization is used for
/// indexing and for user queries so prefixes behave consistently.
fn normalize(value: &str) -> String {
    let mut normalized = String::with_capacity(value.len());

    for character in value.chars().flat_map(char::to_lowercase) {
        match character {
            'à' | 'á' | 'â' | 'ã' | 'ä' | 'å' => normalized.push('a'),
            'ç' => normalized.push('c'),
            'è' | 'é' | 'ê' | 'ë' => normalized.push('e'),
            'ì' | 'í' | 'î' | 'ï' => normalized.push('i'),
            'ñ' => normalized.push('n'),
            'ò' | 'ó' | 'ô' | 'õ' | 'ö' => normalized.push('o'),
            'ù' | 'ú' | 'û' | 'ü' => normalized.push('u'),
            'ý' | 'ÿ' => normalized.push('y'),
            'æ' => normalized.push_str("ae"),
            'œ' => normalized.push_str("oe"),
            other => normalized.push(other),
        }
    }

    normalized
}

fn db_error(error: rusqlite::Error) -> String {
    format!("reference database error: {error}")
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct FixtureSpecies {
    id: i64,
    slug: String,
    national_dex_number: u16,
    generation: i64,
    names: FixtureNames,
    types: Vec<String>,
    stats: FixtureStats,
    height_dm: u16,
    weight_hg: u16,
    capture_rate: u16,
    base_happiness: Option<u16>,
    is_legendary: bool,
    is_mythical: bool,
}

#[derive(Debug, Deserialize)]
struct FixtureNames {
    fr: String,
    en: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct FixtureStats {
    hp: u16,
    attack: u16,
    defense: u16,
    sp_atk: u16,
    sp_def: u16,
    speed: u16,
}

impl FixtureStats {
    const fn bst(&self) -> u16 {
        self.hp + self.attack + self.defense + self.sp_atk + self.sp_def + self.speed
    }
}
