from itertools import product
from tqdm import tqdm
import multiprocessing as mp

(HIGH_CARD, PAIR, TWO_PAIR, THREE_OF_A_KIND, STRAIGHT, FLUSH, FULL_HOUSE,
 FOUR_OF_A_KIND, STRAIGHT_FLUSH, ROYAL_FLUSH) = range(10)
RANK, SUIT = range(2)

RANKS = {'A': 14, 'K': 13, 'Q': 12, 'J': 11, 'T': 10, '9': 9, '8': 8, '7': 7,
         '6': 6, '5': 5, '4': 4, '3': 3, '2': 2}
SUITS = ('c', 'd', 'h', 's')


def get_deck():
    """Returns the standard 52-card deck, represented as a list of strings."""
    return [rank + suit for suit in SUITS for rank in RANKS]


def rank(card):
    return card[0]


def suit(card):
    return card[1]


def archetypal_hand(hand):
    # TODO: Make the turn and river be not sorted
    """Returns 'archetypal' hand isomorphic to input hand."""
    hand = sorted(hand[:2]) + sorted(hand[2:])
    suits= ['s', 'h', 'd', 'c']
    suit_mapping = {}
    for i in range(len(hand)):
        card = hand[i]
        if suit(card) in suit_mapping:
            archetypal_card = rank(card) + suit_mapping[suit(card)]
            hand[i] = archetypal_card
        else:
            suit_mapping[suit(card)] = suits.pop(0)
            archetypal_card = rank(card) + suit_mapping[suit(card)]
            hand[i] = archetypal_card
    return tuple(sorted(hand[:2]) + sorted(hand[2:]))


# def isomorphic_hand(hand):
#     hand = sorted(hand)
#     suits= ['s', 'h', 'd', 'c']
#     suit_mapping = {}
#     for i in range(len(hand)):
#         card = hand[i]
#         if suit(card) in suit_mapping:
#             archetypal_card = rank(card) + suit_mapping[suit(card)]
#             hand[i] = archetypal_card
#         else:
#             suit_mapping[suit(card)] = suits.pop(0)
#             archetypal_card = rank(card) + suit_mapping[suit(card)]
#             hand[i] = archetypal_card
#     return tuple(sorted(hand))


def make_isomorphic_suits():
    table = {}
    suits = ['s', 'h', 'd', 'c']
    for non_iso in product(suits, suits, suits, suits, suits):
        remaining = suits.copy()
        result = []
        mapping = {}
        for s in non_iso:
            if s not in mapping:
                mapping[s] = remaining.pop(0)
        table[non_iso] = [mapping[s] for s in non_iso]
    return table

isomorphic_suits = make_isomorphic_suits()

# TODO: This function is the bottleneck of the flop abstraction code. Anything
# to speed it up will help greatly.
def isomorphic_hand(hand):
    hand = sorted(hand)
    suits = tuple([card[SUIT] for card in hand])
    iso_suits = isomorphic_suits[suits]
    result = []
    for i, card in enumerate(hand):
        result.append(card[RANK] + iso_suits[i])
    return tuple(sorted(result))


def pbar_map(function, iterator):
    with mp.Pool(mp.cpu_count()) as p:
        result = list(tqdm(p.imap(function, iterator), total=len(iterator), smoothing=0.1))
    return result


# Profiling code
# import time
# import numpy as np
# from itertools import combinations
# deck = get_deck()
# np.random.shuffle(deck)
# hand = deck[:5]
# start = time.time()
# for i in range(1000000):
#     isomorphic_hand(hand)

# end = time.time()
# print(end - start)
# raise KeyboardInterrupt





