use std::cmp::Ordering;
use std::collections::BinaryHeap;
use robotics_lib::runner::Runnable;
use robotics_lib::world::World;
use robotics_lib::world::tile::{Content, Tile, TileType};
use robotics_lib::interface::*;
use robotics_lib::utils::{calculate_cost_go_with_environment, go_allowed, LibError};




pub fn nearest_tile_type(robot: &impl Runnable, world: &mut World, tile_type: TileType, no_content_tile: bool) -> Option<Vec<Direction>> {
    let (_, (robot_x, robot_y)) = where_am_i(robot, &world);
    let map = robot_map(world).expect("Errore nella mappa");
    let mut costs: Vec<Vec<Option<(Option<(usize, usize)>, u32)>>> = vec![vec![None; map.len()]; map.len()];
    calc_cost(robot_x, robot_y, &map, &mut costs, world);

    let movements: Vec<(i32, i32)> = vec![(1, 0), (-1, 0), (0, 1), (0, -1)];
    let mut min_cost: Option<(Option<(usize, usize)>, u32)> = None;
    let mut min_pos: Option<(usize, usize)> = None;

    // Find the tile of the specified type with the minimum cost
    for x in 0..map.len() {
        for y in 0..map.len() {
            match &map[x][y] {
                None => {}
                Some(tile) => {
                    if tile.tile_type == tile_type && (!no_content_tile || tile.content == Content::None) {
                        match costs[x][y] {
                            None => {}
                            Some((_, cost)) => {
                                match min_cost {
                                    None => {
                                        min_cost = Some((Some((x, y)), cost));
                                        min_pos = Some((x, y));
                                    }
                                    Some((_, min_cost_val)) => {
                                        if cost < min_cost_val {
                                            min_cost = Some((Some((x, y)), cost));
                                            min_pos = Some((x, y));
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }


    // If a teleport position is found, calculate the path to that position
    if let Some(destination) = min_pos {
        let directions = get_directions_to_teleport((robot_x, robot_y), destination, &costs);
        Some(directions)
    } else {
        None
    }
}


pub fn get_directions_to_tile(start: (usize, usize), destination: (usize, usize), costs: &Vec<Vec<Option<(Option<(usize, usize)>, u32)>>>) -> Vec<Direction> {
    let mut path = Vec::new();
    let mut current_pos = destination;

    // Ricostruisci il percorso a ritroso dal tile alla posizione iniziale del robot
    while current_pos != start {
        if let Some(cost_entry) = &costs[current_pos.0][current_pos.1] {
            if let Some(previous) = cost_entry.0 {
                path.push(current_pos);
                current_pos = previous;
            } else {
                break; // Se non c'Ã¨ un predecessore, interrompi il ciclo
            }
        }
    }
    path.push(start); // Aggiungi la posizione iniziale al percorso
    path.reverse(); // Inverti il percorso per avere l'ordine corretto da start a destination

    // Converti il percorso in direzioni
    let mut directions = Vec::new();
    for i in 1..path.len() {
        let (prev_x, prev_y) = path[i - 1];
        let (next_x, next_y) = path[i];
        let dir = match (next_x as isize - prev_x as isize, next_y as isize - prev_y as isize) {
            (0, 1) => Direction::Right,
            (0, -1) => Direction::Left,
            (1, 0) => Direction::Down,
            (-1, 0) => Direction::Up,
            _ => continue, // Ignora movimenti non validi (non dovrebbero presentarsi)
        };
        directions.push(dir);
    }

    directions
}

fn calc_cost(rob_x: usize, rob_y: usize, map: &Vec<Vec<Option<Tile>>>, costs: &mut Vec<Vec<Option<(Option<(usize, usize)>, u32)>>>, world: &World) {
    let mut pq = BinaryHeap::new();
    let movements = vec![(1i32, 0i32), (-1, 0), (0, 1), (0, -1)];

    // Insert elements into the heap.
    pq.push(DijkstraItem { distance: 0, coord: (rob_x, rob_y) });
    while let Some(curr) = pq.pop() {
        let (x, y) = curr.coord;
        let curr_dist = curr.distance;

        for m in &movements {
            if x as i32 + m.0 >= 0 && y as i32 + m.1 >= 0 && x as i32 + m.0 < costs.len() as i32 && y as i32 + m.1 < costs.len() as i32 {
                let nx = (x as i32 + m.0) as usize;
                let ny = (y as i32 + m.1) as usize;
                match &map[nx][ny] {
                    None => {
                        let mut base_cost: usize = 30;
                        let mut elevation_cost = 0;

                        // Get informations that influence the cost
                        let environmental_conditions = look_at_sky(world);

                        base_cost = calculate_cost_go_with_environment(base_cost, environmental_conditions, TileType::Mountain);

                        let nc = curr_dist + base_cost as u32;
                        if nc < costs[nx][ny].clone().unwrap_or((None, u32::MAX)).1 {
                            pq.push(DijkstraItem { distance: nc, coord: (nx, ny) });
                            costs[nx][ny] = Some((Some((x, y)), nc));
                        }
                    }
                    Some(tile) => {
                        if tile.tile_type.properties().walk() {
                            // let cost = TileType::properties(&tile.tile_type).cost() as u32;
                            // Init costs
                            // I guess I should have participated more in the development, that way I would have avoided this copy and paste from the main lib.
                            let mut base_cost = tile.tile_type.properties().cost();
                            let mut elevation_cost = 0;

                            // Get informations that influence the cost
                            let environmental_conditions = look_at_sky(world);
                            print!("{:?}", tile);
                            print!("{:?}", tile.tile_type);
                            print!("{:?}", tile.elevation);
                            let new_elevation = tile.elevation;
                            println!("porcodio");
                            let location = Some(map[x][y].clone());
                            let mut current_elevation = 0 as usize;
                            if let Some(loc) = location.unwrap() {
                                current_elevation = loc.elevation;
                            }
                            
                            // Calculate cost
                            base_cost = calculate_cost_go_with_environment(base_cost, environmental_conditions, tile.tile_type);
                            // Consider elevation cost only if we are going from a lower tile to a higher tile
                            if new_elevation > current_elevation {
                                elevation_cost = (new_elevation - current_elevation).pow(2);
                            }
                            let move_cost = (base_cost + elevation_cost) as u32;
                            let nc = curr_dist + move_cost;
                            if nc < costs[nx][ny].clone().unwrap_or((None, u32::MAX)).1 {
                                pq.push(DijkstraItem { distance: nc, coord: (nx, ny) });
                                costs[nx][ny] = Some((Some((x, y)), nc));
                            }
                        }
                    }
                };
            }
        }
    }
}


pub fn get_directions_to_teleport(start: (usize, usize), destination: (usize, usize), costs: &Vec<Vec<Option<(Option<(usize, usize)>, u32)>>>) -> Vec<Direction> {
    let mut path = Vec::new();
    let mut current_pos = destination;

    // Ricostruisci il percorso a ritroso dal teleport alla posizione iniziale del robot
    while current_pos != start {
        if let Some(cost_entry) = &costs[current_pos.0][current_pos.1] {
            if let Some(previous) = cost_entry.0 {
                path.push(current_pos);
                current_pos = previous;
            } else {
                break; // Se non c'è un predecessore, interrompi il ciclo
            }
        }
    }
    path.push(start); // Aggiungi la posizione iniziale al percorso
    path.reverse(); // Inverti il percorso per avere l'ordine corretto da start a destination

    // Converti il percorso in direzioni
    let mut directions = Vec::new();
    for i in 1..path.len() {
        let (prev_x, prev_y) = path[i - 1];
        let (next_x, next_y) = path[i];
        let dir = match (next_x as isize - prev_x as isize, next_y as isize - prev_y as isize) {
            (0, 1) => Direction::Right,
            (0, -1) => Direction::Left,
            (1, 0) => Direction::Down,
            (-1, 0) => Direction::Up,
            _ => continue, // Ignora movimenti non validi (non dovrebbero presentarsi)
        };
        directions.push(dir);
    }

    directions
}


#[derive(Eq, PartialEq)]
struct DijkstraItem {
    distance: u32,
    coord: (usize, usize),
}

// Implement the Ord trait in reverse order to create a min-heap.
impl Ord for DijkstraItem {
    fn cmp(&self, other: &Self) -> Ordering {
        // Reverse the order to make it a min-heap
        other.distance.cmp(&self.distance)
    }
}

// PartialOrd must be implemented as well.
impl PartialOrd for DijkstraItem {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}