// Importa librerie
use robotics_lib::world::tile::{Tile, TileType};
use std::collections::{VecDeque, HashSet};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Point {
    x: usize,
    y: usize,
}

mod private {
    use super::*;
    use std::collections::{VecDeque, HashSet};

    pub fn bfs_shortest_path(start: Point, target: TileType, grid: &Vec<Vec<Option<Tile>>>) -> Option<(Vec<Point>, usize)> {
        let mut queue = VecDeque::new();
        let mut visited = HashSet::new();
        let directions = [(0, 1), (0, -1), (1, 0), (-1, 0)];

        queue.push_back((start, Vec::new()));

        while let Some((current, path)) = queue.pop_front() {
            if visited.contains(&current) {
                continue;
            }
            visited.insert(current);

            if let Some(tile) = grid.get(current.y).and_then(|row| row.get(current.x)) {
                match tile {
                    Some(Tile { tile_type, .. }) if tile_type == &target => { // Corretto qui
                        let mut final_path = path.clone();
                        final_path.push(current);
                        return Some((final_path.clone(), final_path.len() - 1));
                    },
                    Some(_) | None => {
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
                    _ => continue,
                }
            }
        }

        None
    }
}

pub fn nearest_tp(point: (usize, usize), visited: Option<Vec<Vec<Option<Tile>>>>) -> Option<(Vec<Point>, usize)> {
    if let Some(grid) = visited {
        let start = Point { x: point.0, y: point.1 };
        let target = TileType::Teleport(true);

        let (path, distance) = private::bfs_shortest_path(start, target, &grid)?;

        Some((path, distance))
    } else {
        None
    }
}
