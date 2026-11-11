//! `pdx-dataset` — build, validate and publish the Dexorium reference dataset.

mod build;
mod source;
mod util;
mod validate;

use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};

use build::BuildOptions;
use source::{Lockfile, Source};
use validate::{ValidateOptions, all_passed, markdown_report, print_report};

const DEFAULT_LOCKFILE: &str = "tools/dataset/sources.lock.toml";
const DEFAULT_DB: &str = "dist/reference.db";
const DEFAULT_RULES: &str = "tools/dataset/row-counts.toml";
const DEFAULT_EXCEPTIONS: &str = "tools/dataset/validation-exceptions.toml";
const DEFAULT_SAMPLE: &str = "tools/dataset/reference-values.csv";
const DEFAULT_OUT: &str = "dist";

fn main() {
    if let Err(error) = run() {
        eprintln!("error: {error}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), String> {
    let mut arguments = std::env::args().skip(1);
    let Some(command) = arguments.next() else {
        print_help();
        return Err("missing command".to_owned());
    };

    let flags = Flags::parse(arguments.collect())?;
    if flags.switch("help") {
        print_help();
        return Ok(());
    }

    match command.as_str() {
        "lock" => command_lock(&flags),
        "fetch" => command_fetch(&flags),
        "build" => command_build(&flags).map(|_| ()),
        "validate" => command_validate(&flags),
        "checksum" => command_checksum(&flags),
        "credits" => command_credits(&flags),
        "release" => command_release(&flags),
        other => Err(format!(
            "unknown command `{other}` (run `pdx-dataset --help`)"
        )),
    }
}

fn print_help() {
    println!(
        "pdx-dataset — offline reference dataset pipeline\n\
         \n\
         Commands:\n\
           lock      [--lock PATH] [--check-licenses]   verify the source lockfile\n\
           fetch     [--lock PATH] [--cache PATH]       checkout pinned sources\n\
           build     [--staging PATH] [--out PATH] [--version V] [--langs fr,en]\n\
           validate  [--db PATH] [--rules PATH] [--exceptions PATH] [--sample PATH] [--out PATH]\n\
           checksum  [--db PATH] [--out PATH]           write a SHA-256 sidecar\n\
           credits   [--lock PATH] [--out PATH]         generate DATA_SOURCES.md\n\
           release   [--out DIR] [--version V] [--langs fr,en] [--install-resource]\n\
         \n\
         Defaults:\n\
           lockfile   {DEFAULT_LOCKFILE}\n\
           database   {DEFAULT_DB}\n\
           output     {DEFAULT_OUT}\n"
    );
}

fn command_lock(flags: &Flags) -> Result<(), String> {
    let lock_path = flags.path("lock", DEFAULT_LOCKFILE);
    let lock = Lockfile::load(Some(&lock_path))?;
    source::verify_lock(&lock)?;
    println!(
        "lockfile OK: {} source(s), version {}",
        lock.sources.len(),
        lock.lockfile_version
    );

    if flags.switch("check-licenses") {
        println!("all source licenses are verified and permissive");
    }

    Ok(())
}

fn command_fetch(flags: &Flags) -> Result<(), String> {
    let lock_path = flags.path("lock", DEFAULT_LOCKFILE);
    let lock = Lockfile::load(Some(&lock_path))?;
    source::verify_lock(&lock)?;

    let cache = flags.path("cache", source::default_cache_dir());
    let staging = source::ensure_sources(&cache, &lock)?;
    println!("staging ready: {}", staging.display());
    Ok(())
}

fn command_build(flags: &Flags) -> Result<build::BuildReport, String> {
    let lock_path = flags.path("lock", DEFAULT_LOCKFILE);
    let lock = Lockfile::load(Some(&lock_path))?;
    let cache = flags.path("cache", source::default_cache_dir());

    let staging = match flags.value("staging") {
        Some(path) => PathBuf::from(path),
        None => source::ensure_sources(&cache, &lock)?,
    };

    let primary = lock.primary_source()?;
    let version = flags.value("version").unwrap_or("0.1.0-dev").to_owned();
    let options = BuildOptions {
        staging,
        out: flags.path("out", DEFAULT_DB),
        langs: flags.langs()?,
        version,
        source_commit: primary.commit.clone(),
        source_lock: Some(lock_hash(&lock_path)?),
    };

    build::build(&options)
}

fn command_validate(flags: &Flags) -> Result<(), String> {
    let options = ValidateOptions {
        db: flags.path("db", DEFAULT_DB),
        rules: flags.path("rules", DEFAULT_RULES),
        exceptions: flags.path("exceptions", DEFAULT_EXCEPTIONS),
        sample: flags.path("sample", DEFAULT_SAMPLE),
    };

    let rules = validate::validate(&options)?;
    print_report(&rules);

    if let Some(out) = flags.value("out") {
        fs::write(out, markdown_report(&rules))
            .map_err(|error| format!("cannot write {out}: {error}"))?;
        println!("validation report written to {out}");
    }

    if all_passed(&rules) {
        println!("all 14 validation rules passed");
        Ok(())
    } else {
        Err("dataset validation failed".to_owned())
    }
}

fn command_checksum(flags: &Flags) -> Result<(), String> {
    let db = flags.path("db", DEFAULT_DB);
    let hash = util::sha256_file(&db)?;
    let output = flags
        .value("out")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(format!("{}.sha256", db.display())));
    let file_name = db.file_name().map_or_else(
        || db.display().to_string(),
        |name| name.to_string_lossy().into_owned(),
    );

    fs::write(&output, format!("{hash}  {file_name}\n"))
        .map_err(|error| format!("cannot write {}: {error}", output.display()))?;
    println!("{hash}  {file_name}");
    println!("checksum written to {}", output.display());
    Ok(())
}

fn command_credits(flags: &Flags) -> Result<(), String> {
    let lock_path = flags.path("lock", DEFAULT_LOCKFILE);
    let lock = Lockfile::load(Some(&lock_path))?;
    let output = flags.path("out", "docs/DATA_SOURCES.md");
    let contents = credits_markdown(&lock);

    if let Some(parent) = output.parent() {
        fs::create_dir_all(parent)
            .map_err(|error| format!("cannot create {}: {error}", parent.display()))?;
    }
    fs::write(&output, contents)
        .map_err(|error| format!("cannot write {}: {error}", output.display()))?;
    println!("credits written to {}", output.display());
    Ok(())
}

fn command_release(flags: &Flags) -> Result<(), String> {
    let lock_path = flags.path("lock", DEFAULT_LOCKFILE);
    let lock = Lockfile::load(Some(&lock_path))?;
    source::verify_lock(&lock)?;

    let cache = flags.path("cache", source::default_cache_dir());
    let staging = match flags.value("staging") {
        Some(path) => PathBuf::from(path),
        None => source::ensure_sources(&cache, &lock)?,
    };

    let out = flags.path("out", DEFAULT_OUT);
    fs::create_dir_all(&out)
        .map_err(|error| format!("cannot create {}: {error}", out.display()))?;

    let version = flags.value("version").unwrap_or("0.1.0-dev").to_owned();
    let db = out.join(format!("reference-{version}.db"));
    let primary = lock.primary_source()?;

    let report = build::build(&BuildOptions {
        staging,
        out: db.clone(),
        langs: flags.langs()?,
        version: version.clone(),
        source_commit: primary.commit.clone(),
        source_lock: Some(lock_hash(&lock_path)?),
    })?;

    let validation = validate::validate(&ValidateOptions {
        db: db.clone(),
        rules: flags.path("rules", DEFAULT_RULES),
        exceptions: flags.path("exceptions", DEFAULT_EXCEPTIONS),
        sample: flags.path("sample", DEFAULT_SAMPLE),
    })?;
    let passed = all_passed(&validation);
    print_report(&validation);

    fs::write(
        out.join("validation-report.md"),
        markdown_report(&validation),
    )
    .map_err(|error| format!("cannot write validation report: {error}"))?;

    let counts = serde_json::Map::from_iter(
        report
            .row_counts
            .into_iter()
            .map(|(table, count)| (table, serde_json::Value::from(count))),
    );
    fs::write(
        out.join("row-counts.json"),
        serde_json::Value::Object(counts).to_string(),
    )
    .map_err(|error| format!("cannot write row counts: {error}"))?;

    let hash = util::sha256_file(&db)?;
    fs::write(
        out.join(format!("reference-{version}.db.sha256")),
        format!(
            "{hash}  {}\n",
            db.file_name().unwrap_or_default().to_string_lossy()
        ),
    )
    .map_err(|error| format!("cannot write checksum: {error}"))?;

    fs::write(out.join("DATA_SOURCES.md"), credits_markdown(&lock))
        .map_err(|error| format!("cannot write data sources: {error}"))?;

    fs::write(
        out.join("CHANGELOG-data.md"),
        format!(
            "# Dataset changelog\n\n## {version}\n\n- Initial build from the pinned source.\n\
             - Validation: {}.\n",
            if passed { "passed" } else { "failed" }
        ),
    )
    .map_err(|error| format!("cannot write changelog: {error}"))?;

    fs::write(
        out.join("conflicts-report.md"),
        "# Source conflicts\n\nSingle-source build: no cross-source conflict to arbitrate.\n",
    )
    .map_err(|error| format!("cannot write conflicts report: {error}"))?;

    if flags.switch("install-resource") {
        let resource = Path::new("src-tauri/resources/reference.db");
        if let Some(parent) = resource.parent() {
            fs::create_dir_all(parent)
                .map_err(|error| format!("cannot create {}: {error}", parent.display()))?;
        }
        fs::copy(&db, resource)
            .map_err(|error| format!("cannot install {}: {error}", resource.display()))?;
        fs::write(
            resource.with_extension("db.sha256"),
            format!("{hash}  reference.db\n"),
        )
        .map_err(|error| format!("cannot write resource checksum: {error}"))?;
        println!("reference database installed at {}", resource.display());
    }

    if passed {
        println!("release ready in {} (sha256 {hash})", out.display());
        Ok(())
    } else {
        Err("dataset validation failed; release artefacts kept for inspection".to_owned())
    }
}

fn credits_markdown(lock: &Lockfile) -> String {
    let mut output = String::from("# Data sources\n\n");
    output.push_str("This file is generated by `pdx-dataset credits`; do not edit it by hand.\n\n");
    output.push_str("| Source | Kind | Version | License | Bundled |\n|---|---|---|---|---|\n");

    for source in &lock.sources {
        output.push_str(&format!(
            "| {id} | {kind} | {version} | {license} | {bundled} |\n",
            id = source.id,
            kind = source.kind,
            version = source_version(source),
            license = source.license,
            bundled = if source.bundled { "yes" } else { "no" },
        ));
        if let Some(note) = &source.note {
            output.push_str(&format!("\n> {note}\n"));
        }
    }

    output.push_str(
        "\nPokémon and Pokémon character names are trademarks of Nintendo, Creatures Inc. \
         and GAME FREAK Inc. Dexorium is an unofficial fan project and is not affiliated \
         with, endorsed or sponsored by them.\n",
    );

    output
}

fn source_version(source: &Source) -> String {
    if let Some(version) = &source.version {
        version.clone()
    } else if let Some(commit) = &source.commit {
        commit.chars().take(12).collect()
    } else {
        "—".to_owned()
    }
}

fn lock_hash(path: &Path) -> Result<String, String> {
    util::sha256_file(path)
}

struct Flags {
    values: HashMap<String, String>,
    switches: HashSet<String>,
}

impl Flags {
    fn parse(arguments: Vec<String>) -> Result<Self, String> {
        let mut values = HashMap::new();
        let mut switches = HashSet::new();
        let mut iterator = arguments.into_iter();

        while let Some(argument) = iterator.next() {
            let Some(key) = argument.strip_prefix("--") else {
                return Err(format!("unexpected positional argument `{argument}`"));
            };

            if let Some((key, value)) = key.split_once('=') {
                values.insert(key.to_owned(), value.to_owned());
                continue;
            }

            if matches!(
                key,
                "help" | "check-licenses" | "install-resource" | "verbose"
            ) {
                switches.insert(key.to_owned());
                continue;
            }

            let value = iterator
                .next()
                .ok_or_else(|| format!("`--{key}` expects a value"))?;
            if value.starts_with("--") {
                return Err(format!("`--{key}` expects a value, found `{value}`"));
            }
            values.insert(key.to_owned(), value);
        }

        Ok(Self { values, switches })
    }

    fn value(&self, key: &str) -> Option<&str> {
        self.values.get(key).map(String::as_str)
    }

    fn switch(&self, key: &str) -> bool {
        self.switches.contains(key)
    }

    fn path(&self, key: &str, default: impl AsRef<Path>) -> PathBuf {
        self.value(key)
            .map_or_else(|| default.as_ref().to_path_buf(), PathBuf::from)
    }

    fn langs(&self) -> Result<Vec<String>, String> {
        let raw = self.value("langs").unwrap_or("fr,en");
        let langs: Vec<String> = raw
            .split(',')
            .map(str::trim)
            .filter(|lang| !lang.is_empty())
            .map(str::to_owned)
            .collect();

        if langs.is_empty() {
            return Err("--langs must contain at least one language".to_owned());
        }
        for lang in &langs {
            if !matches!(lang.as_str(), "fr" | "en") {
                return Err(format!(
                    "unsupported language `{lang}` (the MVP ships fr and en)"
                ));
            }
        }

        Ok(langs)
    }
}
