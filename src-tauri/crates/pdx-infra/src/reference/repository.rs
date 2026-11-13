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
    BaseStats, SortCriterion, SortDirection, SpeciesDetail, SpeciesListQuery, SpeciesPage,
    SpeciesSortField, SpeciesStat, SpeciesSummary, TypeMatchMode,
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

type SummaryRow = (i64, u16, String, String, u16, i64);

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
            .map(|(id, national_dex_number, slug, name, bst, form_id)| {
                Ok(SpeciesSummary {
                    id,
                    national_dex_number,
                    slug,
                    name,
                    types: load_types(&self.connection, form_id)?,
                    bst,
                })
            })
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
        get_u16(row, 4)?,
        row.get(5)?,
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
