use pdx_app::LocationRepository;
use pdx_domain::{EncounterSummary, LocationAreaSummary, LocationDetail, LocationSummary, NatureSummary};
use rusqlite::{OptionalExtension, params};

use super::repository::SqliteSpeciesRepository;

impl LocationRepository for SqliteSpeciesRepository {
    fn list_locations(
        &self,
        search: Option<&str>,
        lang: &str,
        limit: u32,
        offset: u32,
    ) -> Result<Vec<LocationSummary>, String> {
        let mut sql = String::from(
            "SELECT l.id, l.slug, COALESCE(t.value, t_en.value, l.slug) AS name, l.region_slug
               FROM location l
               LEFT JOIN translation t ON t.entity_type='location' AND t.entity_id=l.id AND t.lang=?1 AND t.field='name'
               LEFT JOIN translation t_en ON t_en.entity_type='location' AND t_en.entity_id=l.id AND t_en.lang='en' AND t.field='name'
               WHERE 1 = 1",
        );
        let mut params: Vec<rusqlite::types::Value> = vec![rusqlite::types::Value::Text(lang.to_owned())];
        if let Some(search) = search {
            sql.push_str(" AND (COALESCE(t.value, t_en.value, l.slug) LIKE ? OR l.slug LIKE ?)");
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
                Ok(LocationSummary {
                    id: row.get(0)?,
                    slug: row.get(1)?,
                    name: row.get(2)?,
                    region_slug: row.get(3)?,
                })
            })
            .map_err(super::repository::db_error)?
            .collect::<rusqlite::Result<Vec<_>>>()
            .map_err(super::repository::db_error)?;
        Ok(rows)
    }

    fn location_detail(&self, id: i64, lang: &str) -> Result<Option<LocationDetail>, String> {
        let location: Option<(i64, String, Option<String>)> = self
            .connection
            .query_row(
                "SELECT l.id, l.slug, l.region_slug FROM location l WHERE l.id = ?1",
                [id],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
            )
            .optional()
            .map_err(super::repository::db_error)?;
        let Some((id, slug, region_slug)) = location else {
            return Ok(None);
        };

        let name: String = self
            .connection
            .query_row(
                "SELECT COALESCE(t.value, t_en.value, ?2) FROM location l
                  LEFT JOIN translation t ON t.entity_type='location' AND t.entity_id=l.id AND t.lang=?1 AND t.field='name'
                  LEFT JOIN translation t_en ON t_en.entity_type='location' AND t_en.entity_id=l.id AND t_en.lang='en' AND t.field='name'
                 WHERE l.id=?3",
                params![lang, &slug, id],
                |row| row.get(0),
            )
            .unwrap_or_else(|_| slug.clone());

        let mut area_stmt = self
            .connection
            .prepare("SELECT id, slug, game_index FROM location_area WHERE location_id = ?1 ORDER BY game_index, id")
            .map_err(super::repository::db_error)?;
        let areas: Vec<LocationAreaSummary> = area_stmt
            .query_map([id], |row| {
                Ok(LocationAreaSummary {
                    id: row.get(0)?,
                    slug: row.get(1)?,
                    location_slug: slug.clone(),
                    game_index: row.get(2)?,
                })
            })
            .map_err(super::repository::db_error)?
            .collect::<rusqlite::Result<Vec<_>>>()
            .map_err(super::repository::db_error)?;

        let area_ids: Vec<i64> = areas.iter().map(|area| area.id).collect();
        let mut encounters: Vec<EncounterSummary> = Vec::new();
        for area_id in area_ids {
            let mut stmt = self
                .connection
                .prepare(
                    "SELECT e.species_id, s.slug, COALESCE(ts.value, ts_en.value, s.slug), e.version_id, gv.slug, la.slug,
                            e.method_slug, e.min_level, e.max_level, e.chance, e.conditions_json
                       FROM encounter e
                       JOIN species s ON s.id = e.species_id
                       JOIN location_area la ON la.id = e.location_area_id
                       JOIN game_version gv ON gv.id = e.version_id
                       LEFT JOIN translation ts ON ts.entity_type='species' AND ts.entity_id=s.id AND ts.lang=?1 AND ts.field='name'
                       LEFT JOIN translation ts_en ON ts_en.entity_type='species' AND ts_en.entity_id=s.id AND ts_en.lang='en' AND ts.field='name'
                      WHERE e.location_area_id = ?2",
                )
                .map_err(super::repository::db_error)?;
            let rows = stmt
                .query_map(params![lang, area_id], |row| {
                    let conditions_raw: Option<String> = row.get(10)?;
                    let conditions = conditions_raw
                        .and_then(|raw| serde_json::from_str::<Vec<String>>(&raw).ok())
                        .unwrap_or_default();
                    Ok(EncounterSummary {
                        species_id: row.get(0)?,
                        species_slug: row.get(1)?,
                        species_name: row.get(2)?,
                        version_slug: row.get(4)?,
                        location_area_slug: row.get(5)?,
                        method_slug: row.get(6)?,
                        min_level: row.get(7)?,
                        max_level: row.get(8)?,
                        chance: row.get(9)?,
                        conditions,
                    })
                })
                .map_err(super::repository::db_error)?
                .collect::<rusqlite::Result<Vec<_>>>()
                .map_err(super::repository::db_error)?;
            encounters.extend(rows);
        }

        Ok(Some(LocationDetail {
            location: LocationSummary {
                id,
                slug,
                name,
                region_slug,
            },
            areas,
            encounters,
        }))
    }

    fn encounters_for_species(
        &self,
        species_id: i64,
        lang: &str,
    ) -> Result<Vec<EncounterSummary>, String> {
        let mut stmt = self.connection.prepare(
            "SELECT e.species_id, s.slug, COALESCE(ts.value, ts_en.value, s.slug), e.version_id, gv.slug, la.slug,
                    e.method_slug, e.min_level, e.max_level, e.chance, e.conditions_json
               FROM encounter e
               JOIN species s ON s.id = e.species_id
               JOIN location_area la ON la.id = e.location_area_id
               JOIN game_version gv ON gv.id = e.version_id
               LEFT JOIN translation ts ON ts.entity_type='species' AND ts.entity_id=s.id AND ts.lang=?1 AND ts.field='name'
               LEFT JOIN translation ts_en ON ts_en.entity_type='species' AND ts_en.entity_id=s.id AND ts_en.lang='en' AND ts.field='name'
              WHERE e.species_id = ?2
              ORDER BY gv.slug, la.slug",
        ).map_err(super::repository::db_error)?;
        let rows = stmt
            .query_map(params![lang, species_id], |row| {
                let conditions_raw: Option<String> = row.get(10)?;
                let conditions = conditions_raw
                    .and_then(|raw| serde_json::from_str::<Vec<String>>(&raw).ok())
                    .unwrap_or_default();
                Ok(EncounterSummary {
                    species_id: row.get(0)?,
                    species_slug: row.get(1)?,
                    species_name: row.get(2)?,
                    version_slug: row.get(4)?,
                    location_area_slug: row.get(5)?,
                    method_slug: row.get(6)?,
                    min_level: row.get(7)?,
                    max_level: row.get(8)?,
                    chance: row.get(9)?,
                    conditions,
                })
            })
            .map_err(super::repository::db_error)?
            .collect::<rusqlite::Result<Vec<_>>>()
            .map_err(super::repository::db_error)?;
        Ok(rows)
    }

    fn list_natures(&self, lang: &str) -> Result<Vec<NatureSummary>, String> {
        let mut stmt = self.connection.prepare(
            "SELECT n.id, n.slug, COALESCE(t.value, t_en.value, n.slug) AS name, n.increased_stat_slug, n.decreased_stat_slug
               FROM nature n
               LEFT JOIN translation t ON t.entity_type='nature' AND t.entity_id=n.id AND t.lang=?1 AND t.field='name'
               LEFT JOIN translation t_en ON t_en.entity_type='nature' AND t_en.entity_id=n.id AND t_en.lang='en' AND t.field='name'
              ORDER BY name",
        ).map_err(super::repository::db_error)?;
        let rows = stmt
            .query_map([lang], |row| {
                Ok(NatureSummary {
                    id: row.get(0)?,
                    slug: row.get(1)?,
                    name: row.get(2)?,
                    increased_stat_slug: row.get(3)?,
                    decreased_stat_slug: row.get(4)?,
                })
            })
            .map_err(super::repository::db_error)?
            .collect::<rusqlite::Result<Vec<_>>>()
            .map_err(super::repository::db_error)?;
        Ok(rows)
    }
}
