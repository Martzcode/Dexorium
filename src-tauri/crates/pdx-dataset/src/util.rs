//! Small filesystem, hashing and text helpers shared by the pipeline.

use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};

use serde::de::DeserializeOwned;
use sha2::{Digest, Sha256};

/// Reads and parses a JSON document, adding the path to any error message.
pub fn read_json<T: DeserializeOwned>(path: &Path) -> Result<T, String> {
    let bytes =
        fs::read(path).map_err(|error| format!("cannot read {}: {error}", path.display()))?;
    serde_json::from_slice(&bytes)
        .map_err(|error| format!("invalid JSON in {}: {error}", path.display()))
}

/// Lists the numeric resource directories of a PokeAPI endpoint, sorted.
pub fn numeric_dirs(dir: &Path) -> Result<Vec<(i64, PathBuf)>, String> {
    let entries =
        fs::read_dir(dir).map_err(|error| format!("cannot list {}: {error}", dir.display()))?;

    let mut resources = Vec::new();
    for entry in entries {
        let entry = entry.map_err(|error| format!("cannot read {}: {error}", dir.display()))?;
        let file_type = entry
            .file_type()
            .map_err(|error| format!("cannot stat {:?}: {error}", entry.path()))?;
        if !file_type.is_dir() {
            continue;
        }

        let Ok(id) = entry.file_name().to_string_lossy().parse::<i64>() else {
            continue;
        };
        resources.push((id, entry.path().join("index.json")));
    }

    resources.sort_by_key(|(id, _)| *id);
    Ok(resources)
}

/// Extracts the trailing numeric identifier of an API URL (`/api/v2/x/42/`).
pub fn id_from_url(url: &str) -> Result<i64, String> {
    url.trim_end_matches('/')
        .rsplit('/')
        .next()
        .and_then(|value| value.parse::<i64>().ok())
        .ok_or_else(|| format!("cannot extract an identifier from `{url}`"))
}

/// Normalizes text for the FTS index: lowercase, no accents, no punctuation.
pub fn normalize(value: &str) -> String {
    let mut normalized = String::with_capacity(value.len());

    for character in value.chars().flat_map(char::to_lowercase) {
        match character {
            'à' | 'á' | 'â' | 'ã' | 'ä' | 'å' | 'ā' => normalized.push('a'),
            'ç' => normalized.push('c'),
            'è' | 'é' | 'ê' | 'ë' | 'ē' => normalized.push('e'),
            'ì' | 'í' | 'î' | 'ï' | 'ī' => normalized.push('i'),
            'ñ' => normalized.push('n'),
            'ò' | 'ó' | 'ô' | 'õ' | 'ö' | 'ō' => normalized.push('o'),
            'ù' | 'ú' | 'û' | 'ü' | 'ū' => normalized.push('u'),
            'ý' | 'ÿ' => normalized.push('y'),
            'æ' => normalized.push_str("ae"),
            'œ' => normalized.push_str("oe"),
            'ß' => normalized.push_str("ss"),
            other if other.is_alphanumeric() => normalized.push(other),
            _other => normalized.push(' '),
        }
    }

    normalized.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// Cleans PokeAPI flavour text: control characters, soft hyphens, padding.
pub fn clean_text(value: &str) -> String {
    let replaced: String = value
        .chars()
        .map(|character| match character {
            '\n' | '\r' | '\u{000c}' | '\u{000b}' | '\u{00ad}' => ' ',
            other => other,
        })
        .collect();

    replaced.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// Computes the SHA-256 of a file, returned as lowercase hexadecimal.
pub fn sha256_file(path: &Path) -> Result<String, String> {
    let mut file =
        fs::File::open(path).map_err(|error| format!("cannot open {}: {error}", path.display()))?;
    let mut hasher = Sha256::new();
    let mut buffer = [0_u8; 64 * 1024];

    loop {
        let read = file
            .read(&mut buffer)
            .map_err(|error| format!("cannot read {}: {error}", path.display()))?;
        if read == 0 {
            break;
        }
        hasher.update(&buffer[..read]);
    }

    Ok(hex(&hasher.finalize()))
}

/// Formats bytes as lowercase hexadecimal.
pub fn hex(bytes: &[u8]) -> String {
    use std::fmt::Write;

    let mut output = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        let _ = write!(output, "{byte:02x}");
    }
    output
}

/// Current UTC timestamp as an ISO-8601 string (seconds precision).
pub fn now_iso8601() -> String {
    let seconds = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |duration| duration.as_secs());

    // Civil-from-days algorithm (Howard Hinnant), no external dependency.
    let days = i64::try_from(seconds / 86_400).unwrap_or(i64::MAX);
    let time = seconds % 86_400;
    let (year, month, day) = civil_from_days(days);
    format!(
        "{year:04}-{month:02}-{day:02}T{:02}:{:02}:{:02}Z",
        time / 3600,
        (time % 3600) / 60,
        time % 60
    )
}

fn civil_from_days(days: i64) -> (i64, u32, u32) {
    let z = days + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let year = if m <= 2 { y + 1 } else { y };

    (
        year,
        u32::try_from(m).unwrap_or(1),
        u32::try_from(d).unwrap_or(1),
    )
}

#[cfg(test)]
mod tests {
    use super::{civil_from_days, clean_text, normalize};

    #[test]
    fn normalizes_accents_and_punctuation() {
        assert_eq!(normalize("Évoli N°025"), "evoli n 025");
        assert_eq!(normalize("Nidoran♀"), "nidoran");
    }

    #[test]
    fn cleans_flavour_text() {
        assert_eq!(
            clean_text("Spits fire that\nis hot enough.\u{000c}Known to cause fires."),
            "Spits fire that is hot enough. Known to cause fires."
        );
    }

    #[test]
    fn converts_known_dates() {
        assert_eq!(civil_from_days(0), (1970, 1, 1));
        assert_eq!(civil_from_days(20_000), (2024, 10, 4));
    }
}
