use crate::cards::card::{CardSettings, CardTheme, Svg};

pub struct ErrorCard {
    pub card_settings: CardSettings,
    pub error_message: String,
}

impl ErrorCard {
    // Constants for rendering the error card (in pixels).
    const MAX_ERROR_MSG_LEN: usize = 45;
    const TITLE_BODY_OFFSET: u32 = 16;
    const MESSAGE_LINE_HEIGHT: u32 = 22;
    const LINK_OFFSET: u32 = 20;
    const CARD_PADDING: u32 = 16;
    const DOCS_URL: &'static str =
        "https://github.com/samgozman/github-statcrab?tab=readme-ov-file#github-statcrab";

    /// Creates a new ErrorCard with the given error message.
    /// Uses light theme by default with appropriate styling for errors.
    pub fn new(error_message: String) -> Self {
        Self {
            card_settings: CardSettings {
                offset_x: Self::CARD_PADDING,
                offset_y: Self::CARD_PADDING,
                theme: CardTheme::Light, // Use light theme for error cards
                hide_title: false,
                hide_background: false,
                hide_background_stroke: false,
            },
            error_message,
        }
    }

    /// Renders the ErrorCard as an SVG string.
    pub fn render(&self) -> Svg {
        use crate::cards::card::Card;

        // Break the error message into lines if it's too long
        let message_lines = self.break_message_into_lines(&self.error_message);

        // Title block height (title + offset)
        let header_size_y = Card::TITLE_FONT_SIZE + Self::TITLE_BODY_OFFSET;

        // Calculate positions
        let icon_x = self.card_settings.offset_x;
        let icon_y = header_size_y + self.card_settings.offset_y;

        let message_x = icon_x + 40; // Account for circle diameter (32) + offset
        let mut message_y = icon_y + 24; // Start message at a good vertical position        // Render error icon and message lines
        let mut body_parts = Vec::new();

        // Add error icon
        body_parts.push(self.render_error_icon(icon_x, icon_y));

        // Add message lines
        for line in &message_lines {
            body_parts.push(format!(
                r#"<text x="{}" y="{}" class="error-message">{}</text>"#,
                message_x, message_y, line
            ));
            message_y += Self::MESSAGE_LINE_HEIGHT;
        }

        // Add documentation link
        let link_y = message_y + Self::LINK_OFFSET;
        body_parts.push(self.render_docs_link(message_x + 10, link_y));

        // Calculate card dimensions
        let content_height = (message_lines.len() as u32) * Self::MESSAGE_LINE_HEIGHT
            + Self::LINK_OFFSET
            + Self::MESSAGE_LINE_HEIGHT
            + 10; // Extra space for button
        let height = header_size_y + content_height + self.card_settings.offset_y * 2;
        let width = 380;

        let body = body_parts.join("\n");

        let card = Card::new(
            width,
            height,
            String::from("Error"),
            String::from("An error occurred while processing your request"),
            body,
            "errorCard".to_string(),
            self.card_settings.clone(),
        );

        match card {
            Ok(card) => self.add_error_styles(&card.render()),
            Err(e) => format!("Failed to render ErrorCard: {e}"),
        }
    }

    /// Breaks a long error message into multiple lines.
    fn break_message_into_lines(&self, message: &str) -> Vec<String> {
        if message.len() <= Self::MAX_ERROR_MSG_LEN {
            return vec![message.to_string()];
        }

        let mut lines = Vec::new();
        let words: Vec<&str> = message.split_whitespace().collect();
        let mut current_line = String::new();

        for word in words {
            if current_line.is_empty() {
                current_line = word.to_string();
            } else if current_line.len() + word.len() < Self::MAX_ERROR_MSG_LEN {
                current_line.push(' ');
                current_line.push_str(word);
            } else {
                lines.push(current_line);
                current_line = word.to_string();
            }
        }

        if !current_line.is_empty() {
            lines.push(current_line);
        }

        lines
    }

    /// Renders the error icon using emoji.
    fn render_error_icon(&self, x: u32, y: u32) -> String {
        // Use a more visible exclamation mark in a circle
        let circle_cx = x + 16;
        let circle_cy = y + 16;
        let icon_x = x + 16;
        let icon_y = y + 16; // Center vertically with the circle

        format!(
            "<g class=\"error-icon-container\">\n  <circle cx=\"{}\" cy=\"{}\" r=\"16\" fill=\"#fee2e2\" stroke=\"#fca5a5\" stroke-width=\"1.5\"/>\n  <text x=\"{}\" y=\"{}\" font-size=\"20\" font-weight=\"bold\" class=\"error-icon\" text-anchor=\"middle\" dominant-baseline=\"central\">!</text>\n</g>",
            circle_cx, circle_cy, icon_x, icon_y
        )
    }

