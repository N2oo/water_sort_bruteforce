mod game;
mod pipe;
use std::collections::{HashMap, HashSet};
use std::error::Error;
use std::fs::File;
use std::path::Path;

use crate::game::Game;
use crate::pipe::{Color, Pipe};

fn main() {
    let json_path = std::env::args().nth(1).unwrap_or_else(|| "level.json".to_string());

    let game = match load_game_from_json(Path::new(&json_path)) {
        Ok(game) => game,
        Err(err) => {
            eprintln!("Erreur lors du chargement du JSON '{}': {}", json_path, err);
            eprintln!("Utilisation: cargo run -- <fichier_json>");
            return;
        }
    };

    if let Some(moves) = solve_puzzle_by_bruteforce(&game) {
        println!("✓ Solution trouvée en {} mouvements !\n", moves.len());
        for (i, movement) in moves.iter().enumerate() {
            println!("  Mouvement {} : {}", i + 1, movement);
        }
    } else {
        println!("✗ Aucune solution trouvée");
    }
}

fn parse_color(color: &str) -> Result<Color, String> {
    match color.to_lowercase().as_str() {
        "grey" | "gray" => Ok(Color::Grey),
        "blue" => Ok(Color::Blue),
        "lemon" => Ok(Color::Lemon),
        "brown" => Ok(Color::Brown),
        "green" => Ok(Color::Green),
        "red" => Ok(Color::Red),
        "lightgreen" | "light_green" => Ok(Color::LightGreen),
        "lightblue" | "light_blue" => Ok(Color::LightBlue),
        "pink" => Ok(Color::Pink),
        "orange" => Ok(Color::Orange),
        "purple" => Ok(Color::Purple),
        "yellow" => Ok(Color::Yellow),
        _ => Err(format!("Couleur inconnue : '{}'.", color)),
    }
}

fn load_game_from_json(path: &Path) -> Result<Game, Box<dyn Error>> {
    println!("{path:#?}");
    let file = File::open(path)?;
    let json: serde_json::Value = serde_json::from_reader(file)?;

    let pipes_obj = json
        .get("pipes")
        .and_then(|v| v.as_object())
        .ok_or("La clé 'pipes' doit être un objet JSON")?;

    let mut pipes = HashMap::new();

    for (ident, value) in pipes_obj.iter() {
        let colors = value
            .as_array()
            .ok_or_else(|| format!("La valeur pour '{}' doit être un tableau", ident))?;

        let mut pipe = Pipe::new(ident.clone());
        for color_value in colors {
            let color_str = color_value
                .as_str()
                .ok_or_else(|| format!("Couleur non valide dans '{}'", ident))?;
            pipe.add_color(parse_color(color_str)?);
        }

        pipes.insert(ident.clone(), pipe);
    }

    Ok(Game::new(pipes))
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
    const MAX_DEPTH: usize = 1000;

    if path.len() > MAX_DEPTH {
        eprintln!("⚠️  Profondeur maximale atteinte ({}) - chemin actuel ({} moves)", MAX_DEPTH, path.len());
        eprintln!("Derniers mouvements: {:?}", path.iter().skip(path.len().saturating_sub(10)).collect::<Vec<_>>());
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

            path.push(format!("{}->{}", src, dst));

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

fn optimize(path: Vec<String>) -> Vec<String> {
    return path;
    let mut result = Vec::new();
    let arrow = "->";

    for item in path {
        if let Some(previous) = result.last() {
            let splitted: Vec<&str> = item.split(arrow).collect();
            if splitted.len() == 2 {
                let inversed_current = format!("{}{}{}", splitted[1], arrow, splitted[0]);
                if *previous == inversed_current {
                    result.pop();
                }
            }
        }
        result.push(item);
    }

    result
}
