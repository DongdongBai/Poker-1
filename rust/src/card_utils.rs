use std::thread;
use std::fmt;
use std::time::Duration;
use std::sync::mpsc;
use std::fs::File;
use std::io::Read;
use std::io::Write;
use std::collections::{HashMap, HashSet};
use rand::thread_rng;
use rand::prelude::SliceRandom;
use serde::Serialize;
use serde::Deserialize;
use rayon::prelude::*;
use crate::itertools::Itertools;
use crate::rand::prelude::IteratorRandom;

const HAND_TABLE_PATH: &str = "products/strengths.json";
const EQUITY_TABLE_PATH: &str = "products/river_equities.json";
const CANONICAL_HANDS_PATH: &str = "products/canonical_hands.json";

lazy_static! {
    pub static ref HAND_TABLE: HandTable = HandTable::new();
    pub static ref EQUITY_TABLE: EquityTable = EquityTable::new();
}

pub const CLUBS: i32 = 0;
pub const DIAMONDS: i32 = 1;
pub const HEARTS: i32 = 2;
pub const SPADES: i32 = 3;

#[derive(Hash, Debug, Clone, PartialOrd, Eq, Serialize, Deserialize)]
pub struct Card {
    pub rank: u8,
    pub suit: u8,
}

impl Card {

    pub fn new(card: &str) -> Card {
        let rank = match &card[0..1] {
            "2" => 2,
            "3" => 3,
            "4" => 4,
            "5" => 5,
            "6" => 6,
            "7" => 7,
            "8" => 8,
            "9" => 9,
            "T" => 10,
            "J" => 11,
            "Q" => 12,
            "K" => 13,
            "A" => 14,
            _ => panic!("bad card string")
        };
        let suit = match &card[1..2] {
            "c" => CLUBS,
            "d" => DIAMONDS,
            "h" => HEARTS,
            "s" => SPADES,
            _ => panic!("bad card string")
        };
        return Card { rank: rank, suit: suit as u8};
    }
}


impl PartialEq<Card> for Card {
    fn eq(&self, other: &Self) -> bool {
        self.rank == other.rank && self.suit == other.suit
    }
}

impl Ord for Card {
    // orders first based on rank, and if ranks are equal, then on alphebetical
    // order of the suit
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        (self.rank, self.suit.clone()).cmp(&(other.rank, other.suit.clone()))
    }
}

impl fmt::Display for Card {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let rank = match self.rank {
            2 => "2",
            3 => "3",
            4 => "4",
            5 => "5",
            6 => "6",
            7 => "7",
            8 => "8",
            9 => "9",
            10 => "T",
            11 => "J",
            12 => "Q",
            13 => "K",
            14 => "A",
            _ => panic!("Bad rank value")
        };
        let suit = match self.suit as i32 {
            CLUBS => "c",
            DIAMONDS => "d",
            HEARTS => "h",
            SPADES => "s",
            _ => panic!("Bad suit value")
        };
        write!(f, "{}{}", rank, suit)
    }
}

pub fn deck() -> Vec<Card> {
    let mut deck = Vec::new();
    let ranks = std::ops::Range { start: 2, end: 15};
    for rank in ranks {
        for suit in 0..4 {
            deck.push(Card { rank: rank, suit: suit});
        }
    }
    return deck;
}

pub fn deepcopy(vec: &Vec<&Card>) -> Vec<Card> {
    let vec = vec.clone();
    let mut result: Vec<Card> = Vec::new();
    for v in vec {
        result.push(v.clone());
    }
    result
}

pub fn cards2str(cards: &[Card]) -> String {
    let mut result = String::from("");
    for card in cards {
        result.push_str(&card.to_string());
    }
    result
}

pub fn strvec2cards(strvec: &[&str]) -> Vec<Card> {
    let mut cardvec = Vec::new();
    for card in strvec {
        cardvec.push(Card::new(card));
    }
    cardvec
}

pub fn pbar(n: u64) -> indicatif::ProgressBar {
    let bar = indicatif::ProgressBar::new(n);
    bar.set_style(indicatif::ProgressStyle::default_bar()
        .template("[{elapsed_precise}/{eta_precise}] {wide_bar} {pos:>7}/{len:7} {msg}"));
    // make sure the drawing doesn't dominate computation for large n
    // bar.set_draw_delta(n / 10000);
    bar
}


