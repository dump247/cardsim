pub mod french;

use rand;

pub trait Shuffler {
  fn shuffle<C>(&mut self, deck: &mut Vec<C>);
}

pub struct RngShuffler<T: rand::Rng> {
  rng: T,
}

impl<T: rand::Rng> RngShuffler<T> {
  pub fn with_rng(rng: T) -> RngShuffler<T> {
    RngShuffler{ rng: rng }
  }
}

impl<T: rand::Rng> Shuffler for RngShuffler<T> {
  fn shuffle<C>(&mut self, deck: &mut Vec<C>) {
    let deck = deck.as_mut_slice();
    self.rng.shuffle(deck);
  }
}

pub type StdRngShuffler = RngShuffler<rand::StdRng>;

impl StdRngShuffler {
  pub fn new() -> StdRngShuffler {
    RngShuffler::with_rng(rand::StdRng::new().unwrap())
  }
}
