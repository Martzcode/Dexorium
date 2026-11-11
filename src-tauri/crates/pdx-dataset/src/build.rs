//! Builds the SQLite reference database from a fetched PokeAPI staging tree.

use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};

use pdx_infra::REFERENCE_SCHEMA;
use rusqlite::{Connection, params};
use serde::Deserialize;

use crate::util::{clean_text, id_from_url, normalize, now_iso8601, numeric_dirs, read_json};

/// Options accepted by `pdx-dataset build`.
pub struct BuildOptions {
    pub staging: PathBuf,
    pub out: PathBuf,
    pub langs: Vec<String>,
    pub version: String,
    pub source_commit: Option<String>,
    pub source_lock: Option<String>,
}

/// Result of a build, used by the `release` command.
pub struct BuildReport {
    pub row_counts: Vec<(String, i64)>,
}

/// Runs the full extraction and load.
pub fn build(options: &BuildOptions) -> Result<BuildReport, String> {
    let staging = &options.staging;
    if !staging.is_dir() {
        return Err(format!(
            "staging directory {} does not exist",
            staging.display()
        ));
    }

    if options.out.exists() {
        fs::remove_file(&options.out)
            .map_err(|error| format!("cannot replace {}: {error}", options.out.display()))?;
    }
    if let Some(parent) = options.out.parent() {
        fs::create_dir_all(parent)
            .map_err(|error| format!("cannot create {}: {error}", parent.display()))?;
    }

    let mut connection = Connection::open(&options.out)
        .map_err(|error| format!("cannot create {}: {error}", options.out.display()))?;
    connection
        .execute_batch(
            "PRAGMA journal_mode = MEMORY;
             PRAGMA synchronous = OFF;
             PRAGMA temp_store = MEMORY;
             PRAGMA cache_size = -200000;
             PRAGMA foreign_keys = ON;",
        )
        .map_err(|error| format!("cannot configure SQLite: {error}"))?;
    connection
        .execute_batch(REFERENCE_SCHEMA)
        .map_err(|error| format!("cannot apply reference schema: {error}"))?;

    let mut context = Context {
        langs: options.langs.clone(),
        ..Context::default()
    };

    {
        let tx = connection
            .transaction()
            .map_err(|error| format!("cannot start transaction: {error}"))?;

        insert_generations(&tx, staging, &mut context)?;
        insert_version_groups(&tx, staging, &mut context)?;
        insert_versions(&tx, staging, &mut context)?;
        insert_types(&tx, staging, &mut context)?;
        insert_type_efficacy(&tx, staging, &context)?;
        insert_egg_groups(&tx, staging, &context)?;
        insert_stat_names(&tx, staging, &context)?;
        insert_abilities(&tx, staging, &mut context)?;
        insert_moves(&tx, staging, &mut context)?;
        insert_evolution_chain_stubs(&tx, staging)?;
        insert_species(&tx, staging, &mut context)?;
        resolve_evolves_from(&tx, staging, &context)?;
        insert_forms(&tx, staging, &mut context)?;
        insert_evolution_steps(&tx, staging, &context)?;
        insert_meta(&tx, options, &context)?;

        tx.commit()
            .map_err(|error| format!("cannot commit reference database: {error}"))?;
    }

    connection
        .execute_batch("PRAGMA journal_mode = DELETE; VACUUM; ANALYZE;")
        .map_err(|error| format!("cannot finalize reference database: {error}"))?;

    let row_counts = read_row_counts(&connection)?;
    println!(
        "reference database written to {} ({})",
        options.out.display(),
        human_size(options.out.metadata().map_or(0, |metadata| metadata.len()))
    );
    if context.fallback_names > 0 {
        println!(
            "  {} localized name(s) fell back to English (missing source translation)",
            context.fallback_names
        );
    }
    for (table, count) in &row_counts {
        println!("  {table:<20} {count}");
    }

    Ok(BuildReport { row_counts })
}

#[derive(Default)]
struct Context {
    langs: Vec<String>,
    generations: HashMap<String, i64>,
    version_groups: HashMap<String, i64>,
    versions: HashMap<String, i64>,
    types: HashMap<String, i64>,
    type_introduced: HashMap<i64, i64>,
    abilities: HashMap<String, i64>,
    moves: HashMap<String, i64>,
    skipped_moves: HashSet<String>,
    species: HashMap<String, i64>,
    species_names: HashMap<String, (String, String)>,
    species_generation: HashMap<i64, i64>,
    fallback_names: usize,
}

// ---------------------------------------------------------------------------
// PokeAPI payloads (only the fields the MVP needs)
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
struct Named {
    name: String,
}

#[derive(Debug, Deserialize)]
struct NamedUrl {
    #[serde(default)]
    name: String,
    url: String,
}

#[derive(Debug, Deserialize)]
struct LocalizedName {
    name: String,
    language: Named,
}

#[derive(Debug, Deserialize)]
struct GenerationJson {
    id: i64,
    name: String,
    main_region: Named,
}

#[derive(Debug, Deserialize)]
struct VersionGroupJson {
    id: i64,
    name: String,
    order: i64,
    generation: Named,
}

#[derive(Debug, Deserialize)]
struct VersionJson {
    id: i64,
    name: String,
    version_group: Named,
}

#[derive(Debug, Deserialize)]
struct TypeJson {
    id: i64,
    name: String,
    generation: Named,
    damage_relations: DamageRelations,
    #[serde(default)]
    past_damage_relations: Vec<PastDamageRelations>,
    #[serde(default)]
    names: Vec<LocalizedName>,
}

#[derive(Debug, Default, Deserialize)]
struct DamageRelations {
    #[serde(default)]
    no_damage_to: Vec<Named>,
    #[serde(default)]
    half_damage_to: Vec<Named>,
    #[serde(default)]
    double_damage_to: Vec<Named>,
}

#[derive(Debug, Deserialize)]
struct PastDamageRelations {
    generation: Named,
    damage_relations: DamageRelations,
}

#[derive(Debug, Deserialize)]
struct SpeciesJson {
    id: i64,
    name: String,
    generation: Named,
    gender_rate: i64,
    capture_rate: i64,
    base_happiness: Option<i64>,
    hatch_counter: Option<i64>,
    growth_rate: Named,
    color: Option<Named>,
    shape: Option<Named>,
    habitat: Option<Named>,
    is_baby: bool,
    is_legendary: bool,
    is_mythical: bool,
    has_gender_differences: bool,
    #[serde(default)]
    evolves_from_species: Option<Named>,
    #[serde(default)]
    evolution_chain: Option<NamedUrl>,
    #[serde(default)]
    names: Vec<LocalizedName>,
    #[serde(default)]
    genera: Vec<LocalizedGenus>,
    #[serde(default)]
    flavor_text_entries: Vec<SpeciesFlavorText>,
    #[serde(default)]
    egg_groups: Vec<Named>,
}

