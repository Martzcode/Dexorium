use pdx_app::ItemRepository;
use pdx_domain::{BerryDetail, ItemDetail, ItemSummary};
use rusqlite::{OptionalExtension, params};

use super::repository::SqliteSpeciesRepository;

impl ItemRepository for SqliteSpeciesRepository {
    fn list_items(
        &self,
        search: Option<&str>,
        category: Option<&str>,
        lang: &str,
        limit: u32,
        offset: u32,
    ) -> Result<Vec<ItemSummary>, String> {
        let mut sql = String::from(
            "SELECT i.id, i.slug,
                    COALESCE(t.value, t_en.value, i.slug) AS name,
                    ic.slug, i.pocket_slug, i.cost, i.is_berry, i.fling_power
               FROM item i
               LEFT JOIN item_category ic ON ic.id = i.category_id
               LEFT JOIN translation t
                      ON t.entity_type = 'item' AND t.entity_id = i.id
                     AND t.lang = ?1 AND t.field = 'name'
               LEFT JOIN translation t_en
                      ON t_en.entity_type = 'item' AND t_en.entity_id = i.id
                     AND t_en.lang = 'en' AND t_en.field = 'name'
               WHERE 1 = 1",
        );
        let mut params: Vec<rusqlite::types::Value> = vec![rusqlite::types::Value::Text(lang.to_owned())];

        if let Some(category) = category {
            sql.push_str(" AND ic.slug = ?");
            params.push(rusqlite::types::Value::Text(category.to_owned()));
        }
        if let Some(search) = search {
            sql.push_str(" AND (COALESCE(t.value, t_en.value, i.slug) LIKE ? OR i.slug LIKE ?)");
            let needle = format!("%{}%", search.to_lowercase());
            params.push(rusqlite::types::Value::Text(needle.clone()));
            params.push(rusqlite::types::Value::Text(needle));
        }
        sql.push_str(" ORDER BY name LIMIT ? OFFSET ?");
        params.push(rusqlite::types::Value::Integer(i64::from(limit)));
        params.push(rusqlite::types::Value::Integer(i64::from(offset)));

        let mut statement = self.connection.prepare(&sql).map_err(super::repository::db_error)?;
        let rows = statement
            .query_map(rusqlite::params_from_iter(params.iter()), |row| {
                Ok(ItemSummary {
                    id: row.get(0)?,
                    slug: row.get(1)?,
                    name: row.get(2)?,
                    category_slug: row.get(3)?,
                    pocket_slug: row.get(4)?,
                    cost: row.get(5)?,
                    is_berry: row.get::<_, i64>(6)? == 1,
                    fling_power: row.get(7)?,
                })
            })
            .map_err(super::repository::db_error)?
            .collect::<rusqlite::Result<Vec<_>>>()
            .map_err(super::repository::db_error)?;
        Ok(rows)
    }

