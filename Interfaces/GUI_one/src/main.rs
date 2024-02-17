//TODO:
//SISTEMARE POSITIONING/CONTROLLO MATRICE
//MINIMAPPA
//BOTTONE ZOOM MAPPA
//INSERIRE TUTTE LE INFO(TEMPO, ENERGIA, COSTO, ECC..)
//DESPAWN CONTENT QUANDO SI PRENDONO/DISTRUGGONO 
//CREAZIONE BACKPACK/INSERIMENTO VISUALIZZA OGGETTI
//MODIFICA TILE QUANDO SI POSIZIONA LA ROCCIA


use bevy::{ecs::system::FunctionSystem, prelude::*,render::view::RenderLayers, render::texture, transform::commands, utils::petgraph::dot};
use robotics_lib::world::{self, environmental_conditions::WeatherType, tile::{self, Content, Tile, TileType}, world_generator::Generator};
use bevy::render::camera::Viewport;
use rand::Rng;
use bevy::window::WindowResized;
use bevy::core_pipeline::clear_color::ClearColorConfig;
use std::{collections::HashMap, ptr::null};
use std::thread::sleep;

use bessie::bessie::State;
use crab_rave_explorer::algorithm::{cheapest_border, move_to_cheapest_border};
use oxagaudiotool::sound_config::{self, OxAgSoundConfig};
use robotics_lib::event::events::Event;
use robotics_lib::interface::Direction::Up;
use ohcrab_weather::weather_tool::WeatherPredictionTool;
use arrusticini_destroy_zone::DestroyZone;
use oxagaudiotool::OxAgAudioTool;
use robotics_lib::interface::{ go, one_direction_view, robot_map, robot_view, Direction, look_at_sky};
use robotics_lib::{
    energy::Energy,
    runner::{backpack::BackPack, Robot, Runnable, Runner},
    world::coordinates::Coordinate
};


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
//risorsa generale per vedere/settare se il robot Ã¨ in pausa o meno(mettere in pausa il tutto)
struct RobotState{
    is_moving: bool,
}

#[derive(Resource, Debug, Default)]
struct TileSize{
    tile_size: f32,
}

#[derive(Component, Debug)]
//componente per la mappa zoommata
struct MainCamera;

#[derive(Component, Debug)]
//componente per la minimappa
struct MyMinimapCamera;

#[derive(Component, Debug)]
//componente per la minimappa
struct MainCameraEntity {
    entity: Entity,
}

//componente per la zona rossa della minimappa 
#[derive(Component, Debug)]
struct MinimapOutline;


// Puoi aggiungere altri campi se necessario, per esempio per memorizzare la posizione o altri parametri della camera.

const WORLD_SIZE:u32 = 60;


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

fn get_content_color(content: Tile) -> Color {
    match content.content {
        Content::Rock(_) => Color::rgb_u8(0xB0, 0xC4, 0xDE), // Light Steel Blue
        Content::Tree(_) => Color::rgb_u8(0x00, 0x64, 0x00), // Dark Green
        Content::Garbage(_) => Color::rgb_u8(0x8B, 0x45, 0x13), // Saddle Brown
        Content::Fire => Color::rgb_u8(0xFF, 0x45, 0x00), // Orange Red
        Content::Coin(_) => Color::rgb_u8(0xFF, 0xD7, 0x00), // Gold
        Content::Water(_) => Color::rgb_u8(0x87, 0xCE, 0xEB), // Sky Blue
        Content::Bin(_) => Color::rgb_u8(0x70, 0x80, 0x90), // Slate Gray
        Content::Crate(_) => Color::rgb_u8(0xF5, 0xF5, 0xDC), // Beige
        Content::Bank(_) => Color::rgb_u8(0x85, 0xBB, 0x65), // Dollar Bill
        Content::Market(_) => Color::rgb_u8(0xB2, 0x22, 0x22), // Firebrick
        Content::Fish(_) => Color::rgb_u8(0x00, 0xFF, 0xFF), // Aqua
        Content::Building => Color::rgb_u8(0x80, 0x00, 0x80), // Purple
        Content::Bush(_) => Color::rgb_u8(0x90, 0xEE, 0x90), // Light Green
        Content::JollyBlock(_) => Color::rgb_u8(0xFF, 0xC0, 0xCB), // Pink
        Content::Scarecrow => Color::rgb_u8(0xFF, 0xA5, 0x00), // Orange
        Content::None => Color::NONE, // Transparent or keep the tile color
        _ => Color::YELLOW_GREEN, // Fallback color for unspecified contents
    }
}


