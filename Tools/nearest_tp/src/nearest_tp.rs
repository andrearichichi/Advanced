use robotics_lib::world::{self, tile::{Tile, TileType}};

// The `nearest_tp` function in Rust is designed to find the nearest unvisited point from a given location within an explored world, represented by a two-dimensional array of optional `Tile` states (visited or unvisited). It accepts two parameters: the starting point as `(usize, usize)` coordinates, and an optional grid of `Option<Tile>` indicating visited status.
// Here's how it works:
// 1. **Initialization**: Checks if the grid of visited places exists (`Some`) and initializes variables to track the nearest unvisited point and the minimum distance, initially set to the maximum `usize` value.
// 2. **Grid Iteration**: Iterates through each row and column of the grid. For each position (`tile`), it checks if it's marked as unvisited (`Tile::Unvisited`).
// 3. **Distance Calculation**: For each unvisited tile, calculates the Manhattan distance from the given location to the current grid position. If this distance is less than the current `min_distance`, updates `min_distance` and stores the coordinates as `nearest_point`.
// 4. **Result**: If an unvisited point is found, returns an `Option` containing a pair: a vector with the nearest unvisited point and the minimum distance to it. Returns `None` if no unvisited points are found or if the grid is not provided.
// This algorithm efficiently maps the explored area around a given point, quickly identifying the shortest path to the nearest unvisited point, useful in navigation or map exploration contexts in games or simulation applications.

struct Point {
    x: usize,
    y: usize,
}

pub fn nearest_tp(point:(usize,usize), visited: Option<Vec<Vec<Option<Tile>>>>)-> Option<(Vec<Point>, usize)>{
    use std::collections::HashSet;

    // Definizione di un punto con coordinate (x, y)
    

    fn distance(p1: Point, p2: Point) -> usize {
        (p1.x as isize - p2.x as isize).abs() as usize + (p1.y as isize - p2.y as isize).abs() as usize
    }

    // Funzione per calcolare la distanza minima tra due punti nel vettore
    fn min_distance(v: &[(usize, usize)], start: Point, end: Point) -> Option<Path> {
        // Converto il vettore di tuple in un set di punti
        let points: HashSet<Point> = v.iter().map(|&(x, y)| Point { x, y }).collect();

        // Inizializzo il percorso minimo a None
        let mut min_path = None;

        // Funzione ricorsiva per esplorare tutte le possibili mosse e trovare il percorso minimo
        fn explore(
            current_point: Point,
            end_point: Point,
            visited: &mut HashSet<Point>,
            current_path: &mut Path,
            min_path: &mut Option<Path>,
            points: &HashSet<Point>,
        ) {
            // Se la distanza attuale è maggiore della distanza minima trovata finora, termino la ricorsione
            if let Some(min) = min_path.as_ref().map(|p| p.distance) {
                if current_path.distance >= min {
                    return;
                }
            }

            // Se ho raggiunto la destinazione, aggiorno il percorso minimo
            if current_point == end_point {
                *min_path = Some(current_path.clone());
                return;
            }

            // Aggiungo il punto attuale ai punti visitati
            visited.insert(current_point);

            // Definisco le possibili direzioni di movimento (sopra, sotto, sinistra, destra)
            let directions: Vec<(isize, isize)> = vec![(0, 1), (0, -1), (1, 0), (-1, 0)];

            // Esploro tutte le possibili mosse
            for &(dx, dy) in &directions {
                let new_point = Point {
                    x: (current_point.x as isize + dx) as usize,
                    y: (current_point.y as isize + dy) as usize,
                };

                // Se il nuovo punto non è stato visitato e fa parte del vettore, continuo la ricorsione
                if !visited.contains(&new_point) && points.contains(&new_point) {
                    // Aggiorno il percorso attuale
                    current_path.points.push(new_point);
                    current_path.distance += distance(current_point, new_point);

                    // Ricorsione per il nuovo punto
                    explore(
                        new_point,
                        end_point,
                        visited,
                        current_path,
                        min_path,
                        points,
                    );

                    // Ripristino lo stato del percorso per il backtracking
                    current_path.points.pop();
                    current_path.distance -= distance(current_point, new_point);
                }
            }

            // Rimuovo il punto attuale dai punti visitati prima di tornare indietro nella ricorsione
            visited.remove(&current_point);
        }

        // Inizio l'esplorazione dalla posizione di partenza
        let mut start_path = Path {
            distance: 0,
            points: vec![start],
        };

        explore(
            start,
            end,
            &mut HashSet::new(),
            &mut start_path,
            &mut min_path,
            &points,
        );

        // Restituisco il percorso minimo trovato (o None se non è possibile raggiungere la destinazione)
        min_path
    }


    let tiles = visited.as_ref().map_or_else(|| vec![], |v| v.iter().cloned().flatten().collect::<Vec<_>>());

    let locked_plot = PLOT.lock().unwrap();
    let coordinates = locked_plot.clone();

    let result: Vec<_> = coordinates.iter().zip(tiles).map(|(&(x, y), tile)| {
        (x, y, tile)
    }).collect();
    let simplified_coordinates: Vec<_> = coordinates.iter().map(|&(x, y)| (x, y)).collect();

    let simplified_coordinates_ref: &[(usize, usize)] = &simplified_coordinates;

    let min=result.len();
    let mut path_min:Path=Path { distance: result.len(), points: vec![Point{x: 0, y:0}] }; 
    let confront:Path=Path { distance: result.len(), points: vec![Point{x: 0, y:0}] }; 
    for (end_x, end_y, tile) in &result {
        // Verifica se il tile contiene `tp`
        if tile.clone().unwrap().tile_type == TileType::Teleport(true) {
            // Calcola la distanza minima tra il punto di partenza e la destinazione
            if let Some(min_path) = min_distance(simplified_coordinates_ref, Point { x: point.0, y: point.1 }, Point{x: *end_x, y: *end_y}) {
                if min_path.distance<min{
                    path_min = min_path.clone();
                }
                println!("Percorso: {:?}", min_path.points.clone());
            } else {
                println!("Impossibile raggiungere la destinazione da ({}, {})", point.0, point.1);
            }
        }
    }
    if path_min!= confront{
        Some((path_min.points,min))
    }else {
        None
    }
}
