mod cards;

use cards::card::Card;

fn main() {
    let card = Card::new(100, 80, "Hello".to_string(), "World".to_string(), "".to_string()).render();
    println!("{}", card);
}
