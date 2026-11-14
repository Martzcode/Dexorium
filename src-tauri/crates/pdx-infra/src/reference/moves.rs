//! SQLite implementation of the move catalogue and learnsets.

use std::collections::HashMap;

use pdx_app::MoveRepository;
use pdx_domain::{
    LearnsetEntry, LearnsetPage, MoveDetail, MoveLearner, MoveListQuery, MovePage,
    MoveSortCriterion, MoveSortField, MoveSummary, SortDirection, VersionGroupRef,
};
use rusqlite::types::Value;
use rusqlite::{Connection, OptionalExtension, Row, params, params_from_iter};

use super::repository::{
    SqliteSpeciesRepository, db_error, fts_query, get_optional_u16, get_u16,
};

const MOVE_FROM: &str = r"
FROM move_ m
JOIN poke_type pt ON pt.id = m.type_id
LEFT JOIN translation t
       ON t.entity_type = 'move'
      AND t.entity_id = m.id
      AND t.lang = ?
      AND t.field = 'name'
LEFT JOIN translation t_en
       ON t_en.entity_type = 'move'
      AND t_en.entity_id = m.id
      AND t_en.lang = 'en'
      AND t_en.field = 'name'
";

const MOVE_COLUMNS: &str = r"
SELECT m.id,
       m.slug,
       COALESCE(t.value, t_en.value, m.slug) AS name,
       pt.slug,
       m.damage_class,
       m.power,
       m.accuracy,
       m.pp,
       m.generation_id
";

impl MoveRepository for SqliteSpeciesRepository {
    fn list_moves(&self, query: &MoveListQuery) -> Result<MovePage, String> {
        let Some(sql) = build_move_sql(query) else {
            return Ok(MovePage {
                items: Vec::new(),
                total: 0,
                offset: query.offset,
                limit: query.limit,
            });
        };

        let count_sql = format!("SELECT COUNT(*) {MOVE_FROM}{}", sql.where_clause);
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
            "{MOVE_COLUMNS}{MOVE_FROM}{}{} LIMIT ? OFFSET ?",
            sql.where_clause, sql.order_clause
        );
        let mut statement = self.connection.prepare(&page_sql).map_err(db_error)?;
        let items = statement
            .query_map(params_from_iter(page_params.iter()), map_move_row)
            .map_err(db_error)?
            .collect::<rusqlite::Result<Vec<_>>>()
            .map_err(db_error)?;

        Ok(MovePage {
            items,
            total: u32::try_from(total).unwrap_or(u32::MAX),
            offset: query.offset,
            limit: query.limit,
        })
    }

    fn move_detail(&self, id: i64, lang: &str) -> Result<Option<MoveDetail>, String> {
        let mut statement = self.connection.prepare(MOVE_DETAIL_SQL).map_err(db_error)?;
        let row = statement
            .query_row(params![id, lang], map_move_detail_row)
            .optional()
            .map_err(db_error)?;

        let Some(mut detail) = row else {
            return Ok(None);
        };

        (detail.learner_version_group, detail.learners) =
            load_learners(&self.connection, id, lang)?;

        Ok(Some(detail))
    }

    fn learnset(
        &self,
        form_id: i64,
        lang: &str,
        version_group: Option<&str>,
    ) -> Result<LearnsetPage, String> {
        let version_groups = load_version_groups(&self.connection, form_id)?;
        let selected = version_group
            .and_then(|slug| {
                version_groups
                    .iter()
                    .find(|group| group.slug == slug)
                    .cloned()
            })
            .or_else(|| version_groups.first().cloned());

        let Some(selected) = selected else {
            return Ok(LearnsetPage {
                form_id,
                version_group: None,
                version_groups,
                entries: Vec::new(),
                total: 0,
            });
        };

        let entries = load_learnset_entries(&self.connection, form_id, selected.id, lang)?;

        Ok(LearnsetPage {
            form_id,
            version_group: Some(selected),
            version_groups,
            total: u32::try_from(entries.len()).unwrap_or(u32::MAX),
            entries,
        })
    }
}

struct MoveSql {
    where_clause: String,
    order_clause: String,
    params: Vec<Value>,
}

