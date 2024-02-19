//TODO:
//SISTEMARE POSITIONING/CONTROLLO MATRICE
//MINIMAPPA
//BOTTONE ZOOM MAPPA
//INSERIRE TUTTE LE INFO(TEMPO, ENERGIA, COSTO, ECC..)
//DESPAWN CONTENT QUANDO SI PRENDONO/DISTRUGGONO
//CREAZIONE BACKPACK/INSERIMENTO VISUALIZZA OGGETTI
//MODIFICA TILE QUANDO SI POSIZIONA LA ROCCIA

use bevy::core_pipeline::clear_color::ClearColorConfig;
use bevy::log;
use bevy::render::camera::Viewport;
use bevy::window::PrimaryWindow;
use bevy::window::WindowMode;
use bevy::window::WindowResized;
use bevy::{app::AppExit, prelude::*, render::view::RenderLayers};
use op_map::op_pathfinding::{
    get_best_action_to_element, OpActionInput, OpActionOutput, ShoppingList,
};
use rand::Rng;
use robotics_lib::world::coordinates;
use robotics_lib::{
    interface::{destroy, discover_tiles, go, put, Direction},
    utils::LibError,
    world::{
        environmental_conditions::WeatherType,
        tile::{Content, Tile, TileType},
    },
};
use std::thread::sleep;
use std::{collections::HashMap, ops::Range};

use crab_rave_explorer::algorithm::{cheapest_border, move_to_cheapest_border};
use oxagaudiotool::sound_config::OxAgSoundConfig;

use arrusticini_destroy_zone::DestroyZone;
use ohcrab_weather::weather_tool::WeatherPredictionTool;
use oxagaudiotool::OxAgAudioTool;
use robotics_lib::interface::{look_at_sky, robot_map, robot_view};
use robotics_lib::{
    energy::Energy,
    runner::{backpack::BackPack, Robot, Runnable, Runner},
    world::coordinates::Coordinate,
};

const MIN_ZOOM: f32 = 0.05; // Sostituisci con il valore minimo desiderato
const MAX_ZOOM: f32 = 0.25; //1.0 se 150, 0.25 se 250


const AI: AiLogic = AiLogic::Ricercatore;
const WORLD_SIZE: u32 = 250;
const TILE_SIZE: f32 = 3.0; // Dimensione di ogni quadrato

#[derive(Component, Debug)]
struct Roboto;

//aggiungere sempre le risorse nell'app del main
#[derive(Default, Resource)]
struct RobotPosition {
    x: f32,
    y: f32,
}

#[derive(Resource, Debug, Default)]
struct TileSize {
    tile_size: f32,
}

#[derive(Component, Debug)]
//componente per la mappa zoommata
struct MainCamera;

#[derive(Component, Debug)]
//componente per la minimappa
struct MyMinimapCamera;

//componente per la zona rossa della minimappa
#[derive(Component, Debug)]
struct MinimapOutline;

#[derive(Component, Debug)]
struct TagEnergy;

#[derive(Component, Debug)]
struct TagTime;

#[derive(Component, Debug)]
struct TagBackPack;

#[derive(Component, Debug)]
struct EnergyBar;

#[derive(Component, Debug)]
struct SunTime;

#[derive(Component, Debug)]
struct WeatherIcon;

#[derive(Default, Resource)]
struct WeatherIcons {
    sunny_day: Handle<Image>,
    sunny_night: Handle<Image>,
    foggy_day: Handle<Image>,
    foggy_night: Handle<Image>,
    rainy_day: Handle<Image>,
    rainy_night: Handle<Image>,
    trentino_snow_day: Handle<Image>,
    trentino_snow_night: Handle<Image>,
    tropical_monsoon_day: Handle<Image>,
    tropical_monsoon_night: Handle<Image>,
}

// Funzione per convertire un numero da 1 a 5 in un colore
fn get_color(tile: Tile) -> Color {
    match tile.tile_type {
        TileType::DeepWater => Color::rgb_u8(0x00, 0x00, 0x80), // Blu Scuro
        TileType::Grass => Color::rgb_u8(0x00, 0xFF, 0x00),     // Verde Vivo
        TileType::Hill => Color::rgb_u8(0x7C, 0xFC, 0x00),      // Verde Chiaro
        TileType::Lava => Color::rgb_u8(0xFF, 0x45, 0x00),      // Arancione-Rosso
        TileType::Mountain => Color::rgb_u8(0x8B, 0x45, 0x13),  // Marrone
        TileType::Sand => Color::rgb_u8(0xFF, 0xF0, 0x80),      // Sabbia
        TileType::ShallowWater => Color::rgb_u8(0x00, 0xFF, 0xFF), // Azzurro
        TileType::Snow => Color::rgb_u8(0xFF, 0xFF, 0xFF),      // Bianco
        TileType::Street => Color::rgb_u8(0x69, 0x69, 0x69),    // Grigio Scuro
        TileType::Teleport(_) => Color::rgb_u8(0x94, 0x00, 0xD3), // Viola
        TileType::Wall => Color::rgb_u8(0x8B, 0x00, 0x00),      // Rosso Scuro
    }
}

fn get_content_color(content: Tile) -> Color {
    match content.content {
        Content::Rock(_) => Color::rgb_u8(0xB0, 0xC4, 0xDE), // Light Steel Blue
        Content::Tree(_) => Color::rgb_u8(0x00, 0x64, 0x00), // Dark Green
        Content::Garbage(_) => Color::rgb_u8(0x8B, 0x45, 0x13), // Saddle Brown
        Content::Fire => Color::rgb_u8(0xFF, 0x45, 0x00),    // Orange Red
        Content::Coin(_) => Color::rgb_u8(0xFF, 0xD7, 0x00), // Gold
        Content::Water(_) => Color::rgb_u8(0x87, 0xCE, 0xEB), // Sky Blue
        Content::Bin(_) => Color::rgb_u8(0x70, 0x80, 0x90),  // Slate Gray
        Content::Crate(_) => Color::rgb_u8(0xF5, 0xF5, 0xDC), // Beige
        Content::Bank(_) => Color::rgb_u8(0x85, 0xBB, 0x65), // Dollar Bill
        Content::Market(_) => Color::rgb_u8(0xB2, 0x22, 0x22), // Firebrick
        Content::Fish(_) => Color::rgb_u8(0x00, 0xFF, 0xFF), // Aqua
        Content::Building => Color::rgb_u8(0x80, 0x00, 0x80), // Purple
        Content::Bush(_) => Color::rgb_u8(0x90, 0xEE, 0x90), // Light Green
        Content::JollyBlock(_) => Color::rgb_u8(0xFF, 0xC0, 0xCB), // Pink
        Content::Scarecrow => Color::rgb_u8(0xFF, 0xA5, 0x00), // Orange
        Content::None => Color::NONE,                        // Transparent or keep the tile color
        _ => Color::YELLOW_GREEN, // Fallback color for unspecified contents
    }
}

