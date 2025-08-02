pub struct Card {
    weight: i32,
    height: i32,
    title: String,
}

impl Card {
    pub fn new(weight: i32, height: i32, title: String) -> Self {
        Card {
            weight,
            height,
            title,
        }
    }

    pub fn render(&self) -> String {
        format!(
            "Card: {}, Weight: {}, Height: {}",
            self.title, self.weight, self.height
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    mod fn_new {
        use super::*;

        #[test]
        fn test_card_creation() {
            let card = Card::new(10, 20, "Test Card".to_string());
            assert_eq!(card.weight, 10);
            assert_eq!(card.height, 20);
            assert_eq!(card.title, "Test Card");
        }
    }
}
