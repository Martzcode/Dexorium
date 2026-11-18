use pdx_domain::{ItemDetail, ItemSummary};

pub trait ItemRepository {
    fn list_items(&self, search: Option<&str>, category: Option<&str>, lang: &str, limit: u32, offset: u32) -> Result<Vec<ItemSummary>, String>;
    fn item_detail(&self, id: i64, lang: &str) -> Result<Option<ItemDetail>, String>;
    fn search_items(&self, query: &str, lang: &str, limit: usize) -> Result<Vec<ItemSummary>, String>;
}

pub fn list_items(
    repository: &dyn ItemRepository,
    search: Option<&str>,
    category: Option<&str>,
    lang: &str,
    limit: u32,
    offset: u32,
) -> Result<Vec<ItemSummary>, String> {
    let lang = super::species::normalize_language(lang).to_owned();
    let search = search.map(str::trim).filter(|value| !value.is_empty());
    let category = category.map(str::trim).filter(|value| !value.is_empty());
    repository.list_items(search, category, &lang, limit.min(500), offset)
}

pub fn item_detail(
    repository: &dyn ItemRepository,
    id: i64,
    lang: &str,
) -> Result<Option<ItemDetail>, String> {
    if id <= 0 {
        return Ok(None);
    }
    repository.item_detail(id, super::species::normalize_language(lang))
}
