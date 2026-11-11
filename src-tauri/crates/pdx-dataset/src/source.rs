//! Source lockfile handling and acquisition.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use serde::Deserialize;

const DEFAULT_LOCKFILE: &str = "tools/dataset/sources.lock.toml";

/// A pinned source, as described by the annexe.
#[derive(Debug, Clone, Deserialize)]
pub struct Source {
    pub id: String,
    pub kind: String,
    #[serde(default)]
    pub repo: Option<String>,
    #[serde(default)]
    pub commit: Option<String>,
    #[serde(default)]
    pub subpath: Option<String>,
    #[serde(default)]
    #[allow(dead_code)]
    pub package: Option<String>,
    #[serde(default)]
    pub version: Option<String>,
    #[serde(default)]
    pub license: String,
    #[serde(default)]
    pub bundled: bool,
    #[serde(default)]
    pub note: Option<String>,
}

/// Parsed `sources.lock.toml`.
#[derive(Debug, Clone, Deserialize)]
pub struct Lockfile {
    pub lockfile_version: u32,
    #[serde(default)]
    #[allow(dead_code)]
    pub generated_at: String,
    #[serde(default, rename = "source")]
    pub sources: Vec<Source>,
}

impl Lockfile {
    /// Loads the lockfile, using the repository default when no path is given.
    pub fn load(path: Option<&Path>) -> Result<Self, String> {
        let path = path.unwrap_or_else(|| Path::new(DEFAULT_LOCKFILE));
        let contents = fs::read_to_string(path)
            .map_err(|error| format!("cannot read {}: {error}", path.display()))?;
        toml::from_str(&contents)
            .map_err(|error| format!("invalid lockfile {}: {error}", path.display()))
    }

    /// Returns the single source marked as bundled (the dataset source).
    pub fn primary_source(&self) -> Result<&Source, String> {
        let mut bundled = self.sources.iter().filter(|source| source.bundled);
        let source = bundled
            .next()
            .ok_or_else(|| "the lockfile has no bundled source".to_owned())?;

        if bundled.next().is_some() {
            return Err("the lockfile declares more than one bundled source".to_owned());
        }

        Ok(source)
    }
}

/// Default cache directory, overridable through `PDX_DATASET_CACHE`.
#[must_use]
pub fn default_cache_dir() -> PathBuf {
    if let Some(path) = std::env::var_os("PDX_DATASET_CACHE") {
        return PathBuf::from(path);
    }

    let home = std::env::var_os("USERPROFILE")
        .or_else(|| std::env::var_os("HOME"))
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("."));

    home.join(".cache").join("pdx-dataset")
}

/// Directory holding the checked-out source at its pinned commit.
#[must_use]
pub fn checkout_dir(cache: &Path, source: &Source) -> PathBuf {
    let commit = source.commit.as_deref().unwrap_or("unknown");
    let short: String = commit.chars().take(12).collect();
    cache.join(format!("{}-{short}", source.id))
}

/// Returns the staging path (the PokeAPI `data/api/v2` directory).
pub fn staging_path(cache: &Path, source: &Source) -> Result<PathBuf, String> {
    let mut path = checkout_dir(cache, source);
    if let Some(subpath) = source.subpath.as_deref() {
        path = path.join(subpath);
    }

    if !path.is_dir() {
        return Err(format!(
            "source `{}` is not fetched yet: run `pdx-dataset fetch` first",
            source.id
        ));
    }

    Ok(path)
}

/// Fetches (or verifies) every bundled source and returns its staging path.
pub fn ensure_sources(cache: &Path, lock: &Lockfile) -> Result<PathBuf, String> {
    let source = lock.primary_source()?;
    ensure_source(cache, source)?;
    staging_path(cache, source)
}

fn ensure_source(cache: &Path, source: &Source) -> Result<(), String> {
    match source.kind.as_str() {
        "git" => ensure_git_source(cache, source),
        other => Err(format!(
            "source kind `{other}` is not supported yet by pdx-dataset (source `{}`)",
            source.id
        )),
    }
}

