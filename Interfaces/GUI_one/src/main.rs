//TODO:
//SISTEMARE POSITIONING/CONTROLLO MATRICE
//MINIMAPPA
//BOTTONE ZOOM MAPPA
//INSERIRE TUTTE LE INFO(TEMPO, ENERGIA, COSTO, ECC..)
//DESPAWN CONTENT QUANDO SI PRENDONO/DISTRUGGONO 
//CREAZIONE BACKPACK/INSERIMENTO VISUALIZZA OGGETTI
//MODIFICA TILE QUANDO SI POSIZIONA LA ROCCIA


use bevy::{ecs::system::FunctionSystem, prelude::*,render::view::RenderLayers, render::texture, transform::commands, utils::petgraph::dot};
use robotics_lib::{world, world::{tile::{self, Tile, TileType, Content}, world_generator::Generator}};
use bevy::render::camera::Viewport;
use rand::Rng;
use bevy::window::WindowResized;
use bevy::core_pipeline::clear_color::ClearColorConfig;





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


#[derive(Component, Debug)]
struct Robot;

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
fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    // Matrice di esempio

//TODO: controllare se positioning si basa effettivamente sulla matrice
//TODO: cabiare dimenzione con l'utilizzo della risorsa tile
    let mut world_gen = ghost_amazeing_island::world_generator::WorldGenerator::new(300, false, 1, 1.1);
    let mut world =world_gen.gen().0;
    let square_size = 3.0; // Dimensione di ogni quadrato
    let spacing = 3.0; // Spaziatura tra i quadrati
    //sotto funzione per telecamera
    //commands.spawn(Camera2dBundle::default()); 

    for (y, row) in world.iter().enumerate() {
        for (x, tile) in row.iter().enumerate() {
            let tile_color = get_color(tile.clone());
            let content_color = get_content_color(tile.clone());

            // Create a base sprite for the tile
            commands.spawn(SpriteBundle {
                sprite: Sprite {
                    color: tile_color, // Use the tile color
                    custom_size: Some(Vec2::new(square_size, square_size)),
                    ..Default::default()
                },
                transform: Transform::from_xyz(
                    x as f32 * spacing - 300.0, // X position with an offset
                    y as f32 * spacing - 300.0, // Y position with an offset
                    0.0,
                ),
                ..Default::default()
            }).insert(RenderLayers::layer(0));

            // Optionally spawn an additional sprite for the content if it's not None
            if tile.content != Content::None {
                commands.spawn(SpriteBundle {
                    sprite: Sprite {
                        color: content_color, // Use the content color
                        custom_size: Some(Vec2::new(square_size / 3.0, square_size / 3.0)), // Smaller than the tile for distinction
                        ..Default::default()
                    },
                    transform: Transform::from_xyz(
                        x as f32 * spacing - 303.0 + square_size, // Centered on the tile
                        y as f32 * spacing - 303.0 + square_size, // Centered on the tile
                        1.0, // Slightly above the tile layer
                    ),
                    ..Default::default()
                }).insert(RenderLayers::layer(0));
            }
        }
    }


    

    // per la posizione centrale della tile
    let center_x = world_gen.gen().1.1 as f32 * spacing - 300.0 + square_size;
    let center_y = world_gen.gen().1.0 as f32 * spacing - 300.0 + square_size;

    let robot_size = 2.0;
    let sunny: Handle<Image> = asset_server.load("img/sunny.png");

        //spawna il robot
    commands.spawn(SpriteBundle {
        sprite: Sprite {
            color: Color::RED,
            custom_size: Some(Vec2::new(robot_size, robot_size)),
            ..Default::default()
        },
        transform: Transform::from_xyz(center_x, center_y, 2.0), // asse z serve per metterlo sopra i tile e i conent
        ..Default::default()
    }).insert(Robot)
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
                        world_gen.gen().2.get_time_of_day_string() + "\n", // Assumendo che questo generi il testo desiderato
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
            transform: Transform::from_xyz(center_x, center_y, 1.0) // Usa la posizione del punto rosso
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
    let world_width: f32 = world.len() as f32 * spacing;
    let world_height = world[0].len() as f32 * spacing;

    // Calcola il centro del mondo
    let world_center_x = world_width / 2.0 - 300.0; // Assumi che 300 sia l'offset usato
    let world_center_y = world_height / 2.0 - 300.0;

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
                world_size.x / 2.0, // centra la telecamera sull'asse X
                world_size.y / 2.0, // centra la telecamera sull'asse Y
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