// Funzione di setup che crea la scena
fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    shared_map: Res<MapResource>,
    robot_resource: Res<RobotResource>,
) {
    //SPRITES
    let weather_icons = WeatherIcons {
        sunny_day: asset_server.load("img/sunny_day.png"),
        sunny_night: asset_server.load("img/sunny_night.png"),
        foggy_day: asset_server.load("img/foggy_day.png"),
        foggy_night: asset_server.load("img/foggy_night.png"),
        rainy_day: asset_server.load("img/rainy_day.png"),
        rainy_night: asset_server.load("img/rainy_night.png"),
        trentino_snow_day: asset_server.load("img/trentino_snow_day.png"),
        trentino_snow_night: asset_server.load("img/trentino_snow_night.png"),
        tropical_monsoon_day: asset_server.load("img/tropical_monsoon_day.png"),
        tropical_monsoon_night: asset_server.load("img/tropical_monsoon_night.png"),
    };

    commands.insert_resource(weather_icons);

    //TODO: controllare se positioning si basa effettivamente sulla matrice
    //TODO: cabiare dimenzione con l'utilizzo della risorsa tile
    //sleep 3 secondi
    sleep(std::time::Duration::from_secs(3));
    let world1 = shared_map.0.lock().unwrap();
    let resource1 = robot_resource.0.lock().unwrap();
    let world = world1.clone();
    let resource = resource1.clone();
    drop(world1);
    drop(resource1);

    let mut count = 0;

    //how many tile is not None
    for row in world.iter() {
        for tile in row.iter() {
            if tile.is_some() {
                count += 1;
            }
        }
    }
    // println!("count mappaa viosualizzaaaaa {:?}", count);
    let mut old_map = OldMapResource {
        //world: vec![vec![None; WORLD_SIZE as usize]; WORLD_SIZE as usize],
        world: vec![vec![None; WORLD_SIZE as usize]; WORLD_SIZE as usize],
    };

    // Dimensione di ogni quadrato
    //sotto funzione per telecamera
    //commands.spawn(Camera2dBundle::default());

    // println!("Robot {:?} {:?}",resource.coordinate_column, resource.coordinate_row);
    update_show_tiles(&world, &mut commands, &mut old_map.world);
    commands.insert_resource(old_map);

    // per la posizione centrale della tile

    let robot_size = 2.0;

    //spawna il robot
    commands
        .spawn(SpriteBundle {
            sprite: Sprite {
                color: Color::RED,
                custom_size: Some(Vec2::new(robot_size, robot_size)),
                ..Default::default()
            },
            transform: Transform::from_xyz(
                TILE_SIZE * resource.coordinate_row as f32,
                TILE_SIZE * resource.coordinate_row as f32,
                2.0,
            ), // asse z serve per metterlo sopra i tile e i conent
            ..Default::default()
        })
        .insert(Roboto)
        .insert(RenderLayers::layer(0));

    //BUTTONS
    commands
        .spawn(NodeBundle {
            style: Style {
                // Imposta le dimensioni del nodo contenitore per occupare l'intera finestra
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                // Allinea i bottoni in alto a sinistra
                align_items: AlignItems::FlexEnd, // Allinea verticalmente i figli all'inizio (alto)
                justify_content: JustifyContent::FlexEnd, // Allinea orizzontalmente i figli all'inizio (sinistra)
                flex_direction: FlexDirection::Row,       // Disponi i figli in orizzontale
                // Aggiungi padding per posizionare i bottoni un po' distanti dal bordo superiore e sinistro
                padding: UiRect {
                    left: Val::Auto,
                    top: Val::Px(10.0),
                    right: Val::Px(50.0),
                    bottom: Val::Px(50.0),
                },
                ..default()
            },
            ..default()
        })
        .with_children(|parent| {
            // Primo bottone
            parent
                .spawn(ButtonBundle {
                    style: Style {
                        width: Val::Px(60.0),
                        height: Val::Px(40.0),
                        margin: UiRect::all(Val::Px(10.0)), // Spazio tra i bottoni
                        border: UiRect::all(Val::Px(4.0)),
                        justify_content: JustifyContent::Center, // Centra orizzontalmente
                        align_items: AlignItems::Center,

                        ..default()
                    },
                    border_color: BorderColor(Color::BLACK),
                    background_color: NORMAL_BUTTON.into(),
                    ..default()
                })
                .with_children(|parent| {
                    parent.spawn(TextBundle::from_section(
                        "-",
                        TextStyle {
                            // Allinea il testo al centro del bottone
                            font_size: 25.0,
                            color: Color::rgb(0.9, 0.9, 0.9),
                            ..default()
                        },
                    ));
                })
                .insert(ZoomIn);

            // Secondo bottone
            parent
                .spawn(ButtonBundle {
                    style: Style {
                        width: Val::Px(60.0),
                        height: Val::Px(40.0),
                        margin: UiRect::all(Val::Px(10.0)), // Spazio tra i bottoni
                        border: UiRect::all(Val::Px(4.0)),
                        justify_content: JustifyContent::Center, // Centra orizzontalmente
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    border_color: BorderColor(Color::BLACK),
                    background_color: NORMAL_BUTTON.into(),
                    ..default()
                })
                .with_children(|parent| {
                    parent.spawn(TextBundle::from_section(
                        "+",
                        TextStyle {
                            font_size: 25.0,
                            color: Color::rgb(0.9, 0.9, 0.9),
                            ..default()
                        },
                    ));
                })
                .insert(ZoomOut);

            //bottone chiusura
            parent
                .spawn(ButtonBundle {
                    style: Style {
                        width: Val::Px(60.0),
                        height: Val::Px(40.0),
                        margin: UiRect::all(Val::Px(10.0)), // Spazio tra i bottoni
                        border: UiRect::all(Val::Px(4.0)),
                        justify_content: JustifyContent::Center, // Centra orizzontalmente
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    border_color: BorderColor(Color::BLACK),
                    background_color: NORMAL_BUTTON.into(),
                    ..default()
                })
                .with_children(|parent| {
                    parent.spawn(TextBundle::from_section(
                        "EXIT",
                        TextStyle {
                            font_size: 25.0,
                            color: Color::rgb(0.9, 0.9, 0.9),
                            ..default()
                        },
                    ));
                })
                .insert(CloseAppButton);
        })
        .insert(RenderLayers::layer(0));

    //menu a tendina parte destra
    commands
        .spawn(NodeBundle {
            style: Style {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),

                justify_content: JustifyContent::FlexEnd,
                ..default()
            },
            ..default()
        })
        .with_children(|parent| {
            parent
                .spawn(ButtonBundle {
                    style: Style {
                        width: Val::Px(20.0),
                        height: Val::Px(65.0),
                        border: UiRect::all(Val::Px(5.0)),
                        // horizontally center child text
                        justify_content: JustifyContent::Center,
                        // vertically center child text
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    border_color: BorderColor(Color::BLACK),
                    background_color: NORMAL_BUTTON.into(),
                    ..default()
                })
                .with_children(|parent| {
                    parent.spawn(TextBundle::from_section(
                        "<",
                        TextStyle {
                            font_size: 25.0,
                            color: Color::rgb(0.9, 0.9, 0.9),

                            ..default()
                        },
                    ));
                })
                .insert(DropdownMenu);

            parent
                .spawn(NodeBundle {
                    style: Style {
                        width: Val::Px(350.0),
                        height: Val::Px(700.0),
                        border: UiRect::all(Val::Px(1.0)),
                        justify_content: JustifyContent::FlexStart, // Centra orizzontalmente il contenuto
                        align_items: AlignItems::Center, // Allinea il contenuto dall'inizio verticalmente
                        flex_direction: FlexDirection::Column, // Disponi i figli in colonna
                        display: Display::None, // Assicurati che il display sia impostato su Flex
                        ..default()
                    },
                    visibility: Visibility::Visible,
                    border_color: BorderColor(Color::BLACK),
                    background_color: BackgroundColor(Color::rgba(255.0, 255.0, 255.0, 0.8)),
                    ..default()
                })
                .with_children(|parent| {
                    // TIME
                    parent
                        .spawn(TextBundle::from_section(
                            "Time \n", // Assumendo che questo generi il testo desiderato
                            TextStyle {
                                font_size: 25.0,
                                color: Color::BLACK,
                                ..default()
                            },
                        ))
                        .insert(TagTime);
                    // IMMAGINE
                    parent
                        .spawn(ImageBundle {
                            style: Style {
                                width: Val::Px(150.0),
                                height: Val::Px(150.0),
                                ..default()
                            },
                            image: UiImage::new(asset_server.load("img/sunny_day.png")), // Usa la texture caricata
                            ..default()
                        })
                        .insert(WeatherIcon);
                    //ENERGY AND COORDINATE
                    parent
                        .spawn(TextBundle::from_section(
                            "ENERGY \n", // Assumendo che questo generi il testo desiderato
                            TextStyle {
                                font_size: 25.0,
                                color: Color::BLACK,
                                ..default()
                            },
                        ))
                        .insert(TagEnergy);

                    // BARRA DELL'ENERGIA
                    // All'interno della funzione dove crei la UI
                    parent
                        .spawn(NodeBundle {
                            style: Style {
                                width: Val::Px(150.0),             // Larghezza del container esterno
                                height: Val::Px(30.0),             // Altezza del container esterno
                                border: UiRect::all(Val::Px(2.0)), // Bordi del container esterno
                                ..Default::default()
                            },
                            background_color: Color::NONE.into(),
                            border_color: Color::BLACK.into(), // Colore di sfondo del container esterno
                            ..Default::default()
                        })
                        .with_children(|parent| {
                            parent
                                .spawn(NodeBundle {
                                    style: Style {
                                        width: Val::Percent(100.0), // Larghezza iniziale del container interno (0% del parent)
                                        height: Val::Percent(100.0), // Altezza del container interno (100% del parent)
                                        ..Default::default()
                                    },
                                    background_color: Color::GREEN.into(),
                                    border_color: Color::BLACK.into(), // Colore del container interno (livello di energia)
                                    ..Default::default()
                                })
                                .insert(EnergyBar); // Assumi che EnergyBar sia un componente che hai definito
                        });
                    //COORDINATE ROBOT
                })
                .insert(Label);
        });

    //menu' a tendina BACKPACK
    commands
        .spawn(NodeBundle {
            style: Style {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                justify_content: JustifyContent::FlexStart, // Sposta orizzontalmente gli elementi a sinistra
                align_items: AlignItems::FlexEnd,           // Sposta
                ..default()
            },
            ..default()
        })
        .with_children(|parent| {
            parent
                .spawn(ButtonBundle {
                    style: Style {
                        width: Val::Px(120.0),
                        height: Val::Px(45.0),
                        border: UiRect::all(Val::Px(5.0)),
                        // horizontally center child text
                        justify_content: JustifyContent::Center, // Centra orizzontalmente il testo del figlio
                        align_items: AlignItems::Center,
                        margin: UiRect {
                            left: Val::Px(10.0),   // Distanzia il menu a tendina dal bordo sinistro
                            bottom: Val::Px(10.0), // Distanzia il menu a tendina dal pulsante
                            ..default()
                        },
                        ..default()
                    },
                    border_color: BorderColor(Color::BLACK),
                    background_color: NORMAL_BUTTON.into(),
                    ..default()
                })
                .with_children(|parent| {
                    parent.spawn(TextBundle::from_section(
                        "BACKPACK",
                        TextStyle {
                            font_size: 25.0,
                            color: Color::rgb(0.9, 0.9, 0.9),

                            ..default()
                        },
                    ));
                })
                .insert(DropdownMenuBackpack);

            parent
                .spawn(NodeBundle {
                    style: Style {
                        width: Val::Px(350.0),
                        height: Val::Px(700.0),
                        border: UiRect::all(Val::Px(1.0)),
                        justify_content: JustifyContent::FlexStart, // Centra orizzontalmente il contenuto
                        align_items: AlignItems::FlexStart, // Allinea il contenuto a sinistra verticalmente
                        flex_direction: FlexDirection::Column, // Disponi i figli in colonna
                        display: Display::None, // Nasconde il menu a tendina inizialmente
                        margin: UiRect {
                            left: Val::Px(-120.0), // Distanzia il menu a tendina dal bordo sinistro
                            bottom: Val::Px(55.0), // Distanzia il menu a tendina dal pulsante
                            ..default()
                        },
                        ..default()
                    },
                    visibility: Visibility::Visible,
                    border_color: BorderColor(Color::BLACK),
                    background_color: BackgroundColor(Color::rgba(255.0, 255.0, 255.0, 0.8)),
                    ..default()
                })
                .with_children(|parent| {
                    // Primo figlio:
                    parent
                        .spawn(TextBundle::from_section(
                            "BACKPACK", // Assumendo che questo generi il testo desiderato
                            TextStyle {
                                font_size: 25.0,
                                color: Color::BLACK,
                                ..default()
                            },
                        ))
                        .insert(TagBackPack);
                })
                .insert(LabelBackPack);
        });

    let main_scale = Vec3::new(0.1, 0.1, 1.0);

    //MAINCAMERA
    // Right Camera
    commands
        .spawn((
            Camera2dBundle {
                transform: Transform::from_xyz(
                    TILE_SIZE * resource.coordinate_row as f32,
                    TILE_SIZE * resource.coordinate_row as f32,
                    1.0,
                ) // Usa la posizione del punto rosso
                .with_scale(main_scale),
                camera: Camera {
                    order: 1,
                    ..default()
                },
                ..Default::default()
            },
            MainCamera,
        ))
        .insert(RenderLayers::from_layers(&[0]));

    // Calcola le dimensioni complessive del mondo
    let world_width: f32 = world.len() as f32 * TILE_SIZE;
    let world_height = world[0].len() as f32 * TILE_SIZE;

    // Calcola il centro del mondo
    let world_center_x = world_width / 2.0; // Assumi che 300 sia l'offset usato
    let world_center_y = world_height / 2.0;

    // Definisci le dimensioni della minimappa e lo spessore del bordo
    let minimap_width = 70.0; // Sostituisci con la larghezza effettiva della tua minimappa
    let minimap_height = 70.0; // Sostituisci con l'altezza effettiva della tua minimappa

    // Scala per la camera della minimappa (aggiusta questo valore in base alla necessita' )
    let minimap_scale = Vec3::new(
        WORLD_SIZE as f32 / minimap_width,
        WORLD_SIZE as f32 / minimap_height,
        1.0,
    ); // Aumenta la scala per visualizzare l'intera matrice

    //CAMERA PER LA MINIMAPPA
    // Left Camera
    commands
        .spawn((
            Camera2dBundle {
                transform: Transform::from_xyz(world_center_x, world_center_y, 555.0)
                    .with_scale(minimap_scale), //usa il centro del mondo come posizione e z alta
                camera: Camera {
                    order: 2, //serve per mettere l'ordine di rendering delle camere, se non settato default a 0(MAINCAMERA)
                    ..default()
                },
                camera_2d: Camera2d {
                    // don't clear on the second camera because the first camera already cleared the window
                    clear_color: ClearColorConfig::None, //area di memoria dei pixel, senza None veniva pulita, si vedeva solo l'ultima camera
                    ..default()
                },
                ..default()
            },
            MyMinimapCamera,
        ))
        .insert(RenderLayers::from_layers(&[0, 1]));

    // Crea l'entita'  per il contorno sulla minimappa
    commands
        .spawn(SpriteBundle {
            sprite: Sprite {
                color: Color::rgba(1.0, 0.0, 0.0, 0.5), // Contorno rosso semitrasparente
                custom_size: Some(Vec2::new(25.0, 25.0)), // Dimensione iniziale, sara'  aggiornata
                ..default()
            },
            transform: Transform::from_xyz(0.0, 0.0, 999.0), // Metti il contorno sopra a tutti gli altri elementi della minimappa
            ..default()
        })
        .insert(MinimapOutline)
        .insert(RenderLayers::layer(1));

    //NERO SOTTO WORLD MAP
    // Cicla attraverso per spawnare i quadrati 3x3
    for x in 0..WORLD_SIZE {
        for y in 0..WORLD_SIZE {
            // Calcola la posizione di ogni quadrato 3x3
            let pos_x = x as f32 * TILE_SIZE;
            let pos_y = y as f32 * TILE_SIZE;

            // Spawn del quadrato 3x3
            commands.spawn(SpriteBundle {
                sprite: Sprite {
                    color: Color::GRAY,                                 // Imposta il colore su nero
                    custom_size: Some(Vec2::new(TILE_SIZE, TILE_SIZE)), // Imposta la dimensione su 3x3 unità
                    ..default()
                },
                transform: Transform::from_xyz(pos_x, pos_y, 0.0), // Posiziona il quadrato
                ..default()
            });
        }
    }

    let border_thickness = 5.0; // Spessore del bordo
    let effective_world_size = WORLD_SIZE as f32 + border_thickness * 2.0;

    // Itera attorno al perimetro della mappa per creare il bordo
    for x in 0..effective_world_size as u32 {
        for y in 0..effective_world_size as u32 {
            // Verifica se la posizione attuale è dentro l'area del bordo
            if x < border_thickness as u32
                || y < border_thickness as u32
                || x >= (WORLD_SIZE as f32 + border_thickness) as u32
                || y >= (WORLD_SIZE as f32 + border_thickness) as u32
            {
                // Calcola la posizione di ogni quadrato del bordo
                let pos_x = (x as f32 - border_thickness) * TILE_SIZE;
                let pos_y = (y as f32 - border_thickness) * TILE_SIZE;

                // Spawn del quadrato del bordo
                commands
                    .spawn(SpriteBundle {
                        sprite: Sprite {
                            color: Color::GREEN,                                // Colore rosso per il bordo
                            custom_size: Some(Vec2::new(TILE_SIZE, TILE_SIZE)), // Dimensione del quadrato
                            ..default()
                        },
                        transform: Transform::from_xyz(pos_x, pos_y, -1.0), // Posiziona il quadrato del bordo
                        ..default()
                    })
                    .insert(RenderLayers::layer(1));
            }
        }
    }

    // Cicla attraverso per spawnare i quadrati 3x3
    for x in 0..WORLD_SIZE {
        for y in 0..WORLD_SIZE {
            // Calcola la posizione di ogni quadrato 3x3
            let pos_x = x as f32 * TILE_SIZE;
            let pos_y = y as f32 * TILE_SIZE;

            // Spawn del quadrato 3x3
            commands
                .spawn(SpriteBundle {
                    sprite: Sprite {
                        color: Color::rgba(0.1, 0.1, 0.3, 0.5), // Colore blu scuro semi-trasparente
                        custom_size: Some(Vec2::new(TILE_SIZE, TILE_SIZE)), // Imposta la dimensione su 3x3 unità
                        ..default()
                    },
                    transform: Transform::from_xyz(pos_x, pos_y, 5.0), // Posiziona il quadrato
                    ..default()
                })
                .insert(SunTime);
        }
    }
}

fn update_infos(
    resource: RobotInfo, // Query per trovare il Text da aggiornare
    weather_icons: Res<WeatherIcons>,
    mut energy_query: Query<
        &mut Text,
        (
            With<TagEnergy>,
            Without<Roboto>,
            Without<TagTime>,
            Without<TagBackPack>,
        ),
    >,
    mut time_query: Query<
        &mut Text,
        (
            With<TagTime>,
            Without<TagEnergy>,
            Without<Roboto>,
            Without<TagBackPack>,
        ),
    >,
    mut backpack_query: Query<
        &mut Text,
        (
            With<TagBackPack>,
            Without<TagEnergy>,
            Without<Roboto>,
            Without<TagTime>,
        ),
    >,
    mut battery_query: Query<(&mut Style, &mut BackgroundColor), With<EnergyBar>>,
    mut sun_query: Query<&mut Sprite, With<SunTime>>,
    mut weather_image_query: Query<&mut UiImage, With<WeatherIcon>>,
) {
    // Ora puoi utilizzare `energy_level` e `time` senza preoccuparti del mutex
    //TESTO ENERGY E COORDINATE
    for mut text in energy_query.iter_mut() {
        text.sections[0].value = format!(
            "Energy: {}\n\n X: {}, Y: {}\n\n",
            resource.energy_level, resource.coordinate_column, resource.coordinate_row
        );
    }
    //TESTO TIME E WEATHER
    for mut text in time_query.iter_mut() {
        if resource.current_weather.is_some() {
            text.sections[0].value = format!(
                "Time: {}\n\n Weather: {:?}\n\n",
                resource.time,
                resource.current_weather.unwrap()
            );
        }
    }
    //TESTO BACKPACK
    for mut text in backpack_query.iter_mut() {
        let mut formatted_string = format!("Backpack Size: {}\n\n", resource.bp_size);
        let mut tot_value = 0;
        for (key, value) in resource.bp_contents.iter() {
            // Appendi ogni coppia chiave-valore come "key: value\n" alla stringa
            formatted_string.push_str(&format!("{}: {}\n", key, value));
            tot_value += value;
        }
        //controllare se sparisce dopo che si svuota lo zaino(dovrebbe)
        if tot_value == 20 {
            formatted_string.push_str(&format!("\nMAX SIZE REACHED"));
        }

        text.sections[0].value = formatted_string;
    }

    //UPDATE BATTERY SPRITE
    for (mut style, mut back_color) in battery_query.iter_mut() {
        // Calcola la percentuale dell'energia
        let percentage = resource.energy_level as f32 / 1000.0; // Assumendo 1000 come valore massimo dell'energia
                                                                // Aggiorna il colore in base alla percentuale
        back_color.0 = match percentage {
            p if p > 0.5 => Color::GREEN.into(),
            p if p > 0.25 => Color::YELLOW.into(),
            _ => Color::RED.into(),
        };
        // Aggiorna la larghezza in base alla percentuale dell'energia
        style.width = Val::Percent(percentage * 100.0);
    }

    //SUN MOVEMENT
    for mut sprite in sun_query.iter_mut() {
        let night_alpha = match parse_time(&resource.time) {
            Ok(time) => {
                if time >= 18.0 && time < 20.0 {
                    // Tramonto: aumenta gradualmente l'alpha
                    (time - 18.0) / 2.0 * 0.4 + 0.3
                } else if (time >= 20.0 && time <= 24.0) || (time >= 0.0 && time < 4.0) {
                    // Notte: alpha massimo, ma non completamente opaco
                    0.7
                } else if time >= 4.0 && time < 6.0 {
                    // Alba: diminuisce gradualmente l'alpha
                    (1.0 - (time - 4.0) / 2.0) * 0.4 + 0.3
                } else {
                    // Giorno: completamente trasparente
                    0.0
                }
            }
            Err(e) => {
                eprintln!("Errore nel parsing del tempo: {}", e);
                0.0 // valore predefinito per assenza di errore
            }
        };

        // Imposta l'alpha del colore dello sprite
        sprite.color.set_a(night_alpha);
    }

    //WEATHER ICON
    for mut image in weather_image_query.iter_mut() {
        if let Ok(time_value) = parse_time(&resource.time) {
            let image_handle = match resource.current_weather {
                Some(WeatherType::Sunny) => {
                    if time_value >= 6.0 && time_value < 18.0 {
                        weather_icons.sunny_day.clone()
                    } else {
                        weather_icons.sunny_night.clone()
                    }
                }
                Some(WeatherType::Rainy) => {
                    if time_value >= 6.0 && time_value < 18.0 {
                        weather_icons.rainy_day.clone()
                    } else {
                        weather_icons.rainy_night.clone()
                    }
                }
                Some(WeatherType::Foggy) => {
                    if time_value >= 6.0 && time_value < 18.0 {
                        weather_icons.foggy_day.clone()
                    } else {
                        weather_icons.foggy_night.clone()
                    }
                }
                Some(WeatherType::TrentinoSnow) => {
                    if time_value >= 6.0 && time_value < 18.0 {
                        weather_icons.trentino_snow_day.clone()
                    } else {
                        weather_icons.trentino_snow_night.clone()
                    }
                }
                Some(WeatherType::TropicalMonsoon) => {
                    if time_value >= 6.0 && time_value < 18.0 {
                        weather_icons.tropical_monsoon_day.clone()
                    } else {
                        weather_icons.tropical_monsoon_night.clone()
                    }
                }

                _ => continue, // Ignora se non c'è un tipo di tempo corrispondente
            };

            image.texture = image_handle; // Aggiorna l'immagine
        } else {
            // Gestisci il caso in cui parse_time restituisce un errore
            // Potresti voler loggare l'errore o prendere un'altra azione
        }
    }
}

//funziona usata in Weather icon e sun movement
//serve per cambiare il valore del tempo da stringa a f32
fn parse_time(time_str: &str) -> Result<f32, &'static str> {
    let parts: Vec<&str> = time_str.split(':').collect();
    if parts.len() != 2 {
        return Err("Formato del tempo non valido");
    }

    let hours = parts[0].parse::<f32>();
    let minutes = parts[1].parse::<f32>();

    match (hours, minutes) {
        (Ok(h), Ok(m)) if h >= 0.0 && h < 24.0 && m >= 0.0 && m < 60.0 => Ok(h + m / 60.0),
        _ => Err("Valori di ore o minuti non validi"),
    }
}

fn cursor_position(q_windows: Query<&Window, With<PrimaryWindow>>) {
    if let Ok(window) = q_windows.get_single() {
        if let Some(position) = window.cursor_position() {
            println!("Cursor is inside the primary window, at {:?}", position);
        } else {
            println!("Cursor is not in the game window.");
        }
    }
}

fn cursor_events(
    minimap_camera_query: Query<
        (&Camera, &Transform),
        (With<MyMinimapCamera>, Without<MainCamera>),
    >,
    mut main_camera_query: Query<&mut Transform, (With<MainCamera>, Without<MyMinimapCamera>)>,
    q_windows: Query<&Window, With<PrimaryWindow>>,
) {
    let window = q_windows.single();
    if let Some(cursor_position) = window.cursor_position() {
        if let Ok((minimap_camera, minimap_transform)) = minimap_camera_query.get_single() {
            if let Some(viewport) = &minimap_camera.viewport {
                // Ottieni la posizione e la dimensione fisica della viewport della minimappa
                let minimap_viewport_position = Vec2::new(
                    viewport.physical_position.x as f32,
                    viewport.physical_position.y as f32,
                );
                let minimap_viewport_size = Vec2::new(
                    viewport.physical_size.x as f32 / 1.5,
                    viewport.physical_size.y as f32 / 1.5,
                );
                // Calcola la posizione del cursore relativa alla minimappa
                let cursor_relative_to_minimap = cursor_position - minimap_viewport_position;

                // Verifica se il cursore è all'interno dei limiti della minimappa
                if cursor_relative_to_minimap.x >= 0.0
                    && cursor_relative_to_minimap.x <= minimap_viewport_size.x
                    && cursor_relative_to_minimap.y >= 0.0
                    && cursor_relative_to_minimap.y <= minimap_viewport_size.y
                {
                    // Il cursore si trova all'interno della minimappa, procedere con la logica del click

                    // Calcola le proporzioni del cursore all'interno della minimappa
                    let click_proportions = cursor_relative_to_minimap / minimap_viewport_size;

                    // Calcola la posizione nel mondo basata sulle proporzioni del click sulla minimappa
                    let world_pos_x = minimap_transform.translation.x
                        + (click_proportions.x - 0.5) * (WORLD_SIZE as f32 * TILE_SIZE);
                    // Inverti l'asse y poiché l'origine dello schermo è in alto a sinistra
                    let world_pos_y = minimap_transform.translation.y
                        + (0.5 - click_proportions.y) * (WORLD_SIZE as f32 * TILE_SIZE);

                    // Sposta la main camera a questa posizione nel mondo
                    for mut transform in main_camera_query.iter_mut() {
                        transform.translation.x = world_pos_x;
                        // Inverti l'asse y durante la traduzione della main camera
                        transform.translation.y = world_pos_y;
                    }
                }
            }
        }
    }
}

// Funzione per aggiornare le dimensioni e la posizione del rettangolo sulla minimappa
fn update_minimap_outline(
    mut commands: Commands,
    main_camera_query: Query<
        (&Transform, &Camera2d, &Camera),
        (With<MainCamera>, Without<MinimapOutline>),
    >,
    mut minimap_outline_query: Query<
        (&mut Sprite, &mut Transform),
        (With<MinimapOutline>, Without<MainCamera>),
    >,
) {
    if let Ok((main_camera_transform, main_camera_2d, main_camera)) = main_camera_query.get_single()
    {
        if let Some(viewport) = &main_camera.viewport {
            let viewport_width = viewport.physical_size.x as f32;
            let viewport_height = viewport.physical_size.y as f32;

            // Usa la scala della camera per calcolare la dimensione visibile
            let camera_scale = main_camera_transform.scale.x;

            // Calcola la dimensione visibile basata sulle dimensioni del viewport e sulla scala della camera
            let visible_width = viewport_width * camera_scale / 1.5;
            let visible_height = viewport_height * camera_scale / 1.5;

            // Calcola le dimensioni del rettangolo sulla minimappa
            let outline_size = Vec2::new(visible_width, visible_height);

            for (mut sprite, mut transform) in minimap_outline_query.iter_mut() {
                // Aggiorna le dimensioni del rettangolo sulla minimappa
                sprite.custom_size = Some(outline_size);

                // Aggiorna la posizione del rettangolo sulla minimappa per corrispondere alla posizione della camera principale
                transform.translation.x = main_camera_transform.translation.x;
                transform.translation.y = main_camera_transform.translation.y;
                // Assicurati che il rettangolo rimanga sempre sopra agli altri elementi della minimappa
                transform.translation.z = 999.0;
            }
        }
    }
}

fn set_camera_viewports(
    windows: Query<&Window>,
    mut resize_events: EventReader<WindowResized>,
    mut minimappa_camera: Query<&mut Camera, (With<MyMinimapCamera>, Without<MainCamera>)>,
    mut main_camera: Query<&mut Camera, With<MainCamera>>,
) {
    // We need to dynamically resize the camera's viewports whenever the window size changes
    // so then each camera always takes up half the screen.
    // A resize_event is sent when the window is first created, allowing us to reuse this system for initial setup.
    for resize_event in resize_events.read() {
        //parte sinistra
        let window = windows.get(resize_event.window).unwrap();
        let mut minimappa_camera = minimappa_camera.single_mut();
        minimappa_camera.viewport = Some(Viewport {
            physical_position: UVec2::new(0, 0),
            physical_size: UVec2::new(
                window.resolution.physical_width() / 6,
                window.resolution.physical_height() / 4,
            ),
            ..default()
        });

        //parte destra
        let mut main_camera = main_camera.single_mut();
        main_camera.viewport = Some(Viewport {
            physical_position: UVec2::new(0, 0),
            physical_size: UVec2::new(
                window.resolution.physical_width(),
                window.resolution.physical_height(),
            ),
            ..default()
        });
    }
}

fn update_show_tiles(
    world: &Vec<Vec<Option<Tile>>>,
    commands: &mut Commands,
    old_world: &mut Vec<Vec<Option<Tile>>>,
) {
    for (x, row) in world.iter().enumerate() {
        for (y, tile) in row.iter().enumerate() {
            let old_tile = &old_world[x][y];
            // Se il nuovo tile non e' None e il vecchio tile e' None, spawnalo
            if tile.is_some()
                && (old_tile.is_none()
                    || old_tile.clone().unwrap().content != tile.clone().unwrap().content)
            {
                let tile = tile.clone().unwrap();
                // println!("x: {:?}, y: {:?}, tile: {:?}", x, y, tile);
                let tile_color = get_color(tile.clone());
                let content_color = get_content_color(tile.clone());
                let mut z_value = 1.0;
                // Optionally spawn an additional sprite for the content if it's not None
                if tile.content != Content::None {
                    commands
                        .spawn(SpriteBundle {
                            sprite: Sprite {
                                color: content_color, // Use the content color
                                custom_size: Some(Vec2::new(TILE_SIZE / 3.0, TILE_SIZE / 3.0)), // Smaller than the tile for distinction
                                ..Default::default()
                            },
                            transform: Transform::from_xyz(
                                x as f32 * TILE_SIZE, // Centered on the tile
                                y as f32 * TILE_SIZE, // Centered on the tile
                                z_value,              // Slightly above the tile layer
                            ),
                            ..Default::default()
                        })
                        .insert(RenderLayers::layer(0));
                    z_value = 0.0;
                }

                // Create a base sprite for the tile
                commands
                    .spawn(SpriteBundle {
                        sprite: Sprite {
                            color: tile_color, // Use the tile color
                            custom_size: Some(Vec2::new(TILE_SIZE, TILE_SIZE)),
                            ..Default::default()
                        },
                        transform: Transform::from_xyz(
                            x as f32 * TILE_SIZE, // X position with an offset
                            y as f32 * TILE_SIZE, // Y position with an offset
                            z_value,
                        ),
                        ..Default::default()
                    })
                    .insert(RenderLayers::layer(0));
            }
        }
    }
    *old_world = world.clone();
}

#[derive(Component)]
struct ZoomIn;

#[derive(Component)]
struct ZoomOut;

#[derive(Component)]
struct DropdownMenu;

#[derive(Component)]
struct DropdownMenuBackpack;

#[derive(Component)]
struct Label;

#[derive(Component)]
struct LabelBackPack;

#[derive(Component)]
struct CloseAppButton;

fn button_system(
    mut interaction_query: Query<
        (
            &Interaction,
            &mut BackgroundColor,
            &mut BorderColor,
            &Children,
            Option<&ZoomIn>,
            Option<&ZoomOut>,
            Option<&DropdownMenu>,
            Option<&DropdownMenuBackpack>,
            Option<&CloseAppButton>,
        ),
        (Changed<Interaction>, With<Button>),
    >,
    mut text_query: Query<&mut Text>,
    mut camera_query: Query<(&mut Transform, &Camera), With<MainCamera>>,
    mut label_query: Query<&mut Style, (With<Label>, Without<LabelBackPack>)>,
    mut label_backpack_query: Query<&mut Style, (With<LabelBackPack>, Without<Label>)>,
    robot_position: Res<RobotPosition>,
    mut exit: EventWriter<AppExit>,
) {
    for (
        interaction,
        mut color,
        mut border_color,
        children,
        zoomin,
        zoomout,
        dropdown,
        dropdownback,
        closeapp,
    ) in &mut interaction_query
    {
        //let mut text = text_query.get_mut(children[0]).unwrap();
        match *interaction {
            Interaction::Pressed => {
                if zoomin.is_some() {
                    adjust_camera_zoom_and_position(0.03, &mut camera_query, &robot_position);
                }
                if zoomout.is_some() {
                    adjust_camera_zoom_and_position(-0.03, &mut camera_query, &robot_position);
                } else if dropdown.is_some() {
                    for mut node_style in label_query.iter_mut() {
                        if node_style.display == Display::None {
                            node_style.display = Display::Flex; // Cambia da None a Flex
                        } else {
                            node_style.display = Display::None; // Cambia da None a Flex
                        }
                    }
                } else if closeapp.is_some() {
                    exit.send(AppExit);
                } else if dropdownback.is_some() {
                    for mut node_style in label_backpack_query.iter_mut() {
                        if node_style.display == Display::None {
                            node_style.display = Display::Flex;
                        } else {
                            node_style.display = Display::None;
                        }
                    }
                }

                *color = PRESSED_BUTTON.into();
                border_color.0 = Color::RED;
            }
            Interaction::Hovered => {
                *color = HOVERED_BUTTON.into();
                border_color.0 = Color::WHITE;
            }
            Interaction::None => {
                *color = NORMAL_BUTTON.into();
                border_color.0 = Color::BLACK;
            }
        }
    }
}

// Funzione per aggiustare lo zoom e la posizione della camera
//chiamata in button sistem
fn adjust_camera_zoom_and_position(
    zoom_change: f32,
    mut camera_query: &mut Query<(&mut Transform, &Camera), With<MainCamera>>,
    robot_position: &Res<RobotPosition>,
) {
    if let Ok((mut transform, camera)) = camera_query.get_single_mut() {
        if let Some(viewport) = &camera.viewport {
            let max_scale_width = (WORLD_SIZE as f32 * TILE_SIZE) / viewport.physical_size.x as f32;
            let max_scale_height =
                (WORLD_SIZE as f32 * TILE_SIZE) / viewport.physical_size.y as f32;

            // Usa il minore dei due per garantire che nessun bordo vada oltre il mondo
            let max_scale = max_scale_width.min(max_scale_height);

            // Aggiorna il valore di MAX_ZOOM in base al calcolo
            let max_zoom = MAX_ZOOM.min(max_scale);

            // Aggiusta lo zoom e assicurati che sia nel range consentito
            let new_scale = (transform.scale.x + zoom_change).clamp(MIN_ZOOM, max_zoom);
            transform.scale.x = new_scale;
            transform.scale.y = new_scale;

            // Assicurati che la vista della camera non sia mai più grande del mondo di gioco
            let camera_half_width = ((viewport.physical_size.x as f32 / new_scale) / 2.0)
                .min(WORLD_SIZE as f32 * TILE_SIZE / 2.0);
            let camera_half_height = ((viewport.physical_size.y as f32 / new_scale) / 2.0)
                .min(WORLD_SIZE as f32 * TILE_SIZE / 2.0);

            // Calcola i confini del mondo di gioco
            let world_min_x = camera_half_width;
            let world_max_x = WORLD_SIZE as f32 * TILE_SIZE - camera_half_width;
            let world_min_y = camera_half_height;
            let world_max_y = WORLD_SIZE as f32 * TILE_SIZE - camera_half_height;

            // Clamp può fallire se min è maggiore di max, quindi aggiungiamo un controllo qui
            if world_min_x > world_max_x || world_min_y > world_max_y {
                eprintln!("Il mondo di gioco è troppo piccolo per il livello di zoom attuale!");
                return;
            }

            // Calcola la nuova posizione della camera limitata dai confini del mondo di gioco
            transform.translation.x = robot_position.x.clamp(world_min_x, world_max_x);
            transform.translation.y = robot_position.y.clamp(world_min_y, world_max_y);
        }
    }
}

const NORMAL_BUTTON: Color = Color::rgb(0.15, 0.15, 0.15);
const HOVERED_BUTTON: Color = Color::rgb(0.25, 0.25, 0.25);
const HOVERED_PRESSED_BUTTON: Color = Color::rgb(0.25, 0.65, 0.25);
const PRESSED_BUTTON: Color = Color::rgb(0.35, 0.75, 0.35);

// Modifica setup_main_camera per accettare posizione x, y del punto rosso
fn setup_main_camera(commands: &mut Commands, x: f32, y: f32) {
    commands
        .spawn(Camera2dBundle {
            transform: Transform::from_xyz(x, y, 1.0) // Usa la posizione del punto rosso
                .with_scale(Vec3::splat(0.15)),
            ..Default::default()
        })
        .insert(MainCamera);
}

pub fn zoom_in(mut query: Query<&mut OrthographicProjection, With<Camera>>) {
    for mut projection in query.iter_mut() {
        projection.scale -= 100.0;

        println!("Current zoom scale: {}", projection.scale);
    }
}
/*
fn tryfunction(
    mut command: Commands,
    robot_resource: Res<RobotResource>,
    battery_query: Query<(&mut Style, &mut Sprite), With<EnergyBar>>,
) {

    let resource = robot_resource.0.lock().unwrap();
    let resource_copy = resource.clone();
    drop(resource);
    update_energy_bar_color(resource_copy.clone(), battery_query);
}
 */

//movimento del robot in base alla grandezza di una tile
fn robot_movement_system(
    mut commands: Commands,
    mut query: Query<
        &mut Transform,
        (
            With<Roboto>,
            Without<TagEnergy>,
            Without<TagTime>,
            Without<TagBackPack>,
            Without<DirectionalLight>,
        ),
    >,
    tile_size: Res<TileSize>, // Utilizza la risorsa TileSize
    robot_resource: Res<RobotResource>,
    world: Res<MapResource>,
    weather_icons: Option<Res<WeatherIcons>>,
    mut old_world: Option<ResMut<OldMapResource>>,
    energy_query: Query<
        &mut Text,
        (
            With<TagEnergy>,
            Without<Roboto>,
            Without<TagTime>,
            Without<TagBackPack>,
        ),
    >,
    time_query: Query<
        &mut Text,
        (
            With<TagTime>,
            Without<TagEnergy>,
            Without<Roboto>,
            Without<TagBackPack>,
        ),
    >,
    backpack_query: Query<
        &mut Text,
        (
            With<TagBackPack>,
            Without<TagEnergy>,
            Without<Roboto>,
            Without<TagTime>,
        ),
    >,
    battery_query: Query<(&mut Style, &mut BackgroundColor), With<EnergyBar>>,
    sun_query: Query<&mut Sprite, With<SunTime>>,
    weather_image_query: Query<&mut UiImage, With<WeatherIcon>>,
) {
    let world = world.0.lock().unwrap();
    if let Some(ref mut old_world) = old_world {
        update_show_tiles(&world, &mut commands, &mut old_world.world);
    }
    let resource = robot_resource.0.lock().unwrap();
    let tile_step = tile_size.tile_size; // Use the dimension of the tile from the resource
    let resource_copy = resource.clone();
    drop(resource);
    if let Some(weather_icons) = weather_icons {
        update_infos(
            resource_copy.clone(),
            weather_icons,
            energy_query,
            time_query,
            backpack_query,
            battery_query,
            sun_query,
            weather_image_query,
        );
    }
    // println!(
    //     "Energy Level: {}\nRow: {}\nColumn: {}\nBackpack Size: {}\nBackpack Contents: {:?}\nCurrent Weather: {:?}\nNext Weather: {:?}\nTicks Until Change: {}",
    //     resource_copy.energy_level,
    //     resource_copy.coordinate_row,
    //     resource_copy.coordinate_column,
    //     resource_copy.bp_size,
    //     resource_copy.bp_contents,
    //     resource_copy.current_weather,
    //     resource_copy.next_weather,
    //     resource_copy.ticks_until_change
    // );

    for mut transform in query.iter_mut() {
        transform.translation.y = tile_step * resource_copy.coordinate_column as f32;
        transform.translation.x = tile_step * resource_copy.coordinate_row as f32;
    }
}

//TODO: CAMBIARE LA POSIZIONE DEL ROBOT BASANDOLA SULLE RIGHE E COLONNE DELLA MATRICE
//serve per avere la posizione del puntino rosso ad ogni movimento
fn update_robot_position(
    mut robot_position: ResMut<RobotPosition>,
    robot_query: Query<&Transform, With<Roboto>>,
) {
    if let Ok(robot_transform) = robot_query.get_single() {
        robot_position.x = robot_transform.translation.x;
        robot_position.y = robot_transform.translation.y;
    }
}

/* //serve per far muovere la camera sopra il puntino rosso, prenderndo le sue coordinate
fn follow_robot_system(
    robot_position: Res<RobotPosition>,
    mut camera_query: Query<&mut Transform, With<MainCamera>>,
) {
    if let Ok(mut camera_transform) = camera_query.get_single_mut() {
        camera_transform.translation.x = robot_position.x;
        camera_transform.translation.y = robot_position.y;
        //serve per la distanza dall'alto dal punto rosso
        camera_transform.translation.z = 10.0; // Mantiene la camera elevata per una vista top-down
    }
} */

fn follow_robot_system(
    robot_position: Res<RobotPosition>,
    mut camera_query: Query<(&mut Transform, &Camera), With<MainCamera>>,
) {
    if let Ok((mut camera_transform, camera)) = camera_query.get_single_mut() {
        if let Some(viewport) = &camera.viewport {
            // Prendi la scala dalla camera transform
            let camera_scale = camera_transform.scale;

            // Calcola la larghezza e l'altezza visibili dalla camera
            let camera_half_width = (viewport.physical_size.x as f32 * camera_scale.x) / 3.1;
            let camera_half_height = (viewport.physical_size.y as f32 * camera_scale.y) / 3.1;

            // Definisci i confini del mondo di gioco
            let world_min_x = camera_half_width;
            let world_max_x = WORLD_SIZE as f32 * TILE_SIZE - camera_half_width;
            let world_min_y = camera_half_height;
            let world_max_y = WORLD_SIZE as f32 * TILE_SIZE - camera_half_height;

            // Calcola la nuova posizione della camera limitata dai confini del mondo di gioco
            let new_camera_x = robot_position.x.clamp(world_min_x, world_max_x);
            let new_camera_y = robot_position.y.clamp(world_min_y, world_max_y);

            // Aggiorna la posizione della camera
            camera_transform.translation.x = new_camera_x;
            camera_transform.translation.y = new_camera_y;
            // La z può rimanere invariata a meno che non si voglia modificare anche l'altezza della camera
            camera_transform.translation.z = 10.0; // Mantiene la camera elevata per una vista top-down
        }
    }
}

//DINAMICA(?)
/* fn follow_robot_system(
    robot_position: Res<RobotPosition>,
    mut camera_query: Query<(&mut Transform, &Camera), With<MainCamera>>,
) {
    if let Ok((mut camera_transform, camera)) = camera_query.get_single_mut() {
        if let Some(viewport) = &camera.viewport {
            let viewport_aspect_ratio =
                viewport.physical_size.x as f32 / viewport.physical_size.y as f32;
            let world_aspect_ratio =
                (WORLD_SIZE as f32 * TILE_SIZE) / (WORLD_SIZE as f32 * TILE_SIZE);

            // Calcola un fattore di scala basato sul rapporto tra l'aspect ratio della viewport e quello del mondo
            let scale_factor = if viewport_aspect_ratio > world_aspect_ratio {
                viewport.physical_size.y as f32 / (WORLD_SIZE as f32 * TILE_SIZE)
            } else {
                viewport.physical_size.x as f32 / (WORLD_SIZE as f32 * TILE_SIZE)
            };

            // println!("SCALEFACTON: {}", scale_factor);

            let camera_scale = camera_transform.scale.x;

            // Utilizza il scale_factor per determinare la larghezza e l'altezza visibili della camera
            let camera_half_width = (viewport.physical_size.x as f32 * camera_scale) / scale_factor;
            let camera_half_height =
                (viewport.physical_size.y as f32 * camera_scale) / scale_factor;

            // Resto della logica per limitare la camera ai bordi del mondo...
            let world_min_x = camera_half_width + SCHERMO;
            let world_max_x = WORLD_SIZE as f32 * TILE_SIZE - camera_half_width + SCHERMO;
            let world_min_y = camera_half_height + SCHERMO;
            let world_max_y = WORLD_SIZE as f32 * TILE_SIZE - camera_half_height + SCHERMO;

            let new_camera_x = robot_position.x.clamp(world_min_x, world_max_x);
            let new_camera_y = robot_position.y.clamp(world_min_y, world_max_y);

            camera_transform.translation.x = new_camera_x;
            camera_transform.translation.y = new_camera_y;
        }
    }
} */
//enum per ai_logic (4 stringhe)
enum AiLogic {
    Falegname,
    Asfaltatore,
    Ricercatore,
    Completo,
}

fn moviment(robot_data: Arc<Mutex<RobotInfo>>, map: Arc<Mutex<Vec<Vec<Option<Tile>>>>>) {
    let audio = get_audio_manager();
    //let background_music = OxAgSoundConfig::new_looped_with_volume("assets/audio/background.ogg", 2.0);

    let mut robot = Robottino {
        shared_map: map,
        shared_robot: robot_data,
        robot: Robot::new(),
        audio: audio,
        weather_tool: WeatherPredictionTool::new(),
        ai_logic: AI,
        maze_discovered: None,
    };

    // world generator initialization
    let mut world_gen =
        ghost_amazeing_island::world_generator::WorldGenerator::new(WORLD_SIZE, false, 1, 1.1);
    // Runnable creation and start

    println!("Generating runnable (world + robot)...");
    // match robot.audio.play_audio(&background_music) {
    //     Ok(_) => {},
    //     Err(e) => {
    //         eprintln!("Failed to play audio: {}", e);
    //         std::process::exit(1);
    //     }
    // }
    let mut world_gen =
        ghost_amazeing_island::world_generator::WorldGenerator::new(WORLD_SIZE, false, 1, 1.1);
    let mut runner = Runner::new(Box::new(robot), &mut world_gen);
    println!("Runnable succesfully generated");
    //sleep 5 second
    sleep(std::time::Duration::from_secs(3));
    for _i in 0..10000 {
        let rtn = runner.as_mut().unwrap().game_tick();
        // sleep(std::time::Duration::from_secs(1));
    }
}

#[derive(Clone)]
struct RobotResource(Arc<Mutex<RobotInfo>>);
struct MapResource(Arc<Mutex<Vec<Vec<Option<Tile>>>>>);
struct OldMapResource {
    world: Vec<Vec<Option<Tile>>>,
}

impl bevy::prelude::Resource for RobotResource {}
impl bevy::prelude::Resource for MapResource {}
impl bevy::prelude::Resource for OldMapResource {}

use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

// Definire una struttura di dati condivisa; ad esempio, una posizione condivisa (x, y)
#[derive(Debug)]
struct Position {
    x: i32,
    y: i32,
}

#[derive(Clone)]
struct RobotInfo {
    energy_level: usize,                  // livello di energia del robot
    coordinate_row: usize,                // posizione del robot
    coordinate_column: usize,             // posizione del robot
    bp_size: usize,                       // dimensione dello zaino
    bp_contents: HashMap<Content, usize>, // contenuto dello zaino
    current_weather: Option<WeatherType>, // tempo attuale
    next_weather: Option<WeatherType>,    // prossima previsione del tempo
    ticks_until_change: u32,              // tempo per la prossima previsione del tempo2
    time: String,
}

// Tag component used to tag entities added on the main menu screen
#[derive(Component, Debug)]
struct OnMainMenuScreen;

#[derive(Component, Debug)]
struct OnMainMenuCamera;

fn main() {
    // Dati condivisi tra thread
    let robot_info = RobotInfo {
        energy_level: 1000,
        coordinate_row: 0,
        coordinate_column: 0,
        bp_size: 10,
        bp_contents: HashMap::new(),
        current_weather: None,
        next_weather: None,
        ticks_until_change: 0,
        time: "0:0".to_string(),
    };

    let robot_data = Arc::new(Mutex::new(robot_info));
    let robot_data_clone = robot_data.clone();

    let map: Arc<Mutex<Vec<Vec<Option<Tile>>>>> =
        Arc::new(Mutex::new(vec![
            vec![None; WORLD_SIZE as usize];
            WORLD_SIZE as usize
        ]));
    let map_clone = map.clone();

    let moviment = thread::spawn(move || {
        moviment(robot_data, map);
    });

    let robot_resource = RobotResource(robot_data_clone);
    let map_resource = MapResource(map_clone);

    App::new()
        .init_resource::<RobotPosition>() //ricordarsi di metterlo quando si ha una risorsa
        .insert_resource(TileSize { tile_size: 3.0 }) //setta la risorsa tile per la grandezza di esso
        .insert_resource(robot_resource)
        .insert_resource(map_resource)
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            (
                cursor_events,
                robot_movement_system,
                update_robot_position,
                follow_robot_system,
                button_system,
                set_camera_viewports,
                update_minimap_outline,
            ),
        ) //unpdate every frame
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                mode: WindowMode::Windowed,
                ..default()
            }),
            ..Default::default()
        }))
        .run();

    moviment.join().unwrap();
}