fn ensure_git_source(cache: &Path, source: &Source) -> Result<(), String> {
    let repo = source
        .repo
        .as_deref()
        .ok_or_else(|| format!("source `{}` has no repository", source.id))?;
    let commit = source
        .commit
        .as_deref()
        .ok_or_else(|| format!("source `{}` has no pinned commit", source.id))?;

    if commit.len() != 40
        || !commit
            .chars()
            .all(|character| character.is_ascii_hexdigit())
    {
        return Err(format!(
            "source `{}` has an invalid commit `{commit}` (40 hexadecimal characters expected)",
            source.id
        ));
    }

    let dir = checkout_dir(cache, source);
    if dir.join(".git").is_dir() {
        let head = git(&dir, &["rev-parse", "HEAD"])?;
        if head.trim() == commit {
            println!("source `{}`: cache hit at {}", source.id, &commit[..12]);
            return Ok(());
        }
    }

    if dir.exists() {
        fs::remove_dir_all(&dir)
            .map_err(|error| format!("cannot clean {}: {error}", dir.display()))?;
    }
    fs::create_dir_all(&dir)
        .map_err(|error| format!("cannot create {}: {error}", dir.display()))?;

    println!(
        "source `{}`: fetching {} at {}",
        source.id,
        repo,
        &commit[..12]
    );
    git(&dir, &["init", "--quiet"])?;
    git(&dir, &["remote", "add", "origin", repo])?;
    git(
        &dir,
        &["fetch", "--quiet", "--depth", "1", "origin", commit],
    )?;
    git(
        &dir,
        &[
            "-c",
            "advice.detachedHead=false",
            "checkout",
            "--quiet",
            "FETCH_HEAD",
        ],
    )?;

    let head = git(&dir, &["rev-parse", "HEAD"])?;
    if head.trim() != commit {
        return Err(format!(
            "source `{}`: checked out {} instead of the pinned {commit}",
            source.id,
            head.trim()
        ));
    }

    Ok(())
}

fn git(dir: &Path, args: &[&str]) -> Result<String, String> {
    let output = Command::new("git")
        .arg("-C")
        .arg(dir)
        .args(args)
        .output()
        .map_err(|error| {
            format!(
                "cannot run git (is it installed and on PATH?): {error} (source directory {})",
                dir.display()
            )
        })?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!(
            "git {} failed in {}: {}",
            args.join(" "),
            dir.display(),
            stderr.trim()
        ));
    }

    Ok(String::from_utf8_lossy(&output.stdout).into_owned())
}

const ALLOWED_LICENSES: &[&str] = &[
    "MIT",
    "Apache-2.0",
    "BSD-2-Clause",
    "BSD-3-Clause",
    "CC0-1.0",
    "CC-BY-4.0",
    "ISC",
    "AGPL-3.0-or-later",
    "GPL-3.0-or-later",
    "Unlicense",
];

/// Fails when a source is unverified or not permissively licensed.
pub fn check_licenses(lock: &Lockfile) -> Result<(), String> {
    let mut problems = Vec::new();

    for source in &lock.sources {
        if source.license.trim().is_empty() || source.license == "TO_VERIFY" {
            problems.push(format!(
                "source `{}` has no verified license (license = \"{}\")",
                source.id, source.license
            ));
            continue;
        }

        if !ALLOWED_LICENSES.contains(&source.license.as_str()) {
            problems.push(format!(
                "source `{}` uses unreviewed license `{}`",
                source.id, source.license
            ));
        }
    }

    if problems.is_empty() {
        Ok(())
    } else {
        Err(problems.join("\n"))
    }
}

/// Structural verification of the lockfile (LCK-01 / LCK-05 prerequisites).
pub fn verify_lock(lock: &Lockfile) -> Result<(), String> {
    if lock.lockfile_version != 1 {
        return Err(format!(
            "unsupported lockfile version {}",
            lock.lockfile_version
        ));
    }

    if lock.sources.is_empty() {
        return Err("the lockfile declares no source".to_owned());
    }

    lock.primary_source()?;

    for source in &lock.sources {
        if source.kind == "git" {
            let commit = source
                .commit
                .as_deref()
                .ok_or_else(|| format!("source `{}` has no commit", source.id))?;
            if commit.len() != 40 {
                return Err(format!(
                    "source `{}` commit must be a full 40-character SHA",
                    source.id
                ));
            }
            if source.repo.is_none() {
                return Err(format!("source `{}` has no repository", source.id));
            }
        }
    }

    check_licenses(lock)
}
