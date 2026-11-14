//! Type effectiveness chart adapter.

use pdx_app::TypeRepository;
use pdx_domain::{TypeMatrix, TypeRef};
use rusqlite::params;

use super::repository::{SqliteSpeciesRepository, db_error};

impl TypeRepository for SqliteSpeciesRepository {
    fn type_matrix(&self, generation: u8, lang: &str) -> Result<TypeMatrix, String> {
        let types = load_types(&self.connection, lang)?;
        let generations = load_generations(&self.connection)?;
        let mut multipliers = vec![vec![100_u16; types.len()]; types.len()];

        let mut statement = self
            .connection
            .prepare(
                "SELECT at.slug, dt.slug, te.multiplier_x100
                   FROM type_efficacy te
                   JOIN poke_type at ON at.id = te.attacking_type
                   JOIN poke_type dt ON dt.id = te.defending_type
                  WHERE te.generation_id = ?1",
            )
            .map_err(db_error)?;

        let rows = statement
            .query_map(params![i64::from(generation)], |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, i64>(2)?,
                ))
            })
            .map_err(db_error)?;

        for row in rows {
            let (attacking, defending, value) = row.map_err(db_error)?;
            let Some(attacker) = types.iter().position(|item| item.slug == attacking) else {
                continue;
            };
            let Some(defender) = types.iter().position(|item| item.slug == defending) else {
                continue;
            };
            multipliers[attacker][defender] = u16::try_from(value).unwrap_or(100);
        }

        Ok(TypeMatrix {
            generation,
            generations,
            types,
            multipliers,
        })
    }
}

fn load_types(
    connection: &rusqlite::Connection,
    lang: &str,
) -> Result<Vec<TypeRef>, String> {
    let mut statement = connection
        .prepare(
            "SELECT pt.slug,
                    COALESCE(t.value, t_en.value, pt.slug) AS name,
                    pt.introduced_in
               FROM poke_type pt
               LEFT JOIN translation t
                      ON t.entity_type = 'type'
                     AND t.entity_id = pt.id
                     AND t.lang = ?1
                     AND t.field = 'name'
               LEFT JOIN translation t_en
                      ON t_en.entity_type = 'type'
                     AND t_en.entity_id = pt.id
                     AND t_en.lang = 'en'
                     AND t_en.field = 'name'
              WHERE pt.id < 10000
                AND pt.slug <> 'stellar'
              ORDER BY pt.id",
        )
        .map_err(db_error)?;

    let rows = statement
        .query_map([lang], |row| {
            Ok(TypeRef {
                slug: row.get(0)?,
                name: row.get(1)?,
                introduced_in: row.get(2)?,
            })
        })
        .map_err(db_error)?;

    rows.collect::<rusqlite::Result<Vec<_>>>().map_err(db_error)
}

fn load_generations(connection: &rusqlite::Connection) -> Result<Vec<u8>, String> {
    let mut statement = connection
        .prepare("SELECT id FROM generation ORDER BY id DESC")
        .map_err(db_error)?;

    let rows = statement
        .query_map([], |row| row.get::<_, i64>(0))
        .map_err(db_error)?;

    rows.map(|row| {
        row.map_err(db_error)
            .and_then(|id| u8::try_from(id).map_err(|_| "invalid generation id".to_owned()))
    })
    .collect()
}

#[cfg(test)]
mod tests {
    use pdx_app::TypeRepository;

    use super::SqliteSpeciesRepository;

    #[test]
    fn fixture_exposes_the_eighteen_types_with_a_neutral_chart() {
        let repository =
            SqliteSpeciesRepository::from_fixture().expect("fixture database should build");

        let matrix = repository
            .type_matrix(9, "en")
            .expect("type matrix should build");

        assert_eq!(matrix.generation, 9);
        assert_eq!(matrix.generations, (1..=9).rev().collect::<Vec<u8>>());
        assert_eq!(matrix.types.len(), 18);
        assert!(matrix.types.iter().all(|item| item.slug != "stellar"));
        assert!(
            matrix
                .types
                .iter()
                .any(|item| item.slug == "fairy" && item.introduced_in == 6)
        );
        assert!(matrix.types.iter().all(|item| item.name == item.slug));
        assert!(matrix.multipliers.iter().flatten().all(|value| *value == 100));
    }

    #[test]
    fn generated_database_matches_the_reference_chart() {
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("..")
            .join("..")
            .join("resources")
            .join("reference.db");
        if !path.is_file() {
            return;
        }

        let repository = SqliteSpeciesRepository::open(&path).expect("database should open");
        let matrix = repository
            .type_matrix(9, "fr")
            .expect("type matrix should build");

        assert_eq!(matrix.types.len(), 18);
        assert!(
            matrix
                .types
                .iter()
                .any(|item| item.slug == "fire" && item.name == "Feu")
        );
        assert_eq!(matrix.multiplier("fire", "grass"), Some(200));
        assert_eq!(matrix.multiplier("fire", "water"), Some(50));
        assert_eq!(matrix.multiplier("normal", "ghost"), Some(0));
        assert_eq!(matrix.multiplier("electric", "ground"), Some(0));
        assert_eq!(matrix.multiplier("water", "fire"), Some(200));
        assert_eq!(matrix.multiplier("fire", "fire"), Some(50));

        let old = repository
            .type_matrix(1, "fr")
            .expect("generation 1 matrix should build");
        assert_eq!(old.multiplier("dark", "psychic"), Some(100));
        // The Generation I ghost/psychic quirk is preserved by the dataset.
        assert_eq!(old.multiplier("ghost", "psychic"), Some(0));
    }
}