// canonical / archetypal hand methods
// thanks to stackoverflow user Daniel Slutzbach: https://stackoverflow.com/a/3831682

// returns true if the given list of ints contains duplicate elements.
fn contains_duplicates(list: &[u8]) -> bool {
    for i in 0..list.len() {
        for j in i+1..list.len() {
            if &list[i] == &list[j] {
                return true;
            }
        }
    }
    false
}

fn suit_first_greater_than(card1: &Card, card2: &Card) -> bool {
    if card1.suit < card2.suit {
        return false;
    }
    if card1.suit == card2.suit && card1.rank <= card2.rank {
        return false;
    }
    return true;
}

fn sorted_correctly(cards: &[Card], streets: bool) -> bool {
    if streets {
        if cards.len() >= 2 && suit_first_greater_than(&cards[0], &cards[1]) {
            // preflop is out of order
            return false;
        }
        if cards.len() > 2 {
            // Check that the board cards are sorted relative to each other
            let board = &cards[2..].to_vec();
            let mut sorted_board = board.clone();
            sorted_board.sort_by_key(|c| (c.suit.clone(), c.rank));
            return board == &sorted_board;
        }
        // 1 or 0 cards, always sorted correctly
        return true;
    } else {
        // make sure that all the cards are sorted since we aren't looking at streets
        let mut sorted_cards = cards.to_vec();
        sorted_cards.sort_by_key(|c| (c.suit.clone(), c.rank));
        return sorted_cards == cards;
    }
}

// Given a list of cards, returns true if they are canonical. the rules for being
// canonical are as follows:
// 1. cards must be in sorted order (suit-first, alphabetic order of suits)
// 2. each suit must have at least as many cards as later suits
// 3. when two suits have the same number of cards, the first suit must have
//    lower or equal ranks lexicographically, eg ([1, 5] < [2, 4])
// 4. no duplicate cards
// Thanks again to StackOverflow user Daniel Stutzbach for thinking of these rules.
//
// Inputs:
// cards - array of card instances representing the hand
// streets - whether to distinguish cards from different streets. meaning,
//    if the first two cards represent the preflop and the last three are the,
//    flop, that asad|jh9c2s is different than as2s|jh9cad. both hands have a
//    pair of aces, but in the first hand, the player has pocket aces on the
//    preflop and is in a much stronger position than the second hand.
pub fn is_canonical(cards: &[Card], streets: bool) -> bool {
    if !sorted_correctly(cards, streets) {
        // rule 1
        return false;
    }
    // by_suits is a different way of representing the hand -- it maps suits to
    // the ranks present for that suit
    let mut by_suits: Vec<Vec<u8>> = Vec::new();
    for suit in 0..4 {
        let ranks = c![card.rank, for card in cards, if card.suit == suit];
        by_suits.push(ranks.to_vec());
        if contains_duplicates(&ranks) {
            // duplicate cards have been provided, so this cannot be a real hand
            // rule 4
            return false;
        }

    }
    for i in 1..4 {
        let suit1 = &by_suits[i-1];
        let suit2 = &by_suits[i];
        if suit1.len() < suit2.len() {
            // rule 2
            return false;
        }
        if suit1.len() == suit2.len() && suit1 > suit2 {
            // rule 3. the ranks of the cards are compared for lexicographic ordering.
            return false;
        }
    }
    true
}

// recursively deals canonical hands of a given length.
fn deal_cards(mut permutations: &mut Vec<Vec<Card>>, n: u32, cards: &[Card], streets: bool) {
    let mut cards = cards.to_vec();
    if cards.len() as u32 == n {
        permutations.push(cards);
        return
    }
    for card in deck() {
        cards.push(card);
        if is_canonical(&cards, streets) || (streets && cards.len() < n as usize) {
            deal_cards(&mut permutations, n, &cards, streets);
        }
        cards.pop();
    }
}

