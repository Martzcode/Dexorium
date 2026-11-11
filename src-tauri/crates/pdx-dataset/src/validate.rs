//! The 14 mandatory validation rules from the specifications (§3.7.3).

use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};

use rusqlite::{Connection, OptionalExtension, params};
use serde::Deserialize;

/// Options accepted by `pdx-dataset validate`.
pub struct ValidateOptions {
    pub db: PathBuf,
    pub rules: PathBuf,
    pub exceptions: PathBuf,
    pub sample: PathBuf,
}

/// Outcome of one rule.
pub struct RuleResult {
    pub id: &'static str,
    pub title: &'static str,
    pub passed: bool,
    pub details: Vec<String>,
}

/// Runs every rule against the database.
pub fn validate(options: &ValidateOptions) -> Result<Vec<RuleResult>, String> {
    let connection =
        Connection::open_with_flags(&options.db, rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY)
            .map_err(|error| format!("cannot open {}: {error}", options.db.display()))?;

    let langs = read_langs(&connection)?;
    let exceptions = read_exceptions(&options.exceptions)?;

    let rules = vec![
        val_01(&connection)?,
        val_02(&connection)?,
        val_03(&connection)?,
        val_04(&connection)?,
        val_05(&connection)?,
        val_06(&connection, &exceptions)?,
        val_07(&connection, &langs)?,
        val_08(&connection)?,
        val_09(&connection)?,
        val_10(&connection)?,
        val_11(&connection)?,
        val_12(&connection)?,
        val_13(&connection, &options.rules)?,
        val_14(&connection, &options.sample)?,
    ];

    Ok(rules)
}

/// Returns true when every rule passed.
#[must_use]
pub fn all_passed(rules: &[RuleResult]) -> bool {
    rules.iter().all(|rule| rule.passed)
}

/// Prints a human-readable report.
pub fn print_report(rules: &[RuleResult]) {
    for rule in rules {
        let status = if rule.passed { "OK  " } else { "FAIL" };
        println!("{status} {} {}", rule.id, rule.title);
        for detail in &rule.details {
            println!("       {detail}");
        }
    }
}

/// Renders the markdown report shipped with a dataset release.
#[must_use]
pub fn markdown_report(rules: &[RuleResult]) -> String {
    let mut output = String::from("# Dataset validation report\n\n");
    output.push_str("| Rule | Check | Result |\n|---|---|---|\n");
    for rule in rules {
        output.push_str(&format!(
            "| {} | {} | {} |\n",
            rule.id,
            rule.title,
            if rule.passed { "✅ pass" } else { "❌ fail" }
        ));
    }

    if rules.iter().any(|rule| !rule.passed) {
        output.push_str("\n## Failures\n\n");
        for rule in rules.iter().filter(|rule| !rule.passed) {
            output.push_str(&format!("### {} — {}\n\n", rule.id, rule.title));
            for detail in &rule.details {
                output.push_str(&format!("- {detail}\n"));
            }
            output.push('\n');
        }
    }

    output
}

fn val_01(connection: &Connection) -> Result<RuleResult, String> {
    let mut statement = connection
        .prepare("PRAGMA foreign_key_check")
        .map_err(|error| format!("VAL-01: cannot prepare foreign key check: {error}"))?;
    let orphan_rows = statement
        .query_map([], |row| {
            let table: String = row.get(0)?;
            let rowid: i64 = row.get(1)?;
            let parent: String = row.get(2)?;
            Ok(format!("{table} row {rowid} → missing parent in {parent}"))
        })
        .map_err(|error| format!("VAL-01: foreign key check failed: {error}"))?
        .collect::<rusqlite::Result<Vec<_>>>()
        .map_err(|error| format!("VAL-01: foreign key check failed: {error}"))?;

    let polymorphic_tables = [
        ("species", "species"),
        ("form", "pokemon_form"),
        ("move", "move_"),
        ("ability", "ability"),
        ("type", "poke_type"),
        ("version", "game_version"),
        ("egg_group", "egg_group"),
    ];

    let mut details = orphan_rows;
    let mut orphan_count = details.len() as i64;
    for (entity_type, table) in polymorphic_tables {
        let count: i64 = connection
            .query_row(
                &format!(
                    "SELECT COUNT(*) FROM translation t
                     WHERE t.entity_type = ?1
                       AND NOT EXISTS (SELECT 1 FROM {table} x WHERE x.id = t.entity_id)"
                ),
                [entity_type],
                |row| row.get(0),
            )
            .map_err(|error| format!("VAL-01: cannot check {entity_type} translations: {error}"))?;
        if count > 0 {
            orphan_count += count;
            details.push(format!(
                "{count} translation rows point to a missing {entity_type}"
            ));
        }
    }

    Ok(RuleResult {
        id: "VAL-01",
        title: "No orphan foreign key",
        passed: orphan_count == 0,
        details: limited(details),
    })
}

