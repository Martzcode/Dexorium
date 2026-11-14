use pdx_domain::{FavoriteCollection, FavoriteEntry, FavoriteKind, NationalDexNumber};

/// Persistence port for the user collection.
///
/// The MVP stores favourites for species only. The identifier is kept generic
/// so teams and notes can reuse the same table later.
pub trait FavoriteRepository {
    /// Returns every starred entity, newest first.
    fn favorite_entries(&self) -> Result<Vec<FavoriteEntry>, String>;

    /// Adds or removes one star. The call is idempotent.
    fn set_favorite(
        &self,
        kind: FavoriteKind,
        entity_id: i64,
        favorite: bool,
    ) -> Result<(), String>;
}

/// Lists the user's favourites.
pub fn list_favorites(repository: &dyn FavoriteRepository) -> Result<FavoriteCollection, String> {
    let items = repository.favorite_entries()?;

    Ok(FavoriteCollection {
        total: items.len(),
        items,
    })
}

/// Stars or unstars one entity and returns the updated collection.
pub fn set_favorite(
    repository: &dyn FavoriteRepository,
    kind: FavoriteKind,
    entity_id: i64,
    favorite: bool,
) -> Result<FavoriteCollection, String> {
    validate_target(kind, entity_id)?;
    repository.set_favorite(kind, entity_id, favorite)?;
    list_favorites(repository)
}

/// Returns the national dex numbers of the starred species.
///
/// The Pokédex list uses them to restrict its SQL query to the collection.
pub fn favorite_species_ids(repository: &dyn FavoriteRepository) -> Result<Vec<i64>, String> {
    Ok(repository
        .favorite_entries()?
        .into_iter()
        .filter(|entry| entry.kind == FavoriteKind::Species)
        .map(|entry| entry.entity_id)
        .collect())
}

fn validate_target(kind: FavoriteKind, entity_id: i64) -> Result<(), String> {
    match kind {
        FavoriteKind::Species => {
            let number = u16::try_from(entity_id)
                .ok()
                .and_then(NationalDexNumber::new);
            if number.is_none() {
                return Err(format!(
                    "species identifier {entity_id} is outside the supported national dex range"
                ));
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use std::cell::RefCell;

    use pdx_domain::FavoriteKind;

    use super::{FavoriteRepository, favorite_species_ids, list_favorites, set_favorite};

    #[derive(Default)]
    struct FakeFavorites {
        entries: RefCell<Vec<i64>>,
    }

    impl FavoriteRepository for FakeFavorites {
        fn favorite_entries(&self) -> Result<Vec<pdx_domain::FavoriteEntry>, String> {
            Ok(self
                .entries
                .borrow()
                .iter()
                .map(|id| pdx_domain::FavoriteEntry {
                    kind: FavoriteKind::Species,
                    entity_id: *id,
                    created_at: "2026-01-01T00:00:00Z".to_owned(),
                })
                .collect())
        }

        fn set_favorite(
            &self,
            _kind: FavoriteKind,
            entity_id: i64,
            favorite: bool,
        ) -> Result<(), String> {
            let mut entries = self.entries.borrow_mut();
            entries.retain(|id| *id != entity_id);
            if favorite {
                entries.insert(0, entity_id);
            }
            Ok(())
        }
    }

    #[test]
    fn stars_and_unstars_a_species() {
        let repository = FakeFavorites::default();

        let collection = set_favorite(&repository, FavoriteKind::Species, 25, true)
            .expect("starring should succeed");
        assert_eq!(collection.total, 1);
        assert_eq!(collection.items[0].entity_id, 25);
        assert_eq!(
            favorite_species_ids(&repository).expect("ids should be readable"),
            vec![25]
        );

        let collection = set_favorite(&repository, FavoriteKind::Species, 25, false)
            .expect("unstarring should succeed");
        assert!(collection.items.is_empty());
    }

    #[test]
    fn rejects_out_of_range_species() {
        let repository = FakeFavorites::default();

        assert!(set_favorite(&repository, FavoriteKind::Species, 0, true).is_err());
        assert!(set_favorite(&repository, FavoriteKind::Species, 1026, true).is_err());
        assert!(
            list_favorites(&repository)
                .expect("listing an empty collection should succeed")
                .items
                .is_empty()
        );
    }
}
