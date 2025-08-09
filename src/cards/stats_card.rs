use crate::cards::card::{CardSettings, CardTheme, Svg};

pub struct StatsCard {
    pub card_settings: CardSettings,
    pub username: String,
    pub stars_count: Option<u32>,
    pub commits_ytd_count: Option<u32>,
    pub issues_count: Option<u32>,
    pub pull_requests_count: Option<u32>,
    pub merge_requests_count: Option<u32>,
    pub reviews_count: Option<u32>,
    pub started_discussions_count: Option<u32>,
    pub answered_discussions_count: Option<u32>,
}

impl Default for StatsCard {
    fn default() -> Self {
        StatsCard {
            card_settings: CardSettings {
                offset_x: 1,
                offset_y: 1,
                theme: CardTheme::TransparentBlue,
                hide_title: false,
                hide_background: false,
                hide_background_stroke: false,
            },
            username: String::new(),
            stars_count: None,
            commits_ytd_count: None,
            issues_count: None,
            pull_requests_count: None,
            merge_requests_count: None,
            reviews_count: None,
            started_discussions_count: None,
            answered_discussions_count: None,
        }
    }
}

impl StatsCard {
    // Constants for rendering the card (in pixels).
    const MAX_USERNAME_LEN: usize = 13;
    const VALUE_SIZE: u32 = 31;
    const LABEL_SIZE: u32 = 220;
    const ICON_SIZE: u32 = 15;
    const ICON_OFFSET: u32 = 8;
    const TITLE_BODY_OFFSET: u32 = 1;
    const ROW_Y_STEP: u32 = 27;

    /// Renders the [StatsCard] as an [Svg] string.
    pub fn render(&self) -> Svg {
        use crate::cards::card::Card;

        // Prepare stat lines (label, value, Option)
        let mut lines = Vec::new();
        // Title block height (title + small gap) unless title is hidden
        let header_size_y = if self.card_settings.hide_title {
            0
        } else {
            Card::TITLE_FONT_SIZE + Self::TITLE_BODY_OFFSET
        };

        // Starting baseline (text y) for the first stat row.
        // If title is visible: keep previous spacing (title height + row step + top offset).
        // If title is hidden: start so that the icon's top sits exactly at offset_y, giving
        // symmetric padding top/bottom. Baseline = offset_y + ICON_SIZE.
        let mut y: u32 = if self.card_settings.hide_title {
            self.card_settings.offset_y + Self::ICON_SIZE
        } else {
            header_size_y + Self::ROW_Y_STEP + self.card_settings.offset_y
        };

        if let Some(val) = self.stars_count {
            lines.push(self.render_line(
                StatIcon::Stars,
                "Stars",
                val,
                self.card_settings.offset_x,
                y,
            ));
            y += Self::ROW_Y_STEP;
        }
        if let Some(val) = self.commits_ytd_count {
            lines.push(self.render_line(
                StatIcon::CommitsYTD,
                "Commits YTD",
                val,
                self.card_settings.offset_x,
                y,
            ));
            y += Self::ROW_Y_STEP;
        }
        if let Some(val) = self.issues_count {
            lines.push(self.render_line(
                StatIcon::Issues,
                "Issues",
                val,
                self.card_settings.offset_x,
                y,
            ));
            y += Self::ROW_Y_STEP;
        }
        if let Some(val) = self.pull_requests_count {
            lines.push(self.render_line(
                StatIcon::PullRequests,
                "Pull Requests",
                val,
                self.card_settings.offset_x,
                y,
            ));
            y += Self::ROW_Y_STEP;
        }
        if let Some(val) = self.merge_requests_count {
            lines.push(self.render_line(
                StatIcon::MergeRequests,
                "Merge Requests",
                val,
                self.card_settings.offset_x,
                y,
            ));
            y += Self::ROW_Y_STEP;
        }
        if let Some(val) = self.reviews_count {
            lines.push(self.render_line(
                StatIcon::Reviews,
                "Reviews",
                val,
                self.card_settings.offset_x,
                y,
            ));
            y += Self::ROW_Y_STEP;
        }
        if let Some(val) = self.started_discussions_count {
            lines.push(self.render_line(
                StatIcon::StartedDiscussions,
                "Started Discussions",
                val,
                self.card_settings.offset_x,
                y,
            ));
            y += Self::ROW_Y_STEP;
        }
        if let Some(val) = self.answered_discussions_count {
            lines.push(self.render_line(
                StatIcon::AnsweredDiscussions,
                "Answered Discussions",
                val,
                self.card_settings.offset_x,
                y,
            ));
        }

        // Calculate card height: top margin + (lines * step) + bottom margin
        let line_count = lines.len().max(1) as u32;
        let height = if self.card_settings.hide_title {
            // Height so last baseline + offset_y is the bottom edge.
            // last_baseline = first_baseline + (lines-1)*ROW_Y_STEP
            // first_baseline = offset_y + ICON_SIZE
            // height = last_baseline + offset_y
            self.card_settings.offset_y * 2 + Self::ICON_SIZE + (line_count - 1) * Self::ROW_Y_STEP
        } else {
            header_size_y + line_count * Self::ROW_Y_STEP + self.card_settings.offset_y * 2
        };
        let width: u32 = Self::LABEL_SIZE
            + Self::ICON_SIZE
            + Self::ICON_OFFSET
            + Self::VALUE_SIZE
            + self.card_settings.offset_x * 2;

        let body = lines.join("\n");

        // Build title respecting username length limit.
        let display_title =
            if self.username.is_empty() || self.username.len() > Self::MAX_USERNAME_LEN {
                "GitHub Stats".to_string()
            } else {
                format!("@{}: GitHub Stats", self.username)
            };

        let card = Card::new(
            width,
            height,
            display_title,
            String::from("GitHub statistics summary"),
            body,
            self.card_settings.clone(),
        );
        match card {
            Ok(card) => card.render(),
            Err(e) => format!("Failed to render StatsCard: {e}"),
        }
    }