#[derive(Debug, Deserialize)]
struct LocalizedGenus {
    genus: String,
    language: Named,
}

#[derive(Debug, Deserialize)]
struct SpeciesFlavorText {
    flavor_text: String,
    language: Named,
    version: Named,
}

#[derive(Debug, Deserialize)]
struct PokemonJson {
    id: i64,
    name: String,
    height: i64,
    weight: i64,
    base_experience: Option<i64>,
    #[serde(default)]
    is_default: bool,
    species: Named,
    #[serde(default)]
    abilities: Vec<PokemonAbility>,
    #[serde(default)]
    forms: Vec<NamedUrl>,
    #[serde(default)]
    stats: Vec<PokemonStat>,
    #[serde(default)]
    types: Vec<PokemonType>,
    #[serde(default)]
    moves: Vec<PokemonMove>,
}

#[derive(Debug, Deserialize)]
struct PokemonAbility {
    ability: Named,
    #[serde(default)]
    is_hidden: bool,
    slot: i64,
}

#[derive(Debug, Deserialize)]
struct PokemonStat {
    base_stat: i64,
    #[serde(default)]
    effort: i64,
    stat: Named,
}

#[derive(Debug, Deserialize)]
struct PokemonType {
    slot: i64,
    #[serde(rename = "type")]
    type_: Named,
}

#[derive(Debug, Deserialize)]
struct PokemonMove {
    #[serde(rename = "move")]
    move_: Named,
    #[serde(default)]
    version_group_details: Vec<PokemonMoveDetail>,
}

#[derive(Debug, Deserialize)]
struct PokemonMoveDetail {
    level_learned_at: i64,
    move_learn_method: Named,
    version_group: Named,
}

#[derive(Debug, Deserialize)]
struct PokemonFormJson {
    #[serde(default)]
    form_order: i64,
    #[serde(default)]
    is_mega: bool,
    #[serde(default)]
    form_name: String,
    #[serde(default)]
    form_names: Vec<LocalizedName>,
}

#[derive(Debug, Deserialize)]
struct AbilityJson {
    id: i64,
    name: String,
    generation: Named,
    #[serde(default)]
    is_main_series: bool,
    #[serde(default)]
    names: Vec<LocalizedName>,
    #[serde(default)]
    effect_entries: Vec<EffectEntry>,
    #[serde(default)]
    flavor_text_entries: Vec<AbilityFlavorText>,
}

#[derive(Debug, Deserialize)]
struct EffectEntry {
    effect: String,
    short_effect: String,
    language: Named,
}

#[derive(Debug, Deserialize)]
struct AbilityFlavorText {
    flavor_text: String,
    language: Named,
}

#[derive(Debug, Deserialize)]
struct MoveJson {
    id: i64,
    name: String,
    accuracy: Option<i64>,
    effect_chance: Option<i64>,
    pp: Option<i64>,
    priority: i64,
    power: Option<i64>,
    generation: Named,
    #[serde(rename = "type")]
    type_: Named,
    damage_class: Named,
    target: Named,
    #[serde(default)]
    meta: Option<MoveMeta>,
    #[serde(default)]
    names: Vec<LocalizedName>,
    #[serde(default)]
    effect_entries: Vec<EffectEntry>,
    #[serde(default)]
    flavor_text_entries: Vec<MoveFlavorText>,
    #[serde(default)]
    past_values: Vec<MovePastValue>,
}

#[derive(Debug, Default, Deserialize)]
struct MoveMeta {
    #[serde(default)]
    ailment: Option<Named>,
    #[serde(default)]
    min_hits: Option<i64>,
    #[serde(default)]
    max_hits: Option<i64>,
    #[serde(default)]
    min_turns: Option<i64>,
    #[serde(default)]
    max_turns: Option<i64>,
    #[serde(default)]
    drain: i64,
    #[serde(default)]
    healing: i64,
    #[serde(default)]
    crit_rate: i64,
    #[serde(default)]
    ailment_chance: i64,
    #[serde(default)]
    flinch_chance: i64,
    #[serde(default)]
    stat_chance: i64,
}

#[derive(Debug, Deserialize)]
struct MoveFlavorText {
    flavor_text: String,
    language: Named,
}

#[derive(Debug, Deserialize)]
struct MovePastValue {
    version_group: Named,
    power: Option<i64>,
    accuracy: Option<i64>,
    pp: Option<i64>,
    #[serde(rename = "type")]
    type_: Option<Named>,
    effect_chance: Option<i64>,
}

#[derive(Debug, Deserialize)]
struct EvolutionChainJson {
    id: i64,
    #[serde(default)]
    baby_trigger_item: Option<Named>,
    chain: ChainLink,
}

#[derive(Debug, Deserialize)]
struct ChainLink {
    species: NamedUrl,
    #[serde(default)]
    evolution_details: Vec<EvolutionDetail>,
    #[serde(default)]
    evolves_to: Vec<ChainLink>,
}

#[derive(Debug, Deserialize)]
struct EvolutionDetail {
    #[serde(default)]
    trigger: Option<Named>,
    #[serde(default)]
    min_level: Option<i64>,
    #[serde(default)]
    min_happiness: Option<i64>,
    #[serde(default)]
    min_affection: Option<i64>,
    #[serde(default)]
    min_beauty: Option<i64>,
    #[serde(default)]
    needs_overworld_rain: bool,
    #[serde(default)]
    time_of_day: String,
    #[serde(default)]
    gender: Option<i64>,
    #[serde(default)]
    held_item: Option<Named>,
    #[serde(default)]
    item: Option<Named>,
    #[serde(default)]
    known_move: Option<Named>,
    #[serde(default)]
    known_move_type: Option<Named>,
    #[serde(default)]
    party_species: Option<Named>,
    #[serde(default)]
    party_type: Option<Named>,
    #[serde(default)]
    trade_species: Option<Named>,
    #[serde(default)]
    location: Option<Named>,
    #[serde(default)]
    relative_physical_stats: Option<i64>,
    #[serde(default)]
    turn_upside_down: bool,
    #[serde(default)]
    condition_expression: Option<serde_json::Value>,
    #[serde(flatten)]
    extra: HashMap<String, serde_json::Value>,
}

// ---------------------------------------------------------------------------
// Referentials
// ---------------------------------------------------------------------------

