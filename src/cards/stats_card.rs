use crate::cards::card::{CardSettings, SVG};

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
                hide_title: false,
                hide_background: false,
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
    const VALUE_SIZE: u32 = 27;
    const LABEL_SIZE: u32 = 220;
    const ICON_SIZE: u32 = 16;
    const ICON_OFFSET: u32 = 8;
    const TITLE_BODY_OFFSET: u32 = 1;
    const ROW_Y_STEP: u32 = 28;

    /// Renders the [StatsCard] as an [SVG] string.
    pub fn render(&self) -> SVG {
        use crate::cards::card::Card;

        // Prepare stat lines (label, value, Option)
        let mut lines = Vec::new();
        let header_size_y = Card::TITLE_FONT_SIZE + Self::TITLE_BODY_OFFSET;

        let mut y: u32 = header_size_y + Self::ROW_Y_STEP + self.card_settings.offset_y;

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
        let height =
            header_size_y + line_count * Self::ROW_Y_STEP + self.card_settings.offset_y * 2;
        let width: u32 = Self::LABEL_SIZE
            + Self::ICON_SIZE
            + Self::ICON_OFFSET
            + Self::VALUE_SIZE
            + self.card_settings.offset_x * 2;

        let body = lines.join("\n");

        let card = Card::new(
            width,
            height,
            format!("@{}'s GitHub Stats", self.username),
            String::from("GitHub statistics summary"),
            body,
            self.card_settings.clone(),
        );
        match card {
            Ok(card) => card.render(),
            Err(e) => format!("Failed to render StatsCard: {}", e),
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
            format!(
                "{} x=\"{}\" y=\"{}\" width=\"16\" height=\"16\"{}",
                start, x, y, rest
            )
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
            r#"<g class="stat_row">
  {icon}
  <text x="{pos_x_label}" y="{pos_y}">{label}:</text>
  <text x="{pos_x_value}" y="{pos_y}">{value}</text>
</g>"#,
            icon = self.load_icon(icon, pos_x, pos_y - Self::ICON_SIZE),
            pos_x_label = pos_x_label,
            pos_y = pos_y,
            label = label,
            pos_x_value = pos_x_value,
            value = value
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
            let mut card = StatsCard::default();
            card.username = "testuser".to_string();
            let line = card.render_line(StatIcon::Stars, "Stars", 42, 10, 20);
            assert!(line.contains("<g class=\"stat_row\">"));
            assert!(line.contains(">Stars:</text>"));
            assert!(line.contains(">42</text>"));
            assert!(line.contains("x=\"10"));
            assert!(line.contains("y=\"20"));
        }
    }

    mod fn_render {
        use super::*;

        #[test]
        fn with_some_fields() {
            let mut card = StatsCard::default();
            card.username = "octocat".to_string();
            card.stars_count = Some(10);
            card.commits_ytd_count = Some(20);
            let svg = card.render();
            println!("{}", svg);
            assert!(svg.contains("@octocat's GitHub Stats"));
            assert!(svg.contains(">Stars:</text>"));
            assert!(svg.contains(">10</text>"));
            assert!(svg.contains(">Commits YTD:</text>"));
            assert!(svg.contains(">20</text>"));
            // Should not contain other stat labels
            assert!(!svg.contains(">Issues:</text>"));
        }

        #[test]
        fn svg_is_valid_xml() {
            let mut card = StatsCard::default();
            card.username = "xmluser".to_string();
            card.stars_count = Some(1);
            card.commits_ytd_count = Some(2);
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
                    Err(e) => panic!("Invalid SVG/XML: {}", e),
                }
                buf.clear();
            }
            assert!(found_svg, "SVG root element not found");
        }
    }
}