    /// Renders a clickable link to the documentation.
    fn render_docs_link(&self, x: u32, y: u32) -> String {
        format!(
            "<g class=\"docs-link-container\">\n  <rect x=\"{}\" y=\"{}\" width=\"266\" height=\"28\" rx=\"6\" fill=\"#f0f9ff\" stroke=\"#0ea5e9\" stroke-width=\"1\" class=\"docs-link-bg\"/>\n  <a href=\"{}\" target=\"_blank\" class=\"docs-link\">\n    <text x=\"{}\" y=\"{}\" class=\"link-text\">📚 Readme: samgozman/github-statcrab</text>\n  </a>\n</g>",
            x - 8,
            y - 20, // Background rectangle position
            Self::DOCS_URL,
            x + 4,
            y - 2 // Text position (centered in the rectangle)
        )
    }

    /// Adds error-specific styles to the SVG.
    fn add_error_styles(&self, svg: &str) -> String {
        let error_styles = include_str!("../../assets/css/error-card.css");

        // Insert the error styles into the existing style block
        if let Some(style_end) = svg.find("  </style>") {
            let (before, after) = svg.split_at(style_end);
            format!("{}{}\n{}", before, error_styles, after)
        } else {
            svg.to_string()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_card_creation() {
        let card = ErrorCard::new("Test error message".to_string());
        assert_eq!(card.error_message, "Test error message");
        // CardTheme doesn't implement PartialEq, so we can't use assert_eq!
        // The theme is set to Light in the constructor
    }

    #[test]
    fn test_break_message_into_lines_short() {
        let card = ErrorCard::new("Short".to_string());
        let lines = card.break_message_into_lines("Short message");
        assert_eq!(lines, vec!["Short message"]);
    }

    #[test]
    fn test_break_message_into_lines_long() {
        let card = ErrorCard::new("".to_string());
        let long_message = "This is a very long error message that should be broken into multiple lines for better readability in the error card";
        let lines = card.break_message_into_lines(long_message);

        assert!(lines.len() > 1);
        for line in &lines {
            assert!(line.len() <= ErrorCard::MAX_ERROR_MSG_LEN);
        }
    }

    #[test]
    fn test_render_produces_valid_svg() {
        let card = ErrorCard::new("Test error".to_string());
        let svg = card.render();

        // Basic SVG structure checks
        assert!(svg.contains("<svg"));
        assert!(svg.contains("</svg>"));
        assert!(svg.contains("Error"));
        assert!(svg.contains("Test error"));
        assert!(svg.contains("Readme: samgozman/github-statcrab"));
        assert!(svg.contains(ErrorCard::DOCS_URL));
    }

    #[test]
    fn test_render_error_icon() {
        let card = ErrorCard::new("Test".to_string());
        let icon = card.render_error_icon(10, 20);

        assert!(icon.contains("error-icon"));
        assert!(icon.contains("!"));
        assert!(icon.contains("circle"));
        assert!(icon.contains("#fee2e2"));
        assert!(icon.contains("font-weight=\"bold\""));
    }

    #[test]
    fn test_render_docs_link() {
        let card = ErrorCard::new("Test".to_string());
        let link = card.render_docs_link(50, 100);

        assert!(link.contains("<a href="));
        assert!(link.contains(ErrorCard::DOCS_URL));
        assert!(link.contains("Readme: samgozman/github-statcrab"));
        assert!(link.contains("docs-link"));
        assert!(link.contains("rect")); // Button background
    }

    #[test]
    fn test_add_error_styles() {
        let card = ErrorCard::new("Test".to_string());
        let base_svg = r#"<svg><style>
  .title { fill: black; }
  </style></svg>"#;

        let styled_svg = card.add_error_styles(base_svg);

        assert!(styled_svg.contains(".error-message"));
        assert!(styled_svg.contains("#991b1b")); // Updated error color
        assert!(styled_svg.contains(".link-text"));
        assert!(styled_svg.contains("#0284c7")); // Updated link color
        assert!(styled_svg.contains("drop-shadow")); // Icon styling
    }
}
