extern crate serde;
extern crate serde_json;
extern crate itertools;
extern crate indicatif;
#[macro_use(c)]
extern crate cute;
extern crate rand;

mod card_abstraction;
mod card_utils;
mod tests;

use card_utils::Card;
use std::fs::File;
use std::io::Write;

fn main() {


    let cards = vec![Card::new("Ac"), Card::new("Kc"), Card::new("Qc"),
                     Card::new("Jc"), Card::new("Tc")];

    let table = card_utils::HandTable::new();
    println!("{}", table.hand_strength(&cards));
    // let a = card_abstraction::Abstraction::new();
    // let bin = a.abstract_id(&cards);
    // println!("Bin: {}", bin);
}
