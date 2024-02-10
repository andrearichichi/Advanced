 **## discovered_tiles_stats.rs**

**Purpose:**

- Calculates statistics about the types of tiles that have been discovered in a world map.
- Provides information on the distribution of discovered tile types, aiding in understanding the world's composition.

**Function:**

- Analyzes a list of visited tiles and their types.
- Returns a `HashMap` containing:
    - Keys: Discovered tile types (`TileType` enum).
    - Values: Percentage of each tile type's occurrence among visited tiles.

**Usage:**

```rust
let visited_tiles = vec![TileType::Grass, TileType::Water, TileType::Grass];
let tile_stats = discovered_tiles_stats(&visited_tiles);

// Example output: { TileType::Grass: 66.67, TileType::Water: 33.33 }
```

**Key Variables:**

- `visited_tiles`: A vector containing the types of tiles that have been visited.
- `tile_counts`: A temporary HashMap to count occurrences of each tile type.
- `total_tiles`: The total number of visited tiles.
- `discovered_tiles_count`: The final HashMap storing tile types and their percentages.

**Steps:**

1. Initializes `tile_counts` and `total_tiles`.
2. Iterates through `visited_tiles`:
    - Increments the count for each tile type in `tile_counts`.
3. Calculates percentages for each tile type in `tile_counts`.
4. Inserts percentage values into `discovered_tiles_count`.
5. Returns `discovered_tiles_count`.

**Additional Notes:**

- Necessary imports: `std::collections::HashMap` and `robotics_lib::world::tile::TileType`.
- Assumes a `world` map data structure (not explicitly shown in this code snippet).

**Example:**

```rust
// Example usage in a game context:
let player_position = (10, 20);
let visited_tiles = get_visited_tiles(&world, player_position);
let tile_stats = discovered_tiles_stats(&visited_tiles);

println!("Discovered tile percentages: {:?}", tile_stats);
```