fn insert_generations(
    tx: &rusqlite::Transaction<'_>,
    staging: &Path,
    context: &mut Context,
) -> Result<(), String> {
    for (id, path) in numeric_dirs(&staging.join("generation"))? {
        let generation: GenerationJson = read_json(&path)?;
        tx.execute(
            "INSERT INTO generation (id, slug, region_slug, order_index) VALUES (?1, ?2, ?3, ?4)",
            params![
                generation.id,
                generation.name,
                generation.main_region.name,
                generation.id
            ],
        )
        .map_err(|error| format!("cannot insert generation {}: {error}", generation.name))?;
        context.generations.insert(generation.name, generation.id);
        debug_assert_eq!(id, generation.id);
    }

    Ok(())
}

fn insert_version_groups(
    tx: &rusqlite::Transaction<'_>,
    staging: &Path,
    context: &mut Context,
) -> Result<(), String> {
    for (_, path) in numeric_dirs(&staging.join("version-group"))? {
        let group: VersionGroupJson = read_json(&path)?;
        let generation_id = *context
            .generations
            .get(&group.generation.name)
            .ok_or_else(|| {
                format!(
                    "version group `{}` references unknown generation `{}`",
                    group.name, group.generation.name
                )
            })?;

        tx.execute(
            "INSERT INTO version_group (id, slug, generation_id, order_index) VALUES (?1, ?2, ?3, ?4)",
            params![group.id, group.name, generation_id, group.order],
        )
        .map_err(|error| format!("cannot insert version group {}: {error}", group.name))?;

        context.version_groups.insert(group.name, group.id);
    }

    Ok(())
}

fn insert_versions(
    tx: &rusqlite::Transaction<'_>,
    staging: &Path,
    context: &mut Context,
) -> Result<(), String> {
    for (_, path) in numeric_dirs(&staging.join("version"))? {
        let version: VersionJson = read_json(&path)?;
        let group_id = *context
            .version_groups
            .get(&version.version_group.name)
            .ok_or_else(|| {
                format!(
                    "version `{}` references unknown version group `{}`",
                    version.name, version.version_group.name
                )
            })?;

        tx.execute(
            "INSERT INTO game_version (id, slug, version_group_id) VALUES (?1, ?2, ?3)",
            params![version.id, version.name, group_id],
        )
        .map_err(|error| format!("cannot insert version {}: {error}", version.name))?;

        context.versions.insert(version.name, version.id);
    }

    Ok(())
}

fn insert_types(
    tx: &rusqlite::Transaction<'_>,
    staging: &Path,
    context: &mut Context,
) -> Result<(), String> {
    for (id, path) in numeric_dirs(&staging.join("type"))? {
        // PokeAPI models `unknown` (10001) and `shadow` (10002) as pseudo-types.
        // They are not part of the 18-type chart used by the MVP.
        if id >= 10_000 {
            continue;
        }

        let type_: TypeJson = read_json(&path)?;
        let generation_id = *context
            .generations
            .get(&type_.generation.name)
            .ok_or_else(|| {
                format!(
                    "type `{}` references unknown generation `{}`",
                    type_.name, type_.generation.name
                )
            })?;

        tx.execute(
            "INSERT INTO poke_type (id, slug, introduced_in) VALUES (?1, ?2, ?3)",
            params![type_.id, type_.name, generation_id],
        )
        .map_err(|error| format!("cannot insert type {}: {error}", type_.name))?;

        context.types.insert(type_.name.clone(), type_.id);
        context.type_introduced.insert(type_.id, generation_id);

        for lang in &context.langs {
            if let Some(name) = localized_name(&type_.names, lang) {
                put_translation(tx, "type", type_.id, lang, "name", name)?;
            }
        }

        index_entity(
            tx,
            "type",
            type_.id,
            localized_name(&type_.names, "en").unwrap_or(type_.name.as_str()),
            &type_.name,
            &type_.name,
        )?;
    }

    Ok(())
}

fn insert_type_efficacy(
    tx: &rusqlite::Transaction<'_>,
    staging: &Path,
    context: &Context,
) -> Result<(), String> {
    let mut current: HashMap<i64, DamageRelations> = HashMap::new();
    let mut past: HashMap<i64, HashMap<i64, DamageRelations>> = HashMap::new();

    for (id, path) in numeric_dirs(&staging.join("type"))? {
        if id >= 10_000 {
            continue;
        }
        let type_: TypeJson = read_json(&path)?;
        let Some(&attacker) = context.types.get(&type_.name) else {
            continue;
        };

        current.insert(attacker, type_.damage_relations);
        for past_entry in type_.past_damage_relations {
            let Some(&generation) = context.generations.get(&past_entry.generation.name) else {
                continue;
            };
            past.entry(generation)
                .or_default()
                .insert(attacker, past_entry.damage_relations);
        }
    }

    let mut statement = tx
        .prepare(
            "INSERT INTO type_efficacy
             (generation_id, attacking_type, defending_type, multiplier_x100)
             VALUES (?1, ?2, ?3, ?4)",
        )
        .map_err(|error| format!("cannot prepare type efficacy insert: {error}"))?;

    for generation in 1..=9_i64 {
        for attacker in 1..=18_i64 {
            // Relations for this generation: a past override when present,
            // otherwise the current chart.
            let relations = past
                .get(&generation)
                .and_then(|by_attacker| by_attacker.get(&attacker))
                .or_else(|| current.get(&attacker));

            for defender in 1..=18_i64 {
                let both_exist = context
                    .type_introduced
                    .get(&attacker)
                    .is_some_and(|introduced| *introduced <= generation)
                    && context
                        .type_introduced
                        .get(&defender)
                        .is_some_and(|introduced| *introduced <= generation);

                let multiplier = if both_exist {
                    relations.map_or(100, |relations| {
                        multiplier_for(relations, defender, context)
                    })
                } else {
                    100
                };

                statement
                    .execute(params![generation, attacker, defender, multiplier])
                    .map_err(|error| format!("cannot insert type efficacy: {error}"))?;
            }
        }
    }

    Ok(())
}

fn multiplier_for(relations: &DamageRelations, defender: i64, context: &Context) -> i64 {
    let matches = |names: &[Named]| {
        names
            .iter()
            .any(|name| context.types.get(&name.name).copied() == Some(defender))
    };

    if matches(&relations.no_damage_to) {
        0
    } else if matches(&relations.half_damage_to) {
        50
    } else if matches(&relations.double_damage_to) {
        200
    } else {
        100
    }
}

