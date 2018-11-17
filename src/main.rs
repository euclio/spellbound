extern crate spellbound;

use std::env;

use spellbound::Checker;

fn main() {
    let text = env::args().skip(1).collect::<Vec<String>>().join(" ");

    let mut checker = Checker::new();

    for error in checker.check(&text) {
        println!("ERROR: {}", error.text());
    }
}