// Funzione per stampare la posizione del tile in cui si trova il robot
fn print_robot_tile_position(robot_position: &Transform, tile_size: f32) {
    // Calcola le coordinate del tile dividendo le coordinate del robot per la dimensione del tile
    // e aggiungendo l'offset se necessario per allinearsi con l'origine della griglia dei tile
    let tile_x = ((robot_position.translation.x + 300.0) / tile_size).floor() as i32;
    let tile_y = ((robot_position.translation.y + 300.0) / tile_size).floor() as i32;

    println!("Il robot si trova nel tile: ({}, {})", tile_x, tile_y);
}



//movimento del robot in base alla grandezza di una tile
fn robot_movement_system(
    keyboard_input: Res<Input<KeyCode>>,
    mut query: Query<&mut Transform, With<Robot>>,
    tile_size: Res<TileSize>, // Utilizza la risorsa TileSize
) {
    let tile_step = tile_size.tile_size; // Usa la dimensione del tile dalla risorsa

    for mut transform in query.iter_mut() {
        let mut moved = false;
        if keyboard_input.just_pressed(KeyCode::Left) {
            transform.translation.x -= tile_step;
            moved = true;
        } else if keyboard_input.just_pressed(KeyCode::Right) {
            transform.translation.x += tile_step;
            moved = true;
        } else if keyboard_input.just_pressed(KeyCode::Down) {
            transform.translation.y -= tile_step;
            moved = true;
        } else if keyboard_input.just_pressed(KeyCode::Up) {
            transform.translation.y += tile_step;
            moved = true;
        }
        if moved{
            print_robot_tile_position(&transform, tile_size.tile_size);
        }
    }
}

//NON WORKA PERCHE' LETTURA E SCRITTURA NELLO STESSO FRAME, LEGGERE POSIZIONE IN UNA FUNZIONE E SCRIVERE IN UN'ALTRA
//SEPARARE:
/* fn follow_robot_system(
    mut camera_query: Query<&mut Transform, With<MainCamera>>,
    robot_query: Query<&Transform, With<Robot>>,
) {
    // Ottiene la Transform del robot
    if let Ok(robot_transform) = robot_query.get_single() {
        // Ottiene la Transform della camera
        if let Ok(mut camera_transform) = camera_query.get_single_mut() {
            // Aggiorna la posizione della camera per centrarla sul robot
            camera_transform.translation.x = robot_transform.translation.x;
            camera_transform.translation.y = robot_transform.translation.y;
            // Mantiene la camera in una posizione elevata per garantire una vista top-down
            camera_transform.translation.z = 10.0; // Assicurati che questo sia abbastanza alto da vedere il robot
        }
    }
}
 */


 //TODO: CAMBIARE LA POSIZIONE DEL ROBOT BASANDOLA SULLE RIGHE E COLONNE DELLA MATRICE 
 //serve per avere la posizione del puntino rosso ad ogni movimento
 fn update_robot_position(
    mut robot_position: ResMut<RobotPosition>,
    robot_query: Query<&Transform, With<Robot>>,
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

fn main() {
    App::new()
        .init_resource::<RobotState>()// aggiunge la risorsa con default valure, usare per settare values (.insert_resource(RobotState{is_moving:true}))
        .init_resource::<RobotPosition>()//ricordarsi di metterlo quando si ha una risorsa 
        .insert_resource(TileSize{tile_size: 3.0}) //setta la risorsa tile per la grandezza di esso
        .add_systems(Startup,setup)
        //.add_systems(Startup, setup_minimap_camera)
        .add_systems(Update, (robot_movement_system, update_robot_position, follow_robot_system, button_system,set_camera_viewports, update_minimap_outline)) //unpdate every frame
        .add_plugins(DefaultPlugins)
        .run();     
}