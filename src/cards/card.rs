/// SVG is a type alias for [String], representing an SVG representation of a card.
pub type SVG = String;

/// CardSettings holds unique settings for the [Card].
pub struct CardSettings {
    /// Offset (pixels) is used to adjust the position of the [Card] in the SVG relative to its container.
    pub offset: f32,
    /// Hide title title of the [Card].
    pub hide_title: bool,
    /// Hide background of the [Card].
    pub hide_background: bool,
}

/// Card represents a card with a width, height, and title. Its a base wrapper for cards of different types.
/// It provides a method to create a new card and render it as an [SVG] string.
pub struct Card {
    width: i32,
    height: i32,
    title: String,
    description: String,
    body: String,
    /// The CSS base style for the card, loaded from an external file.
    style: String,
    settings: CardSettings,
}

impl Card {
    /// Creates a new [Card] with the specified parameters.
    pub fn new(
        width: i32,
        height: i32,
        title: String,
        description: String,
        body: String,
        settings: CardSettings,
    ) -> Result<Self, String> {
        let card = Card {
            width,
            height,
            description,
            title,
            body,
            style: Self::load_style(),
            settings,
        };
        card.validate()?;
        Ok(card)
    }

    /// Renders the [Card] as an [SVG] string.
    pub fn render(&self) -> SVG {
        let style = Self::indent_style(&self.style);
        let rendered_background = if !self.settings.hide_background {
            self.render_background()
        } else {
            String::new()
        };
        let rendered_title = if !self.settings.hide_title {
            self.render_title()
        } else {
            String::new()
        };

        format!(
            r#"<svg
  width="{width}"
  height="{height}"
  viewBox="0 0 {width} {height}"
  fill="none"
  xmlns="http://www.w3.org/2000/svg"
  role="img"
  aria-labelledby="title-id"
  aria-describedby="description-id"
>
  <style>
{style}  </style>
  <title id="title-id">{title}</title>
  <desc id="description-id">{description}</desc>
  {rendered_background}
  {rendered_title}
  {body}
</svg>
"#,
            width = self.width,
            height = self.height,
            title = self.title,
            description = self.description,
            body = self.body,
            rendered_background = rendered_background,
            rendered_title = rendered_title,
            style = style
        )
    }

    /// Validates the [Card]'s dimensions and settings.
    fn validate(&self) -> Result<(), String> {
        if self.width < 100 {
            return Err(format!(
                "Card width must be at least 100, got {}",
                self.width
            ));
        }
        if self.height < 100 {
            return Err(format!(
                "Card height must be at least 100, got {}",
                self.height
            ));
        }
        let max_offset_w = self.width as f32 * 0.3;
        let max_offset_h = self.height as f32 * 0.3;
        if self.settings.offset > max_offset_w || self.settings.offset > max_offset_h {
            return Err(format!(
                "Card offset must not exceed 30% of width or height (max: {}, {}), got {}",
                max_offset_w, max_offset_h, self.settings.offset
            ));
        }
        Ok(())
    }

    /// Loads the CSS style for the [Card] from a file.
    fn load_style() -> String {
        // Embed the CSS file into the binary at compile time
        include_str!("../../assets/css/card.css").to_string()
    }

    /// Indents each line of the style string by two spaces.
    fn indent_style(style: &str) -> String {
        style.lines().map(|line| format!("  {}\n", line)).collect()
    }

    /// Renders the title of the [Card] as an SVG text element.
    fn render_title(&self) -> String {
        format!(
            r#"<text x="{}" y="16" class="title">{}</text>"#,
            self.settings.offset * 2.0,
            self.title
        )
    }