struct Robottino {
    shared_robot: Arc<Mutex<RobotInfo>>,
    shared_map: Arc<Mutex<Vec<Vec<Option<Tile>>>>>,
    robot: Robot,
    audio: OxAgAudioTool,
    weather_tool: WeatherPredictionTool,
    ai_logic: AiLogic,
    maze_discovered: Option<(usize, usize)>,
}

fn find_entrance(
    robot: &mut Robottino,
    world: &mut robotics_lib::world::World,
    mut last_direction: Direction,
) {
    loop {
        //sleep 300
        
        if last_direction == Direction::Up {
            go(robot, world, Direction::Left);
        } else if last_direction == Direction::Down {
            go(robot, world, Direction::Right);
        } else if last_direction == Direction::Left {
            go(robot, world, Direction::Down);
        } else if last_direction == Direction::Right {
            go(robot, world, Direction::Up);
        }

        let view = robot_view(robot, world);
        update_map(robot, world);
        //get tile 1 unwrap if some
        println!("{:?}",view[0][1].as_ref().unwrap().tile_type==TileType::Wall);//alto
        println!("{:?}",view[1][0].as_ref().unwrap().tile_type==TileType::Wall);//sinistra
        println!("{:?}",view[1][2].as_ref().unwrap().tile_type==TileType::Wall);//destra
        println!("{:?}",view[2][1].as_ref().unwrap().tile_type==TileType::Wall);//basso
        sleep(std::time::Duration::from_millis(1000));

        match last_direction {
            Direction::Up => {
                if let Some(tile) = &view[0][1] {
                    if tile.tile_type == TileType::Wall {
                        print!("su");
                    } else {
                        go(robot, world, Direction::Up);
                        last_direction = Direction::Right;
                    }
                }
            }
            Direction::Down => {
                if let Some(tile) = &view[2][1] {
                    if tile.tile_type == TileType::Wall {
                        print!("giu");
                    } else {
                        go(robot, world, Direction::Down);
                        last_direction = Direction::Left;
                    }
                }
            }
            Direction::Left => {
                if let Some(tile) = &view[1][0] {
                    if tile.tile_type == TileType::Wall {
                        print!("left");
                    } else {
                        go(robot, world, Direction::Left);
                        last_direction = Direction::Down;
                    }
                }
            }
            Direction::Right => {
                if let Some(tile) = &view[1][2] {
                    if tile.tile_type == TileType::Wall {
                        print!("right");
                    } else {
                        go(robot, world, Direction::Right);
                        last_direction = Direction::Up;
                    }
                }
            }
        }
    }
}

