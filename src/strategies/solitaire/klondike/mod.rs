pub mod simple;

use games::solitaire::klondike::KlondikeSolitaireGame;

pub trait KlondikeStrategy {
    fn new() -> Self;
    fn run(&mut self, game: &mut KlondikeSolitaireGame);
}
