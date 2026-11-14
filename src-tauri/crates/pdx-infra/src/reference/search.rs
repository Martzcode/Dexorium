//! Federated search across species, moves and abilities.
//!
//! The FTS5 index answers prefix queries; a Levenshtein fallback tolerates
//! typos when the index returns too few hits. Both paths resolve localized
//! display names, so the command palette can render results directly.

use std::collections::HashSet;

use pdx_app::SearchRepository;
use pdx_domain::{GlobalSearchResult, SearchEntityType, SearchHit};
use rusqlite::{Connection, OptionalExtension, params};

use super::repository::{
    SqliteSpeciesRepository, db_error, fts_query, normalize, translation_value,
};

impl SearchRepository for SqliteSpeciesRepository {
    fn search_global(
        &self,
        query: &str,
        lang: &str,
        limit: usize,
    ) -> Result<GlobalSearchResult, String> {
        let mut hits: Vec<SearchHit> = Vec::with_capacity(limit);
        let mut seen: HashSet<(String, i64)> = HashSet::new();

        if let Some(fts) = fts_query(query) {
            let mut statement = self
                .connection
                .prepare(
                    "SELECT entity_type, CAST(entity_id AS INTEGER)
                       FROM search_index
                      WHERE search_index MATCH ?1
                        AND entity_type IN ('species', 'move', 'ability')
                      ORDER BY bm25(search_index)
                      LIMIT ?2",
                )
                .map_err(db_error)?;

            let rows = statement
                .query_map(
                    params![fts, i64::try_from(limit).unwrap_or(i64::MAX)],
                    |row| Ok((row.get::<_, String>(0)?, row.get::<_, i64>(1)?)),
                )
                .map_err(db_error)?
                .collect::<rusqlite::Result<Vec<_>>>()
                .map_err(db_error)?;

            for (index, (entity_type, id)) in rows.into_iter().enumerate() {
                let Some(entity_type) = parse_entity_type(&entity_type) else {
                    continue;
                };

                let score = 1_000_u32.saturating_sub(
                    u32::try_from(index).unwrap_or(u32::MAX).saturating_mul(10),
                );
                if let Some(hit) = resolve_hit(&self.connection, entity_type, id, lang, score)? {
                    seen.insert((entity_type.as_str().to_owned(), id));
                    hits.push(hit);
                }
            }
        }

        if hits.len() < limit {
            for (entity_type, id, distance) in fuzzy_candidates(&self.connection, query, lang)? {
                if hits.len() >= limit {
                    break;
                }
                if !seen.insert((entity_type.as_str().to_owned(), id)) {
                    continue;
                }

                let score = 600_u32.saturating_sub(
                    u32::try_from(distance).unwrap_or(u32::MAX).saturating_mul(100),
                );
                if let Some(hit) = resolve_hit(&self.connection, entity_type, id, lang, score)? {
                    hits.push(hit);
                }
            }
        }

        hits.truncate(limit);

        Ok(GlobalSearchResult {
            query: query.to_owned(),
            hits,
        })
    }
}

fn parse_entity_type(value: &str) -> Option<SearchEntityType> {
    match value {
        "species" => Some(SearchEntityType::Species),
        "move" => Some(SearchEntityType::Move),
        "ability" => Some(SearchEntityType::Ability),
        _ => None,
    }
}

fn resolve_hit(
    connection: &Connection,
    entity_type: SearchEntityType,
    id: i64,
    lang: &str,
    score: u32,
) -> Result<Option<SearchHit>, String> {
    match entity_type {
        SearchEntityType::Species => resolve_species(connection, id, lang, score),
        SearchEntityType::Move => resolve_move(connection, id, lang, score),
        SearchEntityType::Ability => resolve_ability(connection, id, lang, score),
    }
}

fn resolve_species(
    connection: &Connection,
    id: i64,
    lang: &str,
    score: u32,
) -> Result<Option<SearchHit>, String> {
    let species: Option<(String, u16)> = connection
        .query_row(
            "SELECT slug, national_dex_number FROM species WHERE id = ?1",
            [id],
            |row| Ok((row.get(0)?, row.get::<_, i64>(1)? as u16)),
        )
        .optional()
        .map_err(db_error)?;

    let Some((slug, national_dex_number)) = species else {
        return Ok(None);
    };

    let name =
        translation_value(connection, "species", id, lang, "name")?.unwrap_or_else(|| slug.clone());
    let types = load_species_types(connection, id, lang)?;

    let subtitle = if types.is_empty() {
        format!("#{national_dex_number:04}")
    } else {
        format!("#{national_dex_number:04} · {}", types.join(" / "))
    };

    Ok(Some(SearchHit {
        entity_type: SearchEntityType::Species,
        id,
        slug,
        name,
        subtitle,
        score,
    }))
}

fn load_species_types(
    connection: &Connection,
    species_id: i64,
    lang: &str,
) -> Result<Vec<String>, String> {
    let mut statement = connection
        .prepare(
            "SELECT COALESCE(t.value, t_en.value, pt.slug)
               FROM form_type ft
               JOIN poke_type pt ON pt.id = ft.type_id
               LEFT JOIN translation t
                      ON t.entity_type = 'type'
                     AND t.entity_id = pt.id
                     AND t.lang = ?2
                     AND t.field = 'name'
               LEFT JOIN translation t_en
                      ON t_en.entity_type = 'type'
                     AND t_en.entity_id = pt.id
                     AND t_en.lang = 'en'
                     AND t_en.field = 'name'
              WHERE ft.form_id = (
                      SELECT f.id
                        FROM pokemon_form f
                       WHERE f.species_id = ?1
                       ORDER BY f.is_default DESC, f.form_order
                       LIMIT 1
                    )
              ORDER BY ft.slot",
        )
        .map_err(db_error)?;

    statement
        .query_map(params![species_id, lang], |row| row.get(0))
        .map_err(db_error)?
        .collect::<rusqlite::Result<Vec<_>>>()
        .map_err(db_error)
}