fn build_move_sql(query: &MoveListQuery) -> Option<MoveSql> {
    let mut params = vec![Value::Text(query.lang.clone())];
    let mut where_clause = String::from(" WHERE 1 = 1");

    if let Some(search) = query.search.as_deref() {
        let trimmed = search.trim();
        if !trimmed.is_empty() {
            let fts = fts_query(trimmed)?;
            where_clause.push_str(
                " AND m.id IN (
                    SELECT CAST(entity_id AS INTEGER)
                      FROM search_index
                     WHERE search_index MATCH ?
                       AND entity_type = 'move'
                  )",
            );
            params.push(Value::Text(fts));
        }
    }

    if !query.types.is_empty() {
        where_clause.push_str(" AND pt.slug IN ");
        push_in(&mut where_clause, &mut params, &query.types);
    }

    if !query.damage_classes.is_empty() {
        where_clause.push_str(" AND m.damage_class IN ");
        push_in(&mut where_clause, &mut params, &query.damage_classes);
    }

    if !query.generations.is_empty() {
        where_clause.push_str(" AND m.generation_id IN ");
        push_in(
            &mut where_clause,
            &mut params,
            &query
                .generations
                .iter()
                .map(u16::to_string)
                .collect::<Vec<_>>(),
        );
    }

    if let Some(min) = query.min_power {
        where_clause.push_str(" AND m.power IS NOT NULL AND m.power >= ?");
        params.push(Value::Integer(i64::from(min)));
    }
    if let Some(max) = query.max_power {
        where_clause.push_str(" AND m.power IS NOT NULL AND m.power <= ?");
        params.push(Value::Integer(i64::from(max)));
    }

    Some(MoveSql {
        where_clause,
        order_clause: build_move_order(&query.sort),
        params,
    })
}

fn build_move_order(sort: &[MoveSortCriterion]) -> String {
    let mut parts = Vec::with_capacity(sort.len() + 1);

    for criterion in sort {
        let expression = match criterion.field {
            MoveSortField::Id => "m.id",
            MoveSortField::Name => "name COLLATE NOCASE",
            MoveSortField::Power => "m.power IS NULL, m.power",
            MoveSortField::Accuracy => "m.accuracy IS NULL, m.accuracy",
            MoveSortField::Pp => "m.pp IS NULL, m.pp",
        };
        let direction = match criterion.direction {
            SortDirection::Asc => "ASC",
            SortDirection::Desc => "DESC",
        };
        parts.push(format!("{expression} {direction}"));
    }

    if !matches!(
        sort.last().map(|criterion| criterion.field),
        Some(MoveSortField::Id)
    ) {
        parts.push("m.id ASC".to_owned());
    }

    format!(" ORDER BY {}", parts.join(", "))
}

fn push_in(clause: &mut String, params: &mut Vec<Value>, values: &[String]) {
    clause.push('(');
    for (index, value) in values.iter().enumerate() {
        if index > 0 {
            clause.push_str(", ");
        }
        clause.push('?');
        if value.chars().all(|character| character.is_ascii_digit()) {
            params.push(Value::Integer(value.parse().unwrap_or_default()));
        } else {
            params.push(Value::Text(value.clone()));
        }
    }
    clause.push(')');
}

fn map_move_row(row: &Row<'_>) -> rusqlite::Result<MoveSummary> {
    Ok(MoveSummary {
        id: row.get(0)?,
        slug: row.get(1)?,
        name: row.get(2)?,
        type_slug: row.get(3)?,
        damage_class: row.get(4)?,
        power: get_optional_u16(row, 5)?,
        accuracy: get_optional_u16(row, 6)?,
        pp: get_optional_u16(row, 7)?,
        generation_id: get_u16(row, 8)?,
    })
}

const MOVE_DETAIL_SQL: &str = r"
SELECT m.id,
       m.slug,
       COALESCE(t.value, t_en.value, m.slug) AS name,
       m.generation_id,
       pt.slug,
       m.damage_class,
       m.power,
       m.accuracy,
       m.pp,
       m.priority,
       m.effect_chance,
       m.target_slug,
       m.ailment_slug,
       m.crit_rate_bonus,
       m.drain_pct,
       m.healing_pct,
       m.min_hits,
       m.max_hits,
       m.min_turns,
       m.max_turns,
       m.flinch_chance,
       m.stat_chance,
       m.flags_bitmask,
       ts.value AS effect_short,
       tl.value AS effect_long,
       td.value AS description
FROM move_ m
JOIN poke_type pt ON pt.id = m.type_id
LEFT JOIN translation t
       ON t.entity_type = 'move'
      AND t.entity_id = m.id
      AND t.lang = ?2
      AND t.field = 'name'
LEFT JOIN translation t_en
       ON t_en.entity_type = 'move'
      AND t_en.entity_id = m.id
      AND t_en.lang = 'en'
      AND t_en.field = 'name'