fn insert_egg_groups(
    tx: &rusqlite::Transaction<'_>,
    staging: &Path,
    context: &Context,
) -> Result<(), String> {
    for (_, path) in numeric_dirs(&staging.join("egg-group"))? {
        #[derive(Deserialize)]
        struct EggGroupJson {
            id: i64,
            name: String,
            #[serde(default)]
            names: Vec<LocalizedName>,
        }

        let group: EggGroupJson = read_json(&path)?;
        tx.execute(
            "INSERT INTO egg_group (id, slug) VALUES (?1, ?2)",
            params![group.id, group.name],
        )
        .map_err(|error| format!("cannot insert egg group {}: {error}", group.name))?;

        for lang in &context.langs {
            if let Some(name) = localized_name(&group.names, lang) {
                put_translation(tx, "egg_group", group.id, lang, "name", name)?;
            }
        }
    }

    Ok(())
}

fn insert_stat_names(
    tx: &rusqlite::Transaction<'_>,
    staging: &Path,
    context: &Context,
) -> Result<(), String> {
    for (_, path) in numeric_dirs(&staging.join("stat"))? {
        #[derive(Deserialize)]
        struct StatJson {
            id: i64,
            #[serde(default)]
            names: Vec<LocalizedName>,
        }

        let stat: StatJson = read_json(&path)?;
        for lang in &context.langs {
            if let Some(name) = localized_name(&stat.names, lang) {
                put_translation(tx, "stat", stat.id, lang, "name", name)?;
            }
        }
    }

    Ok(())
}

fn insert_abilities(
    tx: &rusqlite::Transaction<'_>,
    staging: &Path,
    context: &mut Context,
) -> Result<(), String> {
    for (_, path) in numeric_dirs(&staging.join("ability"))? {
        let ability: AbilityJson = read_json(&path)?;
        let generation_id = *context
            .generations
            .get(&ability.generation.name)
            .ok_or_else(|| {
                format!(
                    "ability `{}` references unknown generation `{}`",
                    ability.name, ability.generation.name
                )
            })?;

        tx.execute(
            "INSERT INTO ability (id, slug, generation_id, is_main_series) VALUES (?1, ?2, ?3, ?4)",
            params![
                ability.id,
                ability.name,
                generation_id,
                i64::from(ability.is_main_series)
            ],
        )
        .map_err(|error| format!("cannot insert ability {}: {error}", ability.name))?;

        let english_name = localized_name(&ability.names, "en")
            .unwrap_or(ability.name.as_str())
            .to_owned();
        let langs = context.langs.clone();

        for lang in &langs {
            let name = match localized_name(&ability.names, lang) {
                Some(name) => name,
                None => {
                    // Recent source entries sometimes ship without a French
                    // localisation; the official English name is used and
                    // counted in the build report.
                    context.fallback_names += 1;
                    english_name.as_str()
                }
            };
            put_translation(tx, "ability", ability.id, lang, "name", name)?;
            if let Some(entry) = ability
                .effect_entries
                .iter()
                .find(|entry| entry.language.name == *lang)
            {
                put_translation(
                    tx,
                    "ability",
                    ability.id,
                    lang,
                    "effect_short",
                    &clean_text(&entry.short_effect),
                )?;
                put_translation(
                    tx,
                    "ability",
                    ability.id,
                    lang,
                    "effect_long",
                    &clean_text(&entry.effect),
                )?;
            }
            if let Some(flavor) = last_flavor(&ability.flavor_text_entries, lang) {
                put_translation(tx, "ability", ability.id, lang, "description", &flavor)?;
            }
        }

        index_entity(
            tx,
            "ability",
            ability.id,
            localized_name(&ability.names, "en").unwrap_or(ability.name.as_str()),
            &ability.name,
            &ability.name,
        )?;

        context.abilities.insert(ability.name, ability.id);
    }

    Ok(())
}

fn insert_moves(
    tx: &rusqlite::Transaction<'_>,
    staging: &Path,
    context: &mut Context,
) -> Result<(), String> {
    let mut insert_move = tx
        .prepare(
            "INSERT INTO move_ (
                id, slug, generation_id, type_id, damage_class, power, accuracy, pp,
                priority, target_slug, effect_chance, crit_rate_bonus, drain_pct,
                healing_pct, min_hits, max_hits, min_turns, max_turns, ailment_slug,
                ailment_chance, flinch_chance, stat_chance
             ) VALUES (
                ?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14,
                ?15, ?16, ?17, ?18, ?19, ?20, ?21, ?22
             )",
        )
        .map_err(|error| format!("cannot prepare move insert: {error}"))?;
    let mut insert_past = tx
        .prepare(
            "INSERT INTO move_past_value
             (move_id, version_group_id, power, accuracy, pp, type_id, effect_chance)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        )
        .map_err(|error| format!("cannot prepare move past value insert: {error}"))?;

    for (_, path) in numeric_dirs(&staging.join("move"))? {
        let move_data: MoveJson = read_json(&path)?;
        // PokeAPI models `shadow` (XD/Colosseum) and `unknown` as pseudo-types.
        // The MVP covers the 18 main-series types, so those moves are excluded.
        if is_pseudo_type(&move_data.type_.name) {
            context.skipped_moves.insert(move_data.name.clone());
            continue;
        }
        let generation_id = *context
            .generations
            .get(&move_data.generation.name)
            .ok_or_else(|| {
                format!(
                    "move `{}` references unknown generation `{}`",
                    move_data.name, move_data.generation.name
                )
            })?;
        let type_id = *context.types.get(&move_data.type_.name).ok_or_else(|| {
            format!(
                "move `{}` references unknown type `{}`",
                move_data.name, move_data.type_.name
            )
        })?;
        let meta = move_data.meta.unwrap_or_default();

        insert_move
            .execute(params![
                move_data.id,
                move_data.name,
                generation_id,
                type_id,
                move_data.damage_class.name,
                move_data.power,
                move_data.accuracy,
                move_data.pp,
                move_data.priority,
                move_data.target.name,
                move_data.effect_chance,
                meta.crit_rate,
                meta.drain,
                meta.healing,
                meta.min_hits,
                meta.max_hits,
                meta.min_turns,
                meta.max_turns,
                meta.ailment.as_ref().map(|ailment| ailment.name.as_str()),
                meta.ailment_chance,
                meta.flinch_chance,
                meta.stat_chance,
            ])
            .map_err(|error| format!("cannot insert move {}: {error}", move_data.name))?;

        for past in &move_data.past_values {
            let Some(&version_group_id) = context.version_groups.get(&past.version_group.name)
            else {
                continue;
            };
            let past_type_id = past
                .type_
                .as_ref()
                .and_then(|type_| context.types.get(&type_.name).copied());

            insert_past
                .execute(params![
                    move_data.id,
                    version_group_id,
                    past.power,
                    past.accuracy,
                    past.pp,
                    past_type_id,
                    past.effect_chance,
                ])
                .map_err(|error| {
                    format!("cannot insert past value for {}: {error}", move_data.name)
                })?;
        }

        for lang in &context.langs {
            if let Some(name) = localized_name(&move_data.names, lang) {
                put_translation(tx, "move", move_data.id, lang, "name", name)?;
            }
            if let Some(entry) = move_data
                .effect_entries
                .iter()
                .find(|entry| entry.language.name == *lang)
            {
                put_translation(
                    tx,
                    "move",
                    move_data.id,
                    lang,
                    "effect_short",
                    &clean_text(&entry.short_effect),
                )?;
                put_translation(
                    tx,
                    "move",
                    move_data.id,
                    lang,
                    "effect_long",
                    &clean_text(&entry.effect),
                )?;
            }
            if let Some(flavor) = last_flavor(&move_data.flavor_text_entries, lang) {
                put_translation(tx, "move", move_data.id, lang, "description", &flavor)?;
            }
        }

        index_entity(
            tx,
            "move",
            move_data.id,
            localized_name(&move_data.names, "en").unwrap_or(move_data.name.as_str()),
            &move_data.name,
            &move_data.name,
        )?;

        context.moves.insert(move_data.name, move_data.id);
    }

    Ok(())
}

