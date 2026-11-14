use pdx_domain::{LearnsetPage, MoveDetail, MoveListQuery, MovePage, MoveSortCriterion};

/// Persistence port consumed by the move catalogue use cases.
pub trait MoveRepository {
    fn list_moves(&self, query: &MoveListQuery) -> Result<MovePage, String>;
    fn move_detail(&self, id: i64, lang: &str) -> Result<Option<MoveDetail>, String>;
    fn learnset(
        &self,
        form_id: i64,
        lang: &str,
        version_group: Option<&str>,
    ) -> Result<LearnsetPage, String>;
}

/// Page size used when the caller does not provide one.
pub const DEFAULT_MOVE_PAGE_SIZE: u32 = 80;

/// Upper bound accepted for a single move page.
pub const MAX_MOVE_PAGE_SIZE: u32 = 500;

const MAX_FILTER_TYPES: usize = 18;
const MAX_SORT_CRITERIA: usize = 2;

const DAMAGE_CLASSES: &[&str] = &["physical", "special", "status"];

/// Lists moves with filters, sort and pagination.
pub fn list_moves(
    repository: &dyn MoveRepository,
    query: &MoveListQuery,
) -> Result<MovePage, String> {
    let mut query = query.clone();

    query.search = query
        .search
        .as_deref()
        .map(super::species::sanitize_query)
        .filter(|value| !value.is_empty());
    query.types = super::species::sanitize_slugs(query.types, MAX_FILTER_TYPES);
    query.damage_classes = sanitize_damage_classes(query.damage_classes);
    query.generations = super::species::sanitize_generations(query.generations);
    if let (Some(min), Some(max)) = (query.min_power, query.max_power)
        && min > max
    {
        query.min_power = Some(max);
        query.max_power = Some(min);
    }
    query.sort = sanitize_sort(query.sort);
    query.lang = super::species::normalize_language(&query.lang).to_owned();
    query.limit = if query.limit == 0 {
        DEFAULT_MOVE_PAGE_SIZE
    } else {
        query.limit.min(MAX_MOVE_PAGE_SIZE)
    };

    repository.list_moves(&query)
}

/// Loads one move with its localized effect and its learners.
pub fn move_detail(
    repository: &dyn MoveRepository,
    id: i64,
    lang: &str,
) -> Result<Option<MoveDetail>, String> {
    if id <= 0 {
        return Ok(None);
    }

    repository.move_detail(id, super::species::normalize_language(lang))
}

/// Loads the learnset of one form, optionally for a specific version group.
pub fn learnset(
    repository: &dyn MoveRepository,
    form_id: i64,
    lang: &str,
    version_group: Option<&str>,
) -> Result<LearnsetPage, String> {
    if form_id <= 0 {
        return Ok(LearnsetPage {
            form_id,
            version_group: None,
            version_groups: Vec::new(),
            entries: Vec::new(),
            total: 0,
        });
    }

    let version_group = version_group
        .map(str::trim)
        .filter(|value| !value.is_empty() && value.len() <= 60);

    repository.learnset(
        form_id,
        super::species::normalize_language(lang),
        version_group,
    )
}

fn sanitize_damage_classes(values: Vec<String>) -> Vec<String> {
    let mut sanitized: Vec<String> = Vec::with_capacity(values.len());

    for value in values {
        let slug = value.trim().to_lowercase();
        if DAMAGE_CLASSES.contains(&slug.as_str()) && !sanitized.contains(&slug) {
            sanitized.push(slug);
        }
    }

    sanitized
}

fn sanitize_sort(values: Vec<MoveSortCriterion>) -> Vec<MoveSortCriterion> {
    let mut sanitized: Vec<MoveSortCriterion> = Vec::with_capacity(values.len().min(MAX_SORT_CRITERIA));

    for criterion in values {
        if !sanitized
            .iter()
            .any(|entry| entry.field == criterion.field)
        {
            sanitized.push(criterion);
        }
        if sanitized.len() == MAX_SORT_CRITERIA {
            break;
        }
    }

    sanitized
}

#[cfg(test)]
mod tests {
    use pdx_domain::{MoveListQuery, MovePage, MoveSortCriterion, MoveSortField, SortDirection};

    use super::{DEFAULT_MOVE_PAGE_SIZE, MoveRepository, list_moves};

    struct FakeRepository;

    impl MoveRepository for FakeRepository {
        fn list_moves(&self, query: &MoveListQuery) -> Result<MovePage, String> {
            assert_eq!(query.search.as_deref(), Some("flam"));
            assert_eq!(query.types, vec!["fire"]);
            assert_eq!(query.damage_classes, vec!["special"]);
            assert_eq!(query.generations, vec![1]);
            assert_eq!(query.min_power, Some(40));
            assert_eq!(query.max_power, Some(120));
            assert_eq!(query.limit, DEFAULT_MOVE_PAGE_SIZE);
            assert_eq!(query.lang, "fr");

            Ok(MovePage {
                items: Vec::new(),
                total: 0,
                offset: 0,
                limit: query.limit,
            })
        }

        fn move_detail(
            &self,
            _id: i64,
            _lang: &str,
        ) -> Result<Option<pdx_domain::MoveDetail>, String> {
            Ok(None)
        }

        fn learnset(
            &self,
            form_id: i64,
            _lang: &str,
            _version_group: Option<&str>,
        ) -> Result<pdx_domain::LearnsetPage, String> {
            Ok(pdx_domain::LearnsetPage {
                form_id,
                version_group: None,
                version_groups: Vec::new(),
                entries: Vec::new(),
                total: 0,
            })
        }
    }

    #[test]
    fn sanitizes_the_move_query() {
        let query = MoveListQuery {
            search: Some("  flam  ".to_owned()),
            types: vec![" FIRE ".to_owned(), "fire".to_owned()],
            damage_classes: vec!["special".to_owned(), "nope".to_owned()],
            generations: vec![1, 1, 99],
            min_power: Some(120),
            max_power: Some(40),
            sort: vec![
                MoveSortCriterion {
                    field: MoveSortField::Power,
                    direction: SortDirection::Desc,
                },
                MoveSortCriterion {
                    field: MoveSortField::Power,
                    direction: SortDirection::Asc,
                },
            ],
            lang: "fr".to_owned(),
            ..MoveListQuery::default()
        };

        list_moves(&FakeRepository, &query).expect("list should succeed");
    }
}