    /// Format a numeric value into a shortened human form.
    /// Rules:
    /// - < 1_000 -> plain number (e.g. 999)
    /// - 1_000 ..= 9_999 -> one decimal (floor) unless the decimal would be 0 (e.g. 1k, 1.5k, 9.9k)
    /// - >= 10_000 -> whole thousands with trailing 'k' (e.g. 10k, 11k, 15234 -> 15k)
    fn format_value(&self, value: u32) -> String {
        if value < 1_000 {
            return value.to_string();
        }
        if value < 10_000 {
            let thousands = value / 1_000; // 1..=9
            let tenths = (value % 1_000) / 100; // 0..=9 (floor)
            if tenths == 0 {
                format!("{thousands}k")
            } else {
                format!("{thousands}.{tenths}k")
            }
        } else {
            format!("{}k", value / 1_000)
        }
    }

    fn load_icon(&self, icon: StatIcon, x: u32, y: u32) -> String {
        let svg = match icon {
            StatIcon::Stars => include_str!("../../assets/icons/star.svg"),
            StatIcon::CommitsYTD => include_str!("../../assets/icons/clock-rotate-left.svg"),
            StatIcon::PullRequests => include_str!("../../assets/icons/code-pull-request.svg"),
            StatIcon::Issues => include_str!("../../assets/icons/circle-exclamation.svg"),
            StatIcon::MergeRequests => include_str!("../../assets/icons/code-merge.svg"),
            StatIcon::Reviews => include_str!("../../assets/icons/eye.svg"),
            StatIcon::StartedDiscussions => include_str!("../../assets/icons/messages.svg"),
            StatIcon::AnsweredDiscussions => include_str!("../../assets/icons/message-check.svg"),
        };

        // Insert x and y attributes into the SVG root element
        // Assumes the SVG starts with <svg ...>
        if let Some(idx) = svg.find('>') {
            let (start, rest) = svg.split_at(idx);
            format!("{start} x=\"{x}\" y=\"{y}\" width=\"16\" height=\"16\"{rest}")
        } else {
            svg.to_string()
        }
    }

    /// Renders the line for the [StatsCard].
    fn render_line(
        &self,
        icon: StatIcon,
        label: &str,
        value: u32,
        pos_x: u32,
        pos_y: u32,
    ) -> String {
        let pos_x_label = pos_x + Self::ICON_SIZE + Self::ICON_OFFSET;
        let pos_x_value = pos_x_label + Self::LABEL_SIZE;

        format!(
            r#"<g class="row">
  {icon}
  <text class="label" x="{pos_x_label}" y="{pos_y}">{label}:</text>
  <text class="value" x="{pos_x_value}" y="{pos_y}">{value}</text>
</g>"#,
            icon = self.load_icon(icon, pos_x, pos_y.saturating_sub(Self::ICON_SIZE)),
            pos_x_label = pos_x_label,
            pos_y = pos_y,
            label = label,
            pos_x_value = pos_x_value,
            value = self.format_value(value)
        )
    }
}