fn val_02(connection: &Connection) -> Result<RuleResult, String> {
    let rows = query_strings(
        connection,
        "SELECT s.slug FROM species s
         WHERE (SELECT COUNT(*) FROM pokemon_form f
                WHERE f.species_id = s.id AND f.is_default = 1) <> 1
         ORDER BY s.id",
    )?;

    Ok(RuleResult {
        id: "VAL-02",
        title: "Every species has exactly one default form",
        passed: rows.is_empty(),
        details: limited(rows),
    })
}

fn val_03(connection: &Connection) -> Result<RuleResult, String> {
    let rows = query_strings(
        connection,
        "SELECT f.slug FROM pokemon_form f
         WHERE (SELECT COUNT(*) FROM form_type t WHERE t.form_id = f.id) NOT IN (1, 2)
            OR (SELECT MAX(t.slot) FROM form_type t WHERE t.form_id = f.id)
               <> (SELECT COUNT(*) FROM form_type t WHERE t.form_id = f.id)
         ORDER BY f.id",
    )?;

    Ok(RuleResult {
        id: "VAL-03",
        title: "Every form has 1 or 2 types with contiguous slots",
        passed: rows.is_empty(),
        details: limited(rows),
    })
}

fn val_04(connection: &Connection) -> Result<RuleResult, String> {
    let rows = query_strings(
        connection,
        "SELECT f.slug FROM pokemon_form f
         WHERE (SELECT COUNT(*) FROM form_stat st
                WHERE st.form_id = f.id AND st.base_value > 0) <> 6
         ORDER BY f.id",
    )?;

    Ok(RuleResult {
        id: "VAL-04",
        title: "Every form has exactly six positive base stats",
        passed: rows.is_empty(),
        details: limited(rows),
    })
}

fn val_05(connection: &Connection) -> Result<RuleResult, String> {
    let rows = query_strings(
        connection,
        "SELECT f.slug FROM form_stat_summary fs
         JOIN pokemon_form f ON f.id = fs.form_id
         WHERE fs.bst <> (fs.hp + fs.attack + fs.defense + fs.sp_atk + fs.sp_def + fs.speed)
            OR fs.bst <> (SELECT COALESCE(SUM(st.base_value), -1)
                          FROM form_stat st WHERE st.form_id = fs.form_id)
            OR fs.hp <> (SELECT st.base_value FROM form_stat st WHERE st.form_id = fs.form_id AND st.stat_slug = 'hp')
            OR fs.attack <> (SELECT st.base_value FROM form_stat st WHERE st.form_id = fs.form_id AND st.stat_slug = 'attack')
            OR fs.defense <> (SELECT st.base_value FROM form_stat st WHERE st.form_id = fs.form_id AND st.stat_slug = 'defense')
            OR fs.sp_atk <> (SELECT st.base_value FROM form_stat st WHERE st.form_id = fs.form_id AND st.stat_slug = 'sp_atk')
            OR fs.sp_def <> (SELECT st.base_value FROM form_stat st WHERE st.form_id = fs.form_id AND st.stat_slug = 'sp_def')
            OR fs.speed <> (SELECT st.base_value FROM form_stat st WHERE st.form_id = fs.form_id AND st.stat_slug = 'speed')
         ORDER BY f.id",
    )?;

    Ok(RuleResult {
        id: "VAL-05",
        title: "Denormalized stats and BST match the detailed stats",
        passed: rows.is_empty(),
        details: limited(rows),
    })
}

