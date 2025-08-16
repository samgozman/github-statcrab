use crate::cards::{
    card::{CardSettings, Svg},
    helpers::gel_language_color,
};
use std::{cmp::Ordering, collections::HashMap};

/// Represents an edge in the language statistics graph.
/// Consist of language name and its size in bytes.
pub struct LangEdge {
    /// Main name of the programming language in the repository.
    /// Should correspond to the name in the `assets/configs/language-colors.json` file.
    pub name: String,
    /// The size of the repo in bytes.
    pub size_bytes: usize,
}

/// Represents a single language statistic for the [LangsCard].
#[derive(Clone)]
pub struct LanguageStat {
    /// The name of the programming language.
    /// Should correspond to the name in the `assets/configs/language-colors.json` file.
    pub name: String,
    /// The size of the repo in bytes.
    pub size_bytes: usize,
    /// The number of repositories that use this language.
    pub repo_count: u64,
}

impl LanguageStat {
    /// Converts a vector of [LangEdge] to a vector of [LanguageStat] by grouping
    /// languages, summing their sizes, and counting their occurrences.
    pub fn from_edges(edges: Vec<LangEdge>) -> Vec<Self> {
        let mut stats_map: HashMap<String, LanguageStat> = HashMap::new();

        for edge in edges {
            let entry = stats_map
                .entry(edge.name.clone())
                .or_insert_with(|| LanguageStat {
                    name: edge.name.clone(),
                    size_bytes: 0,
                    repo_count: 0,
                });

            entry.size_bytes += edge.size_bytes;
            entry.repo_count += 1;
        }

        stats_map.into_values().collect()
    }

    /// Calculates the rank of the language based on its size and repository count.
    fn rank(&self, size_weight: f64, count_weight: f64) -> f64 {
        (self.size_bytes as f64).powf(size_weight) * (self.repo_count as f64).powf(count_weight)
    }
}

/// Extension trait for [LanguageStat] slice to provide ranking and top N functionality.
pub trait LanguageStatsExt {
    /// Returns a new [Vec]<[LanguageStat]> sorted by descending rank.
    fn ranked(&self, size_weight: f64, count_weight: f64) -> Vec<LanguageStat>;

    /// Calculates the total rank of the [Vec]<[LanguageStat]>.
    /// Better to be used on a full [Vec]<[LanguageStat]> array rather than on a slice of `top_n`.
    fn total_rank(&self, size_weight: f64, count_weight: f64) -> f64;

    /// Returns top N [LanguageStat] by rank (descending).
    fn top_n(&self, size_weight: f64, count_weight: f64, n: usize) -> Vec<LanguageStat> {
        let mut ranked = self.ranked(size_weight, count_weight);
        ranked.truncate(n);
        ranked
    }
}

impl LanguageStatsExt for [LanguageStat] {
    fn ranked(&self, size_weight: f64, count_weight: f64) -> Vec<LanguageStat> {
        // Precompute ranks to avoid recomputation during sort comparisons.
        let mut with_rank: Vec<(f64, LanguageStat)> = self
            .iter()
            .cloned()
            .map(|s| (s.rank(size_weight, count_weight), s))
            .collect();

        with_rank.sort_unstable_by(|a, b| {
            // Descending by rank
            b.0.partial_cmp(&a.0).unwrap_or(Ordering::Equal)
        });

        with_rank.into_iter().map(|(_, s)| s).collect()
    }

    fn total_rank(&self, size_weight: f64, count_weight: f64) -> f64 {
        self.iter().map(|s| s.rank(size_weight, count_weight)).sum()
    }
}

/// Represents the layout type for the [LangsCard] (how the languages are displayed).
pub enum LayoutType {
    Horizontal,
    Vertical,
}

/// Represents a card that displays language statistics for a GitHub user.
/// It calculates the ranking of languages based on their size and repository count.
/// The ranking is calculated using the formula:
///
/// `rank = (size_bytes ^ size_weight) * (repo_count ^ count_weight)`.
pub struct LangsCard {
    pub card_settings: CardSettings,
    pub layout: LayoutType,
    pub stats: Vec<LanguageStat>,
    /// Used to modify the weight of the language size in the ranking calculation.
    pub size_weight: Option<f64>,
    /// Used to modify the weight of the repositories count in the ranking calculation.
    pub count_weight: Option<f64>,
    /// Maximum number of languages to display in the card.
    pub max_languages: Option<u64>,
}