// ---------------------------------------------------------------------------
// Species and forms
// ---------------------------------------------------------------------------

fn insert_evolution_chain_stubs(
    tx: &rusqlite::Transaction<'_>,
    staging: &Path,
) -> Result<(), String> {
    let chains = numeric_dirs(&staging.join("evolution-chain"))?;
    let mut statement = tx
        .prepare("INSERT INTO evolution_chain (id) VALUES (?1)")
        .map_err(|error| format!("cannot prepare evolution chain insert: {error}"))?;

    for (_, path) in chains {
        let chain: EvolutionChainJson = read_json(&path)?;
        statement
            .execute([chain.id])
            .map_err(|error| format!("cannot insert evolution chain {}: {error}", chain.id))?;
    }

    Ok(())
}

fn insert_species(
    tx: &rusqlite::Transaction<'_>,
    staging: &Path,
    context: &mut Context,
) -> Result<(), String> {
    let mut insert_species = tx
        .prepare(
            "INSERT INTO species (
                id, slug, national_dex_number, generation_id, gender_rate, capture_rate,
                base_happiness, hatch_counter, growth_rate_slug, color_slug, shape_slug,
                habitat_slug, is_baby, is_legendary, is_mythical, is_ultra_beast,
                has_gender_differences, evolution_chain_id
             ) VALUES (
                ?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14,
                ?15, ?16, ?17, ?18
             )",
        )
        .map_err(|error| format!("cannot prepare species insert: {error}"))?;
    let mut insert_egg_group = tx
        .prepare(
            "INSERT OR IGNORE INTO species_egg_group (species_id, egg_group_id)
             SELECT ?1, id FROM egg_group WHERE slug = ?2",
        )
        .map_err(|error| format!("cannot prepare species egg group insert: {error}"))?;

    for (_, path) in numeric_dirs(&staging.join("pokemon-species"))? {
        let species: SpeciesJson = read_json(&path)?;
        let generation_id = *context
            .generations
            .get(&species.generation.name)
            .ok_or_else(|| {
                format!(
                    "species `{}` references unknown generation `{}`",
                    species.name, species.generation.name
                )
            })?;
        let chain_id = species
            .evolution_chain
            .as_ref()
            .map(|chain| id_from_url(&chain.url))
            .transpose()?;
        let ultra_beast = is_ultra_beast(&species.name);

        insert_species
            .execute(params![
                species.id,
                species.name,
                species.id,
                generation_id,
                species.gender_rate,
                species.capture_rate,
                species.base_happiness,
                species.hatch_counter,
                species.growth_rate.name,
                species.color.as_ref().map(|color| color.name.as_str()),
                species.shape.as_ref().map(|shape| shape.name.as_str()),
                species
                    .habitat
                    .as_ref()
                    .map(|habitat| habitat.name.as_str()),
                i64::from(species.is_baby),
                i64::from(species.is_legendary),
                i64::from(species.is_mythical),
                i64::from(ultra_beast),
                i64::from(species.has_gender_differences),
                chain_id,
            ])
            .map_err(|error| format!("cannot insert species {}: {error}", species.name))?;

        for group in &species.egg_groups {
            insert_egg_group
                .execute(params![species.id, group.name])
                .map_err(|error| format!("cannot link egg group for {}: {error}", species.name))?;
        }

        let english = localized_name(&species.names, "en").unwrap_or(species.name.as_str());
        let french = localized_name(&species.names, "fr").unwrap_or(english);
        for lang in &context.langs {
            let name = if lang == "fr" {
                french.to_owned()
            } else {
                english.to_owned()
            };
            put_translation(tx, "species", species.id, lang, "name", &name)?;

            if let Some(genus) = species
                .genera
                .iter()
                .find(|genus| genus.language.name == *lang)
            {
                put_translation(
                    tx,
                    "species",
                    species.id,
                    lang,
                    "genus",
                    &clean_text(&genus.genus),
                )?;
            }
        }

        let mut flavors: HashSet<(String, String)> = HashSet::new();
        for flavor in &species.flavor_text_entries {
            if !context.langs.contains(&flavor.language.name) {
                continue;
            }
            let Some(&version_id) = context.versions.get(&flavor.version.name) else {
                continue;
            };
            if !flavors.insert((version_id.to_string(), flavor.language.name.clone())) {
                continue;
            }
            let text = clean_text(&flavor.flavor_text);
            if text.is_empty() {
                continue;
            }
            tx.execute(
                "INSERT OR REPLACE INTO flavor_text (species_id, version_id, lang, text)
                 VALUES (?1, ?2, ?3, ?4)",
                params![species.id, version_id, flavor.language.name, text],
            )
            .map_err(|error| format!("cannot insert flavor text for {}: {error}", species.name))?;
        }

        let normalized = normalize(&format!(
            "{french} {english} {} {}",
            species.name, species.id
        ));
        index_entity(tx, "species", species.id, english, &normalized, &normalized)?;

        context.species.insert(species.name.clone(), species.id);
        context.species_names.insert(
            species.name.clone(),
            (french.to_owned(), english.to_owned()),
        );
        context.species_generation.insert(species.id, generation_id);
    }

    Ok(())
}