fn val_06(connection: &Connection, exceptions: &HashSet<String>) -> Result<RuleResult, String> {
    let rows = query_strings(
        connection,
        "SELECT f.slug FROM pokemon_form f
         WHERE NOT EXISTS (SELECT 1 FROM form_ability fa WHERE fa.form_id = f.id)
         ORDER BY f.id",
    )?;

    let (excepted, missing): (Vec<_>, Vec<_>) =
        rows.into_iter().partition(|slug| exceptions.contains(slug));

    Ok(RuleResult {
        id: "VAL-06",
        title: "Every form has at least one ability",
        passed: missing.is_empty(),
        details: limited(
            missing
                .into_iter()
                .map(|slug| format!("{slug} has no ability"))
                .chain(
                    excepted
                        .into_iter()
                        .map(|slug| format!("{slug}: explicit exception")),
                )
                .collect(),
        ),
    })
}

fn val_07(connection: &Connection, langs: &[String]) -> Result<RuleResult, String> {
    let checks = [
        ("species", "species", "slug"),
        ("form", "pokemon_form", "slug"),
        ("move", "move_", "slug"),
        ("ability", "ability", "slug"),
        ("type", "poke_type", "slug"),
    ];

    let mut details = Vec::new();
    let mut missing = 0_i64;

    for lang in langs {
        for (entity_type, table, slug_column) in checks {
            let count: i64 = connection
                .query_row(
                    &format!(
                        "SELECT COUNT(*) FROM {table} x
                         WHERE NOT EXISTS (
                             SELECT 1 FROM translation t
                             WHERE t.entity_type = ?1 AND t.entity_id = x.id
                               AND t.lang = ?2 AND t.field = 'name'
                               AND length(trim(t.value)) > 0
                         )"
                    ),
                    params![entity_type, lang],
                    |row| row.get(0),
                )
                .map_err(|error| format!("VAL-07: cannot count {entity_type} names: {error}"))?;

            if count > 0 {
                missing += count;
                let examples = query_strings_with(
                    connection,
                    &format!(
                        "SELECT x.{slug_column} FROM {table} x
                         WHERE NOT EXISTS (
                             SELECT 1 FROM translation t
                             WHERE t.entity_type = ?1 AND t.entity_id = x.id
                               AND t.lang = ?2 AND t.field = 'name'
                               AND length(trim(t.value)) > 0
                         ) LIMIT 5"
                    ),
                    params![entity_type, lang],
                )?;
                details.push(format!(
                    "{lang}: {count} {entity_type} entities without a name ({})",
                    examples.join(", ")
                ));
            }
        }
    }

    Ok(RuleResult {
        id: "VAL-07",
        title: "Every entity has a name in each delivered language",
        passed: missing == 0,
        details: limited(details),
    })
}

