use std::collections::HashMap;

use robotics_lib::world::{tile::{Tile, TileType}, world_generator::get_tiletype_percentage};




/// Represents the Tools
/// The `Tools` trait is used to define the Tools.
///
/// # Usage
/// ```rust
/// use robotics_lib::interface::Tools;
/// ```
///
/// # Example
/// ```rust
/// use robotics_lib::interface::Tools;
///
/// struct Tool;
/// impl Tools for Tool {};
/// ```

pub trait Tools {}

//* STATS based on discovered tails

//* */ La funzione `get_missing_tiles_count` sembra calcolare il numero di tile mancanti in base a una mappa del mondo (`world`), considerando anche le tile già visitate (`visited_tiles`). Restituisce un tuple contenente due valori:

//* */ 1. Una `HashMap` che associa a ciascun tipo di tile (`TileType`) il numero di tile mancanti di quel tipo.
//* */ 2. Una stima del costo medio delle tile mancanti in base al costo associato a ciascun tipo di tile. Questa stima è calcolata dividendo la somma dei costi delle tile mancanti per il numero totale di tile mancanti.

//* */ Ecco una spiegazione più dettagliata:

//* */ - `total_blocks`: rappresenta il numero totale di blocchi nel mondo.
//* */ - `world`: è una mappa del mondo, dove ogni blocco è rappresentato da un oggetto `Tile` e la struttura dati è una matrice bidimensionale di tile.
//* */ - `visited_tiles`: è una lista di tile che sono già state visitate.

//* */ La funzione inizia calcolando le percentuali di ciascun tipo di tile nel mondo chiamando la funzione `get_tiletype_percentage`. Questo può essere un metodo che hai implementato altrove nel codice.

//* */ Successivamente, inizia un ciclo che itera su ciascun tipo di tile presente nella mappa (`tile_percentages`). Per ogni tipo di tile, calcola il conteggio atteso sulla base della percentuale fornita, confronta questo conteggio con il numero effettivo di tile di quel tipo già visitate e calcola il numero di tile mancanti.

//* */ La funzione tiene traccia di questi valori in una `HashMap` chiamata `missing_tiles_count`, che associa ogni tipo di tile al numero di tile mancanti di quel tipo.

//* */ Infine, la funzione esegue un calcolo complessivo per stimare il costo medio delle tile mancanti. Questo calcolo esclude i tipi di tile specifici (`Lava`, `DeepWater`, e `Wall`) dal conteggio e dal calcolo del costo medio. La stima del costo medio è ottenuta dividendo la somma dei costi delle tile mancanti per il numero totale di tile mancanti.

//* */ La restituzione della funzione è quindi una tupla contenente la `HashMap` con il conteggio dei tile mancanti per tipo e la stima del costo medio delle tile mancanti.



pub fn tiles_count(
    total_blocks: usize,
    world: Vec<Vec<Tile>>,
    visited_tiles: &Vec<Tile>,
) -> (HashMap<TileType, usize>, f64) {
    let tile_percentages: HashMap<TileType, f64> = get_tiletype_percentage(&world);
    let mut missing_tiles_count = HashMap::new();
    let mut sum = 0;
    let mut count = 0;

    for (tile_type, &percentage) in tile_percentages.iter() {
        let expected_count = (percentage * total_blocks as f64) as usize;

        let visited_count = visited_tiles
            .iter()
            .filter(|&tile| tile.tile_type == *tile_type)
            .count();

        let missing_count = expected_count - visited_count;
        missing_tiles_count.insert(*tile_type, missing_count);
        
        if tile_type != &TileType::Lava && tile_type != &TileType::DeepWater && tile_type != &TileType::Wall {
            count +=missing_count;
            sum += missing_count*tile_type.properties().cost();
        } 

    }
    (missing_tiles_count, sum as f64/count as f64)
}


