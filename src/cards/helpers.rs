/// A helper functions to retrieve the color associated with a programming language.
pub fn gel_language_color(language: &str) -> String {
    let json_str = include_str!("../../assets/configs/language-colors.json");
    let colors: serde_json::Value =
        serde_json::from_str(json_str).expect("Failed to parse language colors JSON");

    if let Some(color) = colors.get(language) {
        color.as_str().unwrap_or("#000000").to_string()
    } else {
        "#000000".to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_gel_language_color() {
        let color = gel_language_color("Rust");
        assert_eq!(color, "#dea584");
    }
    #[test]
    fn test_gel_language_color_not_found() {
        let color = gel_language_color("NonExistentLanguage");
        assert_eq!(color, "#000000");
    }
}
