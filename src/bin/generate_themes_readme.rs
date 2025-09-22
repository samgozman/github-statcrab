#![cfg(feature = "gen-themes-readme")]

use anyhow::{Context, Result};
use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

use github_statcrab::cards::card::{CardSettings, CardTheme};
use github_statcrab::cards::langs_card::{LangsCard, LanguageStat, LayoutType};
use github_statcrab::cards::stats_card::StatsCard;

// Generate the theme parser function dynamically from CSS files
use card_theme_macros::build_theme_parser;
build_theme_parser!();

// Output paths
const README_PATH: &str = "assets/css/themes/README.md";
const EXAMPLES_DIR: &str = "assets/css/themes/examples";

/// Converts a kebab-case or snake_case string to PascalCase (same logic as the macro).
fn to_pascal_case(s: &str) -> String {
    let mut out = String::new();
    let mut capitalize = true;
    for ch in s.chars() {
        if ch == '-' || ch == '_' || ch == ' ' {
            capitalize = true;
            continue;
        }
        if capitalize {
            out.extend(ch.to_uppercase());
            capitalize = false;
        } else {
            out.extend(ch.to_lowercase());
        }
    }
    out
}

/// Dynamically tries to construct a CardTheme from a filename by using macro-generated code.
/// This leverages the fact that CardTheme variants are generated from CSS files.
fn parse_card_theme_from_filename(filename_stem: &str) -> Option<CardTheme> {
    // Convert filename to PascalCase using the same logic as the macro
    let pascal_case = to_pascal_case(filename_stem);

    // Use the macro-generated parser function (completely dynamic!)
    parse_theme_from_pascal_case(&pascal_case)
}

// Remove the manual implementation - no longer needed!
// The macro generates this function automatically from CSS files

fn main() -> Result<()> {
    // Create examples directory
    fs::create_dir_all(EXAMPLES_DIR).context("Failed to create examples directory")?;

    // Discover themes from CSS files
    let themes = discover_themes()?;

    if themes.is_empty() {
        anyhow::bail!("No themes found in assets/css/themes directory");
    }

    // Generate SVG examples for all themes
    let mut stats_examples = BTreeMap::new();
    let mut langs_examples = BTreeMap::new();
    let mut stats_transparent_examples = BTreeMap::new();
    let mut langs_transparent_examples = BTreeMap::new();

    for (theme_name, theme_variant) in &themes {
        // Generate regular Stats Card example
        let stats_svg = generate_stats_card_example(theme_variant.clone())?;
        let stats_file = format!("stats-card-{}.svg", theme_name);
        fs::write(Path::new(EXAMPLES_DIR).join(&stats_file), &stats_svg)
            .context("Failed to write stats card SVG")?;
        stats_examples.insert(theme_name.clone(), stats_file);

        // Generate regular Langs Card example
        let langs_svg = generate_langs_card_example(theme_variant.clone())?;
        let langs_file = format!("langs-card-{}.svg", theme_name);
        fs::write(Path::new(EXAMPLES_DIR).join(&langs_file), &langs_svg)
            .context("Failed to write langs card SVG")?;
        langs_examples.insert(theme_name.clone(), langs_file);

        // Generate transparent Stats Card example (hide_background & hide_background_stroke)
        let stats_transparent_svg = generate_stats_card_example_transparent(theme_variant.clone())?;
        let stats_transparent_file = format!("stats-card-{}-transparent.svg", theme_name);
        fs::write(
            Path::new(EXAMPLES_DIR).join(&stats_transparent_file),
            &stats_transparent_svg,
        )
        .context("Failed to write transparent stats card SVG")?;
        stats_transparent_examples.insert(theme_name.clone(), stats_transparent_file);

        // Generate transparent Langs Card example (hide_background & hide_background_stroke)
        let langs_transparent_svg = generate_langs_card_example_transparent(theme_variant.clone())?;
        let langs_transparent_file = format!("langs-card-{}-transparent.svg", theme_name);
        fs::write(
            Path::new(EXAMPLES_DIR).join(&langs_transparent_file),
            &langs_transparent_svg,
        )
        .context("Failed to write transparent langs card SVG")?;
        langs_transparent_examples.insert(theme_name.clone(), langs_transparent_file);
    }

    // Generate new README content with fixed intro
    let new_readme = generate_readme_content(
        &stats_examples,
        &langs_examples,
        &stats_transparent_examples,
        &langs_transparent_examples,
    )?;

    // Write updated README
    fs::write(README_PATH, new_readme).context("Failed to write updated README.md")?;

    println!(
        "Generated {} theme examples (regular + transparent)",
        themes.len() * 2
    );
    println!("Updated {}", README_PATH);

    Ok(())
}

