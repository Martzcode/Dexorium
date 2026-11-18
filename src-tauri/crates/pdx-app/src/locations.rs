use pdx_domain::{EncounterSummary, LocationDetail, LocationSummary, NatureSummary};

pub trait LocationRepository {
    fn list_locations(&self, search: Option<&str>, lang: &str, limit: u32, offset: u32) -> Result<Vec<LocationSummary>, String>;
    fn location_detail(&self, id: i64, lang: &str) -> Result<Option<LocationDetail>, String>;
    fn encounters_for_species(&self, species_id: i64, lang: &str) -> Result<Vec<EncounterSummary>, String>;
    fn list_natures(&self, lang: &str) -> Result<Vec<NatureSummary>, String>;
}

pub fn list_locations(
    repository: &dyn LocationRepository,
    search: Option<&str>,
    lang: &str,
    limit: u32,
    offset: u32,
) -> Result<Vec<LocationSummary>, String> {
    repository.list_locations(search.map(str::trim).filter(|value| !value.is_empty()), super::species::normalize_language(lang), limit.min(500), offset)
}

pub fn location_detail(
    repository: &dyn LocationRepository,
    id: i64,
    lang: &str,
) -> Result<Option<LocationDetail>, String> {
    if id <= 0 {
        return Ok(None);
    }
    repository.location_detail(id, super::species::normalize_language(lang))
}

pub fn encounters_for_species(
    repository: &dyn LocationRepository,
    species_id: i64,
    lang: &str,
) -> Result<Vec<EncounterSummary>, String> {
    if species_id <= 0 {
        return Ok(Vec::new());
    }
    repository.encounters_for_species(species_id, super::species::normalize_language(lang))
}

pub fn list_natures(
    repository: &dyn LocationRepository,
    lang: &str,
) -> Result<Vec<NatureSummary>, String> {
    repository.list_natures(super::species::normalize_language(lang))
}
