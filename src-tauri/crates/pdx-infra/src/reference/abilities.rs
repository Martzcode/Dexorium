//! SQLite implementation of the ability catalogue.

use pdx_app::AbilityRepository;
use pdx_domain::{
    AbilityDetail, AbilityHolder, AbilityListQuery, AbilityPage, AbilitySummary,
};
use rusqlite::types::Value;
use rusqlite::{Connection, OptionalExtension, Row, params, params_from_iter};

use super::repository::{SqliteSpeciesRepository, db_error, fts_query, get_u16};

const ABILITY_FROM: &str = r"
FROM ability a
LEFT JOIN translation t
       ON t.entity_type = 'ability'
      AND t.entity_id = a.id
      AND t.lang = ?
      AND t.field = 'name'
LEFT JOIN translation t_en
       ON t_en.entity_type = 'ability'
      AND t_en.entity_id = a.id
      AND t_en.lang = 'en'
      AND t_en.field = 'name'
";

const ABILITY_COLUMNS: &str = r"
SELECT a.id,
       a.slug,
       COALESCE(t.value, t_en.value, a.slug) AS name,
       a.generation_id,
       a.is_main_series,
       (SELECT COUNT(DISTINCT f.species_id)
          FROM form_ability fa
          JOIN pokemon_form f ON f.id = fa.form_id
         WHERE fa.ability_id = a.id) AS holder_count
";

impl AbilityRepository for SqliteSpeciesRepository {
    fn list_abilities(&self, query: &AbilityListQuery) -> Result<AbilityPage, String> {
        let Some(sql) = build_ability_sql(query) else {
            return Ok(AbilityPage {
                items: Vec::new(),
                total: 0,
                offset: query.offset,
                limit: query.limit,
            });
        };

        let count_sql = format!("SELECT COUNT(*) {ABILITY_FROM}{}", sql.where_clause);
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
            "{ABILITY_COLUMNS}{ABILITY_FROM}{} ORDER BY name COLLATE NOCASE LIMIT ? OFFSET ?",
            sql.where_clause
        );
        let mut statement = self.connection.prepare(&page_sql).map_err(db_error)?;
        let items = statement
            .query_map(params_from_iter(page_params.iter()), map_ability_row)
            .map_err(db_error)?
            .collect::<rusqlite::Result<Vec<_>>>()
            .map_err(db_error)?;

        Ok(AbilityPage {
            items,
            total: u32::try_from(total).unwrap_or(u32::MAX),
            offset: query.offset,
            limit: query.limit,
        })
    }

    fn ability_detail(&self, id: i64, lang: &str) -> Result<Option<AbilityDetail>, String> {
        let mut statement = self
            .connection
            .prepare(ABILITY_DETAIL_SQL)
            .map_err(db_error)?;
        let row = statement
            .query_row(params![id, lang], map_ability_detail_row)
            .optional()
            .map_err(db_error)?;

        let Some(mut detail) = row else {
            return Ok(None);
        };

        detail.holders = load_holders(&self.connection, id, lang)?;

        Ok(Some(detail))
    }
}

struct AbilitySql {
    where_clause: String,
    params: Vec<Value>,
}

fn build_ability_sql(query: &AbilityListQuery) -> Option<AbilitySql> {
    let mut params = vec![Value::Text(query.lang.clone())];
    let mut where_clause = String::from(" WHERE 1 = 1");

    if let Some(search) = query.search.as_deref() {
        let trimmed = search.trim();
        if !trimmed.is_empty() {
            let fts = fts_query(trimmed)?;
            where_clause.push_str(
                " AND a.id IN (
                    SELECT CAST(entity_id AS INTEGER)
                      FROM search_index
                     WHERE search_index MATCH ?
                       AND entity_type = 'ability'
                  )",
            );
            params.push(Value::Text(fts));
        }
    }

    if !query.generations.is_empty() {
        where_clause.push_str(" AND a.generation_id IN (");
        for (index, generation) in query.generations.iter().enumerate() {
            if index > 0 {
                where_clause.push_str(", ");
            }
            where_clause.push('?');
            params.push(Value::Integer(i64::from(*generation)));
        }
        where_clause.push(')');
    }

    Some(AbilitySql {
        where_clause,
        params,
    })
}

fn map_ability_row(row: &Row<'_>) -> rusqlite::Result<AbilitySummary> {
    Ok(AbilitySummary {
        id: row.get(0)?,
        slug: row.get(1)?,
        name: row.get(2)?,
        generation_id: get_u16(row, 3)?,
        is_main_series: row.get::<_, i64>(4)? != 0,
        holder_count: u32::try_from(row.get::<_, i64>(5)?).unwrap_or(u32::MAX),
    })
}

const ABILITY_DETAIL_SQL: &str = r"
SELECT a.id,
       a.slug,
       COALESCE(t.value, t_en.value, a.slug) AS name,
       a.generation_id,
       a.is_main_series,
       ts.value AS effect_short,
       tl.value AS effect_long,
       td.value AS description
FROM ability a
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
LEFT JOIN translation ts
       ON ts.entity_type = 'ability'
      AND ts.entity_id = a.id
      AND ts.lang = ?2
      AND ts.field = 'effect_short'
LEFT JOIN translation tl
       ON tl.entity_type = 'ability'
      AND tl.entity_id = a.id
      AND tl.lang = ?2
      AND tl.field = 'effect_long'
LEFT JOIN translation td
       ON td.entity_type = 'ability'
      AND td.entity_id = a.id
      AND td.lang = ?2
      AND td.field = 'description'
WHERE a.id = ?1
";

fn map_ability_detail_row(row: &Row<'_>) -> rusqlite::Result<AbilityDetail> {
    Ok(AbilityDetail {
        id: row.get(0)?,
        slug: row.get(1)?,
        name: row.get(2)?,
        generation_id: get_u16(row, 3)?,
        is_main_series: row.get::<_, i64>(4)? != 0,
        effect_short: row.get(5)?,
        effect_long: row.get(6)?,
        description: row.get(7)?,
        holders: Vec::new(),
    })
}

const ABILITY_HOLDERS_SQL: &str = r"
SELECT f.id,
       f.species_id,
       s.national_dex_number,
       COALESCE(ts.value, ts_en.value, s.slug) AS species_name,
       COALESCE(tf.value, tf_en.value, ts.value, ts_en.value, f.slug) AS form_name,
       fa.slot,
       fa.is_hidden
FROM form_ability fa
JOIN pokemon_form f ON f.id = fa.form_id
JOIN species s ON s.id = f.species_id
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
WHERE fa.ability_id = ?1
ORDER BY s.national_dex_number, f.form_order, fa.slot
LIMIT 500
";

fn load_holders(
    connection: &Connection,
    ability_id: i64,
    lang: &str,
) -> Result<Vec<AbilityHolder>, String> {
    let mut statement = connection.prepare(ABILITY_HOLDERS_SQL).map_err(db_error)?;

    statement
        .query_map(params![ability_id, lang], |row| {
            Ok(AbilityHolder {
                form_id: row.get(0)?,
                species_id: row.get(1)?,
                national_dex_number: get_u16(row, 2)?,
                species_name: row.get(3)?,
                form_name: row.get(4)?,
                slot: u8::try_from(row.get::<_, i64>(5)?).unwrap_or(1),
                is_hidden: row.get::<_, i64>(6)? != 0,
            })
        })
        .map_err(db_error)?
        .collect::<rusqlite::Result<Vec<_>>>()
        .map_err(db_error)
}
