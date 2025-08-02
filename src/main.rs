mod cards;

use cards::card::{Card, CardSettings};

fn main() {
    let card = Card::new(
        200,
        120,
        "Hello".to_string(),
        "World".to_string(),
        "".to_string(),
        CardSettings {
            offset: 0.5,
            hide_title: false,
            hide_background: false,
        },
    )
    .unwrap()
    .render();
    println!("{}", card);
}