fn resolve_evolves_from(
    tx: &rusqlite::Transaction<'_>,
    staging: &Path,
    context: &Context,
) -> Result<(), String> {
    let mut statement = tx
        .prepare("UPDATE species SET evolves_from_species_id = ?1 WHERE id = ?2")
        .map_err(|error| format!("cannot prepare evolves_from update: {error}"))?;

    for (_, path) in numeric_dirs(&staging.join("pokemon-species"))? {
        let species: SpeciesJson = read_json(&path)?;
        if let Some(parent) = species.evolves_from_species {
            let Some(&parent_id) = context.species.get(&parent.name) else {
                return Err(format!(
                    "species `{}` evolves from unknown species `{}`",
                    species.name, parent.name
                ));
            };
            statement
                .execute(params![parent_id, species.id])
                .map_err(|error| format!("cannot update {}: {error}", species.name))?;
        }
    }

    Ok(())
}

fn insert_forms(
    tx: &rusqlite::Transaction<'_>,
    staging: &Path,
    context: &mut Context,
) -> Result<(), String> {
    let mut insert_form = tx
        .prepare(
            "INSERT INTO pokemon_form (
                id, species_id, slug, form_order, is_default, is_mega, is_gigantamax,
                is_regional, region_slug, introduced_generation_id, height_dm, weight_hg,
                base_experience
             ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13)",
        )
        .map_err(|error| format!("cannot prepare form insert: {error}"))?;
    let mut insert_type = tx
        .prepare("INSERT INTO form_type (form_id, slot, type_id) VALUES (?1, ?2, ?3)")
        .map_err(|error| format!("cannot prepare form type insert: {error}"))?;
    let mut insert_stat = tx
        .prepare(
            "INSERT INTO form_stat (form_id, stat_slug, base_value, effort_yield)
             VALUES (?1, ?2, ?3, ?4)",
        )
        .map_err(|error| format!("cannot prepare form stat insert: {error}"))?;
    let mut insert_summary = tx
        .prepare(
            "INSERT INTO form_stat_summary (form_id, hp, attack, defense, sp_atk, sp_def, speed, bst)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
        )
        .map_err(|error| format!("cannot prepare form stat summary insert: {error}"))?;
    let mut insert_ability = tx
        .prepare(
            "INSERT INTO form_ability (form_id, ability_id, slot, is_hidden)
             VALUES (?1, ?2, ?3, ?4)",
        )
        .map_err(|error| format!("cannot prepare form ability insert: {error}"))?;
    let mut insert_move = tx
        .prepare(
            "INSERT OR IGNORE INTO form_move (
                form_id, move_id, version_group_id, method_slug, level
             ) VALUES (?1, ?2, ?3, ?4, ?5)",
        )
        .map_err(|error| format!("cannot prepare learnset insert: {error}"))?;

    for (_, path) in numeric_dirs(&staging.join("pokemon"))? {
        let pokemon: PokemonJson = read_json(&path)?;
        let Some(&species_id) = context.species.get(&pokemon.species.name) else {
            return Err(format!(
                "pokemon form `{}` references unknown species `{}`",
                pokemon.name, pokemon.species.name
            ));
        };

        let form_json = pokemon
            .forms
            .first()
            .map(|form| {
                let id = id_from_url(&form.url)?;
                read_json::<PokemonFormJson>(
                    &staging
                        .join("pokemon-form")
                        .join(id.to_string())
                        .join("index.json"),
                )
            })
            .transpose()?;

        let form_order = form_json.as_ref().map_or(1, |form| form.form_order.max(1));
        let is_mega =
            form_json.as_ref().is_some_and(|form| form.is_mega) || pokemon.name.contains("-mega");
        let is_gigantamax = pokemon.name.ends_with("-gmax");
        let region = regional_region(&pokemon.name);
        let is_regional = region.is_some();
        let species_generation = context
            .species_generation
            .get(&species_id)
            .copied()
            .unwrap_or(1);
        let introduced = if pokemon.is_default {
            species_generation
        } else {
            form_generation(&pokemon.name).unwrap_or(species_generation)
        };

        insert_form
            .execute(params![
                pokemon.id,
                species_id,
                pokemon.name,
                form_order,
                i64::from(pokemon.is_default),
                i64::from(is_mega),
                i64::from(is_gigantamax),
                i64::from(is_regional),
                region,
                introduced,
                pokemon.height,
                pokemon.weight,
                pokemon.base_experience,
            ])
            .map_err(|error| format!("cannot insert form {}: {error}", pokemon.name))?;

        for type_ in &pokemon.types {
            if is_pseudo_type(&type_.type_.name) {
                continue;
            }
            let type_id = *context.types.get(&type_.type_.name).ok_or_else(|| {
                format!(
                    "form `{}` references unknown type `{}`",
                    pokemon.name, type_.type_.name
                )
            })?;
            insert_type
                .execute(params![pokemon.id, type_.slot, type_id])
                .map_err(|error| format!("cannot insert type for {}: {error}", pokemon.name))?;
        }

        let mut stats = HashMap::new();
        for stat in &pokemon.stats {
            let Some(slug) = stat_slug(&stat.stat.name) else {
                continue;
            };
            insert_stat
                .execute(params![pokemon.id, slug, stat.base_stat, stat.effort])
                .map_err(|error| format!("cannot insert stat for {}: {error}", pokemon.name))?;
            stats.insert(slug, stat.base_stat);
        }

        let value = |slug: &str| stats.get(slug).copied().unwrap_or(0);
        let bst = value("hp")
            + value("attack")
            + value("defense")
            + value("sp_atk")
            + value("sp_def")
            + value("speed");
        insert_summary
            .execute(params![
                pokemon.id,
                value("hp"),
                value("attack"),
                value("defense"),
                value("sp_atk"),
                value("sp_def"),
                value("speed"),
                bst,
            ])
            .map_err(|error| format!("cannot insert stat summary for {}: {error}", pokemon.name))?;

        for ability in &pokemon.abilities {
            let Some(&ability_id) = context.abilities.get(&ability.ability.name) else {
                return Err(format!(
                    "form `{}` references unknown ability `{}`",
                    pokemon.name, ability.ability.name
                ));
            };
            insert_ability
                .execute(params![
                    pokemon.id,
                    ability_id,
                    ability.slot,
                    i64::from(ability.is_hidden)
                ])
                .map_err(|error| format!("cannot insert ability for {}: {error}", pokemon.name))?;
        }

        for move_entry in &pokemon.moves {
            let Some(&move_id) = context.moves.get(&move_entry.move_.name) else {
                if context.skipped_moves.contains(&move_entry.move_.name) {
                    continue;
                }
                return Err(format!(
                    "form `{}` references unknown move `{}`",
                    pokemon.name, move_entry.move_.name
                ));
            };

            for detail in &move_entry.version_group_details {
                let Some(&version_group_id) =
                    context.version_groups.get(&detail.version_group.name)
                else {
                    continue;
                };
                insert_move
                    .execute(params![
                        pokemon.id,
                        move_id,
                        version_group_id,
                        detail.move_learn_method.name,
                        detail.level_learned_at,
                    ])
                    .map_err(|error| {
                        format!("cannot insert learnset for {}: {error}", pokemon.name)
                    })?;
            }
        }

        // Localized form name. Default forms inherit the species name; alternate
        // forms use the PokeAPI form names, with an explicit English fallback.
        let (species_fr, species_en) = context
            .species_names
            .get(&pokemon.species.name)
            .cloned()
            .unwrap_or_else(|| (pokemon.species.name.clone(), pokemon.species.name.clone()));
        let form_fr = form_json
            .as_ref()
            .and_then(|form| localized_name(&form.form_names, "fr"))
            .map(str::to_owned);
        let form_en = form_json
            .as_ref()
            .and_then(|form| localized_name(&form.form_names, "en"))
            .map(str::to_owned);

        for lang in &context.langs {
            let name = if pokemon.is_default {
                if lang == "fr" {
                    species_fr.clone()
                } else {
                    species_en.clone()
                }
            } else if lang == "fr" {
                form_fr.clone().unwrap_or_else(|| {
                    let label = form_json
                        .as_ref()
                        .map(|form| form.form_name.clone())
                        .filter(|label| !label.is_empty())
                        .unwrap_or_default();
                    if label.is_empty() {
                        pokemon.name.clone()
                    } else {
                        format!("{species_fr} ({label})")
                    }
                })
            } else {
                form_en.clone().unwrap_or_else(|| {
                    let label = form_json
                        .as_ref()
                        .map(|form| form.form_name.clone())
                        .filter(|label| !label.is_empty())
                        .unwrap_or_default();
                    if label.is_empty() {
                        pokemon.name.clone()
                    } else {
                        format!("{species_en} ({label})")
                    }
                })
            };

            put_translation(tx, "form", pokemon.id, lang, "name", &name)?;
        }
    }

    Ok(())
}

