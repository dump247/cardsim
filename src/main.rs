extern crate rand;

pub mod cards;
pub mod games;

fn main() {
  for card in cards::french::new_standard_deck() {
    println!("{:?}", card);
  }
}