impl LangsCard {
    const MAX_LANGUAGES: u64 = 20;
    const TITLE_BODY_OFFSET: u32 = 10;
    const ROW_Y_STEP: u32 = 36;
    const HORIZONTAL_BAR_WIDTH: u32 = 220;
    const VALUE_SIZE: u32 = 46;
    const HORIZONTAL_VALUE_X_OFFSET: u32 = 10;

    pub fn render(&self) -> Svg {
        use crate::cards::card::Card;
        // Title block height (title + small gap) unless title is hidden
        let header_size_y = if self.card_settings.hide_title {
            0
        } else {
            Card::TITLE_FONT_SIZE + Self::TITLE_BODY_OFFSET
        };

        // Starting baseline (text y) for the first row.
        let mut y: u32 = header_size_y + self.card_settings.offset_y;

        let max_langs = self
            .max_languages
            .unwrap_or(Self::MAX_LANGUAGES)
            .min(Self::MAX_LANGUAGES);

        let top_langs = self.stats.top_n(
            self.size_weight.unwrap_or(1.0),
            self.count_weight.unwrap_or(0.0),
            max_langs as usize,
        );

        let mut lines = Vec::new();
        let total_rank = self.stats.total_rank(
            self.size_weight.unwrap_or(1.0),
            self.count_weight.unwrap_or(0.0),
        );

        for stat in top_langs.iter() {
            let color = gel_language_color(&stat.name);
            let label = &stat.name;
            let rank = stat.rank(
                self.size_weight.unwrap_or(1.0),
                self.count_weight.unwrap_or(0.0),
            );
            // Value is the percentage of the total rank.
            let value = (rank / total_rank * 100.0).round();

            match self.layout {
                LayoutType::Horizontal => {
                    lines.push(Self::render_line_horizontal(
                        &color,
                        label,
                        value,
                        self.card_settings.offset_x,
                        y,
                    ));
                }
                LayoutType::Vertical => {
                    todo!("Vertical layout is not implemented yet");
                }
            }

            y += Self::ROW_Y_STEP;
        }

        let body = lines.join("\n");

        // TODO: Note height calculation is 3px smaller than the actual height. Need to fix it.
        let height = if self.card_settings.hide_title {
            Self::ROW_Y_STEP * top_langs.len() as u32 + self.card_settings.offset_y * 2
        } else {
            Self::ROW_Y_STEP * top_langs.len() as u32
                + header_size_y
                + self.card_settings.offset_y * 2
        };

        let width: u32 = Self::HORIZONTAL_BAR_WIDTH
            + self.card_settings.offset_x * 2
            + Self::HORIZONTAL_VALUE_X_OFFSET
            + Self::VALUE_SIZE;

        let card = Card::new(
            width,
            height,
            String::from("Most used languages"),
            String::from("GitHub top languages"),
            body,
            "langsCard".to_string(),
            self.card_settings.clone(),
        );

        match card {
            Ok(card) => card.render(),
            // TODO: handle error properly
            Err(e) => format!("Failed to render LangsCard: {e}"),
        }
    }