fn go_to_maze(robot: &mut Robottino, world: &mut robotics_lib::world::World, maze: (usize, usize)) {
    //reach the maze
    let cord_r = robot.get_coordinate();
    let mut res: Result<(Vec<Vec<Option<Tile>>>, (usize, usize)), LibError> =
        Err(LibError::CannotWalk);
    let mut last_direction = Direction::Up;
    if cord_r.get_row() < maze.0 {
        res = go(robot, world, Direction::Down);
        last_direction = Direction::Down;
    } else if cord_r.get_row() > maze.0 {
        res = go(robot, world, Direction::Up);
        last_direction = Direction::Up;
    } else if cord_r.get_col() < maze.1 {
        res = go(robot, world, Direction::Right);
        last_direction = Direction::Right;
    } else if cord_r.get_col() > maze.1 {
        res = go(robot, world, Direction::Left);
        last_direction = Direction::Left;
    }
    if res.is_err() {
        //if wall is up go left
        find_entrance(robot, world, last_direction)
    }
}

fn ai_labirint(robot: &mut Robottino, world: &mut robotics_lib::world::World) {
    //maze are 18*18 so we check every 9 tiles
    //if robotmap some save it
    if robot.maze_discovered.is_none() {
        if let Some(map) = robot_map(world) {
            //quanto e' grande la mappa
            let map_size = map.len();
            let times_to_discover_map_for_side = map_size / 9 + 1;
            for i in 1..times_to_discover_map_for_side {
                for j in 1..times_to_discover_map_for_side {
                    if robot.maze_discovered.is_some() {
                        break;
                    }
                    if robot.robot.energy.get_energy_level() < 300 {
                        robot.robot.energy =
                            rust_and_furious_dynamo::dynamo::Dynamo::update_energy();
                    }
                    let row = i * 9;
                    let col = j * 9;
                    let tiles = discover_tiles(
                        robot,
                        world,
                        &[
                            (row - 1, col),
                            (row, col),
                            (row - 1, col - 1),
                            (row, col - 1),
                        ],
                    );
                    //get result
                    if let Ok(tiles) = tiles {
                        //check if a Tile is Some and is a wall
                        //fai if let some tiles[&(row, col)].is_some();
                        if let Some(tile) = &tiles[&(row, col)] {
                            if tile.tile_type == TileType::Wall {
                                robot.maze_discovered = Some((row, col));
                            }
                        }
                        if let Some(tile) = &tiles[&(row - 1, col)] {
                            if tile.tile_type == TileType::Wall {
                                robot.maze_discovered = Some((row - 1, col));
                            }
                        }
                        if let Some(tile) = &tiles[&(row, col - 1)] {
                            if tile.tile_type == TileType::Wall {
                                robot.maze_discovered = Some((row, col - 1));
                            }
                        }
                        if let Some(tile) = &tiles[&(row - 1, col - 1)] {
                            if tile.tile_type == TileType::Wall {
                                robot.maze_discovered = Some((row - 1, col - 1));
                            }
                        }
                    }
                }
            }
        }
    }
    if robot.robot.energy.get_energy_level() < 300 {
        robot.robot.energy = rust_and_furious_dynamo::dynamo::Dynamo::update_energy();
    }
    robot_view(robot, world);
    //move robot to the maze with go function i can move up down left right
    if let Some((row, col)) = robot.maze_discovered {
        go_to_maze(robot, world, (row, col));
    }
}

