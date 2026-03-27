use std::collections::HashMap;
use crate::pipe::{Pipe,Color};
#[derive(Clone)]
pub struct Game {
    pipes: HashMap<String,Pipe>
}

impl Game {
    pub fn new(pipes: HashMap<String,Pipe>)->Self{
        Game {
            pipes: pipes
        }
    }
    pub fn pipes(&self)->HashMap<String,Pipe>{
        return self.pipes.clone();
    }

    pub fn is_finished(& self)->bool{
        for (_name,p) in &self.pipes{
            if !p.is_completed() {
                return false;
            }
        }
        return true;
    }
}



#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn can_be_finished_with_no_empty() {

        let mut pipes = HashMap::new();
        let mut p1 = Pipe::new(String::from("P1"));
        p1.add_color(Color::Blue);
        p1.add_color(Color::Blue);
        p1.add_color(Color::Blue);
        p1.add_color(Color::Blue);
        
        let mut p2 = Pipe::new(String::from("P2"));
        p2.add_color(Color::Red);
        p2.add_color(Color::Red);
        p2.add_color(Color::Red);
        p2.add_color(Color::Red);

        pipes.insert(String::from("P1"), p1);
        pipes.insert(String::from("P2"), p2);
        let game = Game::new(pipes);
        assert!(game.is_finished())
    }
    #[test]
    fn is_not_finished_because_shuffled() {

        let mut pipes = HashMap::new();
        let mut p1 = Pipe::new(String::from("P1"));
        p1.add_color(Color::Blue);
        p1.add_color(Color::Red);
        p1.add_color(Color::Blue);
        p1.add_color(Color::Red);
        
        let mut p2 = Pipe::new(String::from("P2"));
        p2.add_color(Color::Red);
        p2.add_color(Color::Blue);
        p2.add_color(Color::Red);
        p2.add_color(Color::Blue);

        pipes.insert(String::from("P1"), p1);
        pipes.insert(String::from("P2"), p2);
        let game = Game::new(pipes);
        assert!(!game.is_finished())
    }
}