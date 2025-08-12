/// Svg is a type alias for [String], representing an SVG representation of a card.
pub type Svg = String;

use card_theme_macros::build_card_themes;
build_card_themes!();

/// CardSettings holds unique settings for the [Card].
#[derive(Clone)]
pub struct CardSettings {
    /// Offset X (pixels) is used to offset the position of the [Card] in the SVG relative to its container by X axis.
    pub offset_x: u32,
    /// Offset Y (pixels) is used to offset the position of the [Card] in the SVG relative to its container by Y axis.
    pub offset_y: u32,
    /// Theme of the [Card].
    pub theme: CardTheme,
    /// Hide title title of the [Card].
    pub hide_title: bool,
    /// Hide background of the [Card].
    pub hide_background: bool,
    /// Hide stroke (outline) of background rectangle while preserving layout.
    pub hide_background_stroke: bool,
}

/// Card represents a card with a width, height, and title. Its a base wrapper for cards of different types.
/// It provides a method to create a new card and render it as an [Svg] string.
pub struct Card {
    width: u32,
    height: u32,
    title: String,
    description: String,
    body: String,
    /// The CSS base style for the card, loaded from an external file.
    style: String,
    /// The outer class name for the card, used for styling.
    outer_class: String,
    settings: CardSettings,
}

impl Card {
    pub const TITLE_FONT_SIZE: u32 = 18;

    /// Creates a new [Card] with the specified parameters.
    pub fn new(
        width: u32,
        height: u32,
        title: String,
        description: String,
        body: String,
        outer_class: String,
        settings: CardSettings,
    ) -> anyhow::Result<Self, anyhow::Error> {
        let card = Card {
            width,
            height,
            description,
            title,
            body,
            style: Self::load_style(),
            settings,
            outer_class,
        };
        card.validate().map_err(anyhow::Error::msg)?;
        Ok(card)
    }

    /// Renders the [Card] as an [Svg] string.
    pub fn render(&self) -> Svg {
        let theme = self.load_theme_style();
        // Merge the theme style with the base style, indenting it for readability.
        let base_style = self.style.as_str();
        let style = Self::indent(&format!("{base_style}\n{theme}"), 2);

        let body = Self::indent(&self.body, 4);
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
  <g class="{outer_class}" x="0" y="0">
{body}
  </g>
</svg>
"#,
            width = self.width,
            height = self.height,
            title = self.title,
            description = self.description,
            outer_class = self.outer_class,
            body = body,
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
        if self.height < 60 {
            return Err(format!(
                "Card height must be at least 100, got {}",
                self.height
            ));
        }
        let max_offset_w = (self.width as f32 * 0.3) as u32;
        let max_offset_h = (self.height as f32 * 0.3) as u32;
        if self.settings.offset_x > max_offset_w || self.settings.offset_y > max_offset_h {
            return Err(format!(
                "Card offset must not exceed 30% of width or height (max: {}, {}), got x:{} y:{}",
                max_offset_w, max_offset_h, self.settings.offset_x, self.settings.offset_y
            ));
        }

        if self.settings.offset_x >= self.width / 2 || self.settings.offset_y >= self.height / 2 {
            return Err(format!(
                "Card offset must be less than half of width and height (max: {}, {}), got x:{} y:{}",
                self.width as f32 / 2.0,
                self.height as f32 / 2.0,
                self.settings.offset_x,
                self.settings.offset_y
            ));
        }
        Ok(())
    }

    /// Loads the CSS style for the [Card] from a file.
    fn load_style() -> String {
        // Embed the CSS file into the binary at compile time
        include_str!("../../assets/css/card.css").to_string()
    }

    /// Indents each line by the given number of spaces.
    fn indent(lines: &str, spaces: usize) -> String {
        let pad = " ".repeat(spaces);
        lines.lines().map(|line| format!("{pad}{line}\n")).collect()
    }

    /// Renders the title of the [Card] as an SVG text element.
    fn render_title(&self) -> String {
        format!(
            r#"<g transform="translate({}, {})"><text x="0" y="0" class="title">{}</text></g>"#,
            self.settings.offset_x,
            Self::TITLE_FONT_SIZE + self.settings.offset_y,
            self.title
        )
    }

    fn render_background(&self) -> String {
        // If stroke hidden - remove half-pixel inset so fill spans full size.
        let stroke_offset: f32 = if self.settings.hide_background_stroke {
            0.0
        } else {
            0.5
        };
        let stroke_opacity = if self.settings.hide_background_stroke {
            "0"
        } else {
            "1"
        };

        format!(
            r#"<rect class="background" x="{pos_x}" y="{pos_y}" rx="5" width="{width}" height="{height}" stroke-opacity="{stroke_opacity}"/>"#,
            pos_x = stroke_offset,
            pos_y = stroke_offset,
            width = self.width as f32 - stroke_offset * 2.0,
            height = self.height as f32 - stroke_offset * 2.0,
            stroke_opacity = stroke_opacity,
        )
    }

    fn load_theme_style(&self) -> String {
        self.settings.theme.load_css().to_string()
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
                "test-card".to_string(),
                CardSettings {
                    offset_x: 10,
                    offset_y: 10,
                    theme: CardTheme::TransparentBlue,
                    hide_title: false,
                    hide_background: false,
                    hide_background_stroke: false,
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
                "test-card".to_string(),
                CardSettings {
                    offset_x: 10,
                    offset_y: 10,
                    theme: CardTheme::TransparentBlue,
                    hide_title: false,
                    hide_background: false,
                    hide_background_stroke: false,
                },
            );
            assert!(card.is_err());
        }