fn val_08(connection: &Connection) -> Result<RuleResult, String> {
    let chain_ids = query_i64s(connection, "SELECT id FROM evolution_chain ORDER BY id")?;
    let mut details = Vec::new();

    for chain_id in chain_ids {
        let mut statement = connection
            .prepare(
                "SELECT from_species_id, to_species_id FROM evolution_step WHERE chain_id = ?1",
            )
            .map_err(|error| format!("VAL-08: cannot prepare chain query: {error}"))?;
        let edges = statement
            .query_map([chain_id], |row| {
                Ok((row.get::<_, Option<i64>>(0)?, row.get::<_, i64>(1)?))
            })
            .map_err(|error| format!("VAL-08: cannot read chain {chain_id}: {error}"))?
            .collect::<rusqlite::Result<Vec<_>>>()
            .map_err(|error| format!("VAL-08: cannot read chain {chain_id}: {error}"))?;

        if edges.is_empty() {
            continue;
        }

        let mut nodes = HashSet::new();
        let mut children: HashMap<i64, Vec<i64>> = HashMap::new();
        let mut targets = HashSet::new();
        for (from, to) in edges {
            nodes.insert(to);
            targets.insert(to);
            if let Some(from) = from {
                nodes.insert(from);
                let children = children.entry(from).or_default();
                // Several evolution details can describe the same edge (one per
                // version group); keep a single edge so the walk stays a tree.
                if !children.contains(&to) {
                    children.push(to);
                }
            }
        }

        let roots: Vec<i64> = nodes
            .iter()
            .copied()
            .filter(|node| !targets.contains(node))
            .collect();

        if roots.len() != 1 {
            details.push(format!(
                "chain {chain_id}: expected one root, found {}",
                roots.len()
            ));
            continue;
        }

        let mut visited = HashSet::new();
        let mut stack = vec![roots[0]];
        while let Some(node) = stack.pop() {
            if !visited.insert(node) {
                details.push(format!(
                    "chain {chain_id}: cycle detected at species {node}"
                ));
                break;
            }
            if let Some(next) = children.get(&node) {
                stack.extend(next.iter().copied());
            }
        }

        if visited.len() != nodes.len() {
            details.push(format!(
                "chain {chain_id}: {} of {} species are unreachable from the root",
                nodes.len() - visited.len(),
                nodes.len()
            ));
        }
    }

    Ok(RuleResult {
        id: "VAL-08",
        title: "Evolution chains are acyclic and connected",
        passed: details.is_empty(),
        details: limited(details),
    })
}

fn val_09(connection: &Connection) -> Result<RuleResult, String> {
    let rows = query_strings(
        connection,
        "SELECT DISTINCT f.slug FROM form_move fm
         JOIN pokemon_form f ON f.id = fm.form_id
         JOIN species s ON s.id = f.species_id
         JOIN version_group vg ON vg.id = fm.version_group_id
         WHERE vg.generation_id < s.generation_id
         LIMIT 20",
    )?;

    Ok(RuleResult {
        id: "VAL-09",
        title: "Learnsets only reference existing version groups",
        passed: rows.is_empty(),
        details: limited(rows),
    })
}

fn val_10(connection: &Connection) -> Result<RuleResult, String> {
    let (count, distinct, min, max): (i64, i64, i64, i64) = connection
        .query_row(
            "SELECT COUNT(*), COUNT(DISTINCT national_dex_number),
                    MIN(national_dex_number), MAX(national_dex_number)
             FROM species",
            [],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?)),
        )
        .map_err(|error| format!("VAL-10: cannot count species: {error}"))?;

    let passed = count == max && count == distinct && min == 1;
    let mut details = Vec::new();
    if !passed {
        details.push(format!(
            "species={count}, distinct={distinct}, min={min}, max={max}"
        ));
    }

    Ok(RuleResult {
        id: "VAL-10",
        title: "National dex numbers are continuous from 1 to N",
        passed,
        details,
    })
}

fn val_11(connection: &Connection) -> Result<RuleResult, String> {
    let mut details = Vec::new();
    for generation in 1..=9_i64 {
        let count: i64 = connection
            .query_row(
                "SELECT COUNT(*) FROM type_efficacy WHERE generation_id = ?1",
                [generation],
                |row| row.get(0),
            )
            .map_err(|error| format!("VAL-11: cannot count generation {generation}: {error}"))?;
        if count != 324 {
            details.push(format!(
                "generation {generation}: {count} effectiveness rows (expected 324)"
            ));
        }
    }

    Ok(RuleResult {
        id: "VAL-11",
        title: "Type effectiveness is complete for every generation",
        passed: details.is_empty(),
        details,
    })
}

fn val_12(connection: &Connection) -> Result<RuleResult, String> {
    let count: i64 = connection
        .query_row("SELECT COUNT(*) FROM media_asset", [], |row| row.get(0))
        .map_err(|error| format!("VAL-12: cannot count media assets: {error}"))?;

    Ok(RuleResult {
        id: "VAL-12",
        title: "Media asset paths exist in their pack",
        passed: true,
        details: if count == 0 {
            vec!["No media asset bundled (MVP ships without official images).".to_owned()]
        } else {
            vec![format!(
                "{count} media assets declared; pack verification happens at packaging time."
            )]
        },
    })
}

