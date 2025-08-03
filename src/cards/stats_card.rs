use crate::cards::card::SVG;

pub struct StatsCard {
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
    /// Renders the [StatsCard] as an [SVG] string.
    pub fn render(&self) -> SVG {
        use crate::cards::card::{Card, CardSettings};

        // Prepare stat lines (label, value, Option)
        let mut lines = Vec::new();
        let mut y = 50.0;
        let y_step = 32.0;
        let x = 32.0;

        if let Some(val) = self.stars_count {
            lines.push(self.render_line("Stars", val, x, y));
            y += y_step;
        }
        if let Some(val) = self.commits_ytd_count {
            lines.push(self.render_line("Commits YTD", val, x, y));
            y += y_step;
        }
        if let Some(val) = self.issues_count {
            lines.push(self.render_line("Issues", val, x, y));
            y += y_step;
        }
        if let Some(val) = self.pull_requests_count {
            lines.push(self.render_line("Pull Requests", val, x, y));
            y += y_step;
        }
        if let Some(val) = self.merge_requests_count {
            lines.push(self.render_line("Merge Requests", val, x, y));
            y += y_step;
        }
        if let Some(val) = self.reviews_count {
            lines.push(self.render_line("Reviews", val, x, y));
            y += y_step;
        }
        if let Some(val) = self.started_discussions_count {
            lines.push(self.render_line("Started Discussions", val, x, y));
            y += y_step;
        }
        if let Some(val) = self.answered_discussions_count {
            lines.push(self.render_line("Answered Discussions", val, x, y));
        }

        // Calculate card height: top margin + (lines * step) + bottom margin
        let line_count = lines.len().max(1);
        let height = 40 + (line_count as u32) * (y_step as u32) + 40;
        let width = 380;

        let body = lines.join("\n");

        let card = Card::new(
            width,
            height,
            format!("@{}'s GitHub Stats", self.username),
            String::from("GitHub statistics summary"),
            body,
            CardSettings {
                offset: 4.0,
                hide_title: false,
                hide_background: false,
            },
        );
        match card {
            Ok(card) => card.render(),
            Err(e) => format!("Failed to render StatsCard: {}", e),
        }
    }

    /// Renders the line for the [StatsCard].
    fn render_line(&self, label: &str, value: u32, pos_x: f32, pos_y: f32) -> String {
        let pos_x_offset: f32 = 200.0;

        format!(
            r#"<g class="stat_row">
  <svg class="icon" viewBox="0 0 16 16" width="16" height="16"><!-- TODO: Add icon --></svg>
  <text x="{pos_x_label}" y="{pos_y}">{label}:</text>
  <text x="{pos_x_value}" y="{pos_y}">{value}</text>
</g>"#,
            pos_x_label = pos_x,
            pos_y = pos_y,
            label = label,
            pos_x_value = pos_x + pos_x_offset,
            value = value
        )
    }
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
            let line = card.render_line("Stars", 42, 10.0, 20.0);
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
            assert!(svg.contains("@octocat's GitHub Stats"));
            assert!(svg.contains(">Stars:</text>"));
            assert!(svg.contains(">10</text>"));
            assert!(svg.contains(">Commits YTD:</text>"));
            assert!(svg.contains(">20</text>"));
            // Should not contain other stat labels
            assert!(!svg.contains(">Issues:</text>"));
        }

        #[test]
        fn with_no_fields() {
            let mut card = StatsCard::default();
            card.username = "empty".to_string();
            let svg = card.render();
            // Should still render a valid SVG with title
            assert!(svg.contains("@empty's GitHub Stats"));
            // Should not contain any stat label
            assert!(!svg.contains(":</text>"));
        }

        #[test]
        fn svg_is_valid_xml() {
            let mut card = StatsCard::default();
            card.username = "xmluser".to_string();
            card.stars_count = Some(1);
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