/// Discovers themes by scanning the CSS files in assets/css/themes
fn discover_themes() -> Result<BTreeMap<String, CardTheme>> {
    let themes_dir = Path::new("assets/css/themes");
    let mut themes = BTreeMap::new();

    let entries = fs::read_dir(themes_dir).context("Failed to read themes directory")?;

    for entry in entries {
        let entry = entry.context("Failed to read directory entry")?;
        let path = entry.path();

        if path.extension().and_then(|s| s.to_str()) != Some("css") {
            continue;
        }

        let stem = path
            .file_stem()
            .and_then(|s| s.to_str())
            .context("Failed to get theme file stem")?;

        // Convert kebab-case filename to snake_case API name
        let api_name = stem.to_ascii_lowercase().replace('-', "_");

        // Convert to CardTheme enum variant dynamically using the same naming convention as the macro
        // The macro converts kebab-case filenames to PascalCase enum variants
        let theme_variant = match parse_card_theme_from_filename(stem) {
            Some(theme) => theme,
            None => {
                println!("Warning: Failed to parse theme '{}', skipping", api_name);
                continue;
            }
        };

        themes.insert(api_name, theme_variant);
    }

    Ok(themes)
}

/// Generates a Stats Card example with dummy data
fn generate_stats_card_example(theme: CardTheme) -> Result<String> {
    let settings = CardSettings {
        offset_x: 12,
        offset_y: 12,
        theme,
        hide_title: false,
        hide_background: false,
        hide_background_stroke: false,
    };

    let stats_card = StatsCard {
        card_settings: settings,
        username: "octocat".to_string(),
        stars_count: Some(1234),
        commits_ytd_count: Some(567),
        issues_count: Some(89),
        pull_requests_count: Some(123),
        merge_requests_count: Some(45),
        reviews_count: Some(67),
        started_discussions_count: Some(12),
        answered_discussions_count: Some(34),
    };

    Ok(stats_card.render())
}

/// Generates a Langs Card example with dummy data
fn generate_langs_card_example(theme: CardTheme) -> Result<String> {
    let settings = CardSettings {
        offset_x: 12,
        offset_y: 12,
        theme,
        hide_title: false,
        hide_background: false,
        hide_background_stroke: false,
    };

    let dummy_stats = vec![
        LanguageStat {
            name: "TypeScript".to_string(),
            size_bytes: 125000,
            repo_count: 15,
        },
        LanguageStat {
            name: "JavaScript".to_string(),
            size_bytes: 98000,
            repo_count: 12,
        },
        LanguageStat {
            name: "Rust".to_string(),
            size_bytes: 75000,
            repo_count: 8,
        },
        LanguageStat {
            name: "Python".to_string(),
            size_bytes: 45000,
            repo_count: 6,
        },
        LanguageStat {
            name: "Go".to_string(),
            size_bytes: 32000,
            repo_count: 4,
        },
    ];

    let langs_card = LangsCard {
        card_settings: settings,
        layout: LayoutType::Vertical,
        stats: dummy_stats,
        size_weight: Some(1.0),
        count_weight: Some(0.0),
        max_languages: Some(5),
    };

    Ok(langs_card.render())
}