    fn render_line_horizontal(
        color: &str,
        label: &str,
        value: f64,
        pos_x: u32,
        pos_y: u32,
    ) -> String {
        let label_x = pos_x + 2;
        let label_y = pos_y + 18;
        let percent_x = pos_x + Self::HORIZONTAL_BAR_WIDTH + Self::HORIZONTAL_VALUE_X_OFFSET;
        let percent_y = pos_y + Self::ROW_Y_STEP - 2;
        let bar_container_x = pos_x;
        let bar_container_y = pos_y + 25;
        let bar_width: u32 = Self::HORIZONTAL_BAR_WIDTH;

        let percent_str = format!("{value:.2}%");
        let percent_bar_width = (bar_width as f64 * value / 100.0).round() as u32;

        format!(
            r##"<g class="row">
  <text x="{label_x}" y="{label_y}" class="label">{label}</text>
  <text x="{percent_x}" y="{percent_y}" class="value">{percent_str}</text>
  <svg width="{bar_width}" x="{bar_container_x}" y="{bar_container_y}">
      <rect rx="5" ry="5" x="0" y="0" width="{bar_width}" height="8" class="progressBarBackground"/>
      <rect rx="5" ry="5" x="0" y="0" width="{percent_bar_width}" height="8" fill="{color}"/>
  </svg>
</g>"##
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    mod fn_from_edges {
        use super::*;

        #[test]
        fn test_from_edges() {
            let edges = vec![
                LangEdge {
                    name: "Rust".to_string(),
                    size_bytes: 1000,
                },
                LangEdge {
                    name: "Rust".to_string(),
                    size_bytes: 500,
                },
                LangEdge {
                    name: "Python".to_string(),
                    size_bytes: 2000,
                },
            ];

            let stats = LanguageStat::from_edges(edges);
            assert_eq!(stats.len(), 2);

            // Find Rust stats
            let rust_stats = stats
                .iter()
                .find(|s| s.name == "Rust")
                .expect("Rust stats not found");
            assert_eq!(rust_stats.size_bytes, 1500);
            assert_eq!(rust_stats.repo_count, 2);

            // Find Python stats
            let python_stats = stats
                .iter()
                .find(|s| s.name == "Python")
                .expect("Python stats not found");
            assert_eq!(python_stats.size_bytes, 2000);
            assert_eq!(python_stats.repo_count, 1);
        }
    }

    mod fn_rank {
        use super::*;

        #[test]
        fn test_rank_size_1_weight_0() {
            let stat = LanguageStat {
                name: "Rust".to_string(),
                size_bytes: 1000,
                repo_count: 10,
            };

            let rank = stat.rank(1.0, 0.0);
            assert_eq!(rank, 1000.0);
        }

        #[test]
        fn test_rank_size_0_count_1() {
            let stat = LanguageStat {
                name: "Rust".to_string(),
                size_bytes: 1000,
                repo_count: 10,
            };

            let rank = stat.rank(0.0, 1.0);
            assert_eq!(rank, 10.0);
        }

        #[test]
        fn test_rank_size_0_5_count_0_5() {
            let stat = LanguageStat {
                name: "Rust".to_string(),
                size_bytes: 2500,
                repo_count: 10,
            };

            let rank = stat.rank(0.5, 0.5);
            // Check with a tolerance for floating point precision to 2 decimal places
            assert!(
                (rank - 158.11).abs() < 1e-2,
                "rank was {}, expected 158.11",
                rank
            );
        }
    }

    mod fn_ranked {
        use super::*;

        #[test]
        fn test_ranked() {
            let stats = vec![
                LanguageStat {
                    name: "Rust".to_string(),
                    size_bytes: 1000,
                    repo_count: 10,
                },
                LanguageStat {
                    name: "Go".to_string(),
                    size_bytes: 2000,
                    repo_count: 5,
                },
                LanguageStat {
                    name: "JavaScript".to_string(),
                    size_bytes: 1500,
                    repo_count: 8,
                },
            ];

            let ranked = stats.ranked(1.0, 0.0);
            assert_eq!(ranked.len(), 3);
            assert_eq!(ranked[0].name, "Go");
            assert_eq!(ranked[1].name, "JavaScript");
            assert_eq!(ranked[2].name, "Rust");
        }
    }

    mod fn_top_n {
        use super::*;

        #[test]
        fn test_top_n_2_size_1_count_0() {
            let stats = vec![
                LanguageStat {
                    name: "Rust".to_string(),
                    size_bytes: 1000,
                    repo_count: 10,
                },
                LanguageStat {
                    name: "Go".to_string(),
                    size_bytes: 2000,
                    repo_count: 5,
                },
                LanguageStat {
                    name: "JavaScript".to_string(),
                    size_bytes: 1500,
                    repo_count: 8,
                },
            ];

            let top = stats.top_n(1.0, 0.0, 2);
            assert_eq!(top.len(), 2);
            assert_eq!(top[0].name, "Go");
            assert_eq!(top[1].name, "JavaScript");
        }

        #[test]
        fn test_top_n_2_size_0_count_1() {
            let stats = vec![
                LanguageStat {
                    name: "Rust".to_string(),
                    size_bytes: 1000,
                    repo_count: 10,
                },
                LanguageStat {
                    name: "Go".to_string(),
                    size_bytes: 2000,
                    repo_count: 5,
                },
                LanguageStat {
                    name: "JavaScript".to_string(),
                    size_bytes: 1500,
                    repo_count: 8,
                },
            ];

            let top = stats.top_n(0.0, 1.0, 2);
            assert_eq!(top.len(), 2);
            assert_eq!(top[0].name, "Rust");
            assert_eq!(top[1].name, "JavaScript");
        }

        #[test]
        fn test_top_n_2_size_0_5_count_0_5() {
            let stats = vec![
                LanguageStat {
                    name: "Rust".to_string(),
                    size_bytes: 1000,
                    repo_count: 10,
                },
                LanguageStat {
                    name: "Go".to_string(),
                    size_bytes: 2000,
                    repo_count: 5,
                },
                LanguageStat {
                    name: "JavaScript".to_string(),
                    size_bytes: 1500,
                    repo_count: 8,
                },
            ];

            let top = stats.top_n(0.5, 0.5, 2);
            assert_eq!(top.len(), 2);
            assert_eq!(top[0].name, "JavaScript");
            assert_eq!(top[1].name, "Go");
        }
    }

    mod fn_render_line_horizontal {
        use super::*;

        #[test]
        fn test_render_line_horizontal() {
            let color = "#00ADD8";
            let label = "Rust";
            let value = 30.55;
            let pos_x = 10;
            let pos_y = 20;

            let rendered = LangsCard::render_line_horizontal(color, label, value, pos_x, pos_y);
            // Basic structure
            assert!(rendered.contains("<g class=\"row\">"));
            // Label and its coordinates
            assert!(rendered.contains("x=\"12\" y=\"38\" class=\"label\">Rust</text>"));
            // Percentage text and its coordinates, formatted to 2 decimals
            assert!(rendered.contains("x=\"240\" y=\"54\" class=\"value\">30.55%</text>"));
            // Bar container position and width
            assert!(rendered.contains("<svg width=\"220\" x=\"10\" y=\"45\">"));
            // Background bar
            assert!(
                rendered.contains("width=\"220\" height=\"8\" class=\"progressBarBackground\"")
            );
            // Foreground bar width rounding: round(220 * 30.55 / 100) = 63
            assert!(rendered.contains("width=\"67\" height=\"8\" fill=\"#00ADD8\""));
        }
    }

    mod fn_render {
        use super::*;
        use crate::cards::card::{CardSettings, CardTheme};

        #[test]
        fn test_render() {
            let card = LangsCard {
                card_settings: CardSettings {
                    offset_x: 10,
                    offset_y: 20,
                    hide_title: false,
                    theme: CardTheme::TransparentBlue,
                    hide_background: false,
                    hide_background_stroke: false,
                },
                layout: LayoutType::Horizontal,
                stats: vec![
                    LanguageStat {
                        name: "Rust".to_string(),
                        size_bytes: 1000,
                        repo_count: 10,
                    },
                    LanguageStat {
                        name: "Go".to_string(),
                        size_bytes: 2000,
                        repo_count: 5,
                    },
                    LanguageStat {
                        name: "JavaScript".to_string(),
                        size_bytes: 1300,
                        repo_count: 8,
                    },
                ],
                size_weight: Some(1.0),
                count_weight: Some(0.0),
                max_languages: Some(2),
            };

            let svg = card.render();
            // Basic SVG structure and title
            assert!(svg.contains("<svg"));
            assert!(svg.contains("Most used languages"));
            // Only two rows should be rendered (top 2 languages)
            assert_eq!(svg.matches("<g class=\"row\">").count(), 2);
            // Top languages and percentages
            assert!(svg.contains(">Go</text>"));
            assert!(svg.contains(">JavaScript</text>"));
            assert!(svg.contains(">47.00%</text>"));
            assert!(svg.contains(">30.00%</text>"));
            // Progress bar widths for 47% and 30% on 220px width
            assert!(svg.contains("width=\"103\" height=\"8\" fill=\"#00ADD8\""));
            assert!(svg.contains("width=\"66\" height=\"8\" fill=\"#f1e05a\""));
            // Rust should not appear since max_languages is 2
            assert!(!svg.contains(">Rust</text>"));
        }
    }
}
