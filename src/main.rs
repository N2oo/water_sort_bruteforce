mod game;
mod pipe;
use std::collections::HashMap;

use crate::game::Game;
use crate::pipe::{Color, Pipe};

fn main() {
    let mut pipes = HashMap::new();
    let mut p1 = Pipe::new(String::from("P1"));
    p1.add_color(Color::Red);
    p1.add_color(Color::Blue);
    p1.add_color(Color::Blue);
    p1.add_color(Color::Red);

    let mut p2 = Pipe::new(String::from("P2"));
    p2.add_color(Color::Blue);
    p2.add_color(Color::Red);
    p2.add_color(Color::Red);
    p2.add_color(Color::Blue);
    
    let p3 = Pipe::new(String::from("P3"));

    pipes.insert(String::from("P1"), p1);
    pipes.insert(String::from("P2"), p2);
    pipes.insert(String::from("P3"), p3);

    let game = Game::new(pipes);
    //Il faut résoudre le puzzle
}
