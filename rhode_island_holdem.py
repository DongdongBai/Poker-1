# For playing Rhode Island Hold'em
# Rules: https://www.cs.cmu.edu/~gilpin/gsi.html

import functools
import numpy as np
import pdb
import pickle

BET_SIZES = 10, 20, 20
INITIAL_STACK_SIZE = 1000
STRATEGY_DELAY = 1    # How many iterations to wait before starting to keep
                        # track of the cumulative strategy
PREFLOP, FLOP, TURN = range(3)
HIGH_CARD, PAIR, FLUSH, STRAIGHT, THREE_OF_KIND, STRAIGHT_FLUSH = range(6)
N_ACTIONS = 5
ACTIONS = 'fold', 'check', 'call', 'bet', 'raise'
FOLD, CHECK, CALL, BET, RAISE = range(5)
SAVE_PATH = 'rhode_island_nodes.pkl'
FLOP_CARD = 2
TURN_CARD = 3
MAX_RAISES = 3

RANKS = {'A': 14, 'K': 13, 'Q': 12, 'J': 11, 'T': 10, '9': 9, '8': 8, '7': 7,
         '6': 6, '5': 5, '4': 4, '3': 3, '2': 2}
SUITS = ('c', 'd', 'h', 's')

# TODO: I think the bet limit is actually 4 (max 3 raises), not 3.

### Functions ###

def get_deck():
    """Returns the standard 52-card deck, represented as a list of strings."""
    return [rank + suit for suit in SUITS for rank in RANKS]


### Classes ###

@functools.total_ordering
class Card:
    """Class for representing a card using the format '8c', 'Th', etc.

    Example:

    card = Card('9d')
    card2 = Card('Th')
    card2 > card1 == True

    Attributes:
        self.suit - The suit of the card, represented by 'h', 'c', etc.
        self.rank - The rank of the card, given as an integer, so 'A' -> 14

    Input:
        card_str - Input string in the standard card format '2d', 'Jh', etc.

    Throws:
        ArgumentError if the input string is not in the correct format.
    """

    def __init__(self, card_str):
        if card_str[0] not in RANKS or card_str[1] not in SUITS:
            raise ValueError('card_str must be in the format like "Kc", "4h"')
        self.card_str = card_str
        self.rank = RANKS[self.card_str[0]]
        self.suit = self.card_str[1]

    def __eq__(self, other):
        return self.card_str[0] == other.card_str[0]

    def __lt__(self, other):
        return self.rank < other.rank

    def __hash__(self):
        # Simple hash function--return the memory address of the object.
        return id(self)

    def __str__(self):
        return self.card_str


@functools.total_ordering
class RhodeHand:
    """Represents a 3-card hand for Rhode Island Poker.

    Rhode Island Poker hands have different rankings than standard 5-card poker,
    as follows:

        Straight Flush
        Three of a Kind
        Straight
        Flush
        Pair
        High Card

    Inputs:
        card0, card1, card2 -  Three cards represented like '8c', 'Qs', etc.
    """

    def __init__(self, card0, card1, card2):
        self.cards = [Card(card) for card in (card0, card1, card2)]
        self.type = None
        self.rank = None
        self.classify()

    def classify(self):
        self.rank = self.max_rank()
        if self.is_straight_flush():
            self.type = STRAIGHT_FLUSH
        elif self.is_three_of_kind():
            self.type = THREE_OF_KIND
        elif self.is_straight():
            self.type = STRAIGHT
        elif self.is_flush():
            self.type = FLUSH
        elif self.is_pair():
            self.type = PAIR
        else:
            self.type = HIGH_CARD

    def max_rank(self):
        highest_rank = 2
        for card in self.cards:
            if card.rank > highest_rank:
                highest_rank = card.rank
        return highest_rank

    def is_straight_flush(self):
        return self.is_straight() and self.is_flush()

    def is_three_of_kind(self):
        return self.cards[0].rank == self.cards[1].rank == self.cards[2].rank

    def is_straight(self):
        sorted_ranks = sorted([card.rank for card in self.cards])
        if RANKS['A'] in sorted_ranks:
            # Account for ace low straights, where sorted_ranks = [2, 3, 14]
            return sorted_ranks == [12, 13, 14] or sorted_ranks == [2, 3, 14]
        else:
            return (sorted_ranks[0] + 1 == sorted_ranks[1] and sorted_ranks[1] + 1 == sorted_ranks[2])

    def is_flush(self):
        return self.cards[0].suit == self.cards[1].suit == self.cards[2].suit

    def is_pair(self):
        return (self.cards[0].rank == self.cards[1].rank
                or self.cards[1].rank == self.cards[2].rank
                or self.cards[0].rank == self.cards[2].rank)

    def __lt__(self, other):
        if self.type == other.type:
            if self.rank == other.rank:
                # If the kicker is what determines the hand
                our_ranks = sorted(card.rank for card in self.cards)
                other_ranks = sorted(card.rank for card in other.cards)
                for i in range(len(our_ranks)):
                    if our_ranks[i] != other_ranks[i]:
                        return our_ranks[i] < other_ranks[i]
                return False    # The hand ranks are totally equivalent
            return self.rank < other.rank
        else:
            return self.type < other.type

    def __eq__(self, other):
        if self.type != other.type:
            return False
        our_ranks = sorted(card.rank for card in self.cards)
        other_ranks = sorted(card.rank for card in other.cards)
        return our_ranks == other_ranks

    def __str__(self):
        return ' '.join([str(card) for card in self.cards])