fn resolve_move(
    connection: &Connection,
    id: i64,
    lang: &str,
    score: u32,
) -> Result<Option<SearchHit>, String> {
    let move_row: Option<(String, String, String)> = connection
        .query_row(
            "SELECT m.slug, pt.slug, m.damage_class
               FROM move_ m
               JOIN poke_type pt ON pt.id = m.type_id
              WHERE m.id = ?1",
            [id],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
        )
        .optional()
        .map_err(db_error)?;

    let Some((slug, type_slug, damage_class)) = move_row else {
        return Ok(None);
    };

    let name =
        translation_value(connection, "move", id, lang, "name")?.unwrap_or_else(|| slug.clone());
    let type_label = translation_value(connection, "type", type_id(connection, &type_slug)?, lang, "name")?
        .unwrap_or_else(|| type_slug.clone());

    Ok(Some(SearchHit {
        entity_type: SearchEntityType::Move,
        id,
        slug,
        name,
        subtitle: format!("{type_label} · {damage_class}"),
        score,
    }))
}

fn type_id(connection: &Connection, slug: &str) -> Result<i64, String> {
    connection
        .query_row("SELECT id FROM poke_type WHERE slug = ?1", [slug], |row| {
            row.get(0)
        })
        .map_err(db_error)
}

fn resolve_ability(
    connection: &Connection,
    id: i64,
    lang: &str,
    score: u32,
) -> Result<Option<SearchHit>, String> {
    let ability: Option<(String, u16)> = connection
        .query_row(
            "SELECT slug, generation_id FROM ability WHERE id = ?1",
            [id],
            |row| Ok((row.get(0)?, row.get::<_, i64>(1)? as u16)),
        )
        .optional()
        .map_err(db_error)?;

    let Some((slug, generation_id)) = ability else {
        return Ok(None);
    };

    let name = translation_value(connection, "ability", id, lang, "name")?
        .unwrap_or_else(|| slug.clone());

    Ok(Some(SearchHit {
        entity_type: SearchEntityType::Ability,
        id,
        slug,
        name,
        subtitle: format!("Gen. {generation_id}"),
        score,
    }))
}

/// Returns fuzzy candidates ordered by edit distance, then name length.
fn fuzzy_candidates(
    connection: &Connection,
    query: &str,
    lang: &str,
) -> Result<Vec<(SearchEntityType, i64, usize)>, String> {
    let normalized = normalize(query.trim());
    if normalized.is_empty() {
        return Ok(Vec::new());
    }

    let mut statement = connection
        .prepare(
            "SELECT entity_type, entity_id, value
               FROM translation
              WHERE lang = ?1
                AND field = 'name'
                AND entity_type IN ('species', 'move', 'ability')",
        )
        .map_err(db_error)?;

    let rows = statement
        .query_map([lang], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, i64>(1)?,
                row.get::<_, String>(2)?,
            ))
        })
        .map_err(db_error)?
        .collect::<rusqlite::Result<Vec<_>>>()
        .map_err(db_error)?;

    let mut candidates: Vec<(SearchEntityType, i64, usize, usize)> = Vec::new();

    for (entity_type, id, name) in rows {
        let Some(entity_type) = parse_entity_type(&entity_type) else {
            continue;
        };

        let normalized_name = normalize(&name);
        let mut distance = levenshtein(&normalized, &normalized_name);
        for word in normalized_name.split_whitespace() {
            distance = distance.min(levenshtein(&normalized, word));
        }

        if distance <= 2 {
            candidates.push((entity_type, id, distance, normalized_name.len()));
        }
    }

    candidates.sort_by_key(|(entity_type, id, distance, length)| {
        (*distance, *length, entity_type.as_str(), *id)
    });

    Ok(candidates
        .into_iter()
        .map(|(entity_type, id, distance, _)| (entity_type, id, distance))
        .collect())
}

fn levenshtein(left: &str, right: &str) -> usize {
    let left: Vec<char> = left.chars().collect();
    let right: Vec<char> = right.chars().collect();

    if left.len().abs_diff(right.len()) > 2 {
        return usize::MAX / 2;
    }

    let mut previous: Vec<usize> = (0..=right.len()).collect();

    for (left_index, left_character) in left.iter().enumerate() {
        let mut current = vec![left_index + 1; right.len() + 1];
        for (right_index, right_character) in right.iter().enumerate() {
            let substitution = previous[right_index] + usize::from(left_character != right_character);
            current[right_index + 1] = (current[right_index] + 1)
                .min(previous[right_index + 1] + 1)
                .min(substitution);
        }
        previous = current;
    }

    previous[right.len()]
}

#[cfg(test)]
mod tests {
    use super::levenshtein;

    #[test]
    fn computes_edit_distances() {
        assert_eq!(levenshtein("pikachu", "pikachu"), 0);
        assert_eq!(levenshtein("pikachu", "pikach"), 1);
        assert_eq!(levenshtein("dracaufeu", "dracaufau"), 1);
        assert_eq!(levenshtein("charizard", "charizrd"), 1);
    }
}
