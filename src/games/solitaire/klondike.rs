use cards::{Shuffler, french};
use cards::french::{FrenchPlayingCard, Rank, Suit};
use std::cmp;

const MAX_DECK_SIZE: usize = 24;

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

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum KlondikeErr {
  InvalidRank,
  InvalidSuit,
}

pub type KlondikeResult<T> = Result<T, KlondikeErr>;

pub struct KlondikeSolitaireGame {
  cards: Vec<FrenchPlayingCard>,
  foundations: [Foundation; 4],
  piles: [Pile; 7],
  deck: Deck,
}

impl KlondikeSolitaireGame {
  pub fn new<S: Shuffler>(shuffler: &mut S, draw_count: u8) -> KlondikeSolitaireGame {
    let cards = french::new_standard_deck();

    // The order in the game struct initialization must match the indexes
    // returned by foundation_index function.
    debug_assert!(KlondikeSolitaireGame::foundation_index(Suit::Hearts) == 0);
    debug_assert!(KlondikeSolitaireGame::foundation_index(Suit::Diamonds) == 1);
    debug_assert!(KlondikeSolitaireGame::foundation_index(Suit::Spades) == 2);
    debug_assert!(KlondikeSolitaireGame::foundation_index(Suit::Clubs) == 3);

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
    game.new_game(shuffler);

    return game;
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

  pub fn new_game<S: Shuffler>(&mut self, shuffler: &mut S) {
    shuffler.shuffle(&mut self.cards);
    self.reset();
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
}

pub struct Deck {
  cards: Vec<FrenchPlayingCard>,
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

  pub fn reset(&mut self, cards: &[FrenchPlayingCard]) {
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

  pub fn visible_cards(&self) -> Option<&[FrenchPlayingCard]> {
    match self.visible_count {
      0 => None,
      count => Some(&self.cards[self.visible_index..self.visible_index+count]),
    }
  }

  pub fn waste_cards(&self) -> Option<&[FrenchPlayingCard]> {
    match self.visible_index {
      0 => None,
      index => Some(&self.cards[..index]),
    }
  }

  pub fn remaining_cards(&self) -> Option<&[FrenchPlayingCard]> {
    match self.visible_index + self.visible_count {
      index if index < self.cards.len() => Some(&self.cards[index..]),
      _ => None,
    }
  }

  pub fn take_one(&mut self) -> Option<FrenchPlayingCard> {
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
    // &[FrenchPlayingCard]: visible cards
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
  visible_cards: Vec<FrenchPlayingCard>,
  hidden_cards: Vec<FrenchPlayingCard>,
}

impl Pile {
  pub fn new() -> Pile {
    Pile {
      visible_cards: Vec::new(),
      hidden_cards: Vec::with_capacity(6),
    }
  }

  pub fn reset(&mut self, cards: &[FrenchPlayingCard]) {
    assert!(cards.len() <= 7 && cards.len() > 0);

    self.hidden_cards.clear();
    self.hidden_cards.extend_from_slice(&cards[0..cards.len()-1]);

    self.visible_cards.clear();
    self.visible_cards.push(cards[cards.len()-1]);
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

  pub fn top(&self) -> Option<FrenchPlayingCard> {
    match self.current_rank_index {
      Some(i) => Some(FrenchPlayingCard::new(self.suit, RANKS[i])),
      None => None,
    }
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

  pub fn next_card(&self) -> Option<FrenchPlayingCard> {
    match self.next_rank() {
      Some(r) => Some(FrenchPlayingCard::new(self.suit, r)),
      None => None,
    }
  }

  pub fn push(&mut self) -> Option<FrenchPlayingCard> {
    match self.current_rank_index {
      Some(i) if i == RANKS.len() - 1 => None,
      Some(i) => {
        self.current_rank_index = Some(i+1);
        Some(FrenchPlayingCard::new(self.suit, RANKS[i]))
      },
      None => {
        self.current_rank_index = Some(0);
        Some(FrenchPlayingCard::new(self.suit, RANKS[0]))
      }
    }
  }

  pub fn clear(&mut self) {
    self.current_rank_index = None;
  }

  pub fn pop(&mut self) -> Option<FrenchPlayingCard> {
    match self.current_rank_index {
      Some(0) => {
        self.current_rank_index = None;
        Some(FrenchPlayingCard::new(self.suit, RANKS[0]))
      },
      Some(i) => {
        self.current_rank_index = Some(i-1);
        Some(FrenchPlayingCard::new(self.suit, RANKS[i-1]))
      },
      None => None
    }
  }
}

#[cfg(test)]
mod test {
  macro_rules! card {
    ($suit:expr, $rank:expr) => (FrenchPlayingCard::new($suit, $rank));
  }

  mod foundation {
    use super::super::*;
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
  }

  mod deck {
    use super::super::*;
    use cards::french::{FrenchPlayingCard, Suit, Rank};

    #[test]
    fn new_deck() {
      let d = Deck::new(1);
      assert!(d.is_empty());
      assert!(d.len() == 0);
      assert!(d.visible_cards().is_none());
      assert!(d.waste_cards().is_none());
      assert!(d.remaining_cards().is_none());
    }

    #[test]
    fn reset() {
      let mut d = Deck::new(1);

      d.reset(&[]);
      assert!(d.is_empty());

      d.reset(&[card!(Suit::Hearts, Rank::Jack)]);
      assert!(!d.is_empty());
      assert!(d.len() == 1);
      assert!(d.visible_cards().is_none());
      assert!(d.waste_cards().is_none());
      match d.remaining_cards() {
        Some(cards) => {
          assert!(cards.len() == 1);
          assert!(cards[0] == card!(Suit::Hearts, Rank::Jack));
        },
        _ => panic!("unexpected remaining cards"),
      }

      d.reset(&[
        card!(Suit::Hearts, Rank::Queen),
        card!(Suit::Hearts, Rank::King),
        card!(Suit::Spades, Rank::Ace),
      ]);
      assert!(!d.is_empty());
      assert!(d.len() == 3);
      assert!(d.visible_cards().is_none());
      assert!(d.waste_cards().is_none());
      match d.remaining_cards() {
        Some(cards) => {
          assert!(cards.len() == 3);
          assert!(cards[0] == card!(Suit::Hearts, Rank::Queen));
          assert!(cards[1] == card!(Suit::Hearts, Rank::King));
          assert!(cards[2] == card!(Suit::Spades, Rank::Ace));
        },
        _ => panic!("unexpected remaining cards"),
      }
    }

    fn test_cards(name: &str, expected: Option<&[FrenchPlayingCard]>, actual: Option<&[FrenchPlayingCard]>) {
      match expected {
        Some(expected) => {
          assert!(actual.is_some(), "{}", name);
          let actual = actual.unwrap();
          assert!(actual.len() == expected.len(), "{}: {} != {}", name, expected.len(), actual.len());
          for i in 0..expected.len() {
            assert!(expected[i] == actual[i], "{}[{}]: {:?} != {:?}", name, i, expected[i], actual[i]);
          }
        },
        None => assert!(actual.is_none()),
      };
    }

    fn test_deck(deck: &Deck, visible: Option<&[FrenchPlayingCard]>, waste: Option<&[FrenchPlayingCard]>, remaining: Option<&[FrenchPlayingCard]>) {
      test_cards("visible", visible, deck.visible_cards());
      test_cards("waste", waste, deck.waste_cards());
      test_cards("remaining", remaining, deck.remaining_cards());
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
      test_deck(&deck, None, None, Some(&[card!(Suit::Hearts, Rank::Jack)]));

      deck.draw();
      test_deck(&deck, Some(&[card!(Suit::Hearts, Rank::Jack)]), None, None);

      deck.draw();
      test_deck(&deck, None, None, Some(&[card!(Suit::Hearts, Rank::Jack)]));
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
      test_deck(&deck, None, None, Some(&cards[0..7]));

      deck.draw();
      test_deck(&deck, Some(&cards[0..3]), None, Some(&cards[3..7]));

      deck.draw();
      test_deck(&deck, Some(&cards[3..6]), Some(&cards[0..3]), Some(&cards[6..7]));

      deck.draw();
      test_deck(&deck, Some(&cards[6..7]), Some(&cards[0..6]), None);

      deck.draw();
      test_deck(&deck, None, None, Some(&cards[0..7]));
    }

    #[test]
    fn take_one_when_empty() {
      let mut d = Deck::new(1);
      assert!(d.take_one().is_none());
    }

    #[test]
    fn take_one_with_seven_cards_draw3() {
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
      assert!(deck.take_one().is_none());

      // Move to first 3 cards
      deck.draw();

      // Take all 3 visible cards
      assert!(deck.take_one() == Some(card!(Suit::Hearts, Rank::Queen)));
      assert!(deck.len() == 6);
      test_deck(&deck, Some(&cards[0..2]), None, Some(&cards[3..7]));
      assert!(deck.take_one() == Some(card!(Suit::Diamonds, Rank::Number(3))));
      assert!(deck.len() == 5);
      test_deck(&deck, Some(&cards[0..1]), None, Some(&cards[3..7]));
      assert!(deck.take_one() == Some(card!(Suit::Hearts, Rank::Jack)));
      assert!(deck.len() == 4);
      test_deck(&deck, None, None, Some(&cards[3..7]));

      // Move to next 3 cards
      deck.draw();
      test_deck(&deck, Some(&cards[3..6]), None, Some(&cards[6..7]));

      // Take 2 of 3 visible cards
      assert!(deck.take_one() == Some(card!(Suit::Hearts, Rank::Number(10))));
      assert!(deck.len() == 3);
      test_deck(&deck, Some(&cards[3..5]), None, Some(&cards[6..7]));
      assert!(deck.take_one() == Some(card!(Suit::Clubs, Rank::Ace)));
      assert!(deck.len() == 2);
      test_deck(&deck, Some(&cards[3..4]), None, Some(&cards[6..7]));

      // Move to last card in deck (do not take)
      deck.draw();
      test_deck(&deck, Some(&cards[6..7]), Some(&cards[3..4]), None);

      // Reset cards list with current deck contents
      let cards = {
        let mut x = Vec::new();
        x.extend_from_slice(&cards[3..4]);
        x.extend_from_slice(&cards[6..7]);
        x
      };

      // Move to beginning of deck
      deck.draw();
      test_deck(&deck, None, None, Some(&cards[0..2]));

      // Move to remaining 2 cards
      deck.draw();
      test_deck(&deck, Some(&cards[0..2]), None, None);

      // Take 2 visible cards
      assert!(deck.take_one() == Some(card!(Suit::Spades, Rank::Ace)));
      assert!(deck.len() == 1);
      test_deck(&mut deck, Some(&cards[0..1]), None, None);
      assert!(deck.take_one() == Some(card!(Suit::Spades, Rank::Jack)));
      assert!(deck.len() == 0);
      test_deck(&mut deck, None, None, None);

      // Deck is now empty
    }
  }
}
