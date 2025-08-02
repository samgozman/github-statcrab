/// SVG is a type alias for [String], representing an SVG representation of a card.
pub type SVG = String;

/// Card represents a card with a width, height, and title. Its a base wrapper for cards of different types.
/// It provides a method to create a new card and render it as an [SVG] string.
pub struct Card {
    width: i32,
    height: i32,
    title: String,
    description: String,
    body: String,
    style: Option<String>,
}

impl Card {
    /// Creates a new [Card] with the specified parameters.
    pub fn new(width: i32, height: i32, title: String, description: String, body: String) -> Self {
        Card {
            width,
            height,
            description,
            title,
            body,
            style: Some(Self::load_style()),
        }
    }

    /// Renders the [Card] as an [SVG] string.
    pub fn render(&self) -> SVG {
        let style = self
            .style
            .as_deref()
            .map(Self::indent_style)
            .unwrap_or_else(String::new);

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
  {rendered_title}
  {body}
</svg>
"#,
            width = self.width,
            height = self.height,
            title = self.title,
            description = self.description,
            body = self.body,
            rendered_title = self.render_title(),
            style = style
        )
    }

    /// Indents each line of the style string by two spaces.
    fn indent_style(style: &str) -> String {
        style.lines().map(|line| format!("  {}\n", line)).collect()
    }

    /// Renders the title of the [Card] as an SVG text element.
    fn render_title(&self) -> String {
        format!(r#"<text x="0" y="16" class="title">{}</text>"#, self.title)
    }

    /// Loads the CSS style for the [Card] from a file.
    fn load_style() -> String {
        // Embed the CSS file into the binary at compile time
        include_str!("../../assets/css/card.css").to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    mod fn_new {
        use super::*;

        #[test]
        fn test_card_creation() {
            let card = Card::new(
                10,
                20,
                "Test Card".to_string(),
                "Test Desc".to_string(),
                "Test Body".to_string(),
            );
            assert_eq!(card.width, 10);
            assert_eq!(card.height, 20);
            assert_eq!(card.title, "Test Card");
            assert_eq!(card.description, "Test Desc");
            assert_eq!(card.body, "Test Body");
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
                10,
                20,
                "Test Title".to_string(),
                "".to_string(),
                "".to_string(),
            );
            let rendered_title = card.render_title();
            assert_eq!(
                rendered_title,
                r#"<text x="0" y="16" class="title">Test Title</text>"#
            );
        }
    }

    mod fn_render {
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
            );
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
