// This is the main interface for card abstractions. You can think of this as a
// black box that maps a poker hand to an ID number corresponding to the
// hand's abstraction bin. The idea is that similar hands will have the same
// abstraction id number, so we can treat similar hands as the same to reduce
// the number of possibilities in the game.

use crate::card_utils;
use crate::card_utils::Card;
use crate::card_utils::deepcopy;
use std::collections::HashMap;
use std::fs::File;
use std::io::Read;
use std::io::Write;
use itertools::Itertools;
use rand::thread_rng;
use rand::prelude::SliceRandom;

const FLOP_PATH: &str = "products/flop_abstraction.json";
const TURN_PATH: &str = "products/get_turn_abstraction.json";
const FLOP_EQUITY_PATH: &str = "products/flop_equity_distributions.json";
const TURN_EQUITY_PATH: &str = "products/turn_equity_distributions.json";

const EQUITY_BINS: usize = 50;

// flop and turn map card strings such as "As4d8c9h2d" to their corresponding
// abastract bin. Each string key is an archetypal hand, meaning that
// there is just one entry for every equivalent hand. For example, we don't care
// about the order of the flop cards, so we don't need separate entries for
// every permutation.
pub struct Abstraction {
    flop: HashMap<String, i32>,
    turn: HashMap<String, i32>,
}

impl Abstraction {

    pub fn new() -> Abstraction {
        Abstraction {
            flop: load_flop_abstraction(),
            turn: load_turn_abstraction()
        }
    }

    pub fn abstract_id(&self, cards: &[Card]) -> i32 {
        // let cards = card_utils::archetype(cards);
        match cards.len() {
            2 => self.preflop_bin(&cards),
            5 => self.flop_bin(&cards),
            6 => self.turn_bin(&cards),
            7 => self.river_bin(&cards),
            _ => panic!("Bad number of cards"),
        }
    }

    fn preflop_bin(&self, cards: &[Card]) -> i32 {
        let mut cards = cards.to_vec();
        cards.sort_by_key(|c| c.rank);
        let rank1 = cards[0].rank;
        let rank2 = cards[1].rank;
        let mut bin = 2 * (rank1 * 100 + rank2);
        if cards[0].suit == cards[1].suit {
            bin += 1;
        }
        return bin as i32;
    }

    fn flop_bin(&self, cards: &[Card]) -> i32 {
        return 0;
    }

    fn turn_bin(&self, cards: &[Card]) -> i32 {
        return 0;
    }

    fn river_bin(&self, cards: &[Card]) -> i32 {
        return 0;
    }
}

fn load_flop_abstraction() -> HashMap<String, i32> {
    match File::open(FLOP_PATH) {
        Err(_error) => make_flop_abstraction(),
        Ok(mut file) => {
            let mut buffer = String::new();
            file.read_to_string(&mut buffer).expect("Error reading file");
            serde_json::from_str(&buffer).unwrap()
        }
    };
    // TODO: Change
    return HashMap::new();
}

fn load_turn_abstraction() -> HashMap<String, i32> {
    return HashMap::new();
}

fn make_flop_abstraction() -> HashMap<String, i32> {
    let distributions = make_flop_equity();
    cluster(distributions)
}

fn make_flop_equity() -> HashMap<String, Vec<f64>> {
    println!("[INFO] Calculating flop equity distributions...");
    let mut distributions = HashMap::new();
    let flop_hands = card_utils::deal_canonical(5);
    let bar = card_utils::pbar(flop_hands.len() as u64);

    for hand in flop_hands {
        let equity = equity_distribution(&hand);
        let hand_str = card_utils::cards2str(&hand);
        // We store hands as strings in the HashMap for their equity distributions
        // TODO: Really?
        distributions.insert(hand_str, equity);
        bar.inc(1);
    }
    bar.finish();
    return distributions;
}

fn equity_distribution(cards: &[Card]) -> Vec<f64> {
    let cards = cards.to_vec();
    let mut distribution: Vec<f64> = vec![0.0; EQUITY_BINS];
    let board = (&cards[2..]).to_vec();

    let mut deck = card_utils::deck();
    // Remove the already-dealt cards from the deck
    deck.retain(|c| !cards.contains(&c));
    for opp_preflop in deck.iter().combinations(2) {
        let mut n_wins = 0.0;
        let mut n_rollouts = 0;
        // Remove the opponent's hand from the deck
        let mut subdeck = deck.clone();
        subdeck.retain(|c| !opp_preflop.contains(&c));

        for rollout in subdeck.iter().combinations(7 - cards.len()) {
            let rollout = rollout.to_vec();
            n_rollouts += 1;

            // Create the poker hands by concatenating cards
            let my_hand = [cards.clone(), deepcopy(&rollout)].concat();
            let opp_hand = [deepcopy(&opp_preflop), board.clone(), deepcopy(&rollout)].concat();

            // let my_strength = card_utils::hand_strength(&my_hand);
            // let opp_strength = card_utils::hand_strength(&opp_hand);

            // if my_strength > opp_strength {
            //     n_wins += 1.0;
            // } else if my_strength == opp_strength {
            //     n_wins += 0.5;
            // }
        }
        let equity = n_wins / (n_rollouts as f64);
        let equity_bin = (equity * EQUITY_BINS as f64) as usize;
        distribution[equity_bin] += 1.0;
    }
    // distribution = normalize(distribution);
    return distribution;
}

fn cluster(data: HashMap<String, Vec<f64>>) -> HashMap<String, i32> {
    return HashMap::new();
}

