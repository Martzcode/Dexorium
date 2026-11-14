use pdx_domain::{AbilityDetail, AbilityListQuery, AbilityPage};

/// Persistence port consumed by the ability catalogue use cases.
pub trait AbilityRepository {
    fn list_abilities(&self, query: &AbilityListQuery) -> Result<AbilityPage, String>;
    fn ability_detail(&self, id: i64, lang: &str) -> Result<Option<AbilityDetail>, String>;
}

/// Page size used when the caller does not provide one.
pub const DEFAULT_ABILITY_PAGE_SIZE: u32 = 80;

/// Upper bound accepted for a single ability page.
pub const MAX_ABILITY_PAGE_SIZE: u32 = 500;

/// Lists abilities with search, generation filter and pagination.
pub fn list_abilities(
    repository: &dyn AbilityRepository,
    query: &AbilityListQuery,
) -> Result<AbilityPage, String> {
    let mut query = query.clone();

    query.search = query
        .search
        .as_deref()
        .map(super::species::sanitize_query)
        .filter(|value| !value.is_empty());
    query.generations = super::species::sanitize_generations(query.generations);
    query.lang = super::species::normalize_language(&query.lang).to_owned();
    query.limit = if query.limit == 0 {
        DEFAULT_ABILITY_PAGE_SIZE
    } else {
        query.limit.min(MAX_ABILITY_PAGE_SIZE)
    };

    repository.list_abilities(&query)
}

/// Loads one ability with its localized effect and its holders.
pub fn ability_detail(
    repository: &dyn AbilityRepository,
    id: i64,
    lang: &str,
) -> Result<Option<AbilityDetail>, String> {
    if id <= 0 {
        return Ok(None);
    }

    repository.ability_detail(id, super::species::normalize_language(lang))
}
