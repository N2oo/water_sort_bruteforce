//! Driving adapter n°1: the command line.
//!
//! It reads a board from a JSON file and hands it to the very same solver the
//! HTTP API uses. Pipes are identified by UUID inside the hexagon, so moves are
//! translated back to their human readable labels before being printed.

use std::error::Error;
use std::fs::File;
use std::path::Path;
use std::process::ExitCode;

use water_sort_core::{Move, solve_puzzle_by_bruteforce};
use water_sort_format::{Board, BoardDocument};

const USAGE: &str = "Utilisation: water-sort <fichier_json> [--json]";

fn main() -> ExitCode {
    let mut json_output = false;
    let mut json_path: Option<String> = None;

    for argument in std::env::args().skip(1) {
        match argument.as_str() {
            "--json" => json_output = true,
            "-h" | "--help" => {
                println!("{USAGE}");
                return ExitCode::SUCCESS;
            }
            _ => json_path = Some(argument),
        }
    }

    let json_path = json_path.unwrap_or_else(|| "level.json".to_string());

    let board = match load_board_from_json(Path::new(&json_path)) {
        Ok(board) => board,
        Err(err) => {
            eprintln!("Erreur lors du chargement du JSON '{}': {}", json_path, err);
            eprintln!("{USAGE}");
            return ExitCode::FAILURE;
        }
    };

    let puzzle = match board.to_puzzle() {
        Ok(puzzle) => puzzle,
        Err(err) => {
            eprintln!("Erreur lors du chargement du JSON '{}': {}", json_path, err);
            return ExitCode::FAILURE;
        }
    };

    let solution = solve_puzzle_by_bruteforce(&puzzle.to_game())
        .map(|moves| translate(&board, &moves));

    match solution {
        Some(moves) if json_output => {
            let report = serde_json::json!({
                "solved": true,
                "moves": moves.iter().map(Move::to_string).collect::<Vec<_>>(),
            });
            println!("{}", serde_json::to_string_pretty(&report).unwrap());
            ExitCode::SUCCESS
        }
        Some(moves) => {
            println!("✓ Solution trouvée en {} mouvements !\n", moves.len());
            for (i, movement) in moves.iter().enumerate() {
                println!("  Mouvement {} : {}", i + 1, movement);
            }
            ExitCode::SUCCESS
        }
        None if json_output => {
            println!("{}", serde_json::json!({"solved": false, "moves": []}));
            ExitCode::FAILURE
        }
        None => {
            println!("✗ Aucune solution trouvée");
            ExitCode::FAILURE
        }
    }
}

fn load_board_from_json(path: &Path) -> Result<Board, Box<dyn Error>> {
    let file = File::open(path)?;
    let document: BoardDocument = serde_json::from_reader(file)?;
    Ok(document.into_board()?)
}

/// Turn the solver's UUID based moves back into label based ones.
fn translate(board: &Board, moves: &[String]) -> Vec<Move> {
    moves
        .iter()
        .filter_map(|raw| Move::parse(raw))
        .map(|movement| {
            let label = |identifier: &str| {
                identifier
                    .parse()
                    .ok()
                    .and_then(|id| board.label_of(id))
                    .unwrap_or(identifier)
                    .to_string()
            };
            Move::new(label(&movement.from), label(&movement.to))
        })
        .collect()
}