LEFT JOIN translation ts
       ON ts.entity_type = 'move'
      AND ts.entity_id = m.id
      AND ts.lang = ?2
      AND ts.field = 'effect_short'
LEFT JOIN translation tl
       ON tl.entity_type = 'move'
      AND tl.entity_id = m.id
      AND tl.lang = ?2
      AND tl.field = 'effect_long'
LEFT JOIN translation td
       ON td.entity_type = 'move'
      AND td.entity_id = m.id
      AND td.lang = ?2
      AND td.field = 'description'
WHERE m.id = ?1
";

fn map_move_detail_row(row: &Row<'_>) -> rusqlite::Result<MoveDetail> {
    Ok(MoveDetail {
        id: row.get(0)?,
        slug: row.get(1)?,
        name: row.get(2)?,
        generation_id: get_u16(row, 3)?,
        type_slug: row.get(4)?,
        damage_class: row.get(5)?,
        power: get_optional_u16(row, 6)?,
        accuracy: get_optional_u16(row, 7)?,
        pp: get_optional_u16(row, 8)?,
        priority: i16::try_from(row.get::<_, i64>(9)?).unwrap_or(0),
        effect_chance: get_optional_u16(row, 10)?,
        target: row.get(11)?,
        ailment: row.get::<_, Option<String>>(12)?,
        crit_rate_bonus: u8::try_from(row.get::<_, i64>(13)?).unwrap_or(0),
        drain_pct: i16::try_from(row.get::<_, i64>(14)?).unwrap_or(0),
        healing_pct: i16::try_from(row.get::<_, i64>(15)?).unwrap_or(0),
        min_hits: get_optional_u16(row, 16)?,
        max_hits: get_optional_u16(row, 17)?,
        min_turns: get_optional_u16(row, 18)?,
        max_turns: get_optional_u16(row, 19)?,
        flinch_chance: get_u16(row, 20)?,
        stat_chance: get_u16(row, 21)?,
        flags_bitmask: u32::try_from(row.get::<_, i64>(22)?).unwrap_or(0),
        effect_short: row.get(23)?,
        effect_long: row.get(24)?,
        description: row.get(25)?,
        learner_version_group: None,
        learners: Vec::new(),
    })
}

const LATEST_MOVE_GROUP_SQL: &str = r"
SELECT vg.slug
FROM form_move fm
JOIN version_group vg ON vg.id = fm.version_group_id
WHERE fm.move_id = ?1
GROUP BY vg.id
ORDER BY vg.order_index DESC
LIMIT 1
";

const MOVE_LEARNERS_SQL: &str = r"
SELECT f.id,
       f.species_id,
       s.national_dex_number,
       COALESCE(ts.value, ts_en.value, s.slug) AS species_name,
       COALESCE(tf.value, tf_en.value, ts.value, ts_en.value, f.slug) AS form_name,
       fm.method_slug,
       fm.level
FROM form_move fm
JOIN pokemon_form f ON f.id = fm.form_id
JOIN species s ON s.id = f.species_id
JOIN version_group vg ON vg.id = fm.version_group_id
LEFT JOIN translation ts
       ON ts.entity_type = 'species'
      AND ts.entity_id = s.id
      AND ts.lang = ?3
      AND ts.field = 'name'
LEFT JOIN translation ts_en
       ON ts_en.entity_type = 'species'
      AND ts_en.entity_id = s.id
      AND ts_en.lang = 'en'
      AND ts_en.field = 'name'
LEFT JOIN translation tf
       ON tf.entity_type = 'form'
      AND tf.entity_id = f.id
      AND tf.lang = ?3
      AND tf.field = 'name'
LEFT JOIN translation tf_en
       ON tf_en.entity_type = 'form'
      AND tf_en.entity_id = f.id
      AND tf_en.lang = 'en'
      AND tf_en.field = 'name'
WHERE fm.move_id = ?1
  AND vg.slug = ?2
ORDER BY s.national_dex_number, f.form_order, fm.method_slug, fm.level
";