// Funzione di setup che crea la scena
fn setup(mut commands: Commands, asset_server: Res<AssetServer>, shared_map: Res<MapResource>,robot_resource: Res<RobotResource>,) {
    // Matrice di esempio

//TODO: controllare se positioning si basa effettivamente sulla matrice
//TODO: cabiare dimenzione con l'utilizzo della risorsa tile
    //sleep 3 secondi
    sleep(std::time::Duration::from_secs(3));
    let world = shared_map.0.lock().unwrap();
    let mut count = 0;
    let resource = robot_resource.0.lock().unwrap();

    //how many tile is not None
    for row in world.iter() {
        for tile in row.iter() {
            if tile.is_some() {
                count += 1;
            }
        }
    }
    println!("count mappaa viosualizzaaaaa {:?}", count);

    
// Dimensione di ogni quadrato
    //sotto funzione per telecamera
    //commands.spawn(Camera2dBundle::default()); 

    println!("Robot {:?} {:?}",resource.coordinate_column, resource.coordinate_row);    
    update_show_tiles(&world, &mut commands);


    

    // per la posizione centrale della tile
    
    let robot_size = 2.0;
    let sunny: Handle<Image> = asset_server.load("img/sunny.png");

        //spawna il robot
    commands.spawn(SpriteBundle {
        sprite: Sprite {
            color: Color::RED,
            custom_size: Some(Vec2::new(robot_size, robot_size)),
            ..Default::default()
        },
        transform: Transform::from_xyz(TILE_SIZE * resource.coordinate_row as f32, TILE_SIZE * resource.coordinate_row as f32, 2.0), // asse z serve per metterlo sopra i tile e i conent
        ..Default::default()
    }).insert(Roboto)
    .insert(RenderLayers::layer(0));
        
    commands
        .spawn(NodeBundle {
            style: Style {
                // Imposta le dimensioni del nodo contenitore per occupare l'intera finestra
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                // Allinea i bottoni in alto a sinistra
                align_items: AlignItems::FlexEnd, // Allinea verticalmente i figli all'inizio (alto)
                justify_content: JustifyContent::FlexEnd, // Allinea orizzontalmente i figli all'inizio (sinistra)
                flex_direction: FlexDirection::Row, // Disponi i figli in orizzontale
                // Aggiungi padding per posizionare i bottoni un po' distanti dal bordo superiore e sinistro
                padding: UiRect {
                    left: Val::Px(10.0),
                    top: Val::Px(10.0),
                    right: Val::Auto,
                    bottom: Val::Auto,
                },
                ..default()
            },
            ..default()
        })
        .with_children(|parent| {
            // Primo bottone
            parent.spawn(ButtonBundle {
                style: Style {
                    width: Val::Px(30.0),
                    height: Val::Px(30.0),
                    margin: UiRect::all(Val::Px(4.0)), // Spazio tra i bottoni
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
            parent.spawn(ButtonBundle {
                style: Style {
                    width: Val::Px(30.0),
                    height: Val::Px(30.0),
                    margin: UiRect::all(Val::Px(4.0)), // Spazio tra i bottoni
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
            }).insert(ZoomOut);
        }).insert(RenderLayers::layer(0));

        
        //menu a tendina
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
                }).insert(DropdownMenu);
            
                parent.spawn(NodeBundle {
                    style: Style {
                        width: Val::Px(250.0),
                        height: Val::Px(500.0),
                        border: UiRect::all(Val::Px(1.0)),
                        justify_content: JustifyContent::FlexStart, // Centra orizzontalmente il contenuto
                        align_items: AlignItems::Center, // Allinea il contenuto dall'inizio verticalmente
                        flex_direction: FlexDirection::Column, // Disponi i figli in colonna
                        display: Display::None, // Assicurati che il display sia impostato su Flex
                        ..default()
                    },
                    visibility: Visibility::Visible,
                    border_color: BorderColor(Color::BLACK),
                    background_color: BackgroundColor(Color::rgba(255.0,  255.0, 255.0, 0.8)),
                    ..default()
                })
                .with_children(|parent| {
                    // Primo figlio: TextBundle
                    parent.spawn(TextBundle::from_section(
                         "TIME \n", // Assumendo che questo generi il testo desiderato
                        TextStyle {
                            font_size: 25.0,
                            color: Color::BLACK,
                            ..default()
                        },
                    ));
                
                    // Secondo figlio: ImageBundle
                    parent.spawn(ImageBundle {
                        style: Style {
                            width: Val::Px(100.0),
                            height: Val::Px(100.0),
                            ..default()
                        },
                        image: UiImage::new(sunny), // Usa la texture caricata
                        ..default()
                    });
                }).insert(Label);
                
    });

    let main_scale = Vec3::new(0.1, 0.1, 1.0);

    //MAINCAMERA
    // Right Camera
    commands.spawn((
        Camera2dBundle {
            transform: Transform::from_xyz(TILE_SIZE * resource.coordinate_row as f32, TILE_SIZE * resource.coordinate_row as f32, 1.0) // Usa la posizione del punto rosso
            .with_scale(main_scale),
            camera: Camera{
                order: 0,
                ..default()
            },
            ..Default::default()
    
        },
    MainCamera,
    )).insert(RenderLayers::from_layers(&[0]));


    // Calcola le dimensioni complessive del mondo
    let world_width: f32 = world.len() as f32 * TILE_SIZE;
    let world_height = world[0].len() as f32 * TILE_SIZE;

    // Calcola il centro del mondo
    let world_center_x = world_width / 2.0 - WORLD_SIZE as f32; // Assumi che 300 sia l'offset usato
    let world_center_y = world_height / 2.0 - WORLD_SIZE as f32;

    // Scala per la camera della minimappa (aggiusta questo valore in base alla necessitÃ )
    let minimap_scale = Vec3::new(5.0, 5.0, 1.0); // Aumenta la scala per visualizzare l'intera matrice


        //CAMERA PER LA MINIMAPPA
        // Left Camera 
    commands.spawn((
    Camera2dBundle {
        transform: Transform::from_xyz(world_center_x, world_center_y, 555.0)
        .with_scale(minimap_scale), //usa il centro del mondo come posizione e z alta
        camera: Camera{
            order: 1,  //serve per mettere l'ordine di rendering delle camere, se non settato default a 0(MAINCAMERA)
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
    )).insert(RenderLayers::from_layers(&[0, 1]));



    // Crea l'entitÃ  per il contorno sulla minimappa
    commands.spawn(SpriteBundle {
    sprite: Sprite {
        color: Color::rgba(1.0, 0.0, 0.0, 0.5), // Contorno rosso semitrasparente
        custom_size: Some(Vec2::new(25.0, 25.0)), // Dimensione iniziale, sarÃ  aggiornata
        ..default()
    },
    transform: Transform::from_xyz(0.0, 0.0, 999.0), // Metti il contorno sopra a tutti gli altri elementi della minimappa
    ..default()
    }).insert(MinimapOutline)
    .insert(RenderLayers::layer(1)); 


}

// Funzione per aggiornare le dimensioni e la posizione del rettangolo sulla minimappa
fn update_minimap_outline(
    mut commands: Commands,
    main_camera_query: Query<(&Transform, &Camera2d, &Camera), (With<MainCamera>, Without<MinimapOutline>)>,
    mut minimap_outline_query: Query<(&mut Sprite, &mut Transform), (With<MinimapOutline>, Without<MainCamera>)>,
) {
    if let Ok((main_camera_transform, main_camera_2d, main_camera)) = main_camera_query.get_single() {
        if let Some(viewport) = &main_camera.viewport {
            let viewport_width = viewport.physical_size.x as f32;
            let viewport_height = viewport.physical_size.y as f32;
            
            // Usa la scala della camera per calcolare la dimensione visibile
            let camera_scale = main_camera_transform.scale.x;
            
            // Calcola la dimensione visibile basata sulle dimensioni del viewport e sulla scala della camera
            let visible_width = viewport_width * camera_scale;
            let visible_height = viewport_height * camera_scale;
            
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

fn update_show_tiles(world: &Vec<Vec<Option<Tile>>>, commands: &mut Commands){

    for (x, row) in world.iter().enumerate() {
        for (y, tile) in row.iter().enumerate() {
            if let Some (tile) = tile {
                println!("x: {:?}, y: {:?}, tile: {:?}", x, y, tile);
                let tile_color = get_color(tile.clone());
                let content_color = get_content_color(tile.clone());
            

                // Create a base sprite for the tile
                commands.spawn(SpriteBundle {
                    sprite: Sprite {
                        color: tile_color, // Use the tile color
                        custom_size: Some(Vec2::new(TILE_SIZE, TILE_SIZE)),
                        ..Default::default()
                    },
                    transform: Transform::from_xyz(
                        x as f32 * TILE_SIZE, // X position with an offset
                        y as f32 * TILE_SIZE, // Y position with an offset
                        0.0,
                    ),
                    ..Default::default()
                }).insert(RenderLayers::layer(0));

                // Optionally spawn an additional sprite for the content if it's not None
                if tile.content != Content::None {
                    commands.spawn(SpriteBundle {
                        sprite: Sprite {
                            color: content_color, // Use the content color
                            custom_size: Some(Vec2::new(TILE_SIZE / 3.0, TILE_SIZE / 3.0)), // Smaller than the tile for distinction
                            ..Default::default()
                        },
                        transform: Transform::from_xyz(
                            x as f32 * TILE_SIZE, // Centered on the tile
                            y as f32 * TILE_SIZE, // Centered on the tile
                            1.0, // Slightly above the tile layer
                        ),
                        ..Default::default()
                    }).insert(RenderLayers::layer(0));
                }
            }
        }
    }
}

#[derive(Component)]
struct ZoomIn;

#[derive(Component)]
struct ZoomOut;

#[derive(Component)]
struct DropdownMenu;

#[derive(Component)]
struct Label;

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
        ),
        (Changed<Interaction>, With<Button>),
    >,
    mut text_query: Query<&mut Text>,
    mut camera_query: Query<&mut Transform, With<MainCamera>>,
    mut label_query: Query<&mut Style, With<Label>>,
){
    for (interaction, mut color, mut border_color, children,zoomin,zoomout,dropdown) in &mut interaction_query {
        let mut text = text_query.get_mut(children[0]).unwrap();
        match *interaction {
            Interaction::Pressed => {
                if zoomin.is_some() {
                    for mut projection in camera_query.iter_mut() {
                        projection.scale += 0.03;
                        println!("Current zoom scale: {}", projection.scale);
                    }
                } if zoomout.is_some() {
                    for mut projection in camera_query.iter_mut() {
                        projection.scale -= 0.03;
                        println!("Current zoom scale: {}", projection.scale);
                    }
                }else if dropdown.is_some() {
                    for mut node_style in label_query.iter_mut() {
                        if node_style.display == Display::None {
                            node_style.display = Display::Flex; // Cambia da None a Flex
                        } else {
                        node_style.display = Display::None; // Cambia da None a Flex
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

//TODOOOOOOOOOOOOOO CREARE MINIMAPPA, VIEWPORT(?)
//NON WORKA 
//RIVEDERE PER CREARE MINIMAPPA 
/* fn setup_minimap_camera(mut commands: Commands) {
   
 
    commands.spawn((
        Camera2dBundle {
            projection: OrthographicProjection {
                scale: camera_scale.x.min(camera_scale.y), // usa la scala minore per garantire che l'intera mappa sia visibile
                ..Default::default()
            },
            transform: Transform::from_xyz(
                WORLD_SIZE.x / 2.0, // centra la telecamera sull'asse X
                WORLD_SIZE.y / 2.0, // centra la telecamera sull'asse Y
                100.0,             // posiziona la telecamera sopra la mappa
            ),
            // Configura il viewport per posizionare la minimappa nell'angolo in alto a sinistra
            camera: Camera {
                viewport: Some(Viewport {
                    physical_position: UVec2::new(0, 0), // in alto a sinistra
                    physical_size: UVec2::new(256, 256), // dimensioni della viewport della minimappa
                    ..Default::default()
                }),
                ..Default::default()
            },
            ..Default::default()
        },
        MyMinimapCamera,
    ));
}
 */
// ...


const NORMAL_BUTTON: Color = Color::rgb(0.15, 0.15, 0.15);
const HOVERED_BUTTON: Color = Color::rgb(0.25, 0.25, 0.25);
const PRESSED_BUTTON: Color = Color::rgb(0.35, 0.75, 0.35);



// Modifica setup_main_camera per accettare posizione x, y del punto rosso
fn setup_main_camera(commands: &mut Commands, x: f32, y: f32) {
    commands.spawn(Camera2dBundle {
        
        transform: Transform::from_xyz(x, y, 1.0) // Usa la posizione del punto rosso
        .with_scale(Vec3::splat(0.15)),
        ..Default::default()
    }).insert(MainCamera);
}

pub fn zoom_in(mut query: Query<&mut OrthographicProjection, With<Camera>>) {
    for mut projection in query.iter_mut() {
        projection.scale -= 100.0;

        println!("Current zoom scale: {}", projection.scale);
    }
}

//WORKA
/* fn setup_main_camera(mut commands: Commands) {
    commands.spawn((
        Camera2dBundle {
            transform: Transform::from_xyz(0.0, 0.0, 1.0)
                .with_scale(Vec3::splat(0.1)), // Modifica questo valore per ottenere l'area desiderata
            ..Default::default()
        },
        MainCamera,
    ));
} */

 



//WORKA
//MUOVE LA CAM CON LA KEYBOARD
/* fn camera_movement_system(
    keyboard_input: Res<Input<KeyCode>>,
    mut query: Query<&mut Transform, With<MainCamera>>,
) {
    // Scegli quanto spostare la telecamera per ogni pressione del tasto
    let move_speed = 2.0; 

    for mut transform in query.iter_mut() {
        // Muovi verso sinistra
        if keyboard_input.pressed(KeyCode::Left) {
            transform.translation.x -= move_speed;
        }
        // Muovi verso destra
        if keyboard_input.pressed(KeyCode::Right) {
            transform.translation.x += move_speed;
        }
        // Muovi verso il basso
        if keyboard_input.pressed(KeyCode::Down) {
            transform.translation.y -= move_speed;
        }
        // Muovi verso l'alto
        if keyboard_input.pressed(KeyCode::Up) {
            transform.translation.y += move_speed;
        }
    }
} */


//MOVIMENTO LIBERO KEYBOARD
//movimenti keyboard puntino rosso 
/* fn robot_movement_system(
    keyboard_input: Res<Input<KeyCode>>,
    mut query: Query<&mut Transform, With<Robot>>,
) {
    let move_speed = 2.0;

    for mut transform in query.iter_mut() {
        if keyboard_input.pressed(KeyCode::Left) {
            transform.translation.x -= move_speed;
            println!("Mosso a sinistra");
        }
        if keyboard_input.pressed(KeyCode::Right) {
            transform.translation.x += move_speed;
            println!("Destra");
        }
        if keyboard_input.pressed(KeyCode::Down) {
            transform.translation.y -= move_speed;
            println!("Basso");
        }
        if keyboard_input.pressed(KeyCode::Up) {
            transform.translation.y += move_speed;
            println!("Alto");
        }
    }
} */




//movimento del robot in base alla grandezza di una tile
fn robot_movement_system(
    mut commands: Commands,
    mut query: Query<&mut Transform, With<Roboto>>,
    tile_size: Res<TileSize>, // Utilizza la risorsa TileSize
    robot_resource: Res<RobotResource>,
    world: Res<MapResource>,
) {

    let world = world.0.lock().unwrap();
    update_show_tiles(&world, &mut commands);
    let resource = robot_resource.0.lock().unwrap();
    let tile_step = tile_size.tile_size; // Usa la dimensione del tile dalla risorsa

    println!(
        "Energy Level: {}\nRow: {}\nColumn: {}\nBackpack Size: {}\nBackpack Contents: {:?}\nCurrent Weather: {:?}\nNext Weather: {:?}\nTicks Until Change: {}",
        resource.energy_level,
        resource.coordinate_row,
        resource.coordinate_column,
        resource.bp_size,
        resource.bp_contents,
        resource.current_weather,
        resource.next_weather,
        resource.ticks_until_change
    );
    
    for mut transform in query.iter_mut() {
        transform.translation.y = tile_step * resource.coordinate_column as f32;
        transform.translation.x = tile_step * resource.coordinate_row as f32;
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



//serve per far muovere la camera sopra il puntino rosso, prenderndo le sue coordinate 
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
}



//WORKA
//CODICE DI PROVA
/* 
fn spawn_robot(mut commands: Commands){
    //spawna l'entitiÃ  con componente posizione e componente velocitÃ  settate 
    commands.spawn((Position{ x: 0.0, y: 0.0}, Velocity{x:1.0, y:1.0}));
}

//update della posizione di ogni entitÃ 
fn update_position(robot_state: Res<RobotState>, mut query: Query<(&Velocity, &mut Position)>){
    
    //entra solo se il robot sta runnando(true)
    if(robot_state.is_moving){
    for(velocity, mut position) in query.iter_mut(){
        position.x += velocity.x;
        position.y += velocity.y;

    }}

}

//printa la posizione di ogni entitÃ  
fn print_position(query: Query<(Entity, &Position)>){
    for (entity, position) in query.iter(){
        info!("Entity {:?} is at position {:?},", entity, position);
    }

} */

// fn main() {
    
// }



fn moviment(robot_data: Arc<Mutex<RobotInfo>>, map: Arc<Mutex<Vec<Vec<Option<Tile>>>>>){
    println!("Hello, world!");
    let audio = get_audio_manager();
    let background_music = OxAgSoundConfig::new_looped_with_volume("assets/audio/background.ogg", 2.0);

    let mut robot = Robottino {
        shared_map: map,
        shared_robot: robot_data,
        robot: Robot::new(),
        audio: audio,
        weather_tool: WeatherPredictionTool::new()
    };

    // world generator initialization
    let mut world_gen =
        ghost_amazeing_island::world_generator::WorldGenerator::new(WORLD_SIZE, false, 1, 1.1);
    // Runnable creation and start

    println!("Generating runnable (world + robot)...");
    match robot.audio.play_audio(&background_music) {
        Ok(_) => {},
        Err(e) => {
            eprintln!("Failed to play audio: {}", e);
            std::process::exit(1);
        }
    }
    let mut world_gen =
        ghost_amazeing_island::world_generator::WorldGenerator::new(WORLD_SIZE, false, 1, 1.1);
    let mut runner = Runner::new(Box::new(robot), &mut world_gen);
    println!("Runnable succesfully generated");
    //sleep 5 second
    sleep(std::time::Duration::from_secs(5));
    for _i in 0..10000 {
        let rtn = runner.as_mut().unwrap().game_tick();
        sleep(std::time::Duration::from_secs(1));
    }
     
}

//prova commit

struct RobotResource(Arc<Mutex<RobotInfo>>);
struct MapResource(Arc<Mutex<Vec<Vec<Option<Tile>>>>>);

impl bevy::prelude::Resource for RobotResource {}
impl bevy::prelude::Resource for MapResource {}

use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

// Definire una struttura di dati condivisa; ad esempio, una posizione condivisa (x, y)
#[derive(Debug)]
struct Position {
    x: i32,
    y: i32,
}

struct RobotInfo {
    energy_level: usize, // livello di energia del robot
    coordinate_row : usize, // posizione del robot
    coordinate_column : usize, // posizione del robot
    bp_size: usize, // dimensione dello zaino
    bp_contents: HashMap<Content, usize>, // contenuto dello zaino
    current_weather: Option<WeatherType>, // tempo attuale
    next_weather: Option<WeatherType>, // prossima previsione del tempo
    ticks_until_change: u32 // tempo per la prossima previsione del tempo2
}

fn main() {
    
    // Dati condivisi tra thread
    let robot_info= RobotInfo{
        energy_level: 1000,
        coordinate_row: 0,
        coordinate_column: 0,
        bp_size: 10,
        bp_contents: HashMap::new(),
        current_weather: None,
        next_weather: None,
        ticks_until_change: 0
    };
    
    let robot_data = Arc::new(Mutex::new(robot_info));
    let robot_data_clone = robot_data.clone();

    let map: Arc<Mutex<Vec<Vec<Option<Tile>>>>> = Arc::new(Mutex::new(vec![vec![None; WORLD_SIZE as usize]; WORLD_SIZE as usize]));
    let map_clone = map.clone();

    let moviment = thread::spawn(move || {
        moviment(robot_data, map);
    });

    let robot_resource = RobotResource(robot_data_clone);
    let map_resource = MapResource(map_clone);
    App::new()
    .init_resource::<RobotState>()// aggiunge la risorsa con default valure, usare per settare values (.insert_resource(RobotState{is_moving:true}))
    .init_resource::<RobotPosition>()//ricordarsi di metterlo quando si ha una risorsa 
    .insert_resource(TileSize{tile_size: 3.0}) //setta la risorsa tile per la grandezza di esso
    .insert_resource(robot_resource)
    .insert_resource(map_resource)
    .add_systems(Startup,setup)
    //.add_systems(Startup, setup_minimap_camera)
    .add_systems(Update, (robot_movement_system, update_robot_position, follow_robot_system, button_system,set_camera_viewports, update_minimap_outline)) //unpdate every frame
    .add_plugins(DefaultPlugins)
    .run();  

    moviment.join().unwrap();
}

struct Robottino {
    shared_robot: Arc<Mutex<RobotInfo>>,
    shared_map: Arc<Mutex<Vec<Vec<Option<Tile>>>>>,
    robot: Robot,
    audio: OxAgAudioTool,
    weather_tool: WeatherPredictionTool
}


impl Runnable for Robottino {
    fn process_tick(&mut self, world: &mut robotics_lib::world::World) {
        
        sleep(std::time::Duration::from_millis(30));
        //se l'energia e' sotto il 300, la ricarico
        if self.robot.energy.get_energy_level() < 300 {
            self.robot.energy = rust_and_furious_dynamo::dynamo::Dynamo::update_energy();
        }
        // weather_check(self);
        
        // sleep(std::time::Duration::from_millis(300));
        // bessie::bessie::road_paving_machine(self, world, Direction::Up, State::MakeRoad);
        DestroyZone.execute(world, self, Content::Rock(1));
        let a = self.get_backpack();
        print!("{:?}", a);
        
        //print coordinate
        let coordinates = self.get_coordinate();
        println!("{:?}", coordinates);
        robot_view(self, world);
        let tiles_option = cheapest_border(world, self);
        let map= robot_map(world);
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
        println!("{:?}", count);
        if let Some(tiles) = tiles_option {
            //manage the return stat of move to cheapest border
            let result = move_to_cheapest_border(world, self, tiles);
            if let Err((_tiles, error)) = result {
                println!("The robot cannot move due to a {:?}", error);
            }
        }
        //print coordinate
        let actual_energy = self.get_energy().get_energy_level();
        println!("{:?}", actual_energy);
        let coordinates = self.get_coordinate();
        println!("{:?}", coordinates);
        
        let mut shared_map = self.shared_map.lock().unwrap();
        if let Some(new_map) = robot_map(world) {
            *shared_map = new_map;
        }

        
        
        let mut shared_robot = self.shared_robot.lock().unwrap();
        shared_robot.current_weather = Some(look_at_sky(&world).get_weather_condition());
        if let Some((prediction, ticks)) = weather_check(self) {
            shared_robot.next_weather = Some(prediction);
            shared_robot.ticks_until_change = ticks; 
        }
        
        
    }
    
    fn handle_event(&mut self, event: robotics_lib::event::events::Event) {
        self.weather_tool.process_event(&event);
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


fn weather_check(robot: &Robottino) ->Option<(WeatherType, u32)> {
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