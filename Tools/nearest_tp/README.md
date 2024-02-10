 **README.md**

## nearest_tp.rs

**Funzionalità:**

- Trova il teleport non visitato più vicino a un punto di partenza in una griglia.
- Calcola il percorso più breve verso il teleport utilizzando l'algoritmo Breadth-First Search (BFS).

**Funzioni:**

- **nearest_tp(point, visited):**
    - Restituisce il percorso e la distanza del teleport più vicino, o `None` se non trovato.
    - Parametri:
        - `point`: coordinate del punto di partenza (x, y).
        - `visited`: griglia con informazioni di visita (opzionale).
- **private::bfs_shortest_path(start, target, grid):**
    - Implementazione privata del BFS per trovare il percorso più breve.
    - Non accessibile direttamente dall'esterno del modulo.

**Dipendenze:**

- `robotics_lib::world::tile::{Tile, TileType}`
- `std::collections::{VecDeque, HashSet}`

**Esempio d'uso:**

```rust
let grid = ...; // Definire la griglia
let start = (0, 0); // Punto di partenza

let (path, distance) = nearest_tp(start, Some(grid)).unwrap();

println!("Percorso al teleport: {:?}", path);
println!("Lunghezza: {}", distance);
```