fn load_learners(
    connection: &Connection,
    move_id: i64,
    lang: &str,
) -> Result<(Option<String>, Vec<MoveLearner>), String> {
    let version_group: Option<String> = connection
        .query_row(LATEST_MOVE_GROUP_SQL, [move_id], |row| row.get(0))
        .optional()
        .map_err(db_error)?;

    let Some(version_group) = version_group else {
        return Ok((None, Vec::new()));
    };

    let mut statement = connection.prepare(MOVE_LEARNERS_SQL).map_err(db_error)?;
    let rows = statement
        .query_map(params![move_id, version_group, lang], |row| {
            Ok((
                row.get::<_, i64>(0)?,
                row.get::<_, i64>(1)?,
                get_u16(row, 2)?,
                row.get::<_, String>(3)?,
                row.get::<_, String>(4)?,
                row.get::<_, String>(5)?,
                get_u16(row, 6)?,
            ))
        })
        .map_err(db_error)?
        .collect::<rusqlite::Result<Vec<_>>>()
        .map_err(db_error)?;

    let mut learners: Vec<MoveLearner> = Vec::new();
    let mut positions: HashMap<i64, usize> = HashMap::new();

    for (form_id, species_id, national_dex_number, species_name, form_name, method, level) in rows {
        let position = *positions.entry(form_id).or_insert_with(|| {
            learners.push(MoveLearner {
                form_id,
                species_id,
                national_dex_number,
                species_name: species_name.clone(),
                form_name: form_name.clone(),
                methods: Vec::new(),
                level: None,
            });
            learners.len() - 1
        });
        let learner = &mut learners[position];

        if !learner.methods.contains(&method) {
            learner.methods.push(method.clone());
        }

        if method == "level-up" && level > 0 {
            learner.level = Some(learner.level.map_or(level, |current| current.min(level)));
        }
    }

    Ok((Some(version_group), learners))
}

fn load_version_groups(
    connection: &Connection,
    form_id: i64,
) -> Result<Vec<VersionGroupRef>, String> {
    let mut statement = connection
        .prepare(
            "SELECT vg.id, vg.slug, vg.generation_id, vg.order_index
               FROM form_move fm
               JOIN version_group vg ON vg.id = fm.version_group_id
              WHERE fm.form_id = ?1
              GROUP BY vg.id
              ORDER BY vg.order_index DESC",
        )
        .map_err(db_error)?;

    statement
        .query_map([form_id], |row| {
            Ok(VersionGroupRef {
                id: row.get(0)?,
                slug: row.get(1)?,
                generation_id: get_u16(row, 2)?,
                order_index: get_u16(row, 3)?,
            })
        })
        .map_err(db_error)?
        .collect::<rusqlite::Result<Vec<_>>>()
        .map_err(db_error)
}

fn load_learnset_entries(
    connection: &Connection,
    form_id: i64,
    version_group_id: i64,
    lang: &str,
) -> Result<Vec<LearnsetEntry>, String> {
    let mut statement = connection
        .prepare(
            "SELECT m.id,
                    m.slug,
                    COALESCE(t.value, t_en.value, m.slug) AS name,
                    pt.slug,
                    m.damage_class,
                    m.power,
                    m.accuracy,
                    m.pp,
                    fm.method_slug,
                    fm.level,
                    fm.machine_slug
               FROM form_move fm
               JOIN move_ m ON m.id = fm.move_id
               JOIN poke_type pt ON pt.id = m.type_id
               LEFT JOIN translation t
                      ON t.entity_type = 'move'
                     AND t.entity_id = m.id
                     AND t.lang = ?3
                     AND t.field = 'name'
               LEFT JOIN translation t_en
                      ON t_en.entity_type = 'move'
                     AND t_en.entity_id = m.id
                     AND t_en.lang = 'en'
                     AND t_en.field = 'name'
              WHERE fm.form_id = ?1
                AND fm.version_group_id = ?2
              ORDER BY CASE fm.method_slug
                         WHEN 'level-up' THEN 0
                         WHEN 'machine' THEN 1
                         WHEN 'egg' THEN 2
                         WHEN 'tutor' THEN 3
                         ELSE 4
                       END,
                       fm.level,
                       name COLLATE NOCASE",
        )
        .map_err(db_error)?;

    statement
        .query_map(params![form_id, version_group_id, lang], |row| {
            Ok(LearnsetEntry {
                move_id: row.get(0)?,
                slug: row.get(1)?,
                name: row.get(2)?,
                type_slug: row.get(3)?,
                damage_class: row.get(4)?,
                power: get_optional_u16(row, 5)?,
                accuracy: get_optional_u16(row, 6)?,
                pp: get_optional_u16(row, 7)?,
                method: row.get(8)?,
                level: get_u16(row, 9)?,
                machine: row.get::<_, Option<String>>(10)?,
            })
        })
        .map_err(db_error)?
        .collect::<rusqlite::Result<Vec<_>>>()
        .map_err(db_error)
}
