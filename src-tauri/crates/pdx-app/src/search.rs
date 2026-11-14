use pdx_domain::{GlobalSearchQuery, GlobalSearchResult, SearchHit};

/// Persistence port used by the federated search and the command palette.
pub trait SearchRepository {
    fn search_global(
        &self,
        query: &str,
        lang: &str,
        limit: usize,
    ) -> Result<GlobalSearchResult, String>;
}

/// Default number of hits returned by the search page.
pub const DEFAULT_SEARCH_HITS: usize = 20;

/// Upper bound accepted from the IPC boundary.
pub const MAX_SEARCH_HITS: usize = 60;

/// Default number of suggestions returned by the command palette.
pub const DEFAULT_SUGGESTIONS: usize = 8;

/// Runs a federated search across species, moves and abilities.
pub fn search_global(
    repository: &dyn SearchRepository,
    query: &GlobalSearchQuery,
) -> Result<GlobalSearchResult, String> {
    let term = super::species::sanitize_query(&query.query);
    if term.is_empty() {
        return Ok(GlobalSearchResult {
            query: term,
            hits: Vec::new(),
        });
    }

    let limit = if query.limit == 0 {
        DEFAULT_SEARCH_HITS
    } else {
        usize::try_from(query.limit)
            .unwrap_or(MAX_SEARCH_HITS)
            .min(MAX_SEARCH_HITS)
    };

    repository.search_global(
        &term,
        super::species::normalize_language(&query.lang),
        limit,
    )
}

/// Returns a short list of suggestions for the command palette.
pub fn suggest(
    repository: &dyn SearchRepository,
    query: &str,
    lang: &str,
    limit: Option<usize>,
) -> Result<Vec<SearchHit>, String> {
    let term = super::species::sanitize_query(query);
    if term.is_empty() {
        return Ok(Vec::new());
    }

    let limit = limit
        .unwrap_or(DEFAULT_SUGGESTIONS)
        .clamp(1, MAX_SEARCH_HITS);

    Ok(repository
        .search_global(&term, super::species::normalize_language(lang), limit)?
        .hits)
}
