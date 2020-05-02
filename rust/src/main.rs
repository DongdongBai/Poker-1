extern crate indicatif;
extern crate itertools;
extern crate serde;
extern crate serde_json;
#[macro_use(c)]
extern crate cute;
extern crate rand;
#[macro_use]
extern crate lazy_static;
extern crate bincode;
extern crate bio;
extern crate qstring;
extern crate rayon;

mod backend;
mod bot;
mod card_abstraction;
mod card_utils;
mod exploiter;
mod tests;
mod trainer;
mod trainer_utils;
mod validation;

use std::mem::size_of_val;
use std::fs::File;
use std::io::Write;

fn main() {

    // crate::validation::preflop_matrix();

    // backend::main().expect("Could not launch server");

    // trainer::train(100_000);
    // trainer::train(100_000_000);
    let nodes = trainer::load_strategy();
    let compressed = trainer_utils::compress_strategy(&nodes);
    let bincode: Vec<u8> = bincode::serialize(&compressed).unwrap();
    let mut file = File::create("products/compressed.bin").unwrap();
    file.write_all(&bincode).unwrap();

    // view_preflop(&nodes);

    // for (infoset, node) in nodes {
    //     println!("infoset size: {}", size_of_val(&infoset));
    //     println!("node size: {}", size_of_val(&node));
    // }

    // exploiter::exploitability(&nodes);
    // exploiter::exploitability(&std::collections::HashMap::new());
}
