use std::collections::HashMap;
use robotics_lib::world::tile::TileType;
use robotics_lib::world::tile::Tile;

pub fn discovered_tiles_stats(
    visited_tiles: &Option<Vec<Vec<Option<Tile>>>>,
) -> HashMap<TileType, f64> {
    match visited_tiles {
        None => HashMap::new(),
        Some(tiles) => {
            let mut tile_counts = HashMap::new();
            let mut total_tiles = 0.0;

            // Count occurrences of each TileType
            for row in tiles {
                for tile_option in row {
                    if let Some(tile) = tile_option {
                        *tile_counts.entry(tile.tile_type.clone()).or_insert(0) += 1;
                        total_tiles += 1.0;
                    }
                }
            }

            // Calculate percentages and insert into discovered_tiles_count
            let mut discovered_tiles_count = HashMap::new();
            for (tile_type, count) in tile_counts {
                let percentage = ((count as f64 / total_tiles) * 100.0 * 10.0).round() / 10.0;
                discovered_tiles_count.insert(tile_type, percentage);
            }

            discovered_tiles_count
        }
    }
}
