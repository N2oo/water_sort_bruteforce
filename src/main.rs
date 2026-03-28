mod game;
mod pipe;
use std::collections::{HashMap, HashSet};

use crate::game::Game;
use crate::pipe::{Color, Pipe};

fn main() {
    let mut pipes = HashMap::new();
    let mut p1 = Pipe::new(String::from("P1"));
    p1.add_color(Color::Brown);
    p1.add_color(Color::Lemon);
    p1.add_color(Color::Blue);
    p1.add_color(Color::Grey);
    let mut p2 = Pipe::new(String::from("P2"));
    p2.add_color(Color::Green);
    p2.add_color(Color::Red);
    p2.add_color(Color::Red);
    p2.add_color(Color::Green);
    
    let mut p3 = Pipe::new(String::from("P3"));
    p3.add_color(Color::Pink);
    p3.add_color(Color::Blue);
    p3.add_color(Color::LightGreen);
    p3.add_color(Color::LightBlue);

    let mut p4 = Pipe::new(String::from("P4"));
    p4.add_color(Color::LightGreen);
    p4.add_color(Color::Grey);
    p4.add_color(Color::Brown);
    p4.add_color(Color::Orange);

    let mut p5 = Pipe::new(String::from("P5"));
    p5.add_color(Color::Blue);
    p5.add_color(Color::Orange);
    p5.add_color(Color::LightBlue);
    p5.add_color(Color::Purple);

    
    let mut p6 = Pipe::new(String::from("P6"));
    p6.add_color(Color::Yellow);
    p6.add_color(Color::Yellow);
    p6.add_color(Color::Lemon);
    p6.add_color(Color::LightBlue);

    
    let mut p7 = Pipe::new(String::from("P7"));
    p7.add_color(Color::LightGreen);
    p7.add_color(Color::Blue);
    p7.add_color(Color::Purple);
    p7.add_color(Color::Grey);

    
    let mut p8 = Pipe::new(String::from("P8"));
    p8.add_color(Color::Pink);
    p8.add_color(Color::Red);
    p8.add_color(Color::Green);
    p8.add_color(Color::Orange);

    
    let mut p9 = Pipe::new(String::from("P9"));
    p9.add_color(Color::LightBlue);
    p9.add_color(Color::Yellow);
    p9.add_color(Color::Grey);
    p9.add_color(Color::Brown);

    
    let mut p10 = Pipe::new(String::from("P10"));
    p10.add_color(Color::Purple);
    p10.add_color(Color::Pink);
    p10.add_color(Color::Brown);
    p10.add_color(Color::LightGreen);

    
    let mut p11 = Pipe::new(String::from("P11"));
    p11.add_color(Color::Red);
    p11.add_color(Color::Lemon);
    p11.add_color(Color::Green);
    p11.add_color(Color::Purple);

    
    let mut p12 = Pipe::new(String::from("P12"));
    p12.add_color(Color::Yellow);
    p12.add_color(Color::Orange);
    p12.add_color(Color::Lemon);
    p12.add_color(Color::Pink);

    let mut p13 = Pipe::new(String::from("P13"));
    let mut p14 = Pipe::new(String::from("P14"));


    pipes.insert(String::from("P1"), p1);
    pipes.insert(String::from("P2"), p2);
    pipes.insert(String::from("P3"), p3);
    pipes.insert(String::from("P4"), p4);
    pipes.insert(String::from("P5"), p5);
    pipes.insert(String::from("P6"), p6);
    pipes.insert(String::from("P7"), p7);
    pipes.insert(String::from("P8"), p8);
    pipes.insert(String::from("P9"), p9);
    pipes.insert(String::from("P10"), p10);
    pipes.insert(String::from("P11"), p11);
    pipes.insert(String::from("P12"), p12);
    pipes.insert(String::from("P13"), p13);
    pipes.insert(String::from("P14"), p14);

    let game = Game::new(pipes);
    if let Some(solution) = solve_puzzle_by_bruteforce(&game) {
        println!("Solution trouvée en {} coups : {solution:#?}",solution.len());
    } else {
        println!("Aucune solution trouvée");
    }
}

fn game_state_key(game: &Game) -> String {
    let mut names: Vec<String> = game.pipes().keys().cloned().collect();
    names.sort();

    let pipes = game.pipes();
    names
        .iter()
        .map(|name| format!("{}:{:?}", name, pipes.get(name).unwrap()))
        .collect::<Vec<_>>()
        .join("|")
}

fn search_bruteforce(game: &Game, visited: &mut HashSet<String>, path: &mut Vec<String>) -> bool {
    // Limite de profondeur pour éviter l'overflow
    const MAX_DEPTH: usize = 50;
    
    if path.len() > MAX_DEPTH {
        eprintln!("⚠️  Profondeur maximale atteinte ({}) - chemin actuel ({} moves) - terminés ({}/{}):", 
                  MAX_DEPTH, path.len(),game.how_many_finished(),game.pipes().len()
                );
        println!("{path:#?}");
        eprintln!("Derniers mouvements: {:?}", 
                  path.iter().skip(path.len().saturating_sub(MAX_DEPTH)).collect::<Vec<_>>());
        return false;
    }
    
    if game.is_finished() {
        return true;
    }

    let state_key = game_state_key(game);
    if visited.contains(&state_key) {
        return false;
    }
    visited.insert(state_key);

    let mut names: Vec<String> = game.pipes().keys().cloned().collect();
    names.sort();

    for src in &names {
        for dst in &names {
            if src == dst {
                continue;
            }

            let pipes = game.pipes();
            let source_pipe = pipes.get(src).unwrap();
            let destination_pipe = pipes.get(dst).unwrap();

            if !source_pipe.can_pour(destination_pipe) {
                continue;
            }

            let mut next_pipes = game.pipes();
            let mut source_mut = next_pipes.remove(src).unwrap();
            let mut dest_mut = next_pipes.remove(dst).unwrap();

            source_mut.pour_into(&mut dest_mut);

            next_pipes.insert(src.clone(), source_mut);
            next_pipes.insert(dst.clone(), dest_mut);

            let next_game = Game::new(next_pipes);
            
            let next = format!("{}->{}", src, dst);
            let could_previous = format!("{}->{}", dst, src);

            path.push(next);

            if search_bruteforce(&next_game, visited, path) {
                return true;
            }

            path.pop();
        }
    }

    false
}

fn solve_puzzle_by_bruteforce(game: &Game) -> Option<Vec<String>> {
    let mut visited = HashSet::new();
    let mut path = Vec::new();

    if search_bruteforce(game, &mut visited, &mut path) {
        Some(optimize(path))
    } else {
        None
    }
}

fn optimize(path:Vec<String>)->Vec<String>{
    let mut result:Vec<String> = vec![];
    let arrow = "->";

    for (_,item )in path.iter().enumerate(){
        if let Some(previous) = result.last(){
            let splitted:Vec<&str> = item.split(&arrow).collect();
            let inversed_current = format!("{}{}{}",splitted[1],&arrow,splitted[0]);
            if *previous == inversed_current{
                result.pop();
            }
        }
        result.push(item.clone());
    } 

    return result;
}