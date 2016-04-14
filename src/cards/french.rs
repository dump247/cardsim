use std::slice::Iter;

pub const STANDARD_DECK_SIZE: u8 = 52;

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum Color {
  Red,
  Black,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum Suit {
  Diamonds,
  Hearts,
  Clubs,
  Spades,
}

impl Suit {
  pub fn standard_iter() -> Iter<'static, Suit> {
    static SUITS: [Suit; 4] = [Suit::Diamonds, Suit::Hearts, Suit::Clubs, Suit::Spades];
    SUITS.into_iter()
  }

  pub fn color(&self) -> Color {
    match *self {
      Suit::Diamonds | Suit::Hearts => Color::Red,
      Suit::Clubs | Suit::Spades => Color::Black,
    }
  }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum Rank {
  Ace,
  Number(i8),
  Jack,
  Queen,
  King,
}

impl Rank {
  pub fn standard_iter() -> Iter<'static, Rank> {
    static RANKS: [Rank; 13] = [
      Rank::Ace, Rank::Number(2), Rank::Number(3), Rank::Number(4),
      Rank::Number(5), Rank::Number(6), Rank::Number(7), Rank::Number(8),
      Rank::Number(9), Rank::Number(10), Rank::Jack, Rank::Queen,
      Rank::King
    ];
    RANKS.into_iter()
  }
}

/// Common French playing card.
///
/// Each card has a suit (spades, hearts, clubs, diamonds) and a rank (ace, 2,
/// 10, king, etc).
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct FrenchPlayingCard {
  suit: Suit,
  rank: Rank,
}

impl FrenchPlayingCard {
  pub fn new(suit: Suit, rank: Rank) -> FrenchPlayingCard {
    if let Rank::Number(n) = rank {
      assert!(n >= 2 && n <= 10, "Invalid rank number {}. Valid range is 2-10.", n);
    }

    FrenchPlayingCard{suit: suit, rank: rank}
  }

  pub fn color(&self) -> Color {
    self.suit.color()
  }

  pub fn suit(&self) -> Suit {
    self.suit
  }

  pub fn rank(&self) -> Rank {
    self.rank
  }
}

/// Constructs a new deck of standard French playing cards.
///
/// The resulting deck has 52 red and black cards with common ranks
/// (ace, 2-10, jack, queen, and king) and suits (spades, clubs, hearts,
/// and diamonds).
pub fn new_standard_deck() -> Vec<FrenchPlayingCard> {
  let mut deck = Vec::with_capacity(STANDARD_DECK_SIZE as usize);

  for suit in Suit::standard_iter() {
    for rank in Rank::standard_iter() {
      deck.push(FrenchPlayingCard{suit: *suit, rank: *rank});
    }
  }

  debug_assert!(deck.len() == STANDARD_DECK_SIZE as usize);
  return deck;
}

#[cfg(test)]
mod test {
  use super::*;

  #[test]
  fn test_new_french_playing_card() {
    let card = FrenchPlayingCard::new(Suit::Spades, Rank::Ace);
    assert!(card.color() == Color::Black);
    assert!(card.suit() == Suit::Spades);
    assert!(card.rank() == Rank::Ace);

    assert!(card == FrenchPlayingCard::new(Suit::Spades, Rank::Ace));
    assert!(card != FrenchPlayingCard::new(Suit::Hearts, Rank::Ace));
    assert!(card != FrenchPlayingCard::new(Suit::Spades, Rank::Number(2)));
  }

  #[test]
  fn test_new_standard_deck() {
    let deck = new_standard_deck();
    assert!(deck.len() == 52, "{} != 52", deck.len());

    assert!(deck.iter().filter(|c| c.color() == Color::Black).count() == 26);
    assert!(deck.iter().filter(|c| c.color() == Color::Red).count() == 26);

    assert!(deck.iter().filter(|c| c.suit() == Suit::Spades).count() == 13);
    assert!(deck.iter().filter(|c| c.suit() == Suit::Diamonds).count() == 13);
    assert!(deck.iter().filter(|c| c.suit() == Suit::Hearts).count() == 13);
    assert!(deck.iter().filter(|c| c.suit() == Suit::Clubs).count() == 13);

    assert!(deck.iter().filter(|c| c.rank() == Rank::Ace).count() == 4);
    assert!(deck.iter().filter(|c| c.rank() == Rank::Number(2)).count() == 4);
    assert!(deck.iter().filter(|c| c.rank() == Rank::Number(3)).count() == 4);
    assert!(deck.iter().filter(|c| c.rank() == Rank::Number(4)).count() == 4);
    assert!(deck.iter().filter(|c| c.rank() == Rank::Number(5)).count() == 4);
    assert!(deck.iter().filter(|c| c.rank() == Rank::Number(6)).count() == 4);
    assert!(deck.iter().filter(|c| c.rank() == Rank::Number(7)).count() == 4);
    assert!(deck.iter().filter(|c| c.rank() == Rank::Number(8)).count() == 4);
    assert!(deck.iter().filter(|c| c.rank() == Rank::Number(9)).count() == 4);
    assert!(deck.iter().filter(|c| c.rank() == Rank::Number(10)).count() == 4);
    assert!(deck.iter().filter(|c| c.rank() == Rank::Jack).count() == 4);
    assert!(deck.iter().filter(|c| c.rank() == Rank::Queen).count() == 4);
    assert!(deck.iter().filter(|c| c.rank() == Rank::King).count() == 4);
  }
}
