use games::solitaire::klondike::*;
use super::GameStrategy;

pub struct SimpleKlondikeStrategy;

impl GameStrategy for SimpleKlondikeStrategy {
  fn new() -> SimpleKlondikeStrategy {
    SimpleKlondikeStrategy
  }

  fn play(&mut self, game: &mut KlondikeSolitaireGame) {
    let mut moved = false;

    while ! game.is_clear() {
      // Move pile to foundation
      if let Some((source, target)) = check_pile_to_foundation(game) {
        moved = true;
        game.move_cards(source, target).unwrap();
        continue;
      }

      // Move deck to foundation or pile
      if ! game.deck().visible_cards().is_empty() {
        if game.move_cards(MoveSource::Deck, MoveTarget::Foundation).is_ok() {
          moved = true;
          continue;
        }

        if let Some((source, target)) = check_deck_to_pile(game) {
          moved = true;
          game.move_cards(source, target).unwrap();
          continue;
        }
      }

      // TODO move cards between piles if it opens a move to foundation or from deck

      game.draw();

      // Exit if have iterated through deck and no moves occurred
      if is_at_start(game.deck()) {
        if ! moved {
          break;
        }

        moved = false;
      }
    }
  }
}

fn is_at_start(deck: &Deck) -> bool {
  deck.visible_cards().is_empty() && deck.waste_cards().is_empty()
}

fn check_pile_to_foundation(game: &KlondikeSolitaireGame) -> Option<(MoveSource, MoveTarget)> {
  for (index, pile) in game.piles().iter().enumerate().filter(|&(_, p)| ! p.is_empty()) {
    let card = pile.top().unwrap();
    let foundation = game.foundation(card.suit());

    if foundation.next_card() == Some(card) {
      return Some((MoveSource::Pile(index as u8), MoveTarget::Foundation));
    }
  }

  return None;
}

fn check_deck_to_pile(game: &KlondikeSolitaireGame) -> Option<(MoveSource, MoveTarget)> {
  let deck_card = game.deck().top().unwrap();

  match game.piles().iter().position(|p| p.can_push(deck_card).is_ok()) {
    Some(i) => Some((MoveSource::Deck, MoveTarget::Pile(i as u8))),
    None => None,
  }
}
