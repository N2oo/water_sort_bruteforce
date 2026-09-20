//! Bruteforce solver.
//!
//! Moved verbatim out of the original `main.rs`: the search strategy, the depth
//! limit and the produced `"SRC->DST"` move list are unchanged, only the
//! visibility of the entry point was widened so every adapter (CLI, HTTP) can
//! reuse the very same algorithm.

use std::collections::HashSet;

use crate::game::Game;

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

/// Solve `game`, returning the winning moves as `"SRC->DST"` strings.
pub fn solve_puzzle_by_bruteforce(game: &Game) -> Option<Vec<String>> {
    let mut visited = HashSet::new();
    let mut path = Vec::new();

    if search_bruteforce(game, &mut visited, &mut path) {
        Some(optimize(path))
    } else {
        None
    }
}

#[allow(unreachable_code)]
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