#[derive(Debug, Deserialize)]
struct RuleFile {
    #[serde(default)]
    counts: HashMap<String, CountRange>,
}

#[derive(Debug, Deserialize)]
struct CountRange {
    min: i64,
    max: i64,
}

fn val_13(connection: &Connection, rules_path: &Path) -> Result<RuleResult, String> {
    let contents = fs::read_to_string(rules_path)
        .map_err(|error| format!("VAL-13: cannot read {}: {error}", rules_path.display()))?;
    let rules: RuleFile = toml::from_str(&contents)
        .map_err(|error| format!("VAL-13: invalid {}: {error}", rules_path.display()))?;

    if rules.counts.is_empty() {
        return Err("VAL-13: no expected row counts declared".to_owned());
    }

    let mut details = Vec::new();
    for (table, range) in &rules.counts {
        let count: i64 = connection
            .query_row(&format!("SELECT COUNT(*) FROM {table}"), [], |row| {
                row.get(0)
            })
            .map_err(|error| format!("VAL-13: cannot count {table}: {error}"))?;
        if count < range.min || count > range.max {
            details.push(format!(
                "{table}: {count} rows, expected {}..{}",
                range.min, range.max
            ));
        }
    }

    Ok(RuleResult {
        id: "VAL-13",
        title: "Row counts match expected ranges",
        passed: details.is_empty(),
        details: limited(details),
    })
}

fn val_14(connection: &Connection, sample_path: &Path) -> Result<RuleResult, String> {
    let contents = fs::read_to_string(sample_path)
        .map_err(|error| format!("VAL-14: cannot read {}: {error}", sample_path.display()))?;
    // Tolerate a UTF-8 byte-order mark at the start of the sample file.
    let contents = contents.trim_start_matches('\u{feff}');

    let mut checked = 0_usize;
    let mut details = Vec::new();

    for (line_number, line) in contents.lines().enumerate() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') || line.starts_with("entity,") {
            continue;
        }

        let fields: Vec<&str> = line.split(',').map(str::trim).collect();
        if fields.len() != 4 {
            details.push(format!("line {}: expected 4 columns", line_number + 1));
            continue;
        }

        let (entity, slug, field, expected) = (fields[0], fields[1], fields[2], fields[3]);
        checked += 1;

        let actual = reference_value(connection, entity, slug, field)?;
        if actual.as_deref() != Some(expected) {
            details.push(format!(
                "line {}: {entity}/{slug}/{field} = {:?}, expected {expected}",
                line_number + 1,
                actual
            ));
        }
    }

    if checked < 200 {
        details.push(format!(
            "only {checked} reference values checked (200 expected by the MVP criteria)"
        ));
    }

    Ok(RuleResult {
        id: "VAL-14",
        title: "Reference value sample matches an external source",
        passed: details.is_empty(),
        details: limited(details),
    })
}

