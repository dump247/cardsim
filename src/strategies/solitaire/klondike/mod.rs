pub mod simple;

use games::solitaire::klondike::KlondikeSolitaireGame;

pub trait GameFilter {
    fn new() -> Self;
    fn accept(&self, game: &KlondikeSolitaireGame) -> bool;
}

pub trait GameStrategy {
    fn new() -> Self;
    fn play(&mut self, game: &mut KlondikeSolitaireGame);
}

pub struct AllFilter;

impl GameFilter for AllFilter {
  fn new() -> AllFilter {
    AllFilter
  }

  fn accept(&self, _game: &KlondikeSolitaireGame) -> bool {
    true
  }
}