    fn render_background(&self) -> String {
        format!(
            r#"<rect x="{pos_x}" y="{pos_y}" rx="5" width="{width}" height="{height}" stroke="{stroke_color}" fill="{fill_color}" stroke-opacity="{stroke_opacity}"/>"#,
            pos_x = self.settings.offset,
            pos_y = self.settings.offset,
            width = self.width as f32 - self.settings.offset * 2.0,
            height = self.height as f32 - self.settings.offset * 2.0,
            fill_color = "#ffffff00",
            stroke_color = "#e6e1e1ff",
            stroke_opacity = "1",
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    mod fn_new {
        use super::*;

        #[test]
        fn test_card_creation_valid() {
            let card = Card::new(
                100,
                120,
                "Test Card".to_string(),
                "Test Desc".to_string(),
                "Test Body".to_string(),
                CardSettings {
                    offset: 10.0,
                    hide_title: false,
                    hide_background: false,
                },
            )
            .expect("Card should be valid");
            assert_eq!(card.width, 100);
            assert_eq!(card.height, 120);
            assert_eq!(card.title, "Test Card");
            assert_eq!(card.description, "Test Desc");
            assert_eq!(card.body, "Test Body");
        }

        #[test]
        fn test_card_creation_invalid_width() {
            let card = Card::new(
                99,
                120,
                "Test Card".to_string(),
                "Test Desc".to_string(),
                "Test Body".to_string(),
                CardSettings {
                    offset: 10.0,
                    hide_title: false,
                    hide_background: false,
                },
            );
            assert!(card.is_err());
        }

        #[test]
        fn test_card_creation_invalid_height() {
            let card = Card::new(
                100,
                99,
                "Test Card".to_string(),
                "Test Desc".to_string(),
                "Test Body".to_string(),
                CardSettings {
                    offset: 10.0,
                    hide_title: false,
                    hide_background: false,
                },
            );
            assert!(card.is_err());
        }

        #[test]
        fn test_card_creation_invalid_offset() {
            let card = Card::new(
                100,
                120,
                "Test Card".to_string(),
                "Test Desc".to_string(),
                "Test Body".to_string(),
                CardSettings {
                    offset: 50.0,
                    hide_title: false,
                    hide_background: false,
                },
            );
            assert!(card.is_err());
        }
    }

    mod fn_load_style {
        use super::*;

        #[test]
        fn test_load_style() {
            let style = Card::load_style();
            assert!(!style.is_empty(), "Style should not be empty");
        }
    }

    mod fn_render_title {
        use super::*;

        #[test]
        fn test_render_title() {
            let card = Card::new(
                100,
                120,
                "Test Title".to_string(),
                "".to_string(),
                "".to_string(),
                CardSettings {
                    offset: 0.5,
                    hide_title: false,
                    hide_background: false,
                },
            )
            .unwrap();
            let rendered_title = card.render_title();
            assert_eq!(
                rendered_title,
                r#"<text x="1" y="16" class="title">Test Title</text>"#
            );
        }
    }

    mod fn_render {
        #[test]
        fn test_render_hides_title_svg_text() {
            let card = Card::new(
                100,
                120,
                "Test Title".to_string(),
                "Test Desc".to_string(),
                "Test Body".to_string(),
                CardSettings {
                    offset: 0.5,
                    hide_title: true,
                    hide_background: false,
                },
            )
            .unwrap();
            let svg = card.render();
            // The <title> tag should always be present
            assert!(svg.contains("<title id=\"title-id\">Test Title</title>"));
            // The SVG text element should NOT be present
            assert!(!svg.contains("<text x="));
        }

        #[test]
        fn test_render_hides_background_rect() {
            let card = Card::new(
                100,
                120,
                "Test Title".to_string(),
                "Test Desc".to_string(),
                "Test Body".to_string(),
                CardSettings {
                    offset: 0.5,
                    hide_title: false,
                    hide_background: true,
                },
            )
            .unwrap();
            let svg = card.render();
            // The background <rect> should NOT be present
            assert!(!svg.contains("<rect "));
            // The SVG text element should be present
            assert!(svg.contains("<text x="));
        }

        #[test]
        fn test_render_hides_both_title_and_background() {
            let card = Card::new(
                100,
                120,
                "Test Title".to_string(),
                "Test Desc".to_string(),
                "Test Body".to_string(),
                CardSettings {
                    offset: 0.5,
                    hide_title: true,
                    hide_background: true,
                },
            )
            .unwrap();
            let svg = card.render();
            // The <title> tag should always be present
            assert!(svg.contains("<title id=\"title-id\">Test Title</title>"));
            // The SVG text element should NOT be present
            assert!(!svg.contains("<text x="));
            // The background <rect> should NOT be present
            assert!(!svg.contains("<rect "));
        }
        use super::*;
        use quick_xml::Reader;
        use quick_xml::events::Event;

        #[test]
        fn test_render_svg_is_valid_xml() {
            let card = Card::new(
                100,
                200,
                "SVG Card".to_string(),
                "SVG Description".to_string(),
                "<rect width=\"100\" height=\"200\" fill=\"#fff\"/>".to_string(),
                CardSettings {
                    offset: 0.5,
                    hide_title: false,
                    hide_background: false,
                },
            )
            .unwrap();
            let svg = card.render();

            // Validate SVG is well-formed XML by parsing the entire document
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