fn insert_evolution_steps(
    tx: &rusqlite::Transaction<'_>,
    staging: &Path,
    context: &Context,
) -> Result<(), String> {
    let mut insert_step = tx
        .prepare(
            "INSERT INTO evolution_step (
                id, chain_id, from_species_id, to_species_id, trigger_slug, min_level,
                min_happiness, min_affection, min_beauty, needs_overworld_rain, time_of_day,
                gender, held_item_slug, trigger_item_slug, known_move_id, known_move_type_id,
                party_species_id, party_type_id, trade_species_id, location_slug,
                relative_physical_stats, turn_upside_down, condition_expression,
                extra_condition_json
             ) VALUES (
                ?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15,
                ?16, ?17, ?18, ?19, ?20, ?21, ?22, ?23, ?24
             )",
        )
        .map_err(|error| format!("cannot prepare evolution step insert: {error}"))?;
    let mut update_baby = tx
        .prepare("UPDATE evolution_chain SET baby_trigger_item_slug = ?1 WHERE id = ?2")
        .map_err(|error| format!("cannot prepare baby item update: {error}"))?;

    let mut step_id = 0_i64;
    for (_, path) in numeric_dirs(&staging.join("evolution-chain"))? {
        let chain: EvolutionChainJson = read_json(&path)?;

        if let Some(item) = &chain.baby_trigger_item {
            update_baby
                .execute(params![item.name, chain.id])
                .map_err(|error| format!("cannot update baby item: {error}"))?;
        }

        let Some(&root_id) = context.species.get(&chain.chain.species.name) else {
            continue;
        };
        walk_chain(
            &chain.chain,
            root_id,
            chain.id,
            context,
            &mut step_id,
            &mut insert_step,
        )?;
    }

    Ok(())
}

fn walk_chain(
    link: &ChainLink,
    from_id: i64,
    chain_id: i64,
    context: &Context,
    step_id: &mut i64,
    statement: &mut rusqlite::Statement<'_>,
) -> Result<(), String> {
    for child in &link.evolves_to {
        let Some(&to_id) = context.species.get(&child.species.name) else {
            continue;
        };

        for detail in &child.evolution_details {
            *step_id += 1;
            let extra = if detail.extra.is_empty() {
                None
            } else {
                serde_json::to_string(&detail.extra).ok()
            };
            let condition = detail
                .condition_expression
                .as_ref()
                .map(|value| match value {
                    serde_json::Value::String(text) => text.clone(),
                    other => other.to_string(),
                });

            statement
                .execute(params![
                    *step_id,
                    chain_id,
                    from_id,
                    to_id,
                    detail
                        .trigger
                        .as_ref()
                        .map_or("other", |trigger| trigger.name.as_str()),
                    detail.min_level,
                    detail.min_happiness,
                    detail.min_affection,
                    detail.min_beauty,
                    i64::from(detail.needs_overworld_rain),
                    if detail.time_of_day.is_empty() {
                        None
                    } else {
                        Some(detail.time_of_day.as_str())
                    },
                    detail.gender.map(gender_slug),
                    detail.held_item.as_ref().map(|item| item.name.as_str()),
                    detail.item.as_ref().map(|item| item.name.as_str()),
                    detail
                        .known_move
                        .as_ref()
                        .and_then(|move_| context.moves.get(&move_.name).copied()),
                    detail
                        .known_move_type
                        .as_ref()
                        .and_then(|type_| context.types.get(&type_.name).copied()),
                    detail
                        .party_species
                        .as_ref()
                        .and_then(|species| context.species.get(&species.name).copied()),
                    detail
                        .party_type
                        .as_ref()
                        .and_then(|type_| context.types.get(&type_.name).copied()),
                    detail
                        .trade_species
                        .as_ref()
                        .and_then(|species| context.species.get(&species.name).copied()),
                    detail
                        .location
                        .as_ref()
                        .map(|location| location.name.as_str()),
                    detail.relative_physical_stats,
                    i64::from(detail.turn_upside_down),
                    condition,
                    extra,
                ])
                .map_err(|error| format!("cannot insert evolution step: {error}"))?;
        }

        walk_chain(child, to_id, chain_id, context, step_id, statement)?;
    }

    Ok(())
}

// ---------------------------------------------------------------------------
// Metadata and helpers
// ---------------------------------------------------------------------------

