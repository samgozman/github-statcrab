use crate::cards::card::{Card, CardSettings, Svg};

pub struct ErrorCard {
    pub card_settings: CardSettings,
    pub title: String,
    pub error_message: String,
}

impl ErrorCard {
    // Constants for rendering the error card (in pixels).
    const WIDTH: u32 = 400;
    const HEIGHT: u32 = 160;
    const ERROR_ICON_SIZE: u32 = 20;
    const ERROR_ICON_OFFSET: u32 = 8;
    const MESSAGE_Y_OFFSET: u32 = 45;
    const MESSAGE_LINE_HEIGHT: u32 = 16;

    /// Creates a new [ErrorCard] with the specified error message.
    pub fn new(title: String, error_message: String, card_settings: CardSettings) -> Self {
        Self {
            card_settings,
            title,
            error_message,
        }
    }

    /// Renders the [ErrorCard] as an [Svg] string.
    pub fn render(&self) -> Svg {
        let body = self.render_body();

        let card = Card::new(
            Self::WIDTH,
            Self::HEIGHT,
            self.title.clone(),
            "Error displaying GitHub statistics".to_string(),
            body,
            "error-card".to_string(),
            self.card_settings.clone(),
        );

        match card {
            Ok(card) => card.render(),
            Err(_) => {
                // Fallback to simple error SVG if card creation fails
                let width = Self::WIDTH;
                let height = Self::HEIGHT;
                format!(
                    concat!(
                        "<svg width=\"{}\" height=\"{}\" viewBox=\"0 0 {} {}\" xmlns=\"http://www.w3.org/2000/svg\">",
                        "<rect width=\"100%\" height=\"100%\" fill=\"#ff6b6b\" rx=\"5\"/>",
                        "<text x=\"50%\" y=\"50%\" text-anchor=\"middle\" dominant-baseline=\"middle\" fill=\"white\" font-family=\"Arial\" font-size=\"14\">",
                        "Error: Failed to generate card",
                        "</text>",
                        "</svg>"
                    ),
                    width, height, width, height
                )
            }
        }
    }

    /// Renders the body content of the error card.
    fn render_body(&self) -> String {
        let icon_x = self.card_settings.offset_x + Self::ERROR_ICON_OFFSET;
        let icon_y = self.card_settings.offset_y + Self::MESSAGE_Y_OFFSET;
        let text_x = icon_x + Self::ERROR_ICON_SIZE + Self::ERROR_ICON_OFFSET;
        let text_y = icon_y + Self::ERROR_ICON_SIZE / 2;

        // Split error message into lines (max ~45 characters per line for readability)
        let lines = self.wrap_text(&self.error_message, 45);
        let mut message_elements = String::new();

        for (i, line) in lines.iter().enumerate() {
            let line_y = text_y + (i as u32 * Self::MESSAGE_LINE_HEIGHT);
            message_elements.push_str(&format!(
                r#"<text x="{}" y="{}" class="error-message" font-size="12">{}</text>"#,
                text_x,
                line_y,
                html_escape::encode_text(line)
            ));
        }

        format!(
            r#"<g class="error-content">
  {error_icon}
  {message_elements}
</g>"#,
            error_icon = self.render_error_icon(icon_x, icon_y),
            message_elements = message_elements,
        )
    }

    /// Renders an error icon (exclamation triangle).
    fn render_error_icon(&self, x: u32, y: u32) -> String {
        let points = format!(
            "{},{} {},{} {},{}",
            x + Self::ERROR_ICON_SIZE / 2,
            y, // top point
            x,
            y + Self::ERROR_ICON_SIZE, // bottom left
            x + Self::ERROR_ICON_SIZE,
            y + Self::ERROR_ICON_SIZE, // bottom right
        );
        let text_x = x + Self::ERROR_ICON_SIZE / 2;
        let text_y = y + Self::ERROR_ICON_SIZE / 2 + 1;

        format!(
            concat!(
                "<g class=\"error-icon\">",
                "<polygon points=\"{}\" fill=\"#ff6b6b\" stroke=\"#d63031\" stroke-width=\"1\"/>",
                "<text x=\"{}\" y=\"{}\" text-anchor=\"middle\" dominant-baseline=\"central\" fill=\"white\" font-weight=\"bold\" font-size=\"14\">!</text>",
                "</g>"
            ),
            points, text_x, text_y
        )
    }

    /// Wraps text to fit within the specified character width.
    fn wrap_text(&self, text: &str, max_width: usize) -> Vec<String> {
        let mut lines = Vec::new();
        let mut current_line = String::new();

        for word in text.split_whitespace() {
            if current_line.len() + word.len() + 1 > max_width && !current_line.is_empty() {
                lines.push(current_line.clone());
                current_line.clear();
            }
            if !current_line.is_empty() {
                current_line.push(' ');
            }
            current_line.push_str(word);
        }

        if !current_line.is_empty() {
            lines.push(current_line);
        }

        if lines.is_empty() {
            lines.push("Unknown error".to_string());
        }

        lines
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cards::card::CardTheme;

    #[test]
    fn test_error_card_creation() {
        let card = ErrorCard::new(
            "GitHub API Error".to_string(),
            "Failed to fetch user data".to_string(),
            CardSettings {
                offset_x: 12,
                offset_y: 12,
                theme: CardTheme::TransparentBlue,
                hide_title: false,
                hide_background: false,
                hide_background_stroke: false,
            },
        );

        assert_eq!(card.title, "GitHub API Error");
        assert_eq!(card.error_message, "Failed to fetch user data");
    }

    #[test]
    fn test_error_card_render() {
        let card = ErrorCard::new(
            "API Error".to_string(),
            "GitHub API is currently unavailable".to_string(),
            CardSettings {
                offset_x: 12,
                offset_y: 12,
                theme: CardTheme::TransparentBlue,
                hide_title: false,
                hide_background: false,
                hide_background_stroke: false,
            },
        );

        let svg = card.render();
        assert!(svg.contains("<svg"));
        assert!(svg.contains("API Error"));
        assert!(svg.contains("error-card"));
        assert!(svg.contains("error-content"));
        assert!(svg.contains("GitHub API is currently unavailable"));
    }

    #[test]
    fn test_wrap_text() {
        let card = ErrorCard::new(
            "Test".to_string(),
            "".to_string(),
            CardSettings {
                offset_x: 0,
                offset_y: 0,
                theme: CardTheme::TransparentBlue,
                hide_title: false,
                hide_background: false,
                hide_background_stroke: false,
            },
        );

        let lines = card.wrap_text(
            "This is a very long error message that should be wrapped",
            20,
        );
        assert!(lines.len() > 1);
        assert!(lines.iter().all(|line| line.len() <= 25)); // Allow some flexibility
    }

    #[test]
    fn test_wrap_text_empty() {
        let card = ErrorCard::new(
            "Test".to_string(),
            "".to_string(),
            CardSettings {
                offset_x: 0,
                offset_y: 0,
                theme: CardTheme::TransparentBlue,
                hide_title: false,
                hide_background: false,
                hide_background_stroke: false,
            },
        );

        let lines = card.wrap_text("", 20);
        assert_eq!(lines.len(), 1);
        assert_eq!(lines[0], "Unknown error");
    }
}
