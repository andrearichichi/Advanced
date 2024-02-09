use robotics_lib::world;

// The `nearest_tp` function in Rust is designed to find the nearest unvisited point from a given location within an explored world, represented by a two-dimensional array of optional `Tile` states (visited or unvisited). It accepts two parameters: the starting point as `(usize, usize)` coordinates, and an optional grid of `Option<Tile>` indicating visited status.
// Here's how it works:
// 1. **Initialization**: Checks if the grid of visited places exists (`Some`) and initializes variables to track the nearest unvisited point and the minimum distance, initially set to the maximum `usize` value.
// 2. **Grid Iteration**: Iterates through each row and column of the grid. For each position (`tile`), it checks if it's marked as unvisited (`Tile::Unvisited`).
// 3. **Distance Calculation**: For each unvisited tile, calculates the Manhattan distance from the given location to the current grid position. If this distance is less than the current `min_distance`, updates `min_distance` and stores the coordinates as `nearest_point`.
// 4. **Result**: If an unvisited point is found, returns an `Option` containing a pair: a vector with the nearest unvisited point and the minimum distance to it. Returns `None` if no unvisited points are found or if the grid is not provided.
// This algorithm efficiently maps the explored area around a given point, quickly identifying the shortest path to the nearest unvisited point, useful in navigation or map exploration contexts in games or simulation applications.

fn nearest_tp(
    point: (usize, usize),
    visited: Option<Vec<Vec<Option<Tile>>>>,
) -> Option<(Vec<Point>, usize)> {
    if let Some(grid) = visited {
        let mut nearest_point = None;
        let mut min_distance = usize::MAX;
        let (x, y) = point;

        for (i, row) in grid.iter().enumerate() {
            for (j, tile) in row.iter().enumerate() {
                if let Some(Tile::Unvisited) = tile {
                    let distance = (x as isize - i as isize).abs() + (y as isize - j as isize).abs();
                    if distance < min_distance as isize {
                        min_distance = distance as usize;
                        nearest_point = Some(Point { x: i, y: j });
                    }
                }
            }
        }

        nearest_point.map(|p| (vec![p], min_distance))
    } else {
        None
    }
}
