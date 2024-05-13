use std::cmp::Ordering;
use std::collections::BinaryHeap;

use robotics_lib::runner::Runnable;
use robotics_lib::world::{World, SkyCondition};
use robotics_lib::world::tile::{Tile, TileType};
use robotics_lib::utils::{calculate_cost_go_with_environment, look_at_sky};

/// Represents the robot's action towards the teleportation pad.
#[derive(Clone, Debug)]
pub enum Action {
    MoveToTeleport,
    Arrived,
}

/// Provides utilities to navigate the robot to the nearest teleportation pad.
pub struct TeleportNavigator;

impl TeleportNavigator {
    /// Calculates and returns the next action the robot should take to reach the nearest teleport pad.
    /// Returns None if no teleport pads are reachable.
    pub fn find_next_action(robot: &impl Runnable, world: &World) -> Option<Action> {
        let (_, (robot_x, robot_y)) = robot.where_am_i();
        let map = robot.robot_map().expect("Error in map retrieval");

        // Cost calculation grid initialization
        let mut costs: Vec<Vec<Option<(Option<(usize, usize)>, u32)>>> = vec![vec![None; map.len()]; map.len()];
        Self::calculate_costs(robot_x, robot_y, &map, &mut costs);

        // Determine the nearest teleport position
        if let Some(nearest) = Self::find_nearest_teleport(robot_x, robot_y, &map, &costs) {
            if nearest.0 == (robot_x, robot_y) {
                return Some(Action::Arrived);
            } else {
                return Some(Action::MoveToTeleport);
            }
        }
        None
    }

    /// Calculates the pathfinding costs from the robot's position to all reachable tiles.
    fn calculate_costs(robot_x: usize, robot_y: usize, map: &[Vec<Option<Tile>>], costs: &mut Vec<Vec<Option<(Option<(usize, usize)>, u32)>>>) {
        let movements = vec![(1i32, 0i32), (-1, 0), (0, 1), (0, -1)];
        let mut pq = BinaryHeap::new();

        pq.push(DijkstraItem { distance: 0, coord: (robot_x, robot_y) });
        while let Some(curr) = pq.pop() {
            let (x, y) = curr.coord;
            let curr_dist = curr.distance;

            for m in &movements {
                if x as i32 + m.0 >= 0 && y as i32 + m.1 >= 0 && x as i32 + m.0 < costs.len() as i32 && y as i32 + m.1 < costs.len() as i32 {
                    let nx = (x as i32 + m.0) as usize;
                    let ny = (y as i32 + m.1) as usize;
                    if let Some(tile) = &map[nx][ny] {
                        if tile.tile_type.properties().walk() {
                            let move_cost = Self::calculate_move_cost(&tile, &map[x][y], world);
                            let nc = curr_dist + move_cost;
                            if nc < costs[nx][ny].unwrap_or((None, u32::MAX)).1 {
                                pq.push(DijkstraItem { distance: nc, coord: (nx, ny) });
                                costs[nx][ny] = Some((Some((x, y)), nc));
                            }
                        }
                    }
                }
            }
        }
    }

    /// Finds the nearest teleportation pad from the robot's current position.
    fn find_nearest_teleport(robot_x: usize, robot_y: usize, map: &[Vec<Option<Tile>>], costs: &[Vec<Option<(Option<(usize, usize)>, u32)>>]) -> Option<((usize, usize), u32)> {
        map.iter().enumerate().flat_map(|(x, row)| {
            row.iter().enumerate().filter_map(move |(y, tile)| {
                if let Some(tile) = tile {
                    if tile.tile_type == TileType::Teleport {
                        costs[x][y].map(|cost| ((x, y), cost.1))
                    } else {
                        None
                    }
                } else {
                    None
                }
            })
        }).min_by_key(|&(_, cost)| cost)
    }

    /// Calculates the movement cost based on the tile type, environment, and elevation.
    fn calculate_move_cost(tile: &Tile, current_tile: &Option<Tile>, world: &World) -> u32 {
        let base_cost = tile.tile_type.properties().cost();
        let environmental_conditions = look_at_sky(world);
        let new_elevation = tile.elevation;
        let current_elevation = current_tile.as_ref().map_or(0, |t| t.elevation);

        let elevation_cost = if new_elevation > current_elevation {
            (new_elevation - current_elevation).pow(2)
        } else {
            0
        };

        calculate_cost_go_with_environment(base_cost + elevation_cost, environmental_conditions, tile.tile_type)
    }
}

#[derive(Eq, PartialEq)]
struct DijkstraItem {
    distance: u32,
    coord: (usize, usize),
}

impl Ord for DijkstraItem {
    fn cmp(&self, other: &Self) -> Ordering {
        other.distance.cmp(&self.distance)
    }
}

impl PartialOrd for DijkstraItem {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