fn ai_taglialegna(robot: &mut Robottino, world: &mut robotics_lib::world::World) {
    //se l'energia e' sotto il 300, la ricarico
    if robot.robot.energy.get_energy_level() < 300 {
        robot.robot.energy = rust_and_furious_dynamo::dynamo::Dynamo::update_energy();
    }

    let v = robot_view(robot, world);

    let a = robot.get_backpack().get_size();
    let b = robot.get_backpack().get_contents().values().sum::<usize>();
    if (a > b) {
        let tiles_option = cheapest_border(world, robot);
        if let Some(tiles) = tiles_option {
            //manage the return stat of move to cheapest border
            let result = move_to_cheapest_border(world, robot, tiles);

            DestroyZone.execute(world, robot, Content::Tree(0));
        }
    } else {
        let mut shopping_list = ShoppingList {
            list: vec![(
                Content::Crate(Range::default()),
                Some(OpActionInput::Put(Content::Tree(0), 20)),
            )],
        };
        match get_best_action_to_element(robot, world, &mut shopping_list) {
            None => {
                // mmh, the content was not found or you specified no action. Handle this case!
            }
            Some(next_action) => {
                // println!("{:?}", &rand);
                match next_action {
                    OpActionOutput::Move(dir) => {
                        go(robot, world, dir);
                    }
                    OpActionOutput::Destroy(dir) => {
                        // println!("Destroy");
                        destroy(robot, world, dir);
                    }
                    OpActionOutput::Put(c, u, d) => {
                        print!("depositandoooooooooooo");
                        //print c u d
                        println!("{:?} {:?} {:?}", c, u, d);
                        put(robot, world, c, u, d);
                    }
                }
            }
        }
    }
}
fn ai_asfaltatore(robot: &mut Robottino, world: &mut robotics_lib::world::World) {
    if robot.robot.energy.get_energy_level() < 200 {
        robot.robot.energy = rust_and_furious_dynamo::dynamo::Dynamo::update_energy();
    }

    robot_view(robot, world);

    if (robot.get_backpack().get_size()
        > robot.get_backpack().get_contents().values().sum::<usize>())
    {
        let tiles_option = cheapest_border(world, robot);
        if let Some(tiles) = tiles_option {
            //manage the return stat of move to cheapest border
            let result = move_to_cheapest_border(world, robot, tiles);

            DestroyZone.execute(world, robot, Content::Rock(0));
        }
    } else {
        for _i in 0..20 {
            sleep(std::time::Duration::from_millis(300));
            let v = bessie::bessie::road_paving_machine(
                robot,
                world,
                Direction::Up,
                bessie::bessie::State::MakeRoad,
            );
            //if err
            if v.is_err() {
                //random da 0 a 3
                let rand = rand::thread_rng().gen_range(0..4);
                match rand {
                    0 => go(robot, world, Direction::Up),
                    1 => go(robot, world, Direction::Down),
                    2 => go(robot, world, Direction::Left),
                    _ => go(robot, world, Direction::Right),
                };
            }
        }
    }
}
fn ai_completo_con_tool(robot: &mut Robottino, world: &mut robotics_lib::world::World) {
    //durata sleep in millisecondi per velocità robot
    let sleep_time_milly: u64 = 1000;

    sleep(std::time::Duration::from_millis(sleep_time_milly));
    //se l'energia e' sotto il 300, la ricarico

    // weather_check(self);

    // sleep(std::time::Duration::from_millis(300));
    // bessie::bessie::road_paving_machine(self, world, Direction::Up, State::MakeRoad);
    DestroyZone.execute(world, robot, Content::Tree(0));
    let a = robot.get_backpack();
    // print!("{:?}", a);

    //print coordinate
    let coordinates: &Coordinate = robot.get_coordinate();
    // println!("{:?}", coordinates);
    robot_view(robot, world);
    let tiles_option = cheapest_border(world, robot);
    let map = robot_map(world);
    //count how many tiles are not None in map
    let mut count = 0;
    if let Some(unwrapped_map) = map {
        for i in 0..unwrapped_map.len() {
            for j in 0..unwrapped_map[i].len() {
                if unwrapped_map[i][j].is_some() {
                    count += 1;
                }
            }
        }
    }
    // println!("{:?}", count);
    if let Some(tiles) = tiles_option {
        //manage the return stat of move to cheapest border
        let result = move_to_cheapest_border(world, robot, tiles);
        if let Err((_tiles, error)) = result {
            println!("The robot cannot move due to a {:?}", error);
        }
    }
    //print coordinate

    let actual_energy = robot.get_energy().get_energy_level();
    // println!("{:?}", actual_energy);
    let coordinates = robot.get_coordinate();
    // println!("{:?}", coordinates);
}

