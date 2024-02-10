use robotics_lib::world::tile::{Tile, TileType};
use std::collections::{VecDeque, HashSet};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Point {
    x: usize,
    y: usize,
}

// Implementa BFS per trovare il percorso più breve considerando gli ostacoli
fn bfs_shortest_path(start: Point, target: TileType, grid: &Vec<Vec<Option<Tile>>>) -> Option<(Vec<Point>, usize)> {
    let mut queue = VecDeque::new();
    let mut visited = HashSet::new();
    let directions = [(0, 1), (0, -1), (1, 0), (-1, 0)]; // Spostamenti possibili: su, giù, destra, sinistra

    queue.push_back((start, Vec::new())); // Inizializza la coda con il punto di partenza e un percorso vuoto

    while let Some((current, path)) = queue.pop_front() {
        if visited.contains(&current) {
            continue;
        }
        visited.insert(current);

        if let Some(tile) = grid.get(current.y).and_then(|row| row.get(current.x)) {
            match tile {
                Some(Tile { tile_type, .. }) if *tile_type == target => {
                    let mut final_path = path.clone();
                    final_path.push(current);
                    return Some((final_path, final_path.len() - 1)); // Ritorna il percorso e la lunghezza
                },
                Some(_) | None => { // Continua la ricerca se è un tile visitabile o non definito
                    for &(dx, dy) in &directions {
                        let next_x = current.x as isize + dx;
                        let next_y = current.y as isize + dy;
                        if next_x >= 0 && next_y >= 0 {
                            let next_point = Point { x: next_x as usize, y: next_y as usize };
                            let mut next_path = path.clone();
                            next_path.push(current);
                            queue.push_back((next_point, next_path));
                        }
                    }
                },
                _ => continue, // Salta se è un tipo di tile non attraversabile
            }
        }
    }

    None // Nessun percorso trovato
}

pub fn nearest_tp(point: (usize, usize), visited: Option<Vec<Vec<Option<Tile>>>>) -> Option<(Vec<Point>, usize)> {
    if let Some(grid) = visited {
        let start = Point { x: point.0, y: point.1 };
        let target = TileType::Teleport(true); // Definisce il tipo di tile target come un teleport non visitato

        // Esegue la ricerca BFS dal punto di partenza alla ricerca del teleport più vicino
        bfs_shortest_path(start, target, &grid)
    } else {
        None
    }
}