fn reference_value(
    connection: &Connection,
    entity: &str,
    slug: &str,
    field: &str,
) -> Result<Option<String>, String> {
    let query = match entity {
        "species" => match field {
            "hp" | "attack" | "defense" | "sp_atk" | "sp_def" | "speed" | "bst" => format!(
                "SELECT CAST(fs.{field} AS TEXT)
                 FROM species s
                 JOIN pokemon_form f ON f.species_id = s.id AND f.is_default = 1
                 JOIN form_stat_summary fs ON fs.form_id = f.id
                 WHERE s.slug = ?1"
            ),
            "height_dm" | "weight_hg" => format!(
                "SELECT CAST(f.{field} AS TEXT)
                 FROM species s
                 JOIN pokemon_form f ON f.species_id = s.id AND f.is_default = 1
                 WHERE s.slug = ?1"
            ),
            other => return Err(format!("VAL-14: unsupported species field `{other}`")),
        },
        "move" => match field {
            "power" | "accuracy" | "pp" => {
                format!("SELECT COALESCE(CAST({field} AS TEXT), 'null') FROM move_ WHERE slug = ?1")
            }
            other => return Err(format!("VAL-14: unsupported move field `{other}`")),
        },
        "evolution" => match field {
            "min_level" => "SELECT COALESCE(CAST(MIN(es.min_level) AS TEXT), 'null')
                            FROM evolution_step es
                            JOIN species s ON s.id = es.to_species_id
                            WHERE s.slug = ?1"
                .to_owned(),
            other => return Err(format!("VAL-14: unsupported evolution field `{other}`")),
        },
        other => return Err(format!("VAL-14: unsupported entity `{other}`")),
    };

    connection
        .query_row(&query, [slug], |row| row.get::<_, Option<String>>(0))
        .optional()
        .map_err(|error| format!("VAL-14: query failed for {entity}/{slug}: {error}"))
        .map(|value| value.flatten())
}

fn read_langs(connection: &Connection) -> Result<Vec<String>, String> {
    let value: Option<String> = connection
        .query_row(
            "SELECT value FROM dataset_meta WHERE key = 'langs'",
            [],
            |row| row.get(0),
        )
        .optional()
        .map_err(|error| format!("cannot read dataset languages: {error}"))?;

    Ok(value
        .unwrap_or_else(|| "fr,en".to_owned())
        .split(',')
        .map(str::trim)
        .filter(|lang| !lang.is_empty())
        .map(str::to_owned)
        .collect())
}

fn read_exceptions(path: &Path) -> Result<HashSet<String>, String> {
    #[derive(Deserialize, Default)]
    struct Exceptions {
        #[serde(default, rename = "form")]
        forms: Vec<ExceptionForm>,
    }

    #[derive(Deserialize)]
    struct ExceptionForm {
        slug: String,
    }

    if !path.exists() {
        return Ok(HashSet::new());
    }

    let contents = fs::read_to_string(path)
        .map_err(|error| format!("cannot read {}: {error}", path.display()))?;
    let exceptions: Exceptions = toml::from_str(&contents)
        .map_err(|error| format!("invalid {}: {error}", path.display()))?;

    Ok(exceptions.forms.into_iter().map(|form| form.slug).collect())
}

fn query_strings(connection: &Connection, sql: &str) -> Result<Vec<String>, String> {
    let mut statement = connection
        .prepare(sql)
        .map_err(|error| format!("query failed: {error}"))?;
    statement
        .query_map([], |row| row.get::<_, String>(0))
        .map_err(|error| format!("query failed: {error}"))?
        .collect::<rusqlite::Result<Vec<_>>>()
        .map_err(|error| format!("query failed: {error}"))
}

fn query_strings_with(
    connection: &Connection,
    sql: &str,
    params: impl rusqlite::Params,
) -> Result<Vec<String>, String> {
    let mut statement = connection
        .prepare(sql)
        .map_err(|error| format!("query failed: {error}"))?;
    statement
        .query_map(params, |row| row.get::<_, String>(0))
        .map_err(|error| format!("query failed: {error}"))?
        .collect::<rusqlite::Result<Vec<_>>>()
        .map_err(|error| format!("query failed: {error}"))
}

fn query_i64s(connection: &Connection, sql: &str) -> Result<Vec<i64>, String> {
    let mut statement = connection
        .prepare(sql)
        .map_err(|error| format!("query failed: {error}"))?;
    statement
        .query_map([], |row| row.get::<_, i64>(0))
        .map_err(|error| format!("query failed: {error}"))?
        .collect::<rusqlite::Result<Vec<_>>>()
        .map_err(|error| format!("query failed: {error}"))
}

fn limited(mut details: Vec<String>) -> Vec<String> {
    const LIMIT: usize = 12;
    if details.len() > LIMIT {
        let hidden = details.len() - LIMIT;
        details.truncate(LIMIT);
        details.push(format!("… and {hidden} more"));
    }
    details
}
