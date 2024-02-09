The `get_discovered_tiles_stats` function calculates statistics for tiles discovered in a world map, based on a list of visited tiles. It returns a `HashMap` associating each tile type (`TileType`) with its discovery count. Here's a detailed overview:

- `world`: A 2D array representing the world map, where each element is a `Tile` object.
- `visited_tiles`: A list of identifiers (coordinates) for tiles that have been visited.

As the function iterates through `visited_tiles`, it examines each corresponding tile in `world` to identify its type. It maintains a `HashMap` (`discovered_tiles_count`) to track the number of times each tile type has been discovered. This map is updated dynamically, incrementing the count for each tile type as it processes the list of visited tiles.

The main objective is to provide an exhaustive summary of the types and distribution of tiles explored within the game world. The function returns the `HashMap`, offering a detailed account of each tile type's discovery count up to the current point in the game.