/// Generates a Stats Card example with transparent background (hide_background & hide_background_stroke enabled)
fn generate_stats_card_example_transparent(theme: CardTheme) -> Result<String> {
    let settings = CardSettings {
        offset_x: 12,
        offset_y: 12,
        theme,
        hide_title: false,
        hide_background: true,
        hide_background_stroke: true,
    };

    let stats_card = StatsCard {
        card_settings: settings,
        username: "octocat".to_string(),
        stars_count: Some(1234),
        commits_ytd_count: Some(567),
        issues_count: Some(89),
        pull_requests_count: Some(123),
        merge_requests_count: Some(45),
        reviews_count: Some(67),
        started_discussions_count: Some(12),
        answered_discussions_count: Some(34),
    };

    Ok(stats_card.render())
}

/// Generates a Langs Card example with transparent background (hide_background & hide_background_stroke enabled)
fn generate_langs_card_example_transparent(theme: CardTheme) -> Result<String> {
    let settings = CardSettings {
        offset_x: 12,
        offset_y: 12,
        theme,
        hide_title: false,
        hide_background: true,
        hide_background_stroke: true,
    };

    let dummy_stats = vec![
        LanguageStat {
            name: "TypeScript".to_string(),
            size_bytes: 125000,
            repo_count: 15,
        },
        LanguageStat {
            name: "JavaScript".to_string(),
            size_bytes: 98000,
            repo_count: 12,
        },
        LanguageStat {
            name: "Rust".to_string(),
            size_bytes: 75000,
            repo_count: 8,
        },
        LanguageStat {
            name: "Python".to_string(),
            size_bytes: 45000,
            repo_count: 6,
        },
        LanguageStat {
            name: "Go".to_string(),
            size_bytes: 32000,
            repo_count: 4,
        },
    ];

    let langs_card = LangsCard {
        card_settings: settings,
        layout: LayoutType::Vertical,
        stats: dummy_stats,
        size_weight: Some(1.0),
        count_weight: Some(0.0),
        max_languages: Some(5),
    };

    Ok(langs_card.render())
}

