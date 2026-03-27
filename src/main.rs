mod pipe;
mod game;
use std::collections::HashMap;

use crate::pipe::{Pipe,Color};
use crate::game::Game;


fn main() {
    let mut pipes = HashMap::new();
    pipes.insert(String::from("P1"), Pipe::new());
    let game = Game::new(pipes);
}