enum StatIcon {
    Stars,
    CommitsYTD,
    PullRequests,
    Issues,
    MergeRequests,
    Reviews,
    StartedDiscussions,
    AnsweredDiscussions,
}

#[cfg(test)]
mod tests {
    use super::*;

    mod fn_render_line {
        use super::*;

        #[test]
        fn basic() {
            let card = StatsCard {
                username: "testuser".to_string(),
                ..Default::default()
            };
            let line = card.render_line(StatIcon::Stars, "Stars", 42, 10, 20);
            assert!(line.contains("<g class=\"row\">"));
            assert!(line.contains(">Stars:</text>"));
            assert!(line.contains(">42</text>")); // unchanged for small numbers
            assert!(line.contains("x=\"10"));
            assert!(line.contains("y=\"20"));
        }

        #[test]
        fn formatted_thousands_decimal() {
            let card = StatsCard::default();
            let line = card.render_line(StatIcon::Stars, "Stars", 1_500, 0, 0);
            assert!(line.contains(">1.5k</text>"));
        }

        #[test]
        fn formatted_ten_thousands_whole() {
            let card = StatsCard::default();
            let line = card.render_line(StatIcon::Stars, "Stars", 15_234, 0, 0);
            assert!(line.contains(">15k</text>"));
        }
    }

    mod fn_render {
        use super::*;

        #[test]
        fn with_some_fields() {
            let card = StatsCard {
                username: "octocat".to_string(),
                stars_count: Some(10),
                commits_ytd_count: Some(20),
                ..Default::default()
            };
            let svg = card.render();
            assert!(svg.contains("@octocat: GitHub Stats"));
            assert!(svg.contains(">Stars:</text>"));
            assert!(svg.contains(">10</text>"));
            assert!(svg.contains(">Commits YTD:</text>"));
            assert!(svg.contains(">20</text>"));
            // Should not contain other stat labels
            assert!(!svg.contains(">Issues:</text>"));
        }

        #[test]
        fn with_username_longer_than_limit() {
            let card = StatsCard {
                username: "averylongusername".to_string(),
                stars_count: Some(10),
                commits_ytd_count: Some(20),
                ..Default::default()
            };
            let svg = card.render();
            // Should use default title instead of username
            assert!(svg.contains("GitHub Stats"));
            // Should not contain username in title
            assert!(!svg.contains("@averylongusername"));
        }

        #[test]
        fn svg_is_valid_xml() {
            let card = StatsCard {
                username: "xmluser".to_string(),
                stars_count: Some(1),
                commits_ytd_count: Some(2),
                ..Default::default()
            };
            let svg = card.render();
            // Use quick_xml to check well-formedness
            use quick_xml::Reader;
            use quick_xml::events::Event;
            let mut reader = Reader::from_str(&svg);
            let mut buf = Vec::new();
            let mut found_svg = false;
            loop {
                match reader.read_event_into(&mut buf) {
                    Ok(Event::Start(ref e)) if e.name().as_ref() == b"svg" => {
                        found_svg = true;
                    }
                    Ok(Event::Eof) => break,
                    Ok(_) => (),
                    Err(e) => panic!("Invalid SVG/XML: {e}"),
                }
                buf.clear();
            }
            assert!(found_svg, "SVG root element not found");
        }
    }

    mod fn_format_value {
        use super::*;

        #[test]
        fn under_thousand() {
            let card = StatsCard::default();
            assert_eq!(card.format_value(999), "999");
        }

        #[test]
        fn one_thousand_exact() {
            let card = StatsCard::default();
            assert_eq!(card.format_value(1_000), "1k");
        }

        #[test]
        fn one_thousand_with_decimal() {
            let card = StatsCard::default();
            assert_eq!(card.format_value(1_500), "1.5k");
        }

        #[test]
        fn one_thousand_nine_hundred() {
            let card = StatsCard::default();
            assert_eq!(card.format_value(1_999), "1.9k");
        }

        #[test]
        fn ten_thousand_exact() {
            let card = StatsCard::default();
            assert_eq!(card.format_value(10_000), "10k");
        }

        #[test]
        fn over_ten_thousand_floor() {
            let card = StatsCard::default();
            assert_eq!(card.format_value(15_234), "15k");
        }
    }
}
