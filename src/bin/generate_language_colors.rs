#![cfg(feature = "gen-language-colors")]

use anyhow::{Context, Result, anyhow};
use serde::{Deserialize, Serialize};
use serde_json::ser::PrettyFormatter;
use serde_json::{Serializer, json};
use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

// URL to GitHub linguist languages.yml
const LINGUIST_YAML_URL: &str =
    "https://raw.githubusercontent.com/github/linguist/master/lib/linguist/languages.yml";
// Output path inside the repository
const OUTPUT_PATH: &str = "assets/configs/language-colors.json";

// We only care about the optional `color` field per language entry.
#[derive(Debug, Deserialize)]
struct LanguageMeta {
    #[serde(default)]
    color: Option<String>,
}

fn main() -> Result<()> {
    // Fetch YAML (blocking is fine for a build-time style script)
    let yaml_text = fetch_yaml()?;

    // Parse into map of language -> meta
    let langs: BTreeMap<String, LanguageMeta> =
        serde_yaml::from_str(&yaml_text).context("Failed to parse languages.yml as YAML")?;

    // Build colors map preserving key order (BTreeMap gives sorted order)
    let mut colors: BTreeMap<String, String> = BTreeMap::new();
    for (lang, meta) in langs.into_iter() {
        if let Some(c) = meta.color {
            if !c.trim().is_empty() {
                colors.insert(lang, c);
            }
        }
    }

    // Ensure output directory exists
    let out_path = Path::new(OUTPUT_PATH);
    if let Some(parent) = out_path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("Creating directory {}", parent.display()))?;
    }

    // Pretty-print with 4 spaces to match the example style
    let json_value = json!(colors);
    let mut buf = Vec::new();
    let formatter = PrettyFormatter::with_indent(b"  ");
    let mut serializer = Serializer::with_formatter(&mut buf, formatter);
    json_value.serialize(&mut serializer)?;
    let json_str = String::from_utf8(buf).context("Encoding JSON as UTF-8 failed")?;
    fs::write(out_path, format!("{}\n", json_str))
        .with_context(|| format!("Writing {}", out_path.display()))?;

    println!("Wrote {}", out_path.display());
    Ok(())
}

fn fetch_yaml() -> Result<String> {
    let resp =
        reqwest::blocking::get(LINGUIST_YAML_URL).context("HTTP GET languages.yml failed")?;
    if !resp.status().is_success() {
        return Err(anyhow!(
            "HTTP status {} from {}",
            resp.status(),
            LINGUIST_YAML_URL
        ));
    }
    resp.text().context("Reading response body failed")
}
#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::Value;

    // Valid YAML: flush left for keys, 2-space indent for properties, single quotes for color values
    const SAMPLE_YAML: &str = r#"C:
  type: programming
  color: '#555555'
C#:
  type: programming
  color: '#178600'
C++:
  type: programming
  color: '#f34b7d'
NoColorLang:
  type: programming
"#;

    #[test]
    fn test_yaml_parse_and_color_extraction() {
        let langs: BTreeMap<String, LanguageMeta> = serde_yaml::from_str(SAMPLE_YAML).unwrap();
        assert_eq!(langs.len(), 4);
        let mut colors: BTreeMap<String, String> = BTreeMap::new();
        for (lang, meta) in langs.into_iter() {
            if let Some(c) = meta.color {
                if !c.trim().is_empty() {
                    colors.insert(lang, c);
                }
            }
        }
        assert_eq!(colors.len(), 3);
        assert_eq!(colors["C"], "#555555");
        assert_eq!(colors["C#"], "#178600");
        assert_eq!(colors["C++"], "#f34b7d");
        assert!(!colors.contains_key("NoColorLang"));
    }

    #[test]
    fn test_json_formatting_2_space_indent() {
        let mut colors = BTreeMap::new();
        colors.insert("C".to_string(), "#555555".to_string());
        colors.insert("C#".to_string(), "#178600".to_string());
        let json_value = json!(colors);
        let mut buf = Vec::new();
        let formatter = PrettyFormatter::with_indent(b"  ");
        let mut serializer = Serializer::with_formatter(&mut buf, formatter);
        json_value.serialize(&mut serializer).unwrap();
        let json_str = String::from_utf8(buf).unwrap();
        // Check for 2-space indentation
        assert!(json_str.contains("  \"C#\": "));
        // Check valid JSON
        let v: Value = serde_json::from_str(&json_str).unwrap();
        assert_eq!(v["C"], "#555555");
    }
}
