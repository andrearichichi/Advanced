use bevy::prelude::*;
use robotics_lib::{interface, world::{tile::{self, Tile, TileType}, world_generator::Generator}};

// Funzione per convertire un numero da 1 a 5 in un colore
fn get_color(tile: Tile) -> Color {
    match tile.tile_type {
        TileType::DeepWater => Color::rgb_u8(0x00, 0x00, 0x80), // Blu Scuro
        TileType::Grass => Color::rgb_u8(0x00, 0xFF, 0x00),     // Verde Vivo
        TileType::Hill => Color::rgb_u8(0x7C, 0xFC, 0x00),      // Verde Chiaro
        TileType::Lava => Color::rgb_u8(0xFF, 0x45, 0x00),      // Arancione-Rosso
        TileType::Mountain => Color::rgb_u8(0x8B, 0x45, 0x13),  // Marrone
        TileType::Sand => Color::rgb_u8(0xFF, 0xF0, 0x80),    // Sabbia
        TileType::ShallowWater => Color::rgb_u8(0x00, 0xFF, 0xFF), // Azzurro
        TileType::Snow => Color::rgb_u8(0xFF, 0xFF, 0xFF),      // Bianco
        TileType::Street => Color::rgb_u8(0x69, 0x69, 0x69),    // Grigio Scuro
        TileType::Teleport(_) => Color::rgb_u8(0x94, 0x00, 0xD3), // Viola
        TileType::Wall => Color::rgb_u8(0x8B, 0x00, 0x00),      // Rosso Scuro
    }
}


// Funzione di setup che crea la scena
fn setup(mut commands: Commands) {
    // Matrice di esempio
    let matrice = vec![
        vec![1, 2, 3, 4, 5],
        vec![5, 4, 3, 2, 1],
        vec![1, 2, 1, 4, 5],
        vec![1, 2, 3, 4, 5],
        vec![5, 4, 1, 2, 1],
        ];
    let mut world_gen = ghost_amazeing_island::world_generator::WorldGenerator::new(1000, false, 1, 1.1);
    let mut interface =world_gen.gen().0;
    let square_size = 3.0; // Dimensione di ogni quadrato
    let spacing = 3.0; // Spaziatura tra i quadrati
    commands.spawn(Camera2dBundle::default());


    for (y, row) in interface.iter().enumerate() {
        for (x, tile) in row.iter().enumerate() {
            let color = get_color(tile.clone());
            commands.spawn(SpriteBundle {
                sprite: Sprite {
                    color, // Imposta il colore direttamente qui
                    custom_size: Some(Vec2::new(square_size, square_size)),
                    ..Default::default()
                },
                transform: Transform::from_xyz(
                    x as f32 * spacing - 10.0, // Posizione X con un offset
                    y as f32 * spacing - 10.0, // Posizione Y con un offset
                    0.0,
                ),
                ..Default::default()
            });
        }
    }
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_startup_system(setup)
        .run();
}