/// Generates the README content with theme examples
fn generate_readme_content(
    stats_examples: &BTreeMap<String, String>,
    langs_examples: &BTreeMap<String, String>,
    stats_transparent_examples: &BTreeMap<String, String>,
    langs_transparent_examples: &BTreeMap<String, String>,
) -> Result<String> {
    let mut content = String::new();

    // Add fixed intro content
    content.push_str("# How to add new themes?\n\n");
    content.push_str("If you want to contribute a new theme, please add a new CSS file in the `assets/css/themes` directory. The file name should be in kebab-case (e.g., `new-theme.css`). The macro will automatically generate the necessary Rust code for the new theme based on the file name.\n\n");
    content.push_str("The CSS classes defined in the theme file should follow the naming convention used in the existing themes.\n\n");
    content.push_str("> [!NOTE]  \n");
    content.push_str("> While you can use CSS for styling, keep in mind that you are working with SVG elements. This means that some CSS properties may not work as expected.\n\n");
    content.push_str("The **Transparent** column shows theme variants with `hide_background=true` and `hide_background_stroke=true` options enabled, removing the card background for integration into custom layouts.\n\n");

    // Add Stats Card section
    content.push_str("## Stats Card\n\n");
    content.push_str("| Theme | Default | Transparent |\n");
    content.push_str("|-------|---------|-------------|\n");

    for theme_name in stats_examples.keys() {
        let default_svg = stats_examples.get(theme_name).unwrap();
        let transparent_svg = stats_transparent_examples.get(theme_name).unwrap();
        content.push_str(&format!(
            "| `{}` | ![{}](examples/{}) | ![{} transparent](examples/{}) |\n",
            theme_name, theme_name, default_svg, theme_name, transparent_svg
        ));
    }

    content.push('\n');

    // Add Langs Card section
    content.push_str("## Langs Card\n\n");
    content.push_str("| Theme | Default | Transparent |\n");
    content.push_str("|-------|---------|-------------|\n");

    for theme_name in langs_examples.keys() {
        let default_svg = langs_examples.get(theme_name).unwrap();
        let transparent_svg = langs_transparent_examples.get(theme_name).unwrap();
        content.push_str(&format!(
            "| `{}` | ![{}](examples/{}) | ![{} transparent](examples/{}) |\n",
            theme_name, theme_name, default_svg, theme_name, transparent_svg
        ));
    }

    Ok(content)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn test_generate_stats_card_example() {
        let result = generate_stats_card_example(CardTheme::TransparentBlue);
        assert!(result.is_ok());

        let svg = result.unwrap();
        assert!(svg.contains("<svg"));
        assert!(svg.contains("</svg>"));
        assert!(svg.contains("octocat"));
        assert!(svg.contains("1.2k")); // stars count formatted
        assert!(svg.contains("567")); // commits count
    }

    #[test]
    fn test_generate_readme_content_with_empty_examples() {
        let stats_examples = BTreeMap::new();
        let langs_examples = BTreeMap::new();
        let stats_transparent_examples = BTreeMap::new();
        let langs_transparent_examples = BTreeMap::new();

        let result = generate_readme_content(
            &stats_examples,
            &langs_examples,
            &stats_transparent_examples,
            &langs_transparent_examples,
        );
        assert!(result.is_ok());

        let content = result.unwrap();

        // Should still have intro and headers
        assert!(content.contains("# How to add new themes?"));
        assert!(content.contains("## Stats Card"));
        assert!(content.contains("## Langs Card"));

        // But no theme entries
        assert!(!content.contains("| `"));
    }

    #[test]
    fn test_generate_readme_content_preserves_order() {
        let mut stats_examples = BTreeMap::new();
        stats_examples.insert("z_theme".to_string(), "stats-card-z.svg".to_string());
        stats_examples.insert("a_theme".to_string(), "stats-card-a.svg".to_string());
        stats_examples.insert("m_theme".to_string(), "stats-card-m.svg".to_string());

        let mut stats_transparent_examples = BTreeMap::new();
        stats_transparent_examples.insert(
            "z_theme".to_string(),
            "stats-card-z-transparent.svg".to_string(),
        );
        stats_transparent_examples.insert(
            "a_theme".to_string(),
            "stats-card-a-transparent.svg".to_string(),
        );
        stats_transparent_examples.insert(
            "m_theme".to_string(),
            "stats-card-m-transparent.svg".to_string(),
        );

        let langs_examples = BTreeMap::new();
        let langs_transparent_examples = BTreeMap::new();

        let result = generate_readme_content(
            &stats_examples,
            &langs_examples,
            &stats_transparent_examples,
            &langs_transparent_examples,
        );
        assert!(result.is_ok());

        let content = result.unwrap();

        // BTreeMap should preserve alphabetical order
        let a_pos = content.find("| `a_theme`").unwrap();
        let m_pos = content.find("| `m_theme`").unwrap();
        let z_pos = content.find("| `z_theme`").unwrap();

        assert!(a_pos < m_pos);
        assert!(m_pos < z_pos);
    }

    #[test]
    fn test_discover_themes_with_test_directory() {
        // Create a temporary directory structure
        let temp_dir = tempdir().expect("Failed to create temp dir");
        let themes_dir = temp_dir.path().join("assets/css/themes");
        fs::create_dir_all(&themes_dir).expect("Failed to create themes dir");

        // Create test CSS files
        fs::write(themes_dir.join("transparent-blue.css"), "/* test css */")
            .expect("Failed to write test file");
        fs::write(themes_dir.join("dark.css"), "/* dark theme */")
            .expect("Failed to write test file");
        fs::write(themes_dir.join("invalid-theme.css"), "/* unknown theme */")
            .expect("Failed to write test file");
        fs::write(themes_dir.join("not-css.txt"), "not a css file")
            .expect("Failed to write test file");

        // Temporarily change working directory for the test
        let original_dir = std::env::current_dir().expect("Failed to get current dir");
        std::env::set_current_dir(temp_dir.path()).expect("Failed to change dir");

        let result = discover_themes();

        // Restore original directory
        std::env::set_current_dir(original_dir).expect("Failed to restore dir");

        assert!(result.is_ok());
        let themes = result.unwrap();

        // Should find known themes and skip unknown ones
        assert_eq!(themes.len(), 2);
        assert!(themes.contains_key("transparent_blue"));
        assert!(themes.contains_key("dark"));
        assert!(!themes.contains_key("invalid_theme"));
        assert!(!themes.contains_key("not_css"));
    }

    #[test]
    fn test_discover_themes_handles_nonexistent_directory() {
        // Temporarily change to a directory that doesn't have themes
        let temp_dir = tempdir().expect("Failed to create temp dir");
        let original_dir = std::env::current_dir().expect("Failed to get current dir");
        std::env::set_current_dir(temp_dir.path()).expect("Failed to change dir");

        let result = discover_themes();

        // Restore original directory
        std::env::set_current_dir(original_dir).expect("Failed to restore dir");

        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("Failed to read themes directory")
        );
    }

    #[test]
    fn test_generate_langs_card_example() {
        let result = generate_langs_card_example(CardTheme::Dark);
        assert!(result.is_ok());

        let svg = result.unwrap();
        assert!(svg.contains("<svg"));
        assert!(svg.contains("</svg>"));
        assert!(svg.contains("TypeScript"));
        assert!(svg.contains("JavaScript"));
        assert!(svg.contains("Rust"));
    }

    #[test]
    fn test_generate_readme_content_structure() {
        let mut stats_examples = BTreeMap::new();
        stats_examples.insert("dark".to_string(), "stats-card-dark.svg".to_string());
        stats_examples.insert("light".to_string(), "stats-card-light.svg".to_string());

        let mut langs_examples = BTreeMap::new();
        langs_examples.insert("dark".to_string(), "langs-card-dark.svg".to_string());
        langs_examples.insert("light".to_string(), "langs-card-light.svg".to_string());

        let mut stats_transparent_examples = BTreeMap::new();
        stats_transparent_examples.insert(
            "dark".to_string(),
            "stats-card-dark-transparent.svg".to_string(),
        );
        stats_transparent_examples.insert(
            "light".to_string(),
            "stats-card-light-transparent.svg".to_string(),
        );

        let mut langs_transparent_examples = BTreeMap::new();
        langs_transparent_examples.insert(
            "dark".to_string(),
            "langs-card-dark-transparent.svg".to_string(),
        );
        langs_transparent_examples.insert(
            "light".to_string(),
            "langs-card-light-transparent.svg".to_string(),
        );

        let result = generate_readme_content(
            &stats_examples,
            &langs_examples,
            &stats_transparent_examples,
            &langs_transparent_examples,
        );
        assert!(result.is_ok());

        let content = result.unwrap();

        // Check intro content
        assert!(content.contains("# How to add new themes?"));
        assert!(content.contains("kebab-case"));
        assert!(content.contains("> [!NOTE]"));

        // Check sections
        assert!(content.contains("## Stats Card"));
        assert!(content.contains("## Langs Card"));

        // Check table headers
        assert!(content.contains("| Theme | Default | Transparent |"));
        assert!(content.contains("|-------|---------|-------------|"));

        // Check theme entries
        assert!(content.contains("| `dark` | ![dark](examples/stats-card-dark.svg) | ![dark transparent](examples/stats-card-dark-transparent.svg) |"));
        assert!(content.contains("| `light` | ![light](examples/langs-card-light.svg) | ![light transparent](examples/langs-card-light-transparent.svg) |"));
    }

    #[test]
    fn test_langs_card_example_contains_expected_languages() {
        let svg = generate_langs_card_example(CardTheme::Monokai).unwrap();

        // Should contain all the dummy programming languages
        assert!(svg.contains("TypeScript"));
        assert!(svg.contains("JavaScript"));
        assert!(svg.contains("Rust"));
        assert!(svg.contains("Python"));
        assert!(svg.contains("Go"));

        // Should be valid SVG
        assert!(svg.starts_with("<svg"));
        assert!(svg.ends_with("</svg>\n"));
    }
}