// Returns all possible canonical hands of length n.
pub fn deal_canonical(n: u32, streets: bool) -> Vec<Vec<Card>> {
    // if streets {
    //     // Read the canonical hands from the file, or make the file if it doesn't exist
    //     let canonical_hands: HashMap<u32, Vec<Vec<Card>>> = match File::open(CANONICAL_HANDS_PATH) {
    //         Err(e) => make_canonical_hands();
    //         Ok(mut file) => {
    //             // Load up the hand table from the JSON
    //             let mut buffer = String::new();
    //             file.read_to_string(&mut buffer).expect("Error");
    //             serde_json::from_str(&buffer).unwrap()
    //         }
    //     };
    let mut permutations = Vec::new();
    deal_cards(&mut permutations, n, &[], streets);
    permutations
}

// When we want a list of canonical hands with street order information
// preserved, we can't use deal_canonical because that doesn't hit all possible
// canonical hands. So our best bet is to brute-force it and find all canonical
// hands, then write them to a file to read later.
// fn make_canonical_hands() -> HashMap<u32, Vec<Vec<Card>>> {
//     let mut canonical5 = Vec::new();
//     let mut canonical6 = Vec::new();
//     let mut canonical7 = Vec::new();
//     let deck = deck();
//     let bar = pbar()
// }

pub fn deal_river_canonical() -> HashSet<Vec<Card>> {
    let mut canonical: HashSet<Vec<Card>> = HashSet::new();
    let deck = deck();
    let bar = pbar(674274182400);
    let mut count = 0;
    for preflop in deck.iter().combinations(2) {
        let mut subdeck = deck.clone();
        subdeck.retain(|c| !preflop.contains(&c));
        for board in subdeck.iter().combinations(5) {
            let hand = [deepcopy(&preflop), deepcopy(&board)].concat();
            canonical.insert(canonical_hand(&hand, true));
            // if is_canonical(&hand, true) {
            //     count += 1;
            // }
            bar.inc(1);
        }
        println!("{}\n", count);
    }
    println!("{}\n", count);
    bar.finish();
    canonical
}

// Writes canonical hands to a file
pub fn write_canonical(n: u32, fname: &str, streets: bool) {
    let hands = deal_canonical(n, streets);
    let mut hands_str = String::new();
    for hand in hands {
        hands_str += &cards2str(&hand);
        hands_str += "\n";
    }

    let mut file = File::create(fname).unwrap();
    file.write_all(hands_str.as_bytes());
}

fn sort_canonical(cards: &[Card], streets: bool) -> Vec<Card> {
    let mut sorted = Vec::new();
    if streets && cards.len() > 2 {
        let mut preflop = (&cards[..2]).to_vec();
        let mut board = (&cards[2..]).to_vec();
        preflop.sort_by_key(|c| (c.suit.clone(), c.rank));
        board.sort_by_key(|c| (c.suit.clone(), c.rank));
        sorted = [preflop, board].concat();
    } else {
        sorted = cards.to_vec();
        sorted.sort_by_key(|c| (c.suit.clone(), c.rank));
    }
    sorted
}

// Translates the given cards into their equivalent canonical representation.
// When dealing with poker hands that come up in the game, there is some
// information that doesn't matter. For example, we don't care about the order
// of the flop cards or the hole cards. There is also suit isomorphism, where
// for example a 5-card flush of hearts is essentially the same as a 5-card
// flush of diamonds. This function maps the set of all hands to the much
// smaller set of distinct isomorphic hands.
pub fn canonical_hand(cards: &[Card], streets: bool) -> Vec<Card> {
    let cards_copy = cards.clone();
    let cards = &sort_canonical(&cards, streets);
    // Separate the cards by suit
    let mut by_suits: Vec<Vec<u8>> = Vec::new();
    for suit in 0..4 {
        let ranks = c![card.rank, for card in cards, if card.suit == suit];
        by_suits.push(ranks.to_vec());
    }

    // Define a mapping from old suits to new suits. suit_mapping[old_suit] = new_suit.
    let mut suit_mapping = [0, 0, 0, 0];

    // Retrieve the suits in size order with lexicographic tie breaking

    let mut unused_suits = vec![0, 1, 2, 3];
    for new_suit in 0..4 {
        let mut max = unused_suits[0];
        for old_suit in &unused_suits {
            let old_suit = *old_suit as usize;
            // The next suit must have the largest length, using lower lexicographic ordering
            // to break ties.
            if by_suits[old_suit].len() > by_suits[max].len() {
                max = old_suit;
            } else if by_suits[old_suit].len() == by_suits[max].len() && by_suits[old_suit] < by_suits[max] {
                max = old_suit;
            }
        }
        suit_mapping[max] = new_suit;
        // Wipe the current suit in by_suits so it doesn't get used twice
        unused_suits.retain(|s| s != &max);
        // for rank in by_suits[min].clone() {
        //     canonical.push(Card {rank: rank, suit: new_suit})
        // }
    }
    let mut canonical = Vec::new();
    for card in cards {
        canonical.push(Card {rank: card.rank, suit: suit_mapping[card.suit as usize]});
    }
    canonical = sort_canonical(&canonical, streets);

    // TODO: Remove once I'm convinced it's working
    if (!is_canonical(&canonical, streets)) {
        panic!("Not canonical: {}\nOriginal: {}", cards2str(&canonical), cards2str(&cards_copy));
    }

    canonical
}

