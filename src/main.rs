extern crate rand;
#[macro_use(crate_version)]
extern crate clap;

pub mod cards;
pub mod games;
pub mod strategies;

use clap::{Arg, App, SubCommand};
use std::thread;
use rand::Rng;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

use strategies::solitaire::klondike::{GameFilter, GameStrategy, AllFilter};
use strategies::solitaire::klondike::simple::SimpleKlondikeStrategy;

fn validate_num(name: &str, min: usize, max: usize, v: String) -> Result<(), String> {
    match v.parse::<usize>() {
        Ok(v) if v >= min && v <= max => Ok(()),
        _ if max == usize::max_value() => Err(String::from(format!("{} must be a number greater than or equal to {}", name, min))),
        _ => Err(String::from(format!("{} must be a number between {} and {}", name, min, max))),
    }
}

fn run_klondike<S: GameStrategy, F: GameFilter>(game_count: usize, thread_count: usize) {
    let mut threads = Vec::with_capacity(thread_count);
    let games_per_thread = game_count / thread_count;
    let add_game = game_count % thread_count;
    let wins = Arc::new(AtomicUsize::new(0));
    let games = Arc::new(AtomicUsize::new(0));

    for i in 0..thread_count {
        let game_count = games_per_thread + if i + 1 <= add_game { 1 } else { 0 };
        println!("{}", game_count);

        let wins = wins.clone();
        let games = games.clone();

        threads.push(thread::spawn(move || {
            let mut rng = rand::StdRng::new().unwrap();
            let mut strategy = S::new();
            let filter = F::new();

            for _ in 0..game_count {
                loop {
                    let mut game = games::solitaire::klondike::KlondikeSolitaireGame::new_shuffle(1, |mut c| rng.shuffle(&mut c));

                    if filter.accept(&game) {
                      strategy.play(&mut game);

                      if game.is_clear() {
                        wins.fetch_add(1, Ordering::Relaxed);
                      }

                      break;
                    }
                }

                let g = games.fetch_add(1, Ordering::Relaxed);
                if g % 10000 == 0 {
                    println!("{} games", g);
                }
            }
        }));
    }

    for t in threads {
        t.join().unwrap();
    }

    println!("{}/{} wins", wins.load(Ordering::SeqCst), games.load(Ordering::SeqCst));
}

fn main() {
    let matches = App::new("Card Game Simulator")
        .version(crate_version!())
        .about("Simulate card games.")
        .subcommand(SubCommand::with_name("solitaire:klondike")
                    .version(crate_version!())
                    .about("Play klondike solitaire")
                    .arg(Arg::with_name("games")
                         .long("games")
                         .takes_value(true)
                         .default_value("1000000")
                         .validator(|v| validate_num("games", 1, usize::max_value(), v))
                         .help("Number of games to play"))
                    .arg(Arg::with_name("concurrency")
                         .long("concurrency")
                         .takes_value(true)
                         .default_value("1")
                         .validator(|v| validate_num("concurrency", 1, usize::max_value(), v))
                         .help("Number of concurrent games to play")))
        .get_matches();

    if let Some(matches) = matches.subcommand_matches("solitaire:klondike") {
        let game_count = matches.value_of("games").unwrap().parse::<usize>().unwrap();
        let thread_count = matches.value_of("concurrency").unwrap().parse::<usize>().unwrap();
        run_klondike::<SimpleKlondikeStrategy, AllFilter>(game_count, thread_count);
        return;
    }

    panic!("Unhandled command!");
}