class Game:

    def __init__(self):
        self.pot = 0
        self.player1_card = None
        self.player2_card = None
        self.board = []
        self.street = PREFLOP
        self.hand_is_over = False
        self.player_folded = False
        self.deck = get_deck()
        self.stacks = [INITIAL_STACK_SIZE, INITIAL_STACK_SIZE]

    def play(self):
        """Initiate a sequence of hands for human vs. human play."""
        print("Welcome to Rhode Island Hold'em!")
        while not self.hand_is_over:
            self.advance_hand()
            self.betting()
            print()
            self.street += 1
        if not self.player_folded:
            self.showdown()
        print(self.stacks)

    def advance_hand(self):
        if self.street == PREFLOP:
            self.preflop()
        elif self.street == FLOP:
            self.flop()
        elif self.street == TURN:
            self.turn()
            self.hand_is_over = True

    def preflop(self):
        np.random.shuffle(self.deck)
        self.pot = 0
        self.player1_card = self.deck[0]
        self.player2_card = self.deck[1]
        print("Player 1's card:", self.player1_card)
        print("Player 2's card:", self.player2_card)

    def flop(self):
        self.board.append(self.deck[2])
        print('Flop:', self.board[0])

    def turn(self):
        self.board.append(self.deck[3])
        print('Turn:', self.board[1])

    def betting(self):
        """Process player inputs for a round of betting."""
        if self.street == PREFLOP:
            bet_size = BET_SIZES[0]
        else:
            bet_size = BET_SIZES[1]
        # bet, check
        # fold, call, raise | check, bet

        betting_over = False
        action = False     # Whether a player has bet (as opposed to checking)
        n_raises = 0
        n_checks = 0
        player = 0
        while not betting_over:
            player_action = self.input_action('Player ' + str(player+1), action, n_raises == 3)
            if player_action == 'bet':
                self.pot += bet_size
                self.stacks[player] -= bet_size
                action = True
                n_raises += 1
            elif player_action == 'check':
                n_checks += 1
            elif player_action == 'call':
                self.pot += bet_size
                self.stacks[player] -= bet_size
                action = False
                betting_over = True
            elif player_action == 'fold':
                self.hand_is_over = True
                self.player_folded = True
                self.stacks[1 - player] += self.pot
                return
            elif player_action == 'raise':
                self.pot += 2 * bet_size
                self.stacks[player] -= 2 * bet_size
                n_raises += 1

            if not action and (n_checks == 2 or n_raises == 3):
                betting_over = True
            player = 1 - player

    def showdown(self):
        """Gives the pot to the player with the best hand."""
        player1_hand = RhodeHand(self.player1_card, self.board[0], self.board[1])
        player2_hand = RhodeHand(self.player2_card, self.board[0], self.board[1])
        if player1_hand > player2_hand:
            self.stacks[0] += self.pot
        elif player2_hand > player1_hand:
            self.stacks[1] += self.pot
        elif player1_hand == player2_hand:
            self.stacks[0] += self.pot / 2
            self.stacks[1] += self.pot / 2

    @staticmethod
    def input_action(name, previous_bet, bet_limit_reached):
        """Get a bet input from the user.

        Inputs:
            name - The name of the player
            previous_bet - There has been a bet and the player needs to call,
                raise, or fold
            bet_limit_reached - Whether the max number of bets (3) have already
                been bet and the player can only call or fold.
        """
        allowed_actions = []
        if previous_bet:
            allowed_actions += ['call', 'fold']
            if not bet_limit_reached:
                allowed_actions += ['raise']
        else:
            allowed_actions += ['check', 'bet']

        while True:
            print(name + ' action: ')
            action = input('> ').lower()
            if action in allowed_actions:
                return action
            else:
                actions_string = ', '.join(allowed_actions[:-1]) + ' or ' + allowed_actions[-1] + '.'
                print(actions_string)


if __name__ == '__main__':
    game = Game()
    game.play()