// For fast poker hand comparison, look up relative strength values in a table
pub struct HandTable {
    strengths: HashMap<Vec<Card>, i32>
}

impl HandTable {

    pub fn new() -> HandTable {
        HandTable{ strengths: HandTable::load_hand_strengths() }
    }

    pub fn hand_strength(&self, hand: &[Card]) -> i32 {
        // Return the best hand out of all 5-card subsets
        let mut max_strength = 0;
        for five_card in hand.iter().combinations(5) {
            let canonical = canonical_hand(&deepcopy(&five_card), false);
            let strength = self.strengths.get(&canonical).unwrap().clone();
            if strength > max_strength {
                max_strength = strength;
            }
        }
        max_strength
    }

    fn load_hand_strengths() -> HashMap<Vec<Card>, i32> {
        let str_map: HashMap<String, i32> = match File::open(HAND_TABLE_PATH) {
            Err(e) => panic!("Hand table not found"),
            Ok(mut file) => {
                // Load up the hand table from the JSON
                let mut buffer = String::new();
                file.read_to_string(&mut buffer).expect("Error");
                serde_json::from_str(&buffer).unwrap()
            }
        };
        // Translate the card strings to Vec<Card> keys
        // TODO: If necessary for more speedup, change all lookup tables to
        // Vec<Card> keys instead of String
        let mut vec_map: HashMap<Vec<Card>, i32> = HashMap::new();
        for (hand, strength) in str_map {
            let cards = vec![&hand[0..2], &hand[2..4], &hand[4..6], &hand[6..8], &hand[8..10]];
            let cards = strvec2cards(&cards);
            vec_map.insert(cards, strength);
        }
        vec_map
    }
}

// Normalize the vector so that its elements sum to 1.
pub fn normalize(vector: &Vec<f64>) -> Vec<f64> {
    let mut sum = 0.0;
    for elem in vector {
        sum += elem;
    }
    let mut noramlized = Vec::new();
    for elem in vector {
        noramlized.push(elem / sum);
    }
    noramlized
}

// Stores the equities for every canonical river hand (~41 million). This allows
// for fast equity distribution calculation, and fast river card bucket lookup,
// which is the main bottleneck of training.
pub struct EquityTable {
    equities: HashMap<String, f64>
}

impl EquityTable {

    pub fn new() -> EquityTable {
        EquityTable {equities: EquityTable::load_equity_table()}
    }

    pub fn lookup(&self, cards: &[Card]) -> f64 {
        let canonical = canonical_hand(&cards, true);
        let card_str = cards2str(&canonical);
        println!("{}", cards2str(&cards));
        *self.equities.get(&card_str).unwrap()
    }

    fn load_equity_table() -> HashMap<String, f64> {
        match File::open(EQUITY_TABLE_PATH) {
            Err(e) => EquityTable::make_equity_table(),
            Ok(mut file) => {
                let mut buffer = String::new();
                file.read_to_string(&mut buffer).expect("Error");
                serde_json::from_str(&buffer).unwrap()
            }
        }
    }

