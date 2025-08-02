mod cards;

use cards::card::Card;

fn main() {
    println!("Hello, world!");
    let _card = Card::new(1, 2, "Hello".to_string()).render();
}