        #[test]
        fn test_card_creation_invalid_height() {
            let card = Card::new(
                100,
                50,
                "Test Card".to_string(),
                "Test Desc".to_string(),
                "Test Body".to_string(),
                "test-card".to_string(),
                CardSettings {
                    offset_x: 10,
                    offset_y: 10,
                    theme: CardTheme::TransparentBlue,
                    hide_title: false,
                    hide_background: false,
                    hide_background_stroke: false,
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
                "test-card".to_string(),
                CardSettings {
                    offset_x: 50,
                    offset_y: 10,
                    theme: CardTheme::TransparentBlue,
                    hide_title: false,
                    hide_background: false,
                    hide_background_stroke: false,
                },
            );
            assert!(card.is_err());
        }

        #[test]
        fn test_card_creation_offset_too_large() {
            let card = Card::new(
                100,
                120,
                "Test Card".to_string(),
                "Test Desc".to_string(),
                "Test Body".to_string(),
                "test-card".to_string(),
                CardSettings {
                    offset_x: 60,
                    offset_y: 10,
                    theme: CardTheme::TransparentBlue,
                    hide_title: false,
                    hide_background: false,
                    hide_background_stroke: false,
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
                "".to_string(),
                CardSettings {
                    offset_x: 1,
                    offset_y: 1,
                    theme: CardTheme::TransparentBlue,
                    hide_title: false,
                    hide_background: false,
                    hide_background_stroke: false,
                },
            )
            .unwrap();
            let rendered_title = card.render_title();
            assert_eq!(
                rendered_title,
                r#"<g transform="translate(1, 19)"><text x="0" y="0" class="title">Test Title</text></g>"#
            );
        }
    }

    mod fn_render {
        use super::*;

        #[test]
        fn test_render_background_stroke_visible() {
            let card = Card::new(
                120,
                80,
                "Title".to_string(),
                "Desc".to_string(),
                "Body".to_string(),
                "".to_string(),
                CardSettings {
                    offset_x: 1,
                    offset_y: 1,
                    theme: CardTheme::TransparentBlue,
                    hide_title: true,
                    hide_background: false,
                    hide_background_stroke: false,
                },
            )
            .unwrap();
            let svg = card.render();
            assert!(svg.contains("stroke-opacity=\"1\""));
            assert!(svg.contains("x=\"0.5\" y=\"0.5\""));
        }

        #[test]
        fn test_render_background_stroke_hidden() {
            let card = Card::new(
                120,
                80,
                "Title".to_string(),
                "Desc".to_string(),
                "Body".to_string(),
                "".to_string(),
                CardSettings {
                    offset_x: 1,
                    offset_y: 1,
                    theme: CardTheme::TransparentBlue,
                    hide_title: true,
                    hide_background: false,
                    hide_background_stroke: true,
                },
            )
            .unwrap();
            let svg = card.render();
            assert!(svg.contains("stroke-opacity=\"0\""));
            assert!(svg.contains("x=\"0\" y=\"0\""));
        }
        #[test]
        fn test_render_hides_title_svg_text() {
            let card = Card::new(
                100,
                120,
                "Test Title".to_string(),
                "Test Desc".to_string(),
                "Test Body".to_string(),
                "test-card".to_string(),
                CardSettings {
                    offset_x: 1,
                    offset_y: 1,
                    theme: CardTheme::TransparentBlue,
                    hide_title: true,
                    hide_background: false,
                    hide_background_stroke: false,
                },
            )
            .unwrap();
            let svg = card.render();
            // The <title> tag should always be present
            assert!(svg.contains("<title id=\"title-id\">Test Title</title>"));
            // The SVG <g> title group should NOT be present
            assert!(!svg.contains("<g transform="));
        }

        #[test]
        fn test_render_hides_background_rect() {
            let card = Card::new(
                100,
                120,
                "Test Title".to_string(),
                "Test Desc".to_string(),
                "Test Body".to_string(),
                "test-card".to_string(),
                CardSettings {
                    offset_x: 1,
                    offset_y: 1,
                    theme: CardTheme::TransparentBlue,
                    hide_title: false,
                    hide_background: true,
                    hide_background_stroke: false,
                },
            )
            .unwrap();
            let svg = card.render();
            // The background <rect> should NOT be present
            assert!(!svg.contains("<rect "));
            // The SVG <g> title group should be present
            assert!(svg.contains("<g transform="));
        }

        #[test]
        fn test_render_hides_both_title_and_background() {
            let card = Card::new(
                100,
                120,
                "Test Title".to_string(),
                "Test Desc".to_string(),
                "Test Body".to_string(),
                "test-card".to_string(),
                CardSettings {
                    offset_x: 1,
                    offset_y: 1,
                    theme: CardTheme::TransparentBlue,
                    hide_title: true,
                    hide_background: true,
                    hide_background_stroke: false,
                },
            )
            .unwrap();
            let svg = card.render();
            // The <title> tag should always be present
            assert!(svg.contains("<title id=\"title-id\">Test Title</title>"));
            // The SVG <g> title group should NOT be present
            assert!(!svg.contains("<g transform="));
            // The background <rect> should NOT be present
            assert!(!svg.contains("<rect "));
        }

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
                "".to_string(),
                CardSettings {
                    offset_x: 1,
                    offset_y: 1,
                    theme: CardTheme::TransparentBlue,
                    hide_title: false,
                    hide_background: false,
                    hide_background_stroke: false,
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
                    Err(e) => panic!("Invalid SVG/XML: {e}"),
                }
                buf.clear();
            }
            assert!(found_svg, "SVG root element not found");
        }
    }
}