impl Runnable for Robottino {
    fn process_tick(&mut self, world: &mut robotics_lib::world::World) {
        let sleep_time_milly: u64 = 30;
        sleep(std::time::Duration::from_millis(sleep_time_milly));
        // in base alla logica scelta, esegue la funzione corrispondente
        match self.ai_logic {
            AiLogic::Falegname => ai_taglialegna(self, world),
            AiLogic::Asfaltatore => ai_asfaltatore(self, world),
            AiLogic::Ricercatore => ai_labirint(self, world),
            AiLogic::Completo => ai_completo_con_tool(self, world),
        }

        //update map
        update_map(self, world);
    }

    fn handle_event(&mut self, event: robotics_lib::event::events::Event) {
        self.weather_tool.process_event(&event);

        //update info
        {
            let mut shared_robot = self.shared_robot.lock().unwrap();
            shared_robot.energy_level = self.robot.energy.get_energy_level();
            shared_robot.coordinate_row = self.robot.coordinate.get_row();
            shared_robot.coordinate_column = self.robot.coordinate.get_col();
            shared_robot.bp_size = self.robot.backpack.get_size();
            shared_robot.bp_contents = self.robot.backpack.get_contents().clone();
        }
    }

    fn get_energy(&self) -> &Energy {
        &self.robot.energy
    }