fn insert_meta(
    tx: &rusqlite::Transaction<'_>,
    options: &BuildOptions,
    context: &Context,
) -> Result<(), String> {
    let tables = [
        "species",
        "pokemon_form",
        "form_type",
        "form_stat",
        "form_stat_summary",
        "ability",
        "move_",
        "form_move",
        "evolution_chain",
        "evolution_step",
        "translation",
        "flavor_text",
        "type_efficacy",
    ];

    let mut counts = serde_json::Map::new();
    for table in tables {
        let count: i64 = tx
            .query_row(&format!("SELECT COUNT(*) FROM {table}"), [], |row| {
                row.get(0)
            })
            .map_err(|error| format!("cannot count {table}: {error}"))?;
        counts.insert(table.to_owned(), serde_json::Value::from(count));
    }

    let mut values: Vec<(&str, String)> = vec![
        ("version", options.version.clone()),
        ("built_at", now_iso8601()),
        (
            "source_commit",
            options.source_commit.clone().unwrap_or_default(),
        ),
        (
            "source_lock",
            options.source_lock.clone().unwrap_or_default(),
        ),
        ("schema_version", "2".to_owned()),
        ("langs", context.langs.join(",")),
        (
            "row_counts_json",
            serde_json::Value::Object(counts).to_string(),
        ),
    ];
    values.sort_by_key(|(key, _)| *key);

    for (key, value) in values {
        tx.execute(
            "INSERT OR REPLACE INTO dataset_meta (key, value) VALUES (?1, ?2)",
            params![key, value],
        )
        .map_err(|error| format!("cannot insert dataset metadata {key}: {error}"))?;
    }

    Ok(())
}

fn read_row_counts(connection: &Connection) -> Result<Vec<(String, i64)>, String> {
    let tables = [
        "species",
        "pokemon_form",
        "form_type",
        "form_stat",
        "ability",
        "move_",
        "form_move",
        "evolution_chain",
        "evolution_step",
        "translation",
        "flavor_text",
        "type_efficacy",
    ];

    let mut counts = Vec::new();
    for table in tables {
        let count: i64 = connection
            .query_row(&format!("SELECT COUNT(*) FROM {table}"), [], |row| {
                row.get(0)
            })
            .map_err(|error| format!("cannot count {table}: {error}"))?;
        counts.push((table.to_owned(), count));
    }
    Ok(counts)
}

fn put_translation(
    tx: &rusqlite::Transaction<'_>,
    entity_type: &str,
    entity_id: i64,
    lang: &str,
    field: &str,
    value: &str,
) -> Result<(), String> {
    if value.trim().is_empty() {
        return Ok(());
    }

    tx.execute(
        "INSERT OR REPLACE INTO translation (entity_type, entity_id, lang, field, value)
         VALUES (?1, ?2, ?3, ?4, ?5)",
        params![entity_type, entity_id, lang, field, value],
    )
    .map_err(|error| format!("cannot insert translation {entity_type}/{entity_id}: {error}"))?;

    Ok(())
}

fn index_entity(
    tx: &rusqlite::Transaction<'_>,
    entity_type: &str,
    entity_id: i64,
    name: &str,
    normalized: &str,
    aliases: &str,
) -> Result<(), String> {
    tx.execute(
        "INSERT INTO search_index (entity_type, entity_id, name, name_normalized, aliases)
         VALUES (?1, ?2, ?3, ?4, ?5)",
        params![
            entity_type,
            entity_id.to_string(),
            name,
            normalized,
            aliases
        ],
    )
    .map_err(|error| format!("cannot index {entity_type}/{entity_id}: {error}"))?;

    Ok(())
}

fn localized_name<'a>(names: &'a [LocalizedName], lang: &str) -> Option<&'a str> {
    names
        .iter()
        .find(|name| name.language.name == lang)
        .map(|name| name.name.as_str())
}

fn last_flavor<T: FlavorEntry>(entries: &[T], lang: &str) -> Option<String> {
    entries
        .iter()
        .rev()
        .find(|entry| entry.language() == lang)
        .map(|entry| clean_text(entry.text()))
        .filter(|text| !text.is_empty())
}

trait FlavorEntry {
    fn language(&self) -> &str;
    fn text(&self) -> &str;
}

impl FlavorEntry for AbilityFlavorText {
    fn language(&self) -> &str {
        &self.language.name
    }

    fn text(&self) -> &str {
        &self.flavor_text
    }
}

impl FlavorEntry for MoveFlavorText {
    fn language(&self) -> &str {
        &self.language.name
    }

    fn text(&self) -> &str {
        &self.flavor_text
    }
}

/// Pseudo-types used by PokeAPI for spin-offs; outside the 18-type MVP chart.
fn is_pseudo_type(slug: &str) -> bool {
    matches!(slug, "shadow" | "unknown")
}

fn stat_slug(stat: &str) -> Option<&'static str> {
    match stat {
        "hp" => Some("hp"),
        "attack" => Some("attack"),
        "defense" => Some("defense"),
        "special-attack" => Some("sp_atk"),
        "special-defense" => Some("sp_def"),
        "speed" => Some("speed"),
        _ => None,
    }
}

fn gender_slug(gender: i64) -> &'static str {
    match gender {
        1 => "female",
        2 => "male",
        _ => "genderless",
    }
}

fn form_generation(slug: &str) -> Option<i64> {
    if slug.contains("-mega") {
        Some(6)
    } else if slug.ends_with("-gmax") {
        Some(8)
    } else if slug.contains("-alola") || slug.contains("-totem") {
        Some(7)
    } else if slug.contains("-galar") || slug.contains("-hisui") {
        Some(8)
    } else if slug.contains("-paldea") {
        Some(9)
    } else {
        None
    }
}

fn regional_region(slug: &str) -> Option<&'static str> {
    if slug.contains("-alola") {
        Some("alola")
    } else if slug.contains("-galar") {
        Some("galar")
    } else if slug.contains("-hisui") {
        Some("hisui")
    } else if slug.contains("-paldea") {
        Some("paldea")
    } else {
        None
    }
}

fn is_ultra_beast(slug: &str) -> bool {
    matches!(
        slug,
        "nihilego"
            | "buzzwole"
            | "pheromosa"
            | "xurkitree"
            | "celesteela"
            | "kartana"
            | "guzzlord"
            | "poipole"
            | "naganadel"
            | "stakataka"
            | "blacephalon"
    )
}

fn human_size(bytes: u64) -> String {
    const MIB: u64 = 1024 * 1024;
    if bytes >= MIB {
        format!("{:.1} MiB", bytes as f64 / MIB as f64)
    } else {
        format!("{:.1} KiB", bytes as f64 / 1024.0)
    }
}
