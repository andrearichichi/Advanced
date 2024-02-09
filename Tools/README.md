# TILES STATS 📊📈🎯 
## tiles_stats Function Overview

- **Purpose**: Calculates statistics for tiles discovered on a world map, based on visited tiles.
- **Algorithm**: Iterates through `visited_tiles`, updating a `HashMap` (`discovered_tiles_count`) with the count of each `TileType` discovered.
- **Return Value**: A `HashMap` associating each `TileType` with its discovery count.


<img src="img/tiles_stats.webp" width="80%">

---

## NEAREST TP
- **Purpose**: Finds the cheapest path from a starting point to a "Teleport(true)" destination.
- **Algorithm**: Dijkstra algorithm for optimal pathfinding.
- **Return Value**: Returns the minimum path and distance in an option, or `None` if unreachable.

<img src="img/nearest_tp.webp" width="80%">
