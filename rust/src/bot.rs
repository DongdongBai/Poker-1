// Real-time bot logic. Right now this just does action translation, but this
// is where I will add depth-limited solving.
use crate::card_utils::Card;
use crate::trainer_utils::*;
use std::collections::HashMap;

lazy_static! {
    static ref BLUEPRINT: HashMap<CompactInfoSet, Action> = crate::trainer::load_blueprint();
}

pub fn bot_action(hand: &[Card], board: &[Card], history: &ActionHistory) -> Action {
    let translated = history.translate(&BET_ABSTRACTION.to_vec());
    let hand = [hand, board].concat();
    let infoset = InfoSet::from_hand(&hand, &translated).compress();
    let mut action = BLUEPRINT.get(&infoset).expect("Infoset not found in blueprint").clone();
    // The translated action is based off a misunderstanding off the true bet
    // sizes, so we may have to adjust our call amount to line up with what's
    // actually in the pot as opposed to our approximation.
    if action.action == ActionType::Call {
        // TODO: Are there other spots where the altered history brings illegal moves?
        // Hopefully not with a large enough bet abstraction, but still.
        action.amount = history.to_call();
    } else if action.action == ActionType::Bet && action.amount < history.min_bet() {
        action.amount = history.min_bet();
    }
    action
}