    fn make_equity_table() -> HashMap<String, f64> {
        println!("[INFO] Constructing the river equity table...");
        let mut table: HashMap<String, f64> = HashMap::new();
        let river_hands = (&deal_canonical(7, true)).to_vec();
        let bar = pbar(river_hands.len() as u64);
        for chunk in river_hands.chunks(1000000) {
            let equities: Vec<f64> = chunk.par_iter()
                                          .map(|h| {
                                              bar.inc(1);
                                              EquityTable::river_equity(&h)})
                                          .collect();
            for i in 0..equities.len() {
                let hand_str = cards2str(&chunk[i].to_vec());
                table.insert(hand_str, equities[i]);
            }
            let json = serde_json::to_string_pretty(&table).unwrap();
            let mut file = File::create(EQUITY_TABLE_PATH).unwrap();
            file.write_all(json.as_bytes());
            println!("[INFO] Checkpoint");
        }
        bar.finish();
        println!("[INFO] Done.");
        table
    }

    fn river_equity(hand: &[Card]) -> f64 {
        let mut deck = deck();
        // Remove the already-dealt cards from the deck
        deck.retain(|c| !hand.contains(&c));

        let board = (&hand[2..]).to_vec();
        let mut n_wins = 0.0;
        let mut n_runs = 0;

        let mut rng = &mut rand::thread_rng();

        for opp_preflop in deck.iter().combinations(2) {//.choose_multiple(&mut rng, 10) {

            n_runs += 1;

            // Create the poker hands by concatenating cards
            let my_hand = hand.to_vec();
            let opp_hand = [deepcopy(&opp_preflop), board.clone()].concat();

            let my_strength = HAND_TABLE.hand_strength(&my_hand);
            let opp_strength = HAND_TABLE.hand_strength(&opp_hand);

            if my_strength > opp_strength {
                n_wins += 1.0;
            } else if my_strength == opp_strength {
                n_wins += 0.5;
            }
        }
        let equity = n_wins / (n_runs as f64);
        equity
    }
}

// u64 hand representation
// Each card is a single u8 byte, where
//
//      suit = card / 15   (integer division)
//      rank = card % 15
//
// I chose 15 so that ranks can have their true value, ie a 7c has rank 7.
// The cards are stored as bytes from right to left in the u64, which is
// right-padded by zeros. If no card is present, the value will be zero, which
// is how the length is determined. This byte representation allows for a
// small memory footprint without needing to use Rust's lifetime parameters.

pub fn card(hand: u64, card_index: i32) -> i32 {
    ((hand & 0xFF << 8*card_index) >> 8*card_index) as i32
}

pub fn suit(hand: u64, card_index: i32) -> i32 {
    println!("{}", hand);
    card(hand, card_index) / 15 as i32
}

pub fn rank(hand: u64, card_index: i32) -> i32 {
    card(hand, card_index) % 15 as i32
}

pub fn len(hand: u64) -> i32 {
    for n in 0..8 {
        if 256_u64.pow(n) > hand {
            return n as i32;
        }
    }
    -1
}

pub fn str2hand(hand_str: &str) -> u64 {
    let mut result: u64 = 0;
    let hand_str = hand_str.to_string();
    for i in (0..hand_str.len()).step_by(2) {
        let rank = match &hand_str[i..i+1] {
            "2" => 2,
            "3" => 3,
            "4" => 4,
            "5" => 5,
            "6" => 6,
            "7" => 7,
            "8" => 8,
            "9" => 9,
            "T" => 10,
            "J" => 11,
            "Q" => 12,
            "K" => 13,
            "A" => 14,
            _ => panic!("bad card string")
        };
        let suit = match &hand_str[i+1..i+2] {
            "c" => CLUBS,
            "d" => DIAMONDS,
            "h" => HEARTS,
            "s" => SPADES,
            _ => panic!("bad card string")
        };
        let card = (15 * suit + rank) as u64;
        result += card << 4*i
    }
    result
}

pub fn hand2str(hand: u64) -> String {
    let mut hand_str = String::new();
    for card_index in 0..len(hand) {
        let rank = match rank(hand, card_index) {
            2 => "2",
            3 => "3",
            4 => "4",
            5 => "5",
            6 => "6",
            7 => "7",
            8 => "8",
            9 => "9",
            10 => "T",
            11 => "J",
            12 => "Q",
            13 => "K",
            14 => "A",
            _ => panic!("Bad rank value")
        };
        let suit = match suit(hand, card_index) {
            CLUBS => "c",
            DIAMONDS => "d",
            HEARTS => "h",
            SPADES => "s",
            _ => panic!("Bad suit value")
        };
        hand_str.push_str(rank);
        hand_str.push_str(suit);
    }
    hand_str
}
