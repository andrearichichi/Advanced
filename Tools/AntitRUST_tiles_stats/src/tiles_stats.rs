use std::collections::HashMap;
use robotics_lib::world::tile::TileType;

// The `tiles_stats` function provides statistics on all the tiles that have been discovered so far
// in a world map (`world`), considering the tiles already visited (`visited_tiles`). It returns a single value:
// A `HashMap` that maps each tile type (`TileType`) to the number of times that type has been discovered.
//
// Detailed explanation:
// - `world` is a map of the world, where each block is represented by a `Tile` object, and the data structure is a two-dimensional array of tiles.
// - `visited_tiles` is a list of coordinates (or some form of identifier) for tiles that have already been visited.
//
// The function iterates through the `visited_tiles` list, examining each visited tile in the `world` map to determine its type.
// It then updates a `HashMap` called `discovered_tiles_count`, which tracks the count of discovered tiles by type.
//
// This `HashMap` is dynamically updated as the function processes each visited tile, incrementing the count for the respective tile type.
// The primary goal is to compile a comprehensive overview of the variety and distribution of tile types that have been explored within the game world.
//
// The function ultimately returns the `HashMap`, providing a detailed count of each tile type that has been discovered up to the current point in the game.



pub fn discovered_tiles_stats(
    visited_tiles: &Option<Vec<Vec<TileType>>>,
) -> HashMap<TileType, f64> {
    match visited_tiles {
        None => HashMap::new(),
        Some(tiles) => {
            let mut tile_counts = HashMap::new();
            let mut total_tiles = 0.0;

            // Conta le occorrenze di ogni TileType
            for row in tiles {
                for tile in row {
                    *tile_counts.entry(tile.clone()).or_insert(0) += 1;
                    total_tiles += 1.0;
                }
            }

            // Calcola le percentuali e le inserisce in discovered_tiles_count
            let mut discovered_tiles_count = HashMap::new();
            for (tile, count) in tile_counts {
                let percentage = (count as f64 / total_tiles) * 100.0;
                discovered_tiles_count.insert(tile, percentage);
            }

            discovered_tiles_count
        }
    }
}