    fn get_energy_mut(&mut self) -> &mut Energy {
        &mut self.robot.energy
    }

    fn get_coordinate(&self) -> &Coordinate {
        &self.robot.coordinate
    }

    fn get_coordinate_mut(&mut self) -> &mut Coordinate {
        &mut self.robot.coordinate
    }

    fn get_backpack(&self) -> &BackPack {
        &self.robot.backpack
    }

    fn get_backpack_mut(&mut self) -> &mut BackPack {
        &mut self.robot.backpack
    }
}

fn update_map(robot: &mut Robottino, world: &mut robotics_lib::world::World) {
    let mut shared_map = robot.shared_map.lock().unwrap();
    if let Some(new_map) = robot_map(world) {
        *shared_map = new_map;
    }
    let mut shared_robot = robot.shared_robot.lock().unwrap();
    let enviroment = look_at_sky(&world);

    shared_robot.time = enviroment.get_time_of_day_string();
    shared_robot.current_weather = Some(enviroment.get_weather_condition());
    if let Some((prediction, ticks)) = weather_check(robot) {
        shared_robot.next_weather = Some(prediction);
        shared_robot.ticks_until_change = ticks;
    }
}

fn weather_check(robot: &Robottino) -> Option<(WeatherType, u32)> {
    let ticks_until_weather = match robot.weather_tool.ticks_until_weather_change(100000000000) {
        Ok(ticks) => ticks,
        Err(e) => {
            eprintln!("Failed to get ticks until weather change: {:?}", e);
            return None; // Fix: Return None instead of ()
        }
    };
    let predict = match robot.weather_tool.predict(ticks_until_weather) {
        Ok(prediction) => prediction,
        Err(e) => {
            eprintln!("Failed to predict weather: {:?}", e);
            return None; // Fix: Return None instead of ()
        }
    };

    Some((predict, ticks_until_weather as u32))
}