    #[allow(clippy::type_complexity)]
    fn item_detail(&self, id: i64, lang: &str) -> Result<Option<ItemDetail>, String> {
        let row: Option<(i64, String, Option<String>, Option<String>, Option<i64>, i64, Option<i64>, Option<String>)> =
            self.connection
                .query_row(
                    "SELECT i.id, i.slug, ic.slug, i.pocket_slug, i.cost, i.is_berry, i.fling_power, i.fling_effect_slug
                       FROM item i LEFT JOIN item_category ic ON ic.id = i.category_id WHERE i.id = ?1",
                    [id],
                    |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?, row.get(4)?, row.get(5)?, row.get(6)?, row.get(7)?)),
                )
                .optional()
                .map_err(super::repository::db_error)?;
        let Some((id, slug, category_slug, pocket_slug, cost, is_berry, fling_power, fling_effect_slug)) = row else {
            return Ok(None);
        };

        let name: String = self
            .connection
            .query_row(
                "SELECT COALESCE(t.value, t_en.value, ?2) FROM item i
                  LEFT JOIN translation t ON t.entity_type='item' AND t.entity_id=i.id AND t.lang=?1 AND t.field='name'
                  LEFT JOIN translation t_en ON t_en.entity_type='item' AND t_en.entity_id=i.id AND t_en.lang='en' AND t_en.field='name'
                 WHERE i.id=?3",
                params![lang, &slug, id],
                |row| row.get(0),
            )
            .unwrap_or_else(|_| slug.clone());

        let effect = self.connection
            .query_row(
                "SELECT t.value FROM translation t WHERE t.entity_type='item' AND t.entity_id=?1 AND t.lang=?2 AND t.field='effect'",
                params![id, lang],
                |row| row.get(0),
            )
            .optional()
            .map_err(super::repository::db_error)?;
        let short_effect = self.connection
            .query_row(
                "SELECT t.value FROM translation t WHERE t.entity_type='item' AND t.entity_id=?1 AND t.lang=?2 AND t.field='short_effect'",
                params![id, lang],
                |row| row.get(0),
            )
            .optional()
            .map_err(super::repository::db_error)?;
        let description = self.connection
            .query_row(
                "SELECT t.value FROM translation t WHERE t.entity_type='item' AND t.entity_id=?1 AND t.lang=?2 AND t.field='description'",
                params![id, lang],
                |row| row.get(0),
            )
            .optional()
            .map_err(super::repository::db_error)?;

        let mut flag_stmt = self
            .connection
            .prepare("SELECT flag_slug FROM item_flag WHERE item_id = ?1")
            .map_err(super::repository::db_error)?;
        let flags: Vec<String> = flag_stmt
            .query_map([id], |row| row.get(0))
            .map_err(super::repository::db_error)?
            .collect::<rusqlite::Result<Vec<_>>>()
            .map_err(super::repository::db_error)?;

        let berry = self
            .connection
            .query_row(
                "SELECT firmness_slug, growth_time, max_harvest, natural_gift_power, size, smoothness, soil_dryness, natural_gift_type_id, flavors_json FROM berry WHERE item_id = ?1",
                [id],
                |row| {
                    let flavors_raw: Option<String> = row.get(8)?;
                    let flavors = flavors_raw
                        .and_then(|raw| serde_json::from_str::<serde_json::Value>(&raw).ok())
                        .and_then(|value| value.as_array().cloned())
                        .unwrap_or_default()
                        .iter()
                        .filter_map(|entry| {
                            Some((
                                entry.get("flavor")?.as_str()?.to_owned(),
                                entry.get("potency")?.as_i64()?,
                            ))
                        })
                        .collect();
                    let natural_gift_type_slug = row.get::<_, Option<i64>>(7)?
                        .and_then(|type_id| {
                            self.connection
                                .query_row("SELECT slug FROM poke_type WHERE id = ?1", [type_id], |r| r.get(0))
                                .ok()
                        });
                    Ok(BerryDetail {
                        firmness_slug: row.get(0)?,
                        growth_time: row.get(1)?,
                        max_harvest: row.get(2)?,
                        natural_gift_power: row.get(3)?,
                        size: row.get(4)?,
                        smoothness: row.get(5)?,
                        soil_dryness: row.get(6)?,
                        natural_gift_type_slug,
                        flavors,
                    })
                },
            )
            .optional()
            .map_err(super::repository::db_error)?;

        Ok(Some(ItemDetail {
            id,
            slug,
            name,
            category_slug,
            pocket_slug,
            cost,
            is_berry: is_berry == 1,
            fling_power,
            fling_effect_slug,
            flags,
            effect,
            short_effect,
            description,
            berry,
        }))
    }

    fn search_items(&self, query: &str, lang: &str, limit: usize) -> Result<Vec<ItemSummary>, String> {
        let limit = limit.clamp(1, 100);
        let mut statement = self.connection.prepare(
            "SELECT i.id, i.slug, COALESCE(t.value, t_en.value, i.slug) AS name, ic.slug, i.pocket_slug, i.cost, i.is_berry, i.fling_power
               FROM item i
               LEFT JOIN item_category ic ON ic.id = i.category_id
               LEFT JOIN translation t ON t.entity_type='item' AND t.entity_id=i.id AND t.lang=?1 AND t.field='name'
               LEFT JOIN translation t_en ON t_en.entity_type='item' AND t_en.entity_id=i.id AND t_en.lang='en' AND t.field='name'
              WHERE COALESCE(t.value, t_en.value, i.slug) LIKE ?2
              ORDER BY name
              LIMIT ?3",
        ).map_err(super::repository::db_error)?;
        let needle = format!("%{}%", query);
        let rows = statement
            .query_map(params![lang, needle, limit as i64], |row| {
                Ok(ItemSummary {
                    id: row.get(0)?,
                    slug: row.get(1)?,
                    name: row.get(2)?,
                    category_slug: row.get(3)?,
                    pocket_slug: row.get(4)?,
                    cost: row.get(5)?,
                    is_berry: row.get::<_, i64>(6)? == 1,
                    fling_power: row.get(7)?,
                })
            })
            .map_err(super::repository::db_error)?
            .collect::<rusqlite::Result<Vec<_>>>()
            .map_err(super::repository::db_error)?;
        Ok(rows)
    }
}

