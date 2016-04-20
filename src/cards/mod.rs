pub mod french;

use rand;
use rand::Rng;

pub fn default_shuffle<T>(mut cards: &mut Vec<T>) {
    rand::thread_rng().shuffle(&mut cards);
}
