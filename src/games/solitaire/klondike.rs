use cards::{french, default_shuffle};
use cards::french::{Rank, Suit, Color};
use std::cmp;
use std::collections::HashSet;

pub type Card = french::FrenchPlayingCard;

const MAX_DECK_SIZE: usize = 24;
const NUM_PILES: usize = 7;
const NUM_FOUNDATIONS: usize = 4;

pub enum MoveSource {
  Deck,
  Foundation(Suit),
  Pile(u8),
}

pub enum MoveTarget {
  Foundation,
  Pile(u8),
}

static RANKS: &'static [Rank; 13] = &[
    Rank::Ace,
    Rank::Number(2),
    Rank::Number(3),
    Rank::Number(4),
    Rank::Number(5),
    Rank::Number(6),
    Rank::Number(7),
    Rank::Number(8),
    Rank::Number(9),
    Rank::Number(10),
    Rank::Jack,
    Rank::Queen,
    Rank::King,
];

fn rank_index(rank: Rank) -> Result<usize, String> {
  for (i, r) in RANKS.iter().enumerate() {
    if *r == rank {
      return Ok(i);
    }
  }

  return Err(format!("Unsupported rank: {:?}", rank));
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum KlondikeErr {
  Capacity,
  InvalidCard,
  InvalidRank,
  InvalidSuit,
  InvalidColor,
  InvalidMove,
}

pub type KlondikeResult<T> = Result<T, KlondikeErr>;

pub struct KlondikeSolitaireGame {
  cards: Vec<Card>,
  foundations: [Foundation; NUM_FOUNDATIONS],
  piles: [Pile; NUM_PILES],
  deck: Deck,
}

impl KlondikeSolitaireGame {
  pub fn new(draw_count: u8) -> KlondikeSolitaireGame {
    KlondikeSolitaireGame::new_shuffle(draw_count, default_shuffle)
  }

  pub fn new_shuffle<F>(draw_count: u8, mut shuffle: F) -> KlondikeSolitaireGame
    where F: FnMut(&mut Vec<Card>) {
    // The order in the game struct initialization must match the indexes
    // returned by foundation_index function.
    debug_assert!(KlondikeSolitaireGame::foundation_index(Suit::Hearts) == 0);
    debug_assert!(KlondikeSolitaireGame::foundation_index(Suit::Diamonds) == 1);
    debug_assert!(KlondikeSolitaireGame::foundation_index(Suit::Spades) == 2);
    debug_assert!(KlondikeSolitaireGame::foundation_index(Suit::Clubs) == 3);

    let mut cards = french::new_standard_deck();
    shuffle(&mut cards);

    let mut game = KlondikeSolitaireGame {
      cards: cards,
      foundations: [
        Foundation::new(Suit::Hearts),
        Foundation::new(Suit::Diamonds),
        Foundation::new(Suit::Spades),
        Foundation::new(Suit::Clubs),
      ],
      piles: [
        Pile::new(),
        Pile::new(),
        Pile::new(),
        Pile::new(),
        Pile::new(),
        Pile::new(),
        Pile::new(),
      ],
      deck: Deck::new(draw_count),
    };

    // Deal the cards
    game.reset();

    return game;
  }

  pub fn from(deck: Deck, mut foundations: [Foundation; NUM_FOUNDATIONS], piles: [Pile; NUM_PILES]) -> KlondikeSolitaireGame {
    let mut cards = Vec::with_capacity(french::STANDARD_DECK_SIZE as usize);
    cards.extend(deck.waste_cards());
    cards.extend(deck.visible_cards());
    cards.extend(deck.remaining_cards());

    // validate all cards distinct
    {
      let mut set: HashSet<_> = cards.iter().cloned().collect();

      for f in &foundations {
        for card in f.cards().iter() {
          assert!(set.insert(*card), "Duplicate card in game: {:?}", card);
          cards.push(*card);
        }
      }

      for (i, p) in piles.iter().enumerate() {
        assert!(p.hidden_cards().len() <= i, "Pile {} hidden cards exceeds max: max={}", i, i);

        for card in p.hidden_cards().iter().chain(p.visible_cards()) {
          assert!(set.insert(*card), "Duplicate card in game: {:?}", card);
          cards.push(*card);
        }
      }
    }

    // validate 52 total cards (french::STANDARD_DECK_SIZE)
    assert_eq!(cards.len(), french::STANDARD_DECK_SIZE as usize);

    // ensure foundations are in expected order
    foundations.sort_by_key(|f| KlondikeSolitaireGame::foundation_index(f.suit()));

    return KlondikeSolitaireGame {
      cards: cards,
      deck: deck,
      foundations: foundations,
      piles: piles,
    };
  }

  fn foundation_index(suit: Suit) -> usize {
    match suit {
      Suit::Hearts   => 0,
      Suit::Diamonds => 1,
      Suit::Spades   => 2,
      Suit::Clubs    => 3,
    }
  }

  pub fn foundation(&self, suit: Suit) -> &Foundation {
    &self.foundations[KlondikeSolitaireGame::foundation_index(suit)]
  }

  fn foundation_mut(&mut self, suit: Suit) -> &mut Foundation {
    &mut self.foundations[KlondikeSolitaireGame::foundation_index(suit)]
  }

  pub fn deck(&self) -> &Deck {
    &self.deck
  }

  pub fn piles(&self) -> &[Pile] {
    &self.piles[..]
  }

  pub fn pile(&self, index: usize) -> &Pile {
    &self.piles[index]
  }

  pub fn reset(&mut self) {
    for foundation in self.foundations.iter_mut() {
      foundation.clear();
    }

    self.piles[0].reset(&self.cards[0..1]);
    self.piles[1].reset(&self.cards[1..3]);
    self.piles[2].reset(&self.cards[3..6]);
    self.piles[3].reset(&self.cards[6..10]);
    self.piles[4].reset(&self.cards[10..15]);
    self.piles[5].reset(&self.cards[15..21]);
    self.piles[6].reset(&self.cards[21..28]);

    self.deck.reset(&self.cards[28..]);
  }

  /// True if the table is clear (all cards are in foundation).
  pub fn is_clear(&self) -> bool {
    let clear = self.foundations.iter().all(|f| f.is_full());

    if clear {
      debug_assert!(self.piles.iter().all(|p| p.is_empty()));
      debug_assert!(self.deck.is_empty());
    }

    return clear;
  }

  pub fn draw(&mut self) {
    self.deck.draw()
  }

  pub fn move_cards(&mut self, source: MoveSource, target: MoveTarget) -> KlondikeResult<()> {
    match (source, target) {
      (MoveSource::Deck, MoveTarget::Foundation) => {
        let visible_card = {
          match self.deck.top() {
            Some(c) => c,
            None => { return Err(KlondikeErr::InvalidMove); },
          }
        };

        {
          let foundation = self.foundation_mut(visible_card.suit());
          if foundation.next_card() != Some(visible_card) {
            return Err(KlondikeErr::InvalidMove);
          }
          foundation.push();
        }

        self.deck.pop();
        Ok(())
      },
      (MoveSource::Deck, MoveTarget::Pile(pile_index)) => {
        let pile_index = pile_index as usize;
        assert!(pile_index < NUM_PILES);

        let visible_card = {
          match self.deck.top() {
            Some(c) => c,
            None => { return Err(KlondikeErr::InvalidMove); },
          }
        };

        match self.piles[pile_index].push(visible_card) {
          Ok(_) => {
            self.deck.pop();
            Ok(())
          },
          Err(_) => Err(KlondikeErr::InvalidMove),
        }
      },
      (MoveSource::Foundation(_), MoveTarget::Foundation) => {
        // Noop
        Ok(())
      },
      (MoveSource::Foundation(suit), MoveTarget::Pile(pile_index)) => {
        let pile_index = pile_index as usize;
        assert!(pile_index < NUM_PILES);

        let visible_card = {
          match self.foundation(suit).top() {
            Some(c) => c,
            None => { return Err(KlondikeErr::InvalidMove); }
          }
        };

        match self.piles[pile_index].push(visible_card) {
          Ok(_) => {
            self.foundation_mut(suit).pop();
            Ok(())
          },
          Err(_) => Err(KlondikeErr::InvalidMove),
        }
      },
      (MoveSource::Pile(source_pile_index), MoveTarget::Pile(target_pile_index)) => {
        let source_pile_index = source_pile_index as usize;
        assert!(source_pile_index < NUM_PILES);

        let target_pile_index = target_pile_index as usize;
        assert!(target_pile_index < NUM_PILES);

        if source_pile_index == target_pile_index {
          // Noop
          return Ok(());
        }

        let piles_ptr = self.piles.as_mut_ptr();
        let target_pile = &mut self.piles[target_pile_index];

        // Seems like there should be a better way to do this, but can't get
        // two mutable references to elements of `piles` in the same scope.
        // This should be safe since we ensure the source and target indexes
        // are different.
        unsafe {
          (*piles_ptr.offset(source_pile_index as isize)).move_to(target_pile)
        }
      },
      (MoveSource::Pile(pile_index), MoveTarget::Foundation) => {
        let pile_index = pile_index as usize;
        assert!(pile_index < NUM_PILES);

        let visible_card = {
          match self.piles[pile_index].top() {
            Some(c) => c,
            None => { return Err(KlondikeErr::InvalidMove); },
          }
        };

        {
          let foundation = self.foundation_mut(visible_card.suit());
          if foundation.next_card() != Some(visible_card) {
            return Err(KlondikeErr::InvalidMove);
          }
          foundation.push();
        }

        self.piles[pile_index].pop();
        Ok(())
      }
    }
  }
}

pub struct Deck {
  cards: Vec<Card>,
  draw_count: usize,
  visible_index: usize,
  visible_count: usize,
}

impl Deck {
  pub fn new(draw_count: u8) -> Deck {
    assert!(draw_count > 0 && draw_count as usize <= MAX_DECK_SIZE);

    Deck {
      cards: Vec::with_capacity(MAX_DECK_SIZE),
      draw_count: draw_count as usize,
      visible_index: 0,
      visible_count: 0,
    }
  }

  pub fn from(draw_count: u8, waste: &[Card], visible: &[Card], remaining: &[Card]) -> Deck {
    let deck_size = waste.len() + visible.len() + remaining.len();
    let mut cards = Vec::with_capacity(deck_size);

    // draw_count is valid
    assert!(draw_count > 0 && draw_count as usize <= MAX_DECK_SIZE);

    // visible length <= draw_count
    assert!(visible.len() <= draw_count as usize, "number of visible cards can not exceed draw count");

    // deck can not exceed 24 cards (52 - initial pile contents)
    assert!(deck_size <= MAX_DECK_SIZE, "deck size must be <= {}", MAX_DECK_SIZE);

    // cards are all distinct and are standard ranks
    {
      let mut set = HashSet::new();
      for card in waste.iter().chain(visible).chain(remaining) {
        assert!(set.insert(card), "Duplicate card in deck: {:?}", card);
        rank_index(card.rank()).unwrap();
        cards.push(*card);
      }
    }

    return Deck {
      cards: cards,
      draw_count: draw_count as usize,
      visible_index: waste.len(),
      visible_count: visible.len(),
    };
  }

  pub fn reset(&mut self, cards: &[Card]) {
    assert!(cards.len() <= MAX_DECK_SIZE);

    self.cards.clear();
    self.visible_index = 0;
    self.visible_count = 0;
    self.cards.extend_from_slice(cards);
  }

  pub fn is_empty(&self) -> bool {
    self.cards.is_empty()
  }

  pub fn len(&self) -> usize {
    self.cards.len()
  }

  pub fn draw_count(&self) -> u8 {
    self.draw_count as u8
  }

  pub fn top(&self) -> Option<Card> {
    match self.visible_count {
      0 => None,
      count => Some(self.cards[self.visible_index+count-1]),
    }
  }

  pub fn visible_cards(&self) -> &[Card] {
    &self.cards[self.visible_index..self.visible_index+self.visible_count]
  }

  pub fn waste_cards(&self) -> &[Card] {
    &self.cards[..self.visible_index]
  }

  pub fn remaining_cards(&self) -> &[Card] {
    let index = cmp::min(self.visible_index + self.visible_count, self.cards.len());
    &self.cards[index..]
  }

  pub fn pop(&mut self) -> Option<Card> {
    match self.visible_count {
      0 => None,
      _ => {
        self.visible_count -= 1;
        Some(self.cards.remove(self.visible_index + self.visible_count))
      }
    }
  }

  pub fn draw(&mut self) {
    // TODO return value?
    // boolean: true if visible cards changed
    // &[Card]: visible cards
    self.visible_index += self.visible_count;

    if self.visible_index >= self.cards.len() {
      self.visible_index = 0;
      self.visible_count = 0;
    } else {
      self.visible_count = cmp::min(self.draw_count, self.cards.len() - self.visible_index);
    }
  }
}

pub struct Pile {
  visible_cards: Vec<Card>,
  hidden_cards: Vec<Card>,
}

impl Pile {
  pub fn new() -> Pile {
    Pile {
      visible_cards: Vec::new(),
      hidden_cards: Vec::with_capacity(6),
    }
  }

  pub fn from(hidden: &[Card], visible: &[Card]) -> Pile {
    // if there are hidden cards, must be at least one visible on top
    assert!(hidden.is_empty() || visible.len() > 0, "there must be at least one visible card with hidden cards");

    // hidden has <= 6 cards (the right-most pile)
    assert!(hidden.len() <= 6, "no more than 6 hidden cards (the largest, right-most pile initially has 6)");

    // cards are distinct
    // card ranks only from the standard deck
    // `visible` is in valid order (e.g. color and rank)
    {
      let mut set = HashSet::new();

      let mut viter = visible.iter().peekable();
      while let Some(card) = viter.next() {
        assert!(set.insert(card), "Duplicate card in pile: {:?}", card);
        let card_rank_index = rank_index(card.rank()).unwrap();

        // check next card color and rank
        if let Some(next_card) = viter.peek() {
          assert!(card.color().other() == next_card.color(), "invalid visible card ordering");
          assert!(card_rank_index - 1 == rank_index(next_card.rank()).unwrap(), "invalid visible card ordering");
        }
      }

      for card in hidden.iter() {
        assert!(set.insert(card), "Duplicate card in pile: {:?}", card);
        rank_index(card.rank()).unwrap();
      }
    }

    return Pile {
      visible_cards: visible.iter().cloned().collect(),
      hidden_cards: hidden.iter().cloned().collect(),
    };
  }

  pub fn top(&self) -> Option<Card> {
    self.visible_cards.last().map(|c| *c)
  }

  pub fn len(&self) -> usize {
    self.visible_cards.len() + self.hidden_cards.len()
  }

  pub fn is_empty(&self) -> bool {
    self.visible_cards.is_empty()
  }

  pub fn visible_cards(&self) -> &[Card] {
    &self.visible_cards[..]
  }

  pub fn hidden_cards(&self) -> &[Card] {
    &self.hidden_cards[..]
  }

  pub fn reset(&mut self, cards: &[Card]) {
    assert!(cards.len() <= 7 && cards.len() > 0);

    self.hidden_cards.clear();

    if cards.len() > 1 {
      self.hidden_cards.extend_from_slice(&cards[0..cards.len()-1]);
    }

    self.visible_cards.clear();

    match cards.last() {
      Some(c) => self.visible_cards.push(*c),
      None => {},
    };
  }

  pub fn next_card(&self) -> Option<(Option<Color>, Rank)> {
    match self.visible_cards.last() {
      Some(card) => match rank_index(card.rank()).unwrap() {
        0 => None,
        i => Some((Some(card.color().other()), RANKS[i-1])),
      },
      None => Some((None, Rank::King)),
    }
  }

  pub fn can_push(&self, card: Card) -> KlondikeResult<()> {
    match self.next_card() {
      Some((Some(color), rank)) => {
        if card.color() == color && card.rank() == rank {
          Ok(())
        } else {
          Err(KlondikeErr::InvalidCard)
        }
      },
      Some((None, rank)) => {
        if card.rank() == rank {
          Ok(())
        } else {
          Err(KlondikeErr::InvalidCard)
        }
      },
      None => Err(KlondikeErr::Capacity),
    }
  }

  pub fn push(&mut self, card: Card) -> KlondikeResult<()> {
    let result = self.can_push(card);

    if result.is_ok() {
      self.visible_cards.push(card);
    }

    return result;
  }

  pub fn pop(&mut self) -> Option<Card> {
    match self.visible_cards.pop() {
      Some(card) => {
        self.check_visible();
        Some(card)
      },
      None => None,
    }
  }

  pub fn move_to(&mut self, target: &mut Pile) -> KlondikeResult<()> {
    let index = {
      match self.visible_cards.iter().position(|c| target.can_push(*c).is_ok()) {
        Some(i) => i,
        None => { return Err(KlondikeErr::InvalidMove); },
      }
    };

    target.visible_cards.extend_from_slice(&self.visible_cards[index..]);

    self.visible_cards.truncate(index);
    self.check_visible();

    Ok(())
  }

  fn check_visible(&mut self) {
    if self.visible_cards.is_empty() {
      if let Some(next_card) = self.hidden_cards.pop() {
        self.visible_cards.push(next_card);
      }
    }
  }
}

pub struct Foundation {
  suit: Suit,
  current_rank_index: Option<usize>,
}

impl Foundation {
  pub fn new(suit: Suit) -> Foundation {
    Foundation {
      suit: suit,
      current_rank_index: None,
    }
  }

  pub fn from(suit: Suit, rank: Option<Rank>) -> Foundation {
    let mut f = Foundation::new(suit);
    f.current_rank_index = match rank {
      Some(rank) => Some(rank_index(rank).unwrap()),
      None => None,
    };
    return f;
  }

  pub fn new_full(suit: Suit) -> Foundation {
    Foundation::from(suit, Some(Rank::King))
  }

  pub fn top(&self) -> Option<Card> {
    match self.current_rank_index {
      Some(i) => Some(Card::new(self.suit, RANKS[i])),
      None => None,
    }
  }

  pub fn cards(&self) -> Vec<Card> {
    let mut cards = Vec::new();

    if let Some(rank_index) = self.current_rank_index {
      for index in 0..rank_index+1 {
        cards.push(Card::new(self.suit, RANKS[index]))
      }
    }

    return cards;
  }

  pub fn is_full(&self) -> bool {
    self.current_rank_index == Some(RANKS.len() - 1)
  }

  pub fn is_empty(&self) -> bool {
    self.current_rank_index.is_none()
  }

  pub fn suit(&self) -> Suit {
    self.suit
  }

  pub fn next_rank(&self) -> Option<Rank> {
    match self.current_rank_index {
      Some(i) if i == RANKS.len() - 1 => None,
      Some(i) => Some(RANKS[i+1]),
      None => Some(RANKS[0]),
    }
  }

  pub fn next_card(&self) -> Option<Card> {
    match self.next_rank() {
      Some(r) => Some(Card::new(self.suit, r)),
      None => None,
    }
  }

  pub fn push(&mut self) -> Option<Card> {
    match self.current_rank_index {
      Some(i) if i == RANKS.len() - 1 => None,
      Some(i) => {
        self.current_rank_index = Some(i+1);
        Some(Card::new(self.suit, RANKS[i]))
      },
      None => {
        self.current_rank_index = Some(0);
        Some(Card::new(self.suit, RANKS[0]))
      }
    }
  }

  pub fn clear(&mut self) {
    self.current_rank_index = None;
  }

  pub fn pop(&mut self) -> Option<Card> {
    match self.current_rank_index {
      Some(0) => {
        self.current_rank_index = None;
        Some(Card::new(self.suit, RANKS[0]))
      },
      Some(i) => {
        self.current_rank_index = Some(i-1);
        Some(Card::new(self.suit, RANKS[i-1]))
      },
      None => None
    }
  }
}

#[cfg(test)]
mod test {
  pub use super::*;

  macro_rules! card {
    ($suit:expr, $rank:expr) => (Card::new($suit, $rank));
  }

  pub fn test_cards(name: &str, expected: &[Card], actual: &[Card]) {
    assert!(actual.len() == expected.len(), "{}: {} != {}", name, expected.len(), actual.len());
    for i in 0..expected.len() {
      assert!(expected[i] == actual[i], "{}[{}]: {:?} != {:?}", name, i, expected[i], actual[i]);
    }
  }

  pub fn test_deck(deck: &Deck, visible: &[Card], waste: &[Card], remaining: &[Card]) {
    test_cards("visible", visible, deck.visible_cards());
    test_cards("waste", waste, deck.waste_cards());
    test_cards("remaining", remaining, deck.remaining_cards());
  }

  pub fn test_pile(name: &str, pile: &Pile, hidden: &[Card], visible: &[Card]) {
    test_cards(&format!("{}.hidden", name), hidden, pile.hidden_cards());
    test_cards(&format!("{}.visible", name), visible, pile.visible_cards());
  }

  mod game {
    use super::*;
    use cards::french::{Suit, new_standard_deck};

    #[test]
    fn from_new() {
      let cards = new_standard_deck();
      let game = KlondikeSolitaireGame::from(
        Deck::from(3, &[], &[], &cards[28..]),
        [
          Foundation::new(Suit::Clubs),
          Foundation::new(Suit::Hearts),
          Foundation::new(Suit::Spades),
          Foundation::new(Suit::Diamonds),
        ], [
          Pile::from(&cards[0..0], &cards[0..1]),
          Pile::from(&cards[1..2], &cards[2..3]),
          Pile::from(&cards[3..5], &cards[5..6]),
          Pile::from(&cards[6..9], &cards[9..10]),
          Pile::from(&cards[10..14], &cards[14..15]),
          Pile::from(&cards[15..20], &cards[20..21]),
          Pile::from(&cards[21..27], &cards[27..28]),
        ]
      );

      assert_eq!(game.deck().draw_count(), 3);
      test_deck(game.deck(), &[], &[], &cards[28..]);

      assert!(game.foundation(Suit::Clubs).is_empty());
      assert_eq!(game.foundation(Suit::Clubs).suit(), Suit::Clubs);
      assert!(game.foundation(Suit::Spades).is_empty());
      assert_eq!(game.foundation(Suit::Spades).suit(), Suit::Spades);
      assert!(game.foundation(Suit::Diamonds).is_empty());
      assert_eq!(game.foundation(Suit::Diamonds).suit(), Suit::Diamonds);
      assert!(game.foundation(Suit::Hearts).is_empty());
      assert_eq!(game.foundation(Suit::Hearts).suit(), Suit::Hearts);

      test_pile("game.piles[0]", game.pile(0), &[], &cards[0..1]);
      test_pile("game.piles[1]", game.pile(1), &cards[1..2], &cards[2..3]);
      test_pile("game.piles[2]", game.pile(2), &cards[3..5], &cards[5..6]);
      test_pile("game.piles[3]", game.pile(3), &cards[6..9], &cards[9..10]);
      test_pile("game.piles[4]", game.pile(4), &cards[10..14], &cards[14..15]);
      test_pile("game.piles[5]", game.pile(5), &cards[15..20], &cards[20..21]);
      test_pile("game.piles[6]", game.pile(6), &cards[21..27], &cards[27..28]);

      assert!(! game.is_clear());
    }

    #[test]
    fn from_clear() {
      let game = KlondikeSolitaireGame::from(
        Deck::from(3, &[], &[], &[]),
        [
          Foundation::new_full(Suit::Clubs),
          Foundation::new_full(Suit::Hearts),
          Foundation::new_full(Suit::Spades),
          Foundation::new_full(Suit::Diamonds),
        ], [
          Pile::new(),
          Pile::new(),
          Pile::new(),
          Pile::new(),
          Pile::new(),
          Pile::new(),
          Pile::new(),
        ]
      );

      assert_eq!(game.deck().draw_count(), 3);
      test_deck(game.deck(), &[], &[], &[]);

      assert!(game.foundation(Suit::Clubs).is_full());
      assert!(game.foundation(Suit::Spades).is_full());
      assert!(game.foundation(Suit::Diamonds).is_full());
      assert!(game.foundation(Suit::Hearts).is_full());

      assert!(game.pile(0).is_empty());
      assert!(game.pile(1).is_empty());
      assert!(game.pile(2).is_empty());
      assert!(game.pile(3).is_empty());
      assert!(game.pile(4).is_empty());
      assert!(game.pile(5).is_empty());
      assert!(game.pile(6).is_empty());

      assert!(game.is_clear());
    }
  }

  mod pile {
    use super::*;
    use cards::french::{Color, Suit, Rank, new_standard_deck};

    #[test]
    fn new_pile() {
      let pile = Pile::new();
      assert!(pile.is_empty());
      assert!(pile.len() == 0);
      assert!(pile.top().is_none());
      assert!(pile.next_card() == Some((None, Rank::King)));
    }

    #[test]
    fn from_empty() {
      let pile = Pile::from(
        &[],
        &[]);
      assert!(pile.is_empty());
    }

    #[test]
    fn from_empty_hidden_one_visible() {
      let pile = Pile::from(
        &[],
        &[card!(Suit::Hearts, Rank::King)]);
      assert!(pile.len() == 1);
      assert!(pile.top() == Some(card!(Suit::Hearts, Rank::King)));
    }

    #[test]
    fn from_empty_hidden_mult_visible() {
      let pile = Pile::from(
        &[],
        &[card!(Suit::Hearts, Rank::Number(4)), card!(Suit::Spades, Rank::Number(3)), card!(Suit::Hearts, Rank::Number(2))]);
      assert!(pile.len() == 3);
      assert!(pile.hidden_cards().is_empty());
      test_cards("visible", &[card!(Suit::Hearts, Rank::Number(4)), card!(Suit::Spades, Rank::Number(3)), card!(Suit::Hearts, Rank::Number(2))], pile.visible_cards());
    }

    #[test]
    fn from_mult_hidden_mult_visible() {
      let pile = Pile::from(
        &[card!(Suit::Hearts, Rank::Queen), card!(Suit::Spades, Rank::King), card!(Suit::Diamonds, Rank::Number(2))],
        &[card!(Suit::Hearts, Rank::King), card!(Suit::Spades, Rank::Queen), card!(Suit::Hearts, Rank::Jack)]);
      assert!(pile.len() == 6);
      test_cards("hidden", &[card!(Suit::Hearts, Rank::Queen), card!(Suit::Spades, Rank::King), card!(Suit::Diamonds, Rank::Number(2))], pile.hidden_cards());
      test_cards("visible", &[card!(Suit::Hearts, Rank::King), card!(Suit::Spades, Rank::Queen), card!(Suit::Hearts, Rank::Jack)], pile.visible_cards());
    }

    #[test]
    #[should_panic]
    fn from_error_when_empty_visible() {
      Pile::from(
        &[card!(Suit::Hearts, Rank::Queen)],
        &[]);
    }

    #[test]
    #[should_panic]
    fn from_error_when_too_many_hidden() {
      Pile::from(
        &[card!(Suit::Hearts, Rank::Queen), card!(Suit::Spades, Rank::King), card!(Suit::Diamonds, Rank::Number(2)),
        card!(Suit::Hearts, Rank::King), card!(Suit::Spades, Rank::Queen), card!(Suit::Hearts, Rank::Jack),
        card!(Suit::Clubs, Rank::Ace)],
        &[card!(Suit::Clubs, Rank::Number(5))]);
    }

    #[test]
    #[should_panic]
    fn from_error_duplicate_cards() {
      Pile::from(
        &[card!(Suit::Hearts, Rank::Queen)],
        &[card!(Suit::Hearts, Rank::Queen)]);
    }

    #[test]
    #[should_panic]
    fn from_error_duplicate_cards2() {
      Pile::from(
        &[card!(Suit::Diamonds, Rank::Ace), card!(Suit::Diamonds, Rank::Ace)],
        &[card!(Suit::Hearts, Rank::Queen)]);
    }

    #[test]
    #[should_panic]
    fn from_error_duplicate_cards3() {
      Pile::from(
        &[card!(Suit::Hearts, Rank::Queen)],
        &[card!(Suit::Diamonds, Rank::Ace), card!(Suit::Diamonds, Rank::Ace)]);
    }

    #[test]
    #[should_panic]
    fn from_error_invalid_visible_rank() {
      Pile::from(
        &[],
        &[card!(Suit::Diamonds, Rank::Number(3)), card!(Suit::Spades, Rank::Number(4))]);
    }

    #[test]
    #[should_panic]
    fn from_error_invalid_visible_color() {
      Pile::from(
        &[],
        &[card!(Suit::Diamonds, Rank::Number(3)), card!(Suit::Hearts, Rank::Number(2))]);
    }

    #[test]
    fn len() {
      let mut pile = Pile::new();
      pile.reset(&[card!(Suit::Spades, Rank::Number(3))]);
      assert!(pile.len() == 1);
      assert!(!pile.is_empty());

      pile.reset(&[card!(Suit::Spades, Rank::Number(3)), card!(Suit::Hearts, Rank::Queen)]);
      assert!(pile.len() == 2);
      assert!(!pile.is_empty());

      pile.pop().unwrap();
      assert!(pile.len() == 1);
      assert!(!pile.is_empty());

      pile.pop().unwrap();
      assert!(pile.len() == 0);
      assert!(pile.is_empty());
    }

    #[test]
    fn move_to_with_empty_piles() {
      let mut source = Pile::new();
      let mut target = Pile::new();
      assert!(source.move_to(&mut target).is_err());
    }

    #[test]
    fn move_to_with_source_empty() {
      let mut source = Pile::new();

      let mut target = Pile::new();
      target.reset(&[card!(Suit::Hearts, Rank::Queen)]);

      assert!(source.move_to(&mut target).is_err());
    }

    #[test]
    fn move_to_with_empty_target_and_non_king() {
      let mut source = Pile::new();
      source.reset(&[card!(Suit::Hearts, Rank::Queen)]);

      let mut target = Pile::new();

      assert!(source.move_to(&mut target).is_err());
    }

    #[test]
    fn move_to_with_empty_target_and_king() {
      let mut source = Pile::new();
      source.reset(&[card!(Suit::Hearts, Rank::King)]);

      let mut target = Pile::new();

      assert!(source.move_to(&mut target).is_ok());

      assert!(source.is_empty());
      assert!(target.len() == 1);
      test_cards("target", &[card!(Suit::Hearts, Rank::King)], target.visible_cards());
    }

    #[test]
    fn move_to_with_incompatible_piles() {
      let mut source = Pile::new();
      source.reset(&[card!(Suit::Hearts, Rank::Number(4))]);

      let mut target = Pile::new();
      target.reset(&[card!(Suit::Spades, Rank::Number(3))]);

      assert!(source.move_to(&mut target).is_err());

      assert!(source.len() == 1);
      test_cards("source", &[card!(Suit::Hearts, Rank::Number(4))], source.visible_cards());
      assert!(target.len() == 1);
      test_cards("target", &[card!(Suit::Spades, Rank::Number(3))], target.visible_cards());
    }

    #[test]
    fn move_to_with_one_card() {
      let mut source = Pile::new();
      source.reset(&[card!(Suit::Diamonds, Rank::Number(3)), card!(Suit::Hearts, Rank::Number(3))]);

      let mut target = Pile::new();
      target.reset(&[card!(Suit::Spades, Rank::Number(4))]);

      assert!(source.move_to(&mut target).is_ok());

      assert!(source.len() == 1);
      test_cards("source", &[card!(Suit::Diamonds, Rank::Number(3))], source.visible_cards());
      assert!(target.len() == 2);
      test_cards("target", &[card!(Suit::Spades, Rank::Number(4)), card!(Suit::Hearts, Rank::Number(3))], target.visible_cards());
    }

    #[test]
    fn move_to_partial() {
      let mut source = Pile::new();
      source.reset(&[
        card!(Suit::Diamonds, Rank::Number(3)),
        card!(Suit::Hearts, Rank::Number(9))
      ]);

      source.push(card!(Suit::Spades, Rank::Number(8))).unwrap();
      source.push(card!(Suit::Diamonds, Rank::Number(7))).unwrap();
      source.push(card!(Suit::Clubs, Rank::Number(6))).unwrap();

      let mut target = Pile::new();
      target.reset(&[card!(Suit::Clubs, Rank::Number(8))]);

      assert!(source.move_to(&mut target).is_ok());

      assert!(source.len() == 3);
      test_cards("source.visible", &[card!(Suit::Hearts, Rank::Number(9)), card!(Suit::Spades, Rank::Number(8))], source.visible_cards());
      test_cards("source.hidden", &[card!(Suit::Diamonds, Rank::Number(3))], source.hidden_cards());
      assert!(target.len() == 3);
      test_cards("target.visible", &[card!(Suit::Clubs, Rank::Number(8)), card!(Suit::Diamonds, Rank::Number(7)), card!(Suit::Clubs, Rank::Number(6))], target.visible_cards());
      test_cards("target.hidden", &[], target.hidden_cards());
    }

    #[test]
    fn move_to_full() {
      let mut source = Pile::new();
      source.reset(&[
        card!(Suit::Diamonds, Rank::Number(3)),
        card!(Suit::Hearts, Rank::Number(9))
      ]);

      source.push(card!(Suit::Spades, Rank::Number(8))).unwrap();
      source.push(card!(Suit::Diamonds, Rank::Number(7))).unwrap();
      source.push(card!(Suit::Clubs, Rank::Number(6))).unwrap();

      let mut target = Pile::new();
      target.reset(&[card!(Suit::Spades, Rank::Number(10))]);

      assert!(source.move_to(&mut target).is_ok());

      assert!(source.len() == 1);
      test_cards("source.visible", &[card!(Suit::Diamonds, Rank::Number(3))], source.visible_cards());
      test_cards("source.hidden", &[], source.hidden_cards());
      assert!(target.len() == 5);
      test_cards("target.visible", &[
        card!(Suit::Spades, Rank::Number(10)),
        card!(Suit::Hearts, Rank::Number(9)),
        card!(Suit::Spades, Rank::Number(8)),
        card!(Suit::Diamonds, Rank::Number(7)),
        card!(Suit::Clubs, Rank::Number(6))], target.visible_cards());
      test_cards("target.hidden", &[], target.hidden_cards());
    }

    #[test]
    fn visible_and_hidden_cards() {
      let mut pile = Pile::new();
      pile.reset(&[
        card!(Suit::Hearts, Rank::Ace),
        card!(Suit::Diamonds, Rank::Number(10)),
      ]);

      pile.push(card!(Suit::Clubs, Rank::Number(9))).unwrap();

      test_cards("visible1", &[card!(Suit::Diamonds, Rank::Number(10)), card!(Suit::Clubs, Rank::Number(9))], pile.visible_cards());
      test_cards("hidden1", &[card!(Suit::Hearts, Rank::Ace)], pile.hidden_cards());

      pile.pop().unwrap();
      test_cards("visible2", &[card!(Suit::Diamonds, Rank::Number(10))], pile.visible_cards());
      test_cards("hidden2", &[card!(Suit::Hearts, Rank::Ace)], pile.hidden_cards());

      pile.pop().unwrap();
      test_cards("visible3", &[card!(Suit::Hearts, Rank::Ace)], pile.visible_cards());
      test_cards("hidden3", &[], pile.hidden_cards());

      pile.pop().unwrap();
      test_cards("visible4", &[], pile.visible_cards());
      test_cards("hidden4", &[], pile.hidden_cards());
    }

    #[test]
    fn top() {
      let mut pile = Pile::new();
      pile.reset(&[
        card!(Suit::Hearts, Rank::Ace),
        card!(Suit::Diamonds, Rank::Number(10)),
      ]);

      pile.push(card!(Suit::Clubs, Rank::Number(9))).unwrap();

      assert!(pile.top() == Some(card!(Suit::Clubs, Rank::Number(9))));
      pile.pop().unwrap();
      assert!(pile.top() == Some(card!(Suit::Diamonds, Rank::Number(10))));
      pile.pop().unwrap();
      assert!(pile.top() == Some(card!(Suit::Hearts, Rank::Ace)));
      pile.pop().unwrap();
      assert!(pile.top().is_none());
    }

    #[test]
    fn pop_on_empty() {
      let mut pile = Pile::new();
      assert!(pile.pop().is_none());
    }

    #[test]
    fn pop() {
      let mut pile = Pile::new();
      pile.reset(&[
        card!(Suit::Spades, Rank::King),
        card!(Suit::Hearts, Rank::Ace),
        card!(Suit::Diamonds, Rank::Number(10)),
      ]);

      pile.push(card!(Suit::Clubs, Rank::Number(9))).unwrap();
      pile.push(card!(Suit::Hearts, Rank::Number(8))).unwrap();

      assert!(pile.pop() == Some(card!(Suit::Hearts, Rank::Number(8))));
      assert!(pile.pop() == Some(card!(Suit::Clubs, Rank::Number(9))));
      assert!(pile.pop() == Some(card!(Suit::Diamonds, Rank::Number(10))));
      assert!(pile.pop() == Some(card!(Suit::Hearts, Rank::Ace)));
      assert!(pile.pop() == Some(card!(Suit::Spades, Rank::King)));
      assert!(pile.pop().is_none());
    }

    #[test]
    fn push_changes_next_card() {
      let mut pile = Pile::new();
      let cards = &[
        card!(Suit::Spades, Rank::King),
        card!(Suit::Hearts, Rank::Queen),
        card!(Suit::Clubs, Rank::Jack),
        card!(Suit::Diamonds, Rank::Number(10)),
        card!(Suit::Clubs, Rank::Number(9)),
        card!(Suit::Hearts, Rank::Number(8)),
        card!(Suit::Clubs, Rank::Number(7)),
        card!(Suit::Hearts, Rank::Number(6)),
        card!(Suit::Spades, Rank::Number(5)),
        card!(Suit::Diamonds, Rank::Number(4)),
        card!(Suit::Clubs, Rank::Number(3)),
        card!(Suit::Hearts, Rank::Number(2)),
        card!(Suit::Spades, Rank::Ace),
      ];

      let mut iter = cards.iter().map(|c| *c).peekable();

      while let Some(card) = iter.next() {
        pile.push(card).unwrap();

        match iter.peek() {
          Some(next) => assert!(pile.next_card() == Some((Some(next.color()), next.rank()))),
          None => assert!(pile.next_card().is_none()),
        }
      }

      assert!(pile.len() == cards.len());
    }

    #[test]
    fn can_push_king_with_empty_pile() {
      let pile = Pile::new();

      assert!(pile.next_card() == Some((None, Rank::King)));

      for card in new_standard_deck().iter().map(|c| *c) {
        if card.rank() == Rank::King {
          assert!(pile.can_push(card).is_ok());
        } else {
          assert!(pile.can_push(card) == Err(KlondikeErr::InvalidCard));
        }
      }
    }

    #[test]
    fn can_push_nothing_with_ace() {
      let mut pile = Pile::new();
      pile.reset(&[card!(Suit::Hearts, Rank::Ace)]);

      assert!(pile.next_card().is_none());

      for card in new_standard_deck().iter() {
        assert!(pile.can_push(*card) == Err(KlondikeErr::Capacity));
      }
    }

    #[test]
    fn can_push_red_with_black_visible() {
      let mut pile = Pile::new();
      pile.reset(&[card!(Suit::Spades, Rank::King)]);

      assert!(pile.next_card() == Some((Some(Color::Red), Rank::Queen)));

      for card in new_standard_deck().iter().map(|c| *c) {
        if card.color() == Color::Red && card.rank() == Rank::Queen {
          assert!(pile.can_push(card).is_ok());
        } else {
          assert!(pile.can_push(card) == Err(KlondikeErr::InvalidCard));
        }
      }
    }

    #[test]
    fn can_push_black_with_red_visible() {
      let mut pile = Pile::new();
      pile.reset(&[card!(Suit::Hearts, Rank::Number(2))]);

      assert!(pile.next_card() == Some((Some(Color::Black), Rank::Ace)));

      for card in new_standard_deck().iter().map(|c| *c) {
        if card.color() == Color::Black && card.rank() == Rank::Ace {
          assert!(pile.can_push(card).is_ok());
        } else {
          assert!(pile.can_push(card) == Err(KlondikeErr::InvalidCard));
        }
      }
    }
  }

  mod foundation {
    use super::*;
    use cards::french::{Suit, Rank};

    #[test]
    fn new_foundation() {
      let f = Foundation::new(Suit::Hearts);
      assert!(f.top().is_none());
      assert!(f.is_full() == false);
      assert!(f.is_empty() == true);
      assert!(f.suit() == Suit::Hearts);
    }

    #[test]
    fn new_full() {
      let f = Foundation::new_full(Suit::Hearts);
      assert!(f.is_full());
    }

    #[test]
    fn foundation_push() {
      let mut f = Foundation::new(Suit::Hearts);
      assert!(f.next_rank() == Some(Rank::Ace));
      f.push().unwrap();
      assert!(f.next_rank() == Some(Rank::Number(2)));
      f.push().unwrap();
      assert!(f.next_rank() == Some(Rank::Number(3)));
      f.push().unwrap();
      assert!(f.next_rank() == Some(Rank::Number(4)));
      f.push().unwrap();
      assert!(f.next_rank() == Some(Rank::Number(5)));
      f.push().unwrap();
      assert!(f.next_rank() == Some(Rank::Number(6)));
      f.push().unwrap();
      assert!(f.next_rank() == Some(Rank::Number(7)));
      f.push().unwrap();
      assert!(f.next_rank() == Some(Rank::Number(8)));
      f.push().unwrap();
      assert!(f.next_rank() == Some(Rank::Number(9)));
      f.push().unwrap();
      assert!(f.next_rank() == Some(Rank::Number(10)));
      f.push().unwrap();
      assert!(f.next_rank() == Some(Rank::Jack));
      f.push().unwrap();
      assert!(f.next_rank() == Some(Rank::Queen));
      f.push().unwrap();
      assert!(f.next_rank() == Some(Rank::King));
      f.push().unwrap();
      assert!(f.next_rank().is_none());
    }

    #[test]
    fn from_none() {
      let f = Foundation::from(Suit::Clubs, None);
      assert!(f.top().is_none());
      assert!(f.next_rank() == Some(Rank::Ace));
    }

    #[test]
    fn from() {
      let f = Foundation::from(Suit::Clubs, Some(Rank::Jack));
      assert!(f.top() == Some(card!(Suit::Clubs, Rank::Jack)));
      assert!(f.next_rank() == Some(Rank::Queen));
    }
  }

  mod deck {
    use super::*;
    use cards::french::{Suit, Rank};

    #[test]
    fn new_deck() {
      let d = Deck::new(1);
      assert!(d.is_empty());
      assert!(d.len() == 0);
      assert!(d.visible_cards().is_empty());
      assert!(d.waste_cards().is_empty());
      assert!(d.remaining_cards().is_empty());
    }

    #[test]
    fn reset() {
      let mut d = Deck::new(1);

      d.reset(&[]);
      assert!(d.is_empty());

      d.reset(&[card!(Suit::Hearts, Rank::Jack)]);
      assert!(!d.is_empty());
      assert!(d.len() == 1);
      assert!(d.visible_cards().is_empty());
      assert!(d.waste_cards().is_empty());
      test_cards("remaining1", &[card!(Suit::Hearts, Rank::Jack)], d.remaining_cards());

      d.reset(&[
        card!(Suit::Hearts, Rank::Queen),
        card!(Suit::Hearts, Rank::King),
        card!(Suit::Spades, Rank::Ace),
      ]);
      assert!(!d.is_empty());
      assert!(d.len() == 3);
      assert!(d.visible_cards().is_empty());
      assert!(d.waste_cards().is_empty());
      test_cards("remaining2", &[card!(Suit::Hearts, Rank::Queen), card!(Suit::Hearts, Rank::King), card!(Suit::Spades, Rank::Ace)], d.remaining_cards());
    }

    #[test]
    fn draw_when_empty() {
      let mut d = Deck::new(1);
      d.draw();
      assert!(d.is_empty());
    }

    #[test]
    fn draw1_with_single_card() {
      let mut deck = Deck::new(1);

      deck.reset(&[card!(Suit::Hearts, Rank::Jack)]);
      test_deck(&deck, &[], &[], &[card!(Suit::Hearts, Rank::Jack)]);

      deck.draw();
      test_deck(&deck, &[card!(Suit::Hearts, Rank::Jack)], &[], &[]);

      deck.draw();
      test_deck(&deck, &[], &[], &[card!(Suit::Hearts, Rank::Jack)]);
    }

    #[test]
    fn top() {
      let mut deck = Deck::new(3);
      let cards = [
        card!(Suit::Hearts, Rank::Jack),
        card!(Suit::Diamonds, Rank::Number(3)),
        card!(Suit::Hearts, Rank::Queen),
        card!(Suit::Spades, Rank::Jack),
        card!(Suit::Clubs, Rank::Jack),
      ];
      deck.reset(&cards);

      assert!(deck.top().is_none());
      deck.draw();

      assert!(deck.top() == Some(card!(Suit::Hearts, Rank::Queen)), "{:?}", deck.top());
      deck.pop().unwrap();

      assert!(deck.top() == Some(card!(Suit::Diamonds, Rank::Number(3))));
      deck.pop().unwrap();

      assert!(deck.top() == Some(card!(Suit::Hearts, Rank::Jack)));
      deck.pop().unwrap();

      assert!(deck.top().is_none());
      deck.draw();

      assert!(deck.top() == Some(card!(Suit::Clubs, Rank::Jack)));
    }

    #[test]
    fn draw3_with_seven_cards() {
      let mut deck = Deck::new(3);
      let cards = [
        card!(Suit::Hearts, Rank::Jack),
        card!(Suit::Diamonds, Rank::Number(3)),
        card!(Suit::Hearts, Rank::Queen),
        card!(Suit::Spades, Rank::Jack),
        card!(Suit::Clubs, Rank::Ace),
        card!(Suit::Hearts, Rank::Number(10)),
        card!(Suit::Spades, Rank::Ace),
      ];

      deck.reset(&cards);
      test_deck(&deck, &[], &[], &cards[0..7]);

      deck.draw();
      test_deck(&deck, &cards[0..3], &[], &cards[3..7]);

      deck.draw();
      test_deck(&deck, &cards[3..6], &cards[0..3], &cards[6..7]);

      deck.draw();
      test_deck(&deck, &cards[6..7], &cards[0..6], &[]);

      deck.draw();
      test_deck(&deck, &[], &[], &cards[0..7]);
    }

    #[test]
    fn pop_when_empty() {
      let mut d = Deck::new(1);
      assert!(d.pop().is_none());
    }

    #[test]
    fn pop_with_seven_cards_draw3() {
      let mut deck = Deck::new(3);
      let cards = [
        card!(Suit::Hearts, Rank::Jack),
        card!(Suit::Diamonds, Rank::Number(3)),
        card!(Suit::Hearts, Rank::Queen),
        card!(Suit::Spades, Rank::Jack),
        card!(Suit::Clubs, Rank::Ace),
        card!(Suit::Hearts, Rank::Number(10)),
        card!(Suit::Spades, Rank::Ace),
      ];

      deck.reset(&cards);
      assert!(deck.pop().is_none());

      // Move to first 3 cards
      deck.draw();

      // Take all 3 visible cards
      assert!(deck.pop() == Some(card!(Suit::Hearts, Rank::Queen)));
      assert!(deck.len() == 6);
      test_deck(&deck, &cards[0..2], &[], &cards[3..7]);
      assert!(deck.pop() == Some(card!(Suit::Diamonds, Rank::Number(3))));
      assert!(deck.len() == 5);
      test_deck(&deck, &cards[0..1], &[], &cards[3..7]);
      assert!(deck.pop() == Some(card!(Suit::Hearts, Rank::Jack)));
      assert!(deck.len() == 4);
      test_deck(&deck, &[], &[], &cards[3..7]);

      // Move to next 3 cards
      deck.draw();
      test_deck(&deck, &cards[3..6], &[], &cards[6..7]);

      // Take 2 of 3 visible cards
      assert!(deck.pop() == Some(card!(Suit::Hearts, Rank::Number(10))));
      assert!(deck.len() == 3);
      test_deck(&deck, &cards[3..5], &[], &cards[6..7]);
      assert!(deck.pop() == Some(card!(Suit::Clubs, Rank::Ace)));
      assert!(deck.len() == 2);
      test_deck(&deck, &cards[3..4], &[], &cards[6..7]);

      // Move to last card in deck (do not take)
      deck.draw();
      test_deck(&deck, &cards[6..7], &cards[3..4], &[]);

      // Reset cards list with current deck contents
      let cards = {
        let mut x = Vec::new();
        x.extend_from_slice(&cards[3..4]);
        x.extend_from_slice(&cards[6..7]);
        x
      };

      // Move to beginning of deck
      deck.draw();
      test_deck(&deck, &[], &[], &cards[0..2]);

      // Move to remaining 2 cards
      deck.draw();
      test_deck(&deck, &cards[0..2], &[], &[]);

      // Take 2 visible cards
      assert!(deck.pop() == Some(card!(Suit::Spades, Rank::Ace)));
      assert!(deck.len() == 1);
      test_deck(&mut deck, &cards[0..1], &[], &[]);
      assert!(deck.pop() == Some(card!(Suit::Spades, Rank::Jack)));
      assert!(deck.len() == 0);
      test_deck(&mut deck, &[], &[], &[]);

      // Deck is now empty
    }

    #[test]
    fn from_empty() {
      let deck = Deck::from(3, &[], &[], &[]);
      assert!(deck.is_empty());
      assert_eq!(deck.draw_count(), 3);
    }

    #[test]
    fn from() {
      let deck = Deck::from(3, &[card!(Suit::Spades, Rank::Number(3))], &[card!(Suit::Diamonds, Rank::Number(3))], &[card!(Suit::Diamonds, Rank::Jack)]);
      test_cards("waste", &[card!(Suit::Spades, Rank::Number(3))], deck.waste_cards());
      test_cards("visible", &[card!(Suit::Diamonds, Rank::Number(3))], deck.visible_cards());
      test_cards("remaining", &[card!(Suit::Diamonds, Rank::Jack)], deck.remaining_cards());
    }

    #[test]
    fn from_max_cards() {
      // 24 is max deck size; 25 cards results in error
      let deck = Deck::from(3, &[
        card!(Suit::Spades, Rank::Number(10)),
        card!(Suit::Spades, Rank::Number(9)),
        card!(Suit::Spades, Rank::Number(8)),
        card!(Suit::Spades, Rank::Number(7)),
        card!(Suit::Spades, Rank::Number(6)),
        card!(Suit::Spades, Rank::Number(5)),
      ], &[
        card!(Suit::Hearts, Rank::Number(10)),
        card!(Suit::Hearts, Rank::Number(9)),
        card!(Suit::Hearts, Rank::Number(8)),
      ], &[
        card!(Suit::Hearts, Rank::Number(7)),
        card!(Suit::Hearts, Rank::Number(6)),
        card!(Suit::Hearts, Rank::Number(5)),

        card!(Suit::Diamonds, Rank::Number(10)),
        card!(Suit::Diamonds, Rank::Number(9)),
        card!(Suit::Diamonds, Rank::Number(8)),
        card!(Suit::Diamonds, Rank::Number(7)),
        card!(Suit::Diamonds, Rank::Number(6)),
        card!(Suit::Diamonds, Rank::Number(5)),

        card!(Suit::Clubs, Rank::Number(10)),
        card!(Suit::Clubs, Rank::Number(9)),
        card!(Suit::Clubs, Rank::Number(8)),
        card!(Suit::Clubs, Rank::Number(7)),
        card!(Suit::Clubs, Rank::Number(6)),
        card!(Suit::Clubs, Rank::Number(5)),
      ]);

      test_cards("waste", &[
        card!(Suit::Spades, Rank::Number(10)),
        card!(Suit::Spades, Rank::Number(9)),
        card!(Suit::Spades, Rank::Number(8)),
        card!(Suit::Spades, Rank::Number(7)),
        card!(Suit::Spades, Rank::Number(6)),
        card!(Suit::Spades, Rank::Number(5)),
      ], deck.waste_cards());
      test_cards("visible", &[
        card!(Suit::Hearts, Rank::Number(10)),
        card!(Suit::Hearts, Rank::Number(9)),
        card!(Suit::Hearts, Rank::Number(8)),
      ], deck.visible_cards());
      test_cards("remaining", &[
        card!(Suit::Hearts, Rank::Number(7)),
        card!(Suit::Hearts, Rank::Number(6)),
        card!(Suit::Hearts, Rank::Number(5)),

        card!(Suit::Diamonds, Rank::Number(10)),
        card!(Suit::Diamonds, Rank::Number(9)),
        card!(Suit::Diamonds, Rank::Number(8)),
        card!(Suit::Diamonds, Rank::Number(7)),
        card!(Suit::Diamonds, Rank::Number(6)),
        card!(Suit::Diamonds, Rank::Number(5)),

        card!(Suit::Clubs, Rank::Number(10)),
        card!(Suit::Clubs, Rank::Number(9)),
        card!(Suit::Clubs, Rank::Number(8)),
        card!(Suit::Clubs, Rank::Number(7)),
        card!(Suit::Clubs, Rank::Number(6)),
        card!(Suit::Clubs, Rank::Number(5)),
      ], deck.remaining_cards());
    }

    #[test]
    #[should_panic]
    fn from_error_duplicate_card() {
      Deck::from(3, &[card!(Suit::Spades, Rank::Number(3))], &[card!(Suit::Spades, Rank::Number(3))], &[]);
    }

    #[test]
    #[should_panic]
    fn from_error_too_many_visible() {
      Deck::from(1, &[], &[card!(Suit::Spades, Rank::Number(4)), card!(Suit::Spades, Rank::Number(3))], &[]);
    }

    #[test]
    #[should_panic]
    fn from_error_too_many_cards() {
      // 24 is max deck size; 25 cards results in error
      Deck::from(1, &[
        card!(Suit::Spades, Rank::Number(10)),
        card!(Suit::Spades, Rank::Number(9)),
        card!(Suit::Spades, Rank::Number(8)),
        card!(Suit::Spades, Rank::Number(7)),
        card!(Suit::Spades, Rank::Number(6)),
        card!(Suit::Spades, Rank::Number(5)),

        card!(Suit::Hearts, Rank::Number(10)),
        card!(Suit::Hearts, Rank::Number(9)),
        card!(Suit::Hearts, Rank::Number(8)),
        card!(Suit::Hearts, Rank::Number(7)),
        card!(Suit::Hearts, Rank::Number(6)),
        card!(Suit::Hearts, Rank::Number(5)),

        card!(Suit::Diamonds, Rank::Number(10)),
        card!(Suit::Diamonds, Rank::Number(9)),
        card!(Suit::Diamonds, Rank::Number(8)),
        card!(Suit::Diamonds, Rank::Number(7)),
        card!(Suit::Diamonds, Rank::Number(6)),
        card!(Suit::Diamonds, Rank::Number(5)),

        card!(Suit::Clubs, Rank::Number(10)),
        card!(Suit::Clubs, Rank::Number(9)),
        card!(Suit::Clubs, Rank::Number(8)),
        card!(Suit::Clubs, Rank::Number(7)),
        card!(Suit::Clubs, Rank::Number(6)),
        card!(Suit::Clubs, Rank::Number(5)),

        card!(Suit::Clubs, Rank::Queen),
      ], &[], &[]);
    }
  }
}
