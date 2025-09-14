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
    Vertical,
    Horizontal,
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
    const TITLE_BODY_OFFSET: u32 = 25;
    const ROW_Y_STEP: u32 = 36;
    const VERTICAL_BAR_WIDTH: u32 = 220;
    const VALUE_SIZE: u32 = 46;
    const VERTICAL_VALUE_X_OFFSET: u32 = 10;
    const BAR_HEIGHT: u32 = 8;

    // Horizontal layout constants
    const HORIZONTAL_COLUMN_WIDTH: u32 = 130;
    const HORIZONTAL_COLUMN_GAP: u32 = 20;
    const HORIZONTAL_CIRCLE_SIZE: u32 = 8;
    const HORIZONTAL_CIRCLE_TEXT_GAP: u32 = 10;
    const HORIZONTAL_ROW_Y_STEP: u32 = 25;

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

        match self.layout {
            LayoutType::Vertical => {
                for stat in top_langs.iter() {
                    let color = gel_language_color(&stat.name);
                    let label = &stat.name;
                    let rank = stat.rank(
                        self.size_weight.unwrap_or(1.0),
                        self.count_weight.unwrap_or(0.0),
                    );
                    // Value is the percentage of the total rank.
                    let value = rank / total_rank * 100.0;

                    lines.push(Self::render_line_vertical(
                        &color,
                        label,
                        value,
                        self.card_settings.offset_x,
                        y,
                    ));

                    y += Self::ROW_Y_STEP;
                }
            }
            LayoutType::Horizontal => {
                // Create a single horizontal bar with stacked segments
                let total_width = Self::HORIZONTAL_COLUMN_WIDTH * 2 + Self::HORIZONTAL_COLUMN_GAP;
                let bar_spacing = 10;

                lines.push(Self::render_horizontal_bar(
                    &top_langs,
                    self.size_weight.unwrap_or(1.0),
                    self.count_weight.unwrap_or(0.0),
                    self.card_settings.offset_x,
                    y - bar_spacing,
                    total_width,
                ));

                y += Self::BAR_HEIGHT + bar_spacing;

                // Add language labels below the bar
                let mut label_y = y;
                for chunk in top_langs.chunks(2) {
                    let mut row_items = Vec::new();

                    for (col_index, stat) in chunk.iter().enumerate() {
                        let color = gel_language_color(&stat.name);
                        let label = &stat.name;
                        let rank = stat.rank(
                            self.size_weight.unwrap_or(1.0),
                            self.count_weight.unwrap_or(0.0),
                        );
                        // Value is the percentage of the total rank.
                        let value = rank / total_rank * 100.0;

                        let x_offset = self.card_settings.offset_x
                            + col_index as u32
                                * (Self::HORIZONTAL_COLUMN_WIDTH + Self::HORIZONTAL_COLUMN_GAP);

                        row_items.push(Self::render_line_horizontal(
                            &color, label, value, x_offset, label_y,
                        ));
                    }

                    lines.push(format!("<g class=\"row\">\n{}\n</g>", row_items.join("\n")));
                    label_y += Self::HORIZONTAL_ROW_Y_STEP;
                }
            }
        }

        let body = lines.join("\n");

        // TODO: Note height calculation is 3px smaller than the actual height. Need to fix it.
        let height = match self.layout {
            LayoutType::Vertical => {
                if self.card_settings.hide_title {
                    Self::ROW_Y_STEP * top_langs.len() as u32 + self.card_settings.offset_y * 2
                } else {
                    Self::ROW_Y_STEP * top_langs.len() as u32
                        + header_size_y
                        + self.card_settings.offset_y * 2
                }
            }
            LayoutType::Horizontal => {
                // For horizontal layout, we have a bar + grouped labels (2 per row)
                let num_rows = top_langs.len().div_ceil(2); // Ceiling division for label rows

                if self.card_settings.hide_title {
                    Self::BAR_HEIGHT
                        + Self::HORIZONTAL_ROW_Y_STEP * num_rows as u32
                        + self.card_settings.offset_y * 2
                } else {
                    Self::BAR_HEIGHT
                        + Self::HORIZONTAL_ROW_Y_STEP * num_rows as u32
                        + header_size_y
                        + self.card_settings.offset_y * 2
                }
            }
        };

        let width: u32 = match self.layout {
            LayoutType::Vertical => {
                Self::VERTICAL_BAR_WIDTH
                    + self.card_settings.offset_x * 2
                    + Self::VERTICAL_VALUE_X_OFFSET
                    + Self::VALUE_SIZE
            }
            LayoutType::Horizontal => {
                // Width for 2 columns with gap
                Self::HORIZONTAL_COLUMN_WIDTH * 2
                    + Self::HORIZONTAL_COLUMN_GAP
                    + self.card_settings.offset_x * 2
            }
        };

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

    fn render_line_vertical(
        color: &str,
        label: &str,
        value: f64,
        pos_x: u32,
        pos_y: u32,
    ) -> String {
        let bar_height = Self::BAR_HEIGHT;
        let label_x = pos_x + 2;
        let label_y = pos_y;
        let percent_x = pos_x + Self::VERTICAL_BAR_WIDTH + Self::VERTICAL_VALUE_X_OFFSET;
        let percent_y = pos_y + bar_height * 2;
        let bar_container_x = pos_x;
        let bar_container_y = pos_y + bar_height;
        let bar_width: u32 = Self::VERTICAL_BAR_WIDTH;

        let percent_str = format!("{value:.2}%");
        let percent_bar_width = (bar_width as f64 * value / 100.0).round() as u32;

        format!(
            r##"<g class="row">
  <text x="{label_x}" y="{label_y}" class="label">{label}</text>
  <text x="{percent_x}" y="{percent_y}" class="value">{percent_str}</text>
  <svg width="{bar_width}" x="{bar_container_x}" y="{bar_container_y}">
      <rect rx="5" ry="5" x="0" y="0" width="{bar_width}" height="{bar_height}" class="progressBarBackground"/>
      <rect rx="5" ry="5" x="0" y="0" width="{percent_bar_width}" height="{bar_height}" fill="{color}"/>
  </svg>
</g>"##
        )
    }

    fn render_line_horizontal(
        color: &str,
        label: &str,
        value: f64,
        pos_x: u32,
        pos_y: u32,
    ) -> String {
        let circle_x = pos_x + Self::HORIZONTAL_CIRCLE_SIZE / 2;
        let circle_y = pos_y;
        let label_x = pos_x + Self::HORIZONTAL_CIRCLE_SIZE + Self::HORIZONTAL_CIRCLE_TEXT_GAP;
        let label_y = pos_y + 4;

        let percent_str = format!("{value:.2}%");

        format!(
            r##"<circle cx="{circle_x}" cy="{circle_y}" r="{}" fill="{color}"/>
<text x="{label_x}" y="{label_y}" class="label">{label} {percent_str}</text>"##,
            Self::HORIZONTAL_CIRCLE_SIZE / 2
        )
    }

    fn render_horizontal_bar(
        stats: &[LanguageStat],
        size_weight: f64,
        count_weight: f64,
        pos_x: u32,
        pos_y: u32,
        total_width: u32,
    ) -> String {
        let bar_height = Self::BAR_HEIGHT;
        let mut segments = Vec::new();
        let mut current_x = 0f64;

        // Calculate total rank from reduced stats slice to calculate percentages width properly
        let relative_total_rank = stats.total_rank(size_weight, count_weight);

        // Calculate all percentages first
        let percentages: Vec<f64> = stats
            .iter()
            .map(|stat| {
                let rank = stat.rank(size_weight, count_weight);
                rank / relative_total_rank * 100.0
            })
            .collect();

        // Create segments with proper rounding to avoid gaps/overlaps
        for (i, stat) in stats.iter().enumerate() {
            let color = gel_language_color(&stat.name);

            // Calculate the expected end position for this segment
            let expected_end_x =
                total_width as f64 * percentages[0..=i].iter().sum::<f64>() / 100.0;
            let segment_width = (expected_end_x - current_x).round() as u32;

            // Ensure minimum width of 1px for very small segments
            let segment_width = segment_width.max(1);

            segments.push(format!(
                r##"<rect mask="url(#bar-mask)" x="{}" y="0" width="{segment_width}" height="{bar_height}" fill="{color}"/>"##,
                current_x.round() as u32
            ));
            current_x += segment_width as f64;
        }

        format!(
            r##"<g class="horizontal-bar">
  <svg width="{total_width}" x="{pos_x}" y="{pos_y}">
    <mask id="bar-mask">
        <rect x="0" y="0" width="{total_width}" height="{bar_height}" fill="white" rx="5"/>
    </mask>
      {}
  </svg>
</g>"##,
            segments.join("\n      ")
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
            let stats = [
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
            let stats = [
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
            let stats = [
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
            let stats = [
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

    mod fn_render_line_vertical {
        use super::*;

        #[test]
        fn test_render_line_vertical() {
            let color = "#00ADD8";
            let label = "Rust";
            let value = 30.55;
            let pos_x = 10;
            let pos_y = 20;

            let rendered = LangsCard::render_line_vertical(color, label, value, pos_x, pos_y);
            // Basic structure
            assert!(rendered.contains("<g class=\"row\">"));
            // Label and its coordinates
            assert!(rendered.contains("x=\"12\" y=\"20\" class=\"label\">Rust</text>"));
            // Percentage text and its coordinates, formatted to 2 decimals
            assert!(rendered.contains("x=\"240\" y=\"36\" class=\"value\">30.55%</text>"));
            // Bar container position and width
            assert!(rendered.contains("<svg width=\"220\" x=\"10\" y=\"28\">"));
            // Background bar
            assert!(
                rendered.contains("width=\"220\" height=\"8\" class=\"progressBarBackground\"")
            );
            // Foreground bar width rounding: round(220 * 30.55 / 100) = 67
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
                layout: LayoutType::Vertical,
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
            assert!(svg.contains(">46.51%</text>"));
            assert!(svg.contains(">30.23%</text>"));
            // Progress bar widths for 46.51% and 30.23% on 220px width
            assert!(svg.contains("width=\"102\" height=\"8\" fill=\"#00ADD8\""));
            assert!(svg.contains("width=\"67\" height=\"8\" fill=\"#f1e05a\""));
            // Rust should not appear since max_languages is 2
            assert!(!svg.contains(">Rust</text>"));
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
            // Circle with correct position and color (circle_y = pos_y = 20, not pos_y + circle_size/2)
            assert!(rendered.contains("cx=\"14\" cy=\"20\" r=\"4\" fill=\"#00ADD8\""));
            // Label and percentage in the same text element
            assert!(rendered.contains("x=\"28\" y=\"24\" class=\"label\">Rust 30.55%</text>"));
        }
    }

    mod fn_render_horizontal_layout {
        use super::*;
        use crate::cards::card::{CardSettings, CardTheme};

        #[test]
        fn test_render_horizontal_layout() {
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
                    LanguageStat {
                        name: "Python".to_string(),
                        size_bytes: 800,
                        repo_count: 3,
                    },
                ],
                size_weight: Some(1.0),
                count_weight: Some(0.0),
                max_languages: Some(4),
            };

            let svg = card.render();
            // Basic SVG structure and title
            assert!(svg.contains("<svg"));
            assert!(svg.contains("Most used languages"));
            // Should have a horizontal bar
            assert!(svg.contains("<g class=\"horizontal-bar\">"));
            // Should have 2 rows for labels (4 languages grouped by 2)
            assert_eq!(svg.matches("<g class=\"row\">").count(), 2);
            // Top languages should appear in labels
            assert!(svg.contains(">Go"));
            assert!(svg.contains(">JavaScript"));
            assert!(svg.contains(">Rust"));
            assert!(svg.contains(">Python"));
            // Should have circles for each language in labels
            assert_eq!(svg.matches("<circle").count(), 4);
            // Should have horizontal bar segments (rect elements)
            assert!(svg.matches("<rect").count() >= 4); // At least 4 segments for 4 languages
        }

        #[test]
        fn test_render_horizontal_bar() {
            let stats = vec![
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
                LanguageStat {
                    name: "Rust".to_string(),
                    size_bytes: 1000,
                    repo_count: 10,
                },
            ];

            let rendered = LangsCard::render_horizontal_bar(
                &stats, 1.0, 0.0, 10,  // pos_x
                20,  // pos_y
                280, // total_width
            );

            // Should contain horizontal bar structure
            assert!(rendered.contains("<g class=\"horizontal-bar\">"));
            assert!(rendered.contains("x=\"10\" y=\"20\""));
            assert!(rendered.contains("width=\"280\""));

            // Should have 4 rect elements: 1 for mask + 3 segments
            assert_eq!(rendered.matches("<rect").count(), 4);

            // Check colors are present
            assert!(rendered.contains("#00ADD8")); // Go
            assert!(rendered.contains("#f1e05a")); // JavaScript
            assert!(rendered.contains("#dea584")); // Rust

            // Segments should be positioned correctly (first starts at x=0)
            assert!(rendered.contains("x=\"0\""));

            // Should have mask
            assert!(rendered.contains("<mask id=\"bar-mask\">"));
        }
    }
}