fn get_audio_manager() -> OxAgAudioTool {
    let background_music = OxAgSoundConfig::new_looped_with_volume("audio/background.ogg", 2.0);

    let mut events = HashMap::new();
    // events.insert(Event::Ready, OxAgSoundConfig::new("assets/default/event/event_ready.ogg"));
    // events.insert(Event::Terminated, OxAgSoundConfig::new("assets/default/event/event_terminated.ogg"));
    // // events.insert(Event::EnergyRecharged(0), OxAgSoundConfig::new_with_volume("assets/default/event/event_energy_recharged.ogg", 0.1));
    // events.insert(Event::AddedToBackpack(Content::None, 0), OxAgSoundConfig::new("assets/default/event/event_add_to_backpack.ogg"));
    // events.insert(Event::RemovedFromBackpack(Content::None, 0), OxAgSoundConfig::new("assets/default/event/event_remove_from_backpack.ogg"));

    let mut tiles = HashMap::new();
    // tiles.insert(TileType::DeepWater, OxAgSoundConfig::new("assets/default/tile/tile_water.ogg"));
    // tiles.insert(TileType::ShallowWater, OxAgSoundConfig::new("assets/default/tile/tile_water.ogg"));
    // tiles.insert(TileType::Sand, OxAgSoundConfig::new("assets/default/tile/tile_sand.ogg"));
    // tiles.insert(TileType::Grass, OxAgSoundConfig::new("assets/default/tile/tile_grass.ogg"));
    // tiles.insert(TileType::Hill, OxAgSoundConfig::new("assets/default/tile/tile_grass.ogg"));
    // tiles.insert(TileType::Mountain, OxAgSoundConfig::new("assets/default/tile/tile_mountain.ogg"));
    // tiles.insert(TileType::Snow, OxAgSoundConfig::new("assets/default/tile/tile_snow.ogg"));
    // tiles.insert(TileType::Lava, OxAgSoundConfig::new("assets/default/tile/tile_lava.ogg"));
    // tiles.insert(TileType::Teleport(false), OxAgSoundConfig::new("assets/default/tile/tile_teleport.ogg"));
    // tiles.insert(TileType::Street, OxAgSoundConfig::new("assets/default/tile/tile_street.ogg"));

    let mut weather = HashMap::new();
    // weather.insert(WeatherType::Rainy, OxAgSoundConfig::new("assets/default/weather/weather_rainy.ogg"));
    // weather.insert(WeatherType::Foggy, OxAgSoundConfig::new("assets/default/weather/weather_foggy.ogg"));
    // weather.insert(WeatherType::Sunny, OxAgSoundConfig::new("assets/default/weather/weather_sunny.ogg"));
    // weather.insert(WeatherType::TrentinoSnow, OxAgSoundConfig::new("assets/default/weather/weather_winter.ogg"));
    // weather.insert(WeatherType::TropicalMonsoon, OxAgSoundConfig::new("assets/default/weather/weather_tropical.ogg"));

    // Create the audio tool
    let audio = match OxAgAudioTool::new(events, tiles, weather) {
        Ok(audio) => audio,
        Err(e) => {
            eprintln!("Failed to create OxAgAudioTool: {}", e);
            std::process::exit(1);
        }
    };
    return audio;
}
