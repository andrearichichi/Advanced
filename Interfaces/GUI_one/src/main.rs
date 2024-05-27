/* CONTROLLARE  TRANSFORM DI QUESTO CODICE
 commands
        .spawn(SpriteBundle {
            sprite: Sprite {
                color: Color::GRAY,
                custom_size: Some(Vec2::new(robot_size, robot_size)),
                // Flip the logo to the left
           // flip_x: true,
            // And don't flip it upside-down ( the default )
           // flip_y: false,
                ..Default::default()
            },
            texture: texture_robot_handle,
            transform: Transform::from_xyz(
                TILE_SIZE * resource.coordinate_row as f32,
                TILE_SIZE * resource.coordinate_row as f32,
                115.0,
            ), // asse z serve per metterlo sopra i tile e i conent
            ..Default::default()
        })
        .insert(Roboto)
        .insert(RenderLayers::layer(2))
        .insert(Explode) */

use bevy::core_pipeline::clear_color::ClearColorConfig;
use bevy::log;
use bevy::render::camera::Viewport;
use bevy::render::render_resource::encase::rts_array::Length;
use bevy::text;
use bevy::window::PrimaryWindow;
use bevy::window::WindowMode;
use bevy::window::WindowResized;
use bevy::{app::AppExit, prelude::*, render::view::RenderLayers};
use op_map::op_pathfinding::{
    get_best_action_to_element, OpActionInput, OpActionOutput, ShoppingList,
};
use tiles_stats::tiles_stats::discovered_tiles_stats;
use nearest_tp::nearest_tp::{
    nearest_teleport};
mod utils;
use utils::{nearest_tile_type, go_to_coordinate};
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
use std::f64::consts::E;
use std::thread::sleep;
use std::{collections::HashMap, ops::Range};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::atomic::AtomicU64;

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

#[derive(Resource, Default)]
struct FontResource {
    font_handle: Handle<Font>,
    font_handle1: Handle<Font>,
    font_handle2: Handle<Font>,
    font_handle3: Handle<Font>,
}

#[derive(Resource, Default)]
struct ContentTracker {
    counts: HashMap<Content, usize>,           // Mappa attuale dei conteggi per tipo di contenuto
    old_values: HashMap<Content, usize>,      // Mappa dei vecchi valori per tipo di contenuto
    deposited_counts: HashMap<Content, usize> // Mappa delle quantità depositate per tipo di contenuto
}

#[derive(Resource, Default)]
struct OldValueBackPack{
    value: usize,
}


#[derive(Resource)]
struct PreviousX(f32);

#[derive(Resource)]
struct ContentCounter {
    pub count: usize,
}

#[derive(Component)]
struct TagItem {
    item_type: Content,
}

#[derive(Component)]
struct PopupLabel;

#[derive(Component)]
struct PopupLabelText;


//PRIVATE
//use robotics_lib::interface::PLOT;

//NON SI PUò USARE PERCHE PLOT è PRIVATO
/* fn reset_plot() {
    if let Ok(mut plot_guard) = PLOT.lock() {
        plot_guard.clear(); // Clears all entries from the vector
    } else {
        // Handle the error case where the lock could not be acquired
        eprintln!("Failed to lock PLOT for resetting");
    }
} */

const MIN_ZOOM: f32 = 0.05; 
const MAX_ZOOM: f32 = 1.0; //1.0 se 150, 0.25 se 250

const WORLD_SIZE: u32 = 75; //A 200 TROVA IL MAZE
const TILE_SIZE: f32 = 3.0; //LASCIARE A 3!


#[derive(Component, Debug)]
struct WorldSize{
    size: u32,
}


#[derive(Component, Debug)]
struct Roboto;

#[derive(Resource)]
struct CameraFollow {
    follow_robot: bool,
}

#[derive(Resource, Default)]
struct CameraControl {
    is_zooming: bool,
}


#[derive(Component, Debug)]
struct Explode;

#[derive(Component, Debug)]
struct Explodetry;


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
struct TagCoordinate;

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

#[derive(Default, Resource)]
struct TileIcons {
    deepWater: Handle<Image>,
    grass: Handle<Image>,
    hill: Handle<Image>,
    lava: Handle<Image>,
    mountain: Handle<Image>,
    sand: Handle<Image>,
    shallowWater: Handle<Image>,
    snow: Handle<Image>,
    street: Handle<Image>,
    teleport: Handle<Image>,
    wall: Handle<Image>,
}

#[derive(Default, Resource)]
struct ContentIcons {
    rock: Handle<Image>,
    tree: Handle<Image>,
    garbage: Handle<Image>,
    fire: Handle<Image>,
    coin: Handle<Image>,
    water: Handle<Image>,
    bin: Handle<Image>,
    c_crate: Handle<Image>,
    bank: Handle<Image>,
    market: Handle<Image>,
    fish: Handle<Image>,
    building: Handle<Image>,
    bush: Handle<Image>,
    jollyBlock: Handle<Image>,
    scarecrow: Handle<Image>,
}
#[derive(Resource, Debug)]
struct ButtonIcons {
    increase: Handle<Image>,
    decrease: Handle<Image>,
    play: Handle<Image>,
    pause: Handle<Image>,
    teleport: Handle<Image>,
    teleportactive: Handle<Image>,
    actionin: Handle<Image>,
    actionout: Handle<Image>,
}

fn load_texture_tile_assets(commands: &mut Commands, asset_server: &Res<AssetServer>) {

    let tile_icons = TileIcons{
    deepWater: asset_server.load("img/DeepWater.png"),
    grass: asset_server.load("img/Grass.png"),
    hill: asset_server.load("img/Hill.png"),
    lava: asset_server.load("img/Lava.png"),
    mountain: asset_server.load("img/Mountain.png"),
    sand: asset_server.load("img/Sand.png"),
    shallowWater: asset_server.load("img/ShallowWater.png"),
    snow: asset_server.load("img/Snow1.png"),
    street: asset_server.load("img/Street.png"),
    teleport: asset_server.load("img/Teleport.png"),
    wall: asset_server.load("img/Wall.png"),

};

    commands.insert_resource(tile_icons);
}

fn load_texture_content_assets(commands: &mut Commands, asset_server: &Res<AssetServer>) {

    let content_icons = ContentIcons{

    rock: asset_server.load("img/Rock.png"),
    tree: asset_server.load("img/Tree.png"),
    garbage: asset_server.load("img/Trash.png"),
    fire: asset_server.load("img/Fire.png"),
    coin: asset_server.load("img/Coin.png"),
    water: asset_server.load("img/WaterObject.png"),
    bin: asset_server.load("img/Bin.png"),
    c_crate: asset_server.load("img/Crate.png"),
    bank: asset_server.load("img/Bank.png"),
    market: asset_server.load("img/Market.png"),
    fish: asset_server.load("img/Fish.png"),
    building: asset_server.load("img/Building.png"),
    bush: asset_server.load("img/Bush.png"),
    jollyBlock: asset_server.load("img/JollyBlock.png"),
    scarecrow: asset_server.load("img/ScareCrow.png"),

    };

    commands.insert_resource(content_icons);

}





fn get_tile_icons(tile: &Tile, tile_icons: &TileIcons ) -> Handle<Image> {
    match tile.tile_type {
        TileType::DeepWater => tile_icons.deepWater.clone(), // Blu Scuro
        TileType::Grass => tile_icons.grass.clone(),     // Verde Vivo
        TileType::Hill => tile_icons.hill.clone(),      // Verde Chiaro
        TileType::Lava => tile_icons.lava.clone(),      // Arancione-Rosso
        TileType::Mountain => tile_icons.mountain.clone(),  // Marrone
        TileType::Sand => tile_icons.sand.clone(),      // Sabbia
        TileType::ShallowWater => tile_icons.shallowWater.clone(), // Azzurro
        TileType::Snow => tile_icons.snow.clone(),      // Bianco
        TileType::Street => tile_icons.street.clone(),    // Grigio Scuro
        TileType::Teleport(_) => tile_icons.teleport.clone(), // Viola
        TileType::Wall => tile_icons.wall.clone(),      // Rosso Scuro
    }
}


fn get_content_icons(content: &Tile, content_icons: &ContentIcons) -> Option<Handle<Image>> {
    match content.content {
        Content::Rock(_) => Some(content_icons.rock.clone()), // Light Steel Blue
        Content::Tree(_) => Some(content_icons.tree.clone()), // Dark Green
        Content::Garbage(_) => Some(content_icons.garbage.clone()), // Saddle Brown
        Content::Fire => Some(content_icons.fire.clone()),    // Orange Red
        Content::Coin(_) => Some(content_icons.coin.clone()), // Gold
        Content::Water(_) => Some(content_icons.water.clone()), // Sky Blue
        Content::Bin(_) => Some(content_icons.bin.clone()),  // Slate Gray
        Content::Crate(_) => Some(content_icons.c_crate.clone()), // Beige
        Content::Bank(_) => Some(content_icons.bank.clone()), // Dollar Bill
        Content::Market(_) => Some(content_icons.market.clone()), // Firebrick
        Content::Fish(_) => Some(content_icons.fish.clone()), // Aqua
        Content::Building => Some(content_icons.building.clone()), // Purple
        Content::Bush(_) => Some(content_icons.bush.clone()), // Light Green
        Content::JollyBlock(_) => Some(content_icons.jollyBlock.clone()), // Pink
        Content::Scarecrow => Some(content_icons.scarecrow.clone()), // Orange
        Content::None => None,                       
    }
}


//setup main menu
fn initial_menu_setup(mut commands: Commands, asset_server: Res<AssetServer>) {

    let font_handle2: Handle<Font> = asset_server.load("img/LondrinaSketch-Regular.ttf");
    let font_handle3: Handle<Font> = asset_server.load("img/Lobster-Regular.ttf");

    
    // Qui vai a definire lo stile dei bottoni e il testo, simile a quanto fatto in main_menu_setup
    // Common style for all buttons on the screen
     let button_style = Style {
        width: Val::Px(250.0),
        height: Val::Px(65.0),
        margin: UiRect::all(Val::Px(20.0)),
        justify_content: JustifyContent::Center,
        align_items: AlignItems::Center,
        ..default()
    };
    let button_icon_style = Style {
        width: Val::Px(50.0),
        // This takes the icons out of the flexbox flow, to be positioned exactly
        position_type: PositionType::Absolute,
        // The icon will be close to the left border of the button
        left: Val::Px(10.0),
        ..default()
    };
    let button_text_style = TextStyle {
        font: font_handle3,
        font_size: 40.0,
        color: Color::rgb(0.9, 0.9, 0.9),
        ..default()
    };

commands
    .spawn((
        NodeBundle {
            style: Style {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                ..default()
            },
            ..default()
        },
        OnMainMenuScreen,
    ))
    .with_children(|parent| {
        parent
            .spawn(NodeBundle {
                style: Style {
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::Center,
                    ..default()
                },
                background_color: Color::CRIMSON.into(),
                ..default()
            })
            .with_children(|parent| {
                // Display the game name
                parent.spawn(
                    TextBundle::from_section(
                        "AntitRust Project",
                        TextStyle {
                            font: font_handle2,
                            font_size: 80.0,
                            color: Color::rgb(0.9, 0.9, 0.9),
                            ..default()
                        },
                    )
                    .with_style(Style {
                        margin: UiRect::all(Val::Px(50.0)),
                        ..default()
                    }),
                );

                // Display three buttons for each action available from the main menu:
                // - new game
                // - settings
                // - quit
                parent
                    .spawn((
                        ButtonBundle {
                            style: button_style.clone(),
                            background_color: NORMAL_BUTTON.into(),
                            ..default()
                        },
                        MenuButtonAction::AI1,
                    ))
                    .with_children(|parent| {
                        let icon = asset_server.load("img/menu_tree_robot.png");
                        parent.spawn(ImageBundle {
                            style: button_icon_style.clone(),
                            image: UiImage::new(icon),
                            ..default()
                        });
                        parent.spawn(TextBundle::from_section(
                            "AI_1",
                            button_text_style.clone(),
                        ));
                    });
                parent
                    .spawn((
                        ButtonBundle {
                            style: button_style.clone(),
                            background_color: NORMAL_BUTTON.into(),
                            ..default()
                        },
                        MenuButtonAction::AI2,
                    ))
                    .with_children(|parent| {
                        let icon = asset_server.load("img/menu_rock_robot.png");
                        parent.spawn(ImageBundle {
                            style: button_icon_style.clone(),
                            image: UiImage::new(icon),
                            ..default()
                        });
                        parent.spawn(TextBundle::from_section(
                            "AI_2",
                            button_text_style.clone(),
                        ));
                    });
                parent
                    .spawn((
                        ButtonBundle {
                            style: button_style.clone(),
                            background_color: NORMAL_BUTTON.into(),
                            ..default()
                        },
                        MenuButtonAction::AI3,
                    ))
                    .with_children(|parent| {
                        let icon = asset_server.load("img/menu_maze_robot.png");
                        parent.spawn(ImageBundle {
                            style: button_icon_style.clone(),
                            image: UiImage::new(icon),
                            ..default()
                        });
                        parent.spawn(TextBundle::from_section("AI_3", button_text_style.clone()));
                    });

                    parent
                    .spawn((
                        ButtonBundle {
                            style: button_style.clone(),
                            background_color: NORMAL_BUTTON.into(),
                            ..default()
                        },
                        MenuButtonAction::UberAI,
                    ))
                    .with_children(|parent| {
                        let icon = asset_server.load("img/menu_full_robot.png");
                        parent.spawn(ImageBundle {
                            style: button_icon_style.clone(),
                            image: UiImage::new(icon),
                            ..default()
                        });
                        parent.spawn(TextBundle::from_section("Uber_AI", button_text_style.clone()));
                    });

                    parent
                    .spawn((
                        ButtonBundle {
                            style: button_style.clone(),
                            background_color: NORMAL_BUTTON.into(),
                            ..default()
                        },
                        MenuButtonAction::Exit,
                    ))
                    .with_children(|parent| {
                        let icon = asset_server.load("img/exitRight.png");
                        parent.spawn(ImageBundle {
                            style: button_icon_style.clone(),
                            image: UiImage::new(icon),
                            ..default()
                        });
                        parent.spawn(TextBundle::from_section("EXIT", button_text_style));
                    });
            });
            
    });


    commands.spawn(Camera2dBundle {
        camera: Camera {
            order: 0,
            ..default()
        },
        ..default()
    })
    .insert(OnMainMenuScreen);
}


// Funzione di setup che crea la scena
fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    shared_map: Res<MapResource>,
    robot_resource: Res<RobotResource>,
) {
    
    commands.insert_resource(ContentCounter { count: 0 });
    commands.insert_resource(ContentTracker {
        counts: HashMap::new(),
        old_values: HashMap::new(),
        deposited_counts: HashMap::new(), // Add the missing field 'deposited_counts'
    });

    let start_time = Instant::now() - Duration::from_secs(5);
    commands.insert_resource(LastUpdate(start_time));


    load_texture_content_assets(&mut commands, &asset_server);
    load_texture_tile_assets(&mut commands, &asset_server);

    commands.insert_resource(CameraFollow { follow_robot: true });
    commands.insert_resource(CameraControl { is_zooming: false });

   // sleep(std::time::Duration::from_secs(3));
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

    let texture_border1_handle: Handle<Image> = asset_server.load("img/border1.png");
    let texture_border2_handle: Handle<Image> = asset_server.load("img/border2.png");
    let texture_border3_handle: Handle<Image> = asset_server.load("img/border3.png");

    let texture_decrease_handle: Handle<Image> = asset_server.load("img/decrease.png");
    let texture_zoom_in_handle:Handle<Image> = asset_server.load("img/ZoomIn.png");
    let texture_zoom_out_handle:Handle<Image> = asset_server.load("img/ZoomOut.png");

    let button_icons = ButtonIcons {
        increase: asset_server.load("img/increase.png"),
        decrease: asset_server.load("img/decrease.png"),
        play: asset_server.load("img/pause.png"),
        pause: asset_server.load("img/play.png"),
        teleport: asset_server.load("img/TeleportButton.png"),
        teleportactive: asset_server.load("img/TeleportButtonActive.png"),
        actionin: asset_server.load("img/ActionActive.png"),
        actionout: asset_server.load("img/ActionNotActive.png"),
    };


    commands.insert_resource(ButtonIcons {
        increase: button_icons.increase.clone(),
        decrease: button_icons.decrease.clone(),
        play: button_icons.play.clone(),
        pause: button_icons.pause.clone(),
        teleport: button_icons.teleport.clone(),
        teleportactive: button_icons.teleportactive.clone(),
        actionin: button_icons.actionin.clone(),
        actionout: button_icons.actionout.clone(),
    });
/* 
    let texture_increase_handle: Handle<Image> = asset_server.load("img/increase.png");
    let texture_play_handle: Handle<Image> = asset_server.load("img/pause.png");
    let texture_pause_handle: Handle<Image> = asset_server.load("img/play.png");
*/

    let texture_robot_handle: Handle<Image> = asset_server.load("img/Robot.png");
    let texture_battery_handle: Handle<Image> = asset_server.load("img/EmptyBattery.png");

    let content_icons = ContentIcons{

        rock: asset_server.load("img/Rock.png"),
        tree: asset_server.load("img/Tree.png"),
        garbage: asset_server.load("img/Trash.png"),
        fire: asset_server.load("img/Fire.png"),
        coin: asset_server.load("img/Coin.png"),
        water: asset_server.load("img/WaterObject.png"),
        bin: asset_server.load("img/Bin.png"),
        c_crate: asset_server.load("img/Crate.png"),
        bank: asset_server.load("img/Bank.png"),
        market: asset_server.load("img/Market.png"),
        fish: asset_server.load("img/Fish.png"),
        building: asset_server.load("img/Building.png"),
        bush: asset_server.load("img/Bush.png"),
        jollyBlock: asset_server.load("img/JollyBlock.png"),
        scarecrow: asset_server.load("img/ScareCrow.png"),
    
        };
    
    let font_handle: Handle<Font> = asset_server.load("img/Jacquard12-Regular.ttf");
    let font_handle1: Handle<Font> = asset_server.load("img/Jersey25Charted-Regular.ttf");
    let font_handle2: Handle<Font> = asset_server.load("img/AmaticSC-Bold.ttf");
    let font_handle3: Handle<Font> = asset_server.load("img/SpecialElite-Regular.ttf");

    commands.insert_resource(FontResource {
        font_handle: font_handle.clone(),
        font_handle1: font_handle1.clone(),
        font_handle2: font_handle2.clone(),
        font_handle3: font_handle3.clone(),
    });
       
    //sleep 3 secondi
    //sleep(std::time::Duration::from_secs(10));
    let world1 = shared_map.0.lock().unwrap();
    let resource1 = robot_resource.0.lock().unwrap();
    let world = world1.clone();
    let resource = resource1.clone();
    drop(world1);
    drop(resource1);

    let mut count = 0;

    
    for row in world.iter() {
        for tile in row.iter() {
            if tile.is_some() {
                count += 1;
            }
        }
    }
    // println!("count mappaa viosualizzaaaaa {:?}", count);
    /* let mut old_map = OldMapResource {
        //world: vec![vec![None; WORLD_SIZE as usize]; WORLD_SIZE as usize],
        world: vec![vec![None; WORLD_SIZE as usize]; WORLD_SIZE as usize],
    }; */

    commands.spawn(()).insert(OldMapResource {
        world: vec![vec![None; WORLD_SIZE as usize]; WORLD_SIZE as usize],
    })
    .insert(Explodetry);

    // println!("Robot {:?} {:?}",resource.coordinate_column, resource.coordinate_row);
    // update_show_tiles(&world, &mut commands, &mut old_map.world);
    //commands.insert_resource(old_map);



    let robot_size = 2.0;
    commands.insert_resource(PreviousX(resource.coordinate_column as f32));

    //spawna il robot
    commands
        .spawn(SpriteBundle {
            sprite: Sprite {
                color: Color::GRAY,
                custom_size: Some(Vec2::new(robot_size, robot_size)),
                // Flip the logo to the left
           // flip_x: true,
            // And don't flip it upside-down ( the default )
           // flip_y: false,
                ..Default::default()
            },
            texture: texture_robot_handle,
            transform: Transform::from_xyz(
                TILE_SIZE * resource.coordinate_row as f32,
                TILE_SIZE * resource.coordinate_row as f32,
                15.0,
            ), // asse z serve per metterlo sopra i tile e i conent
            ..Default::default()
        })
        .insert(Roboto)
        .insert(RenderLayers::layer(2))
        .insert(Explode);

    //BUTTONS RIGHT
    commands
        .spawn(NodeBundle {
            style: Style {
                // Imposta le dimensioni del rettangolo
                width: Val::Px(250.0),
                height: Val::Px(100.0),
                
                // Posiziona il rettangolo in basso e al centro
                position_type: PositionType::Absolute,
                bottom: Val::Px(25.0), // Distanza dal fondo dello schermo
                left: Val::Percent(80.0), // Centra orizzontalmente
                border: UiRect::all(Val::Px(4.0)),
                // Traslazione di -50% della propria larghezza per centrare esattamente
                //translate: Transform::from_xyz(-100.0, 0.0, 0.0), 
                
                // Assicurati che il contenuto sia centrato (se ne hai)
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
    
                ..default()
            },
            border_color: BorderColor(Color::BLACK),
            background_color: Color::rgba(1.0, 1.0, 1.0, 0.5).into(), // Colore bianco con trasparenza
            ..default() 
            })
        .insert(Explode)
        .with_children(|parent| {
            // Primo bottone
            parent
                .spawn(ButtonBundle {
                    style: Style {
                        width: Val::Px(50.0),
                        height: Val::Px(50.0),
                        margin: UiRect::all(Val::Px(10.0)), 
                       // border: UiRect::all(Val::Px(4.0)),
                        justify_content: JustifyContent::Center, 
                        align_items: AlignItems::Center,

                        ..default()
                    },
                    border_color: BorderColor(Color::BLACK),
                    background_color: NORMAL_BUTTON.into(),
                    image: texture_zoom_out_handle.into(),
                    ..default()
                })
                .with_children(|parent| {
                   /*  parent.spawn(TextBundle::from_section(
                        "-",
                        TextStyle {
                            
                            font_size: 25.0,
                            color: Color::rgb(0.9, 0.9, 0.9),
                            ..default()
                        },
                    )); */
                })
                .insert(ZoomIn);
               

            // Secondo bottone
            parent
                .spawn(ButtonBundle {
                    style: Style {
                        width: Val::Px(50.0),
                        height: Val::Px(50.0),
                        margin: UiRect::all(Val::Px(10.0)), 
                       // border: UiRect::all(Val::Px(4.0)),
                        justify_content: JustifyContent::Center, 
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    border_color: BorderColor(Color::BLACK),
                    background_color: NORMAL_BUTTON.into(),
                    image: texture_zoom_in_handle.clone().into(),
                    ..default()
                })
                .with_children(|parent| {
                    /* parent.spawn(TextBundle::from_section(
                        "+",
                        TextStyle {
                            font_size: 25.0,
                            color: Color::rgb(0.9, 0.9, 0.9),
                            ..default()
                        },
                    )); */
                })
                .insert(ZoomOut);
                

            //bottone chiusura
            parent
                .spawn(ButtonBundle {
                    style: Style {
                        width: Val::Px(60.0),
                        height: Val::Px(40.0),
                        margin: UiRect::all(Val::Px(10.0)), 
                        border: UiRect::all(Val::Px(4.0)),
                        justify_content: JustifyContent::Center, 
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
        .insert(RenderLayers::layer(1));

    //BUTTONS CENTER
    commands.spawn(NodeBundle {
        style: Style {
            // Imposta le dimensioni del rettangolo
            width: Val::Px(250.0),
            height: Val::Px(120.0),
            
            // Posiziona il rettangolo in basso e al centro
            position_type: PositionType::Absolute,
            bottom: Val::Px(25.0), // Distanza dal fondo dello schermo
            left: Val::Percent(40.0), // Centra orizzontalmente
            border: UiRect::all(Val::Px(4.0)),
            // Traslazione di -50% della propria larghezza per centrare esattamente
            //translate: Transform::from_xyz(-100.0, 0.0, 0.0), 
            
            // Assicurati che il contenuto sia centrato (se ne hai)
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,

            ..default()
        },
        border_color: BorderColor(Color::BLACK),
        background_color: Color::rgba(1.0, 1.0, 1.0, 0.5).into(), // Colore bianco con trasparenza
        ..default() 
        })
        .insert(Explode)
        .with_children(|parent| {


        //Bottone diminuisce velocità
        parent
        .spawn(ButtonBundle {
            style: Style {
                width: Val::Px(70.0),
                height: Val::Px(70.0),
                margin: UiRect::all(Val::Px(10.0)), 
               // border: UiRect::all(Val::Px(4.0)),
                justify_content: JustifyContent::Center, 
                align_items: AlignItems::Center,
                ..default()
            },
            border_color: BorderColor(Color::NONE),
            background_color: BackgroundColor(Color::BLACK),
            image: button_icons.decrease.clone().into(),
            ..default()
        })
        .with_children(|parent| {
            /* parent.spawn(ImageBundle {
                image: texture_decrease_handle.clone().into(),
                background_color: BackgroundColor(Color::WHITE),
                style: Style {
                    width: Val::Px(40.0),
                    height: Val::Px(40.0),
                    justify_content: JustifyContent::Center, 
                    align_items: AlignItems::Center,
                    ..default()
                },
                ..default()
              }); */
            })
            .insert(IncreaseSpeed);


        //BOTTONE STOP
        parent
        .spawn(ButtonBundle {
            style: Style {
                width: Val::Px(70.0),
                height: Val::Px(70.0),
                margin: UiRect::all(Val::Px(10.0)), 
                //border: UiRect::all(Val::Px(4.0)),
                justify_content: JustifyContent::Center, 
                align_items: AlignItems::Center,
                ..default()
            },
            border_color: BorderColor(Color::BLACK),
            background_color: BackgroundColor(Color::BLACK),
            image: button_icons.play.clone().into(),
            ..default()
        })
        .with_children(|parent| {
            /* parent.spawn(TextBundle::from_section(
                "STOP",
                TextStyle {
                    font_size: 25.0,
                    color: Color::rgb(0.9, 0.9, 0.9),
                    ..default()
                },
            ));*/
        }) 
        .insert(PauseButton);

        
     //Bottone aumenta velocità
     parent
     .spawn(ButtonBundle {
         style: Style {
             width: Val::Px(70.0),
             height: Val::Px(70.0),
             margin: UiRect::all(Val::Px(10.0)), 
           //  border: UiRect::all(Val::Px(4.0)),
             justify_content: JustifyContent::Center, 
             align_items: AlignItems::Center,
             ..default()
         },
         border_color: BorderColor(Color::BLACK),
         background_color: BackgroundColor(Color::BLACK),
         image: button_icons.increase.clone().into(),
         ..default()
     })
     .with_children(|parent| {
         /* parent.spawn(TextBundle::from_section(
             "SPEED +",
             TextStyle {
                 font_size: 25.0,
                 color: Color::rgb(0.9, 0.9, 0.9),
                 ..default()
             },
         )); */
     })
     .insert(DecreaseSpeed);

     
    })
      .insert(RenderLayers::layer(1));

    //BUTTONS CENTER RIGHT
    commands.spawn(NodeBundle {
        style: Style {
            // Imposta le dimensioni del rettangolo
            width: Val::Px(250.0),
            height: Val::Px(120.0),
            
            // Posiziona il rettangolo in basso e al centro
            position_type: PositionType::Absolute,
            bottom: Val::Px(25.0), // Distanza dal fondo dello schermo
            left: Val::Percent(60.0), // Centra orizzontalmente
            border: UiRect::all(Val::Px(4.0)),
            // Traslazione di -50% della propria larghezza per centrare esattamente
            //translate: Transform::from_xyz(-100.0, 0.0, 0.0), 
            
            // Assicurati che il contenuto sia centrato (se ne hai)
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,

            ..default()
        },
        border_color: BorderColor(Color::BLACK),
        background_color: Color::rgba(1.0, 1.0, 1.0, 0.5).into(), // Colore bianco con trasparenza
        ..default() 
        })
        .insert(Explode)
        .with_children(|parent| {

            //ACTIVITY BUTTON
            parent
        .spawn(ButtonBundle {
            style: Style {
                width: Val::Px(70.0),
                height: Val::Px(70.0),
                margin: UiRect::all(Val::Px(10.0)), 
                //border: UiRect::all(Val::Px(4.0)),
                justify_content: JustifyContent::Center, 
                align_items: AlignItems::Center,
                ..default()
            },
            border_color: BorderColor(Color::BLACK),
            background_color: BackgroundColor(Color::WHITE),
            image: button_icons.actionout.clone().into(),
            ..default()
        })
        .with_children(|parent| {
            /* parent.spawn(TextBundle::from_section(
                "STOP",
                TextStyle {
                    font_size: 25.0,
                    color: Color::rgb(0.9, 0.9, 0.9),
                    ..default()
                },
            ));*/
        }) 
        .insert(ActivityButton);

        //TELEPORT BUTTON
        parent
        .spawn(ButtonBundle {
            style: Style {
                width: Val::Px(70.0),
                height: Val::Px(70.0),
                margin: UiRect::all(Val::Px(10.0)), 
                //border: UiRect::all(Val::Px(4.0)),
                justify_content: JustifyContent::Center, 
                align_items: AlignItems::Center,
                ..default()
            },
            border_color: BorderColor(Color::BLACK),
            background_color: BackgroundColor(Color::WHITE),
            image: button_icons.teleport.clone().into(),
            ..default()
        })
        .with_children(|parent| {
            /* parent.spawn(TextBundle::from_section(
                "STOP",
                TextStyle {
                    font_size: 25.0,
                    color: Color::rgb(0.9, 0.9, 0.9),
                    ..default()
                },
            ));*/
        }) 
        .insert(TeleportButton);
    
        }).insert(RenderLayers::layer(1));
        

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
        .insert(Explode)
        .with_children(|parent| {
            parent
                .spawn(ButtonBundle {
                    style: Style {
                        width: Val::Px(20.0),
                        height: Val::Px(65.0),
                        border: UiRect::all(Val::Px(5.0)),                     
                        justify_content: JustifyContent::Center,
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
                        justify_content: JustifyContent::FlexStart, 
                        align_items: AlignItems::Center, 
                        flex_direction: FlexDirection::Column, 
                        display: Display::None, 
                        ..default()
                    },
                    visibility: Visibility::Visible,
                    border_color: BorderColor(Color::WHITE),
                    background_color: BackgroundColor(Color::rgba(0.3, 0.3, 0.3, 0.8)),
                    ..default()
                })
                .insert(Explode)
                .with_children(|parent| {
                    // TIME
                    parent
                        .spawn(TextBundle::from_section(
                            "Time \n", 
                            TextStyle {
                                font: font_handle3.clone(),
                                font_size: 25.0,
                                color: Color::WHITE,
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
                            image: UiImage::new(asset_server.load("img/sunny_day.png")),
                            ..default()
                        })
                        .insert(WeatherIcon);
                    //ENERGY 
                    parent
                        .spawn(TextBundle::from_section(
                            "ENERGY \n", 
                            TextStyle {
                                font: font_handle3.clone(),
                                font_size: 25.0,
                                color: Color::WHITE,
                                ..default()
                            },
                        ))
                        .insert(TagEnergy);

                    // BARRA DELL'ENERGIA
                    // All'interno del menu a tendina
                    parent.spawn(NodeBundle {
                        style: Style {
                            width: Val::Px(150.0),
                            height: Val::Px(150.0),
                            ..Default::default()
                        },
                      //  background_color: BackgroundColor::default(),  // Se l'immagine deve coprire tutto il nodo
                      //  border_color: BorderColor(Color::BLACK),
                        ..Default::default()
                    }).with_children(|parent| {
                        // Nodo figlio per l'immagine
                        parent.spawn(ImageBundle {
                            style: Style {
                                position_type: PositionType::Absolute,
                                //position: UiRect::all(Val::Auto),
                                width: Val::Percent(100.0),
                                height: Val::Percent(100.0),
                               // border: UiRect { left: Val::Percent(25.0), right: Val::Percent(25.0), top: Val::Percent(25.0), bottom: Val::Percent(25.0) },
                                ..Default::default()
                            },
                           // background_color: BackgroundColor(Color::WHITE),
                            image: texture_battery_handle.into(), 
                            z_index: ZIndex::Global(50), // Qui si imposta l'immagine di sfondo
                            ..Default::default()
                        });
                        
                        // Nodo interno per la barra dell'energia
                        parent.spawn(NodeBundle {
                            style: Style {
                                width: Val::Percent(40.0), // Inizia con il 50%, aggiusta dinamicamente
                                height: Val::Percent(48.0),
                                top: Val::Percent(25.0),
                                left: Val::Percent(2.0),
                                ..Default::default()
                            },
                            background_color: BackgroundColor(Color::GREEN),  // Usa un colore solido per la barra
                            ..Default::default()
                        }).insert(EnergyBar);
                    });

                    //COORDINATES
                    parent
                        .spawn(TextBundle::from_section(
                            "COORDINATES: \n", 
                            TextStyle {
                                font: font_handle3.clone(),
                                font_size: 25.0,
                                color: Color::WHITE,
                                ..default()
                            },
                        ))
                        .insert(TagCoordinate);
                })
                .insert(Label);
                
        });

    //menu' a tendina BACKPACK
    commands
        .spawn(NodeBundle {
            style: Style {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                justify_content: JustifyContent::FlexStart,
                align_items: AlignItems::FlexEnd,           
                ..default()
            },
            ..default()
        })
        .insert(Explode)
        .with_children(|parent| {
            parent
                .spawn(ButtonBundle {
                    style: Style {
                        width: Val::Px(120.0),
                        height: Val::Px(45.0),
                        border: UiRect::all(Val::Px(5.0)),       
                        justify_content: JustifyContent::Center, 
                        align_items: AlignItems::Center,
                        margin: UiRect {
                            left: Val::Px(10.0),   
                            bottom: Val::Px(10.0), 
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
                        width: Val::Px(250.0),
                        height: Val::Px(600.0),
                        border: UiRect::all(Val::Px(1.0)),
                        justify_content: JustifyContent::FlexStart, 
                        align_items: AlignItems::FlexStart, 
                        flex_direction: FlexDirection::Column, 
                        display: Display::None, 
                        margin: UiRect {
                            left: Val::Px(-120.0), 
                            bottom: Val::Px(55.0), 
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
                    let item_types = [
                        (Content::Rock(0), &content_icons.rock), // Assumo 1 come quantità di default
                        (Content::Tree(0), &content_icons.tree),
                        (Content::Garbage(0), &content_icons.garbage),
                        (Content::Fire, &content_icons.fire), // Fire non ha parametri
                        (Content::Coin(0), &content_icons.coin),
                        (Content::Water(0), &content_icons.water),
                        (Content::Bin(0..5), &content_icons.bin), // Assumo un range come esempio
                        (Content::Crate(0..5), &content_icons.c_crate),
                        (Content::Bank(0..5), &content_icons.bank),
                        (Content::Market(0), &content_icons.market),
                        (Content::Fish(0), &content_icons.fish),
                        (Content::Building, &content_icons.building), // Building non ha parametri
                        (Content::Bush(0), &content_icons.bush),
                        (Content::JollyBlock(0), &content_icons.jollyBlock),
                        (Content::Scarecrow, &content_icons.scarecrow), // Scarecrow non ha parametri
                    ];
            
                    for (name, icon) in item_types {
                       let button = parent.spawn(NodeBundle {
                            style: Style {
                                flex_direction: FlexDirection::Row,
                                align_items: AlignItems::Center,
                                ..default()
                            },
                            ..default()
                        }).with_children(|parent| {
                            parent.spawn(ButtonBundle {
                                style: Style {
                                    width: Val::Px(25.0),
                                    height: Val::Px(25.0),
                                    margin: UiRect::all(Val::Px(5.0)),
                                    ..default()
                                },
                                background_color: BackgroundColor(Color::WHITE),
                                image: icon.clone().into(),
                                ..default()
                            }).insert(TagItem { item_type: name.clone() });
                            
                        // .insert(TagBackPack);
                        // button.insert(TagItem { item_type: name.to_string() });

                            parent.spawn(TextBundle::from_section(
                                format!("{:?}:", name.clone()),
                                TextStyle {
                                    font: font_handle2.clone(),
                                    font_size: 30.0,
                                    color: Color::BLACK,
                                    ..default()
                                },
                            )).insert(TagBackPack)
                            .insert(TagItem { item_type: name });
                            
                        });
                    }
                })
            .insert(LabelBackPack);
        });


        //POPUP
        commands.spawn(NodeBundle {
            style: Style {
                display: Display::Flex,
                width: Val::Px(220.0),
                height: Val::Px(130.0),
                position_type: PositionType::Absolute,
                left: Val::Px(10.0),
                top: Val::Percent(20.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                border: UiRect::all(Val::Px(4.0)),
                ..default()
            },
            background_color: Color::rgba(0.0, 0.0, 0.0, 0.8).into(),
            border_color: BorderColor(Color::WHITE),
            ..default()
        }).insert(PopupLabel)
        .insert(Explode)
        .with_children(|parent| {
            parent.spawn(TextBundle::from_section(
                "",
                TextStyle {
                    font_size: 15.0,
                    font: font_handle1.into(),
                    color: Color::WHITE,
                    ..default()
                },
                
            )).insert(PopupLabelText)
            .insert(Explode); // Assicurati di inserire PopupLabelText qui
        });
         // Etichetta per identificare questo NodeBundle

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
                    )
                    .with_scale(main_scale),
                    camera: Camera {
                        order: 1,
                        ..default()
                    },
                    ..Default::default()
                },
                MainCamera,
            ))
            .insert(RenderLayers::from_layers(&[1, 2, 3, 7, 8]))
            .insert(Explode);

    //dimensioni complessive del mondo
    let world_width: f32 = world.len() as f32 * TILE_SIZE;
    let world_height = world[0].len() as f32 * TILE_SIZE;

    //centro del mondo
    let world_center_x = world_width / 2.0; 
    let world_center_y = world_height / 2.0;

    //pixel minimappa
    let minimap_width = 70.0; 
    let minimap_height = 70.0; 

    //scale minimappa
    let minimap_scale = Vec3::new(
        WORLD_SIZE as f32 / minimap_width,
        WORLD_SIZE as f32 / minimap_height,
        1.0,
    ); 

    //CAMERA PER LA MINIMAPPA
    //Left Camera
    commands
        .spawn((
            Camera2dBundle {
                transform: Transform::from_xyz(world_center_x, world_center_y, 555.0)
                    .with_scale(minimap_scale), 
                camera: Camera {
                    order: 2, //serve per mettere l'ordine di rendering delle camere, se non settato default a 0(MAINCAMERA)
                    ..default()
                },
                camera_2d: Camera2d {
                    
                    clear_color: ClearColorConfig::None,
                    ..default()
                },
                ..default()
            },
            MyMinimapCamera,
        ))
        .insert(RenderLayers::from_layers(&[0, 2, 9]))
        .insert(Explode);

    // Crea l'entita' per il contorno sulla minimappa
    commands
        .spawn(SpriteBundle {
            sprite: Sprite {
                color: Color::rgba(1.0, 0.0, 0.0, 0.5), 
                custom_size: Some(Vec2::new(25.0, 25.0)),
                ..default()
            },
            transform: Transform::from_xyz(0.0, 0.0, 857.0), 
            ..default()
        })
        .insert(RenderLayers::layer(0))
        .insert(MinimapOutline)
        .insert(Explode);

    //NERO SOTTO WORLD MAP
    
    for x in 0..WORLD_SIZE {
        for y in 0..WORLD_SIZE {
           
            let pos_x = x as f32 * TILE_SIZE;
            let pos_y = y as f32 * TILE_SIZE;

            // Spawn del quadrato 3x3
            commands.spawn(SpriteBundle {
                sprite: Sprite {
                    color: Color::GRAY,                                 
                    custom_size: Some(Vec2::new(TILE_SIZE, TILE_SIZE)), 
                    ..default()
                },
                transform: Transform::from_xyz(pos_x, pos_y, 0.0), 
                ..default()
            })
            .insert(Explode);
        }
    }

    let border_thickness = 5.0; 
    let effective_world_size = WORLD_SIZE as f32 + border_thickness * 2.0;

    //bordo minimappa
    for x in 0..effective_world_size as u32 {
        for y in 0..effective_world_size as u32 {
           
            if x < border_thickness as u32
                || y < border_thickness as u32
                || x >= (WORLD_SIZE as f32 + border_thickness) as u32
                || y >= (WORLD_SIZE as f32 + border_thickness) as u32
            {
               
                let pos_x = (x as f32 - border_thickness) * TILE_SIZE;
                let pos_y = (y as f32 - border_thickness) * TILE_SIZE;

                
                commands
                    .spawn(SpriteBundle {
                        sprite: Sprite {
                            color: Color::GREEN,                                
                            custom_size: Some(Vec2::new(TILE_SIZE, TILE_SIZE)), 
                            ..default()
                        },
                        transform: Transform::from_xyz(pos_x, pos_y, -1.0), 
                        ..default()
                    })
                    .insert(RenderLayers::layer(0))
                    .insert(Explode);
            }
        }
    }

    let mut texture_counter = 0;


    //bordo main camera
     for x in 0..effective_world_size as u32  {
    for y in 0..effective_world_size as u32  {
        if x < border_thickness as u32
            || y < border_thickness as u32
            || x >= (WORLD_SIZE as f32 + border_thickness) as u32
            || y >= (WORLD_SIZE as f32 + border_thickness) as u32
        {
            let pos_x = (x as f32 - border_thickness) * TILE_SIZE;
            let pos_y = (y as f32 - border_thickness) * TILE_SIZE;

            let texture_handle = match texture_counter % 3 {
                0 => texture_border1_handle.clone(),
                1 => texture_border2_handle.clone(),
                _ => texture_border3_handle.clone(),
            };

            commands.spawn(SpriteBundle {
                sprite: Sprite {
                    color: Color::WHITE,
                    custom_size: Some(Vec2::new(TILE_SIZE, TILE_SIZE)), 
                    ..default()
                },
                texture: texture_handle, 
                transform: Transform::from_xyz(pos_x, pos_y, -1.0),
                ..default()
            })
            .insert(RenderLayers::layer(1))
            .insert(Explode);

            texture_counter += 1;
        }
    }
} 


   //TILES GIORNO NOTTE
    for x in 0..WORLD_SIZE {
        for y in 0..WORLD_SIZE {
            
            let pos_x = x as f32 * TILE_SIZE;
            let pos_y = y as f32 * TILE_SIZE;

            // Spawn del quadrato 3x3
            commands
                .spawn(SpriteBundle {
                    sprite: Sprite {
                        color: Color::rgba(0.1, 0.1, 0.3, 0.5), 
                        custom_size: Some(Vec2::new(TILE_SIZE, TILE_SIZE)), 
                        ..default()
                    },
                    transform: Transform::from_xyz(pos_x, pos_y, 50.0), 
                    ..default()
                })
                .insert(RenderLayers::layer(7))
                .insert(SunTime)
                .insert(Explode);
        }
    }
}

fn update_infos(
    resource: RobotInfo, 
    weather_icons: Res<WeatherIcons>,
    mut tracker: ResMut<ContentTracker>,
   // mut old_value: ResMut<OldValueBackPack>,
    mut energy_query: Query<
        &mut Text,
        (
            With<TagEnergy>,
            Without<Roboto>,
            Without<TagTime>,
            Without<TagBackPack>,
            Without<TagCoordinate>,
        ),
    >,
    mut coordinate_quert: Query<
        &mut Text,
        (
            With<TagCoordinate>,
            Without<Roboto>,
            Without<TagTime>,
            Without<TagBackPack>,
            Without<TagEnergy>,
        )
    >,
    mut time_query: Query<
        &mut Text,
        (
            With<TagTime>,
            Without<TagEnergy>,
            Without<Roboto>,
            Without<TagBackPack>,
            Without<TagCoordinate>,
        ),
    >,
    mut backpack_query: Query<
        ( &mut Text,
           &TagItem),       
            (
            With<TagBackPack>,
            Without<TagEnergy>,
            Without<Roboto>,
            Without<TagTime>,
            Without<TagCoordinate>,
        ),
    >,
    mut battery_query: Query<(&mut Style, &mut BackgroundColor), With<EnergyBar>>,
    mut sun_query: Query<&mut Sprite, With<SunTime>>,
    mut weather_image_query: Query<&mut UiImage, With<WeatherIcon>>,
    font_res: Res<FontResource>,
) {
    
    //TESTO ENERGY E COORDINATE
    for mut text in energy_query.iter_mut() {
        text.sections[0].value = format!(
            "Energy: {}\n\n",
            resource.energy_level,
        );
    }

    for mut text in coordinate_quert.iter_mut() {
        text.sections[0].value = format!(
            "Coordinates: X: {}, Y:{}",  resource.coordinate_column, resource.coordinate_row
        );
    }

    //TESTO TIME E WEATHER
    for mut text in time_query.iter_mut() {
        if resource.current_weather.is_some() {
            text.sections = vec![
                TextSection {
                    value: format!("\nTIME: {:?}\n\n", resource.time),
                    style: TextStyle {
                      font: font_res.font_handle3.clone(), // Assicurati di avere il font handle corretto qui
                        font_size: 40.0, // Font size più piccolo per 'Weather'
                    color: Color::WHITE,
                       
                    },
                },
            TextSection {
                value: format!("Weather: {:?}\n\n", resource.current_weather.unwrap()),
                style: TextStyle {
                  font: font_res.font_handle3.clone(), // Assicurati di avere il font handle corretto qui
                    font_size: 25.0, // Font size più piccolo per 'Weather'
                   color: Color::WHITE,
                   
                },
            },
        ];
        }
    }
    //TESTO BACKPACK
        for (mut text, tag_item) in backpack_query.iter_mut() {
            if let Some(value) = resource.bp_contents.get(&tag_item.item_type) {
                // Costruisci la stringa di output basata solo sul tipo specifico di Content
                let formatted_string = format!("{}: {}", tag_item.item_type, value);
                text.sections[0].value = formatted_string;
            } else {
                // Se non ci sono elementi di quel tipo, imposta un messaggio di default
                text.sections[0].value = format!("{}: None", tag_item.item_type);
            }
        }

        // Aggiornamento delle informazioni nel tracker
        for (_, tag_item) in backpack_query.iter() {
            let item_type = tag_item.item_type.clone();
            let current_value = resource.bp_contents.get(&item_type).cloned().unwrap_or(0);
            let old_value = *tracker.old_values.entry(item_type.clone()).or_insert(current_value);
        
            if current_value != old_value {
                let difference = current_value as isize - old_value as isize; // Differenza calcolata come valore signed
        
                // Aggiorna il vecchio valore
                *tracker.old_values.get_mut(&item_type).unwrap() = current_value;
        
                // Gestione incremento
                if difference > 0 {
                    let increase = difference as usize;  // Converte solo incrementi positivi
                    let entry = tracker.counts.entry(item_type.clone()).or_insert(0);
                    *entry += increase;  // Aggiunge solo incrementi positivi
                }
        
                // Gestione deposito
                if difference < 0 {
                    let deposited_amount = (-difference) as usize;  // Quantità depositata
                    let deposited = tracker.deposited_counts.entry(item_type).or_insert(0);
                    *deposited += deposited_amount;  // Aggiunge al conteggio dei depositati
                }
            }
        }

    //UPDATE BATTERY SPRITE
    for (mut style, mut back_color) in battery_query.iter_mut() {
        // Calcola la percentuale dell'energia
        let percentage = resource.energy_level as f32 / 1000.0; 
                                                                
        back_color.0 = match percentage {
            p if p > 0.5 => Color::GREEN.into(),
            p if p > 0.25 => Color::YELLOW.into(),
            _ => Color::RED.into(),
        };
        // Aggiorna la larghezza in base alla percentuale dell'energia
        style.width = Val::Percent(percentage * 82.0);
    }

    //SUN MOVEMENT
    for mut sprite in sun_query.iter_mut() {
        let night_alpha = match parse_time(&resource.time) {
            Ok(time) => {
                if time >= 18.0 && time < 20.0 {
                    // Tramonto
                    (time - 18.0) / 2.0 * 0.4 + 0.3
                } else if (time >= 20.0 && time <= 24.0) || (time >= 0.0 && time < 4.0) {
                    // Notte
                    0.7
                } else if time >= 4.0 && time < 6.0 {
                    // Alba
                    (1.0 - (time - 4.0) / 2.0) * 0.4 + 0.3
                } else {
                    // Giorno
                    0.0
                }
            }
            Err(e) => {
                eprintln!("Errore nel parsing del tempo: {}", e);
                0.0 
            }
        };

        
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

                _ => continue, 
            };

            image.texture = image_handle;

        } else {
            
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

//EXTRA
fn cursor_position(q_windows: Query<&Window, With<PrimaryWindow>>) {
    if let Ok(window) = q_windows.get_single() {
        if let Some(position) = window.cursor_position() {
            println!("Cursor is inside the primary window, at {:?}", position);
        } else {
            println!("Cursor is not in the game window.");
        }
    }
}

//MOVIMENTO MINIMAPPA
/* fn cursor_events(
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
                println!("Entrato nel cursor");
                //posizione e la dimensione fisica della viewport della minimappa
                let minimap_viewport_position = Vec2::new(
                    viewport.physical_position.x as f32,
                    viewport.physical_position.y as f32,
                );
                let minimap_viewport_size = Vec2::new(
                    viewport.physical_size.x as f32 / 1.5,
                    viewport.physical_size.y as f32 / 1.5,
                );
                //posizione del cursore relativa alla minimappa
                let cursor_relative_to_minimap = cursor_position - minimap_viewport_position;

                //controllo se il cursore è all'interno della minimappa
                if cursor_relative_to_minimap.x >= 0.0
                    && cursor_relative_to_minimap.x <= minimap_viewport_size.x
                    && cursor_relative_to_minimap.y >= 0.0
                    && cursor_relative_to_minimap.y <= minimap_viewport_size.y
                {
                    println!("entrato nell'if");

                    //proporzioni del cursore all'interno della minimappa
                    let click_proportions = cursor_relative_to_minimap / minimap_viewport_size;

                    //posizione nel mondo basata sulle proporzioni della minimappa
                    let world_pos_x = minimap_transform.translation.x
                        + (click_proportions.x - 0.5) * (WORLD_SIZE as f32 * TILE_SIZE);
            
                    let world_pos_y = minimap_transform.translation.y
                        + (0.5 - click_proportions.y) * (WORLD_SIZE as f32 * TILE_SIZE);

                    //trasform finale della maincamera in base alla posizione del cursore sulla minimappa
                    for mut transform in main_camera_query.iter_mut() {
                        transform.translation.x = world_pos_x;
                        transform.translation.y = world_pos_y;
                    }
                }
            }
        }
    }
    println!("CURSOR");
} */

//VECCHIO, DA PROBLEMI CON FOLLOW DEL ROBOT PERCHE ENTRAMBI CHIAMATI ALLO STESSO MOMENTO
/* fn cursor_events(
    minimap_camera_query: Query<
        (&Camera, &Transform),
        (With<MyMinimapCamera>, Without<MainCamera>),
    >,
    mut main_camera_query: Query<&mut Transform, (With<MainCamera>, Without<MyMinimapCamera>)>,
    q_windows: Query<&Window, With<PrimaryWindow>>,
) {
    let window = q_windows.single();
    //println!("Funzione cursor_events avviata");

    if let Some(cursor_position) = window.cursor_position() {
        //println!("Posizione del cursore: {:?}", cursor_position);

        if let Ok((minimap_camera, minimap_transform)) = minimap_camera_query.get_single() {
            //println!("Minimappa trovata con trasformazione: {:?}", minimap_transform);

            if let Some(viewport) = &minimap_camera.viewport {
                //println!("Viewport trovata: {:?}", viewport);

                let minimap_viewport_position = Vec2::new(
                    viewport.physical_position.x as f32,
                    viewport.physical_position.y as f32,
                );
                let minimap_viewport_size = Vec2::new(
                    viewport.physical_size.x as f32 / 1.5,
                    viewport.physical_size.y as f32 / 1.5,
                );
                //println!("Posizione viewport minimappa: {:?}", minimap_viewport_position);
                //println!("Dimensione viewport minimappa: {:?}", minimap_viewport_size);

                let cursor_relative_to_minimap = cursor_position - minimap_viewport_position;
                //println!("Posizione relativa del cursore sulla minimappa: {:?}", cursor_relative_to_minimap);

                if cursor_relative_to_minimap.x >= 0.0
                    && cursor_relative_to_minimap.x <= minimap_viewport_size.x
                    && cursor_relative_to_minimap.y >= 0.0
                    && cursor_relative_to_minimap.y <= minimap_viewport_size.y
                {
                    //println!("Il cursore è all'interno della minimappa");

                    let click_proportions = cursor_relative_to_minimap / minimap_viewport_size;
                    //println!("Proporzioni del click sulla minimappa: {:?}", click_proportions);

                    let world_pos_x = minimap_transform.translation.x
                        + (click_proportions.x - 0.5) * (WORLD_SIZE as f32 * TILE_SIZE);
                    let world_pos_y = minimap_transform.translation.y
                        + (0.5 - click_proportions.y) * (WORLD_SIZE as f32 * TILE_SIZE);
                    //println!("Posizione calcolata nel mondo: x = {}, y = {}", world_pos_x, world_pos_y);

                    for mut transform in main_camera_query.iter_mut() {
                        //println!("Aggiornamento della trasformazione della camera principale");
                        transform.translation.x = world_pos_x;
                        transform.translation.y = world_pos_y;
                    }
                } else {
                   // println!("Il cursore NON è all'interno della minimappa");
                }
            } else {
                //println!("Viewport non trovato nella minimappa");
            }
        } else {
            //println!("Query sulla minimappa non riuscita o minimappa non trovata");
        }
    } else {
        //println!("Posizione del cursore non disponibile");
    }
    //println!("Fine della funzione cursor_events");
} */

fn cursor_events(
    minimap_camera_query: Query<(&Camera, &Transform), (With<MyMinimapCamera>, Without<MainCamera>)>,
    mut main_camera_query: Query<&mut Transform, (With<MainCamera>, Without<MyMinimapCamera>)>,
    q_windows: Query<&Window, With<PrimaryWindow>>,
    mut camera_follow: ResMut<CameraFollow>, // Aggiungi questo
) {
    let window = q_windows.single();

    if let Some(cursor_position) = window.cursor_position() {
        if let Ok((minimap_camera, minimap_transform)) = minimap_camera_query.get_single() {
            if let Some(viewport) = &minimap_camera.viewport {
                let minimap_viewport_position = Vec2::new(
                    viewport.physical_position.x as f32,
                    viewport.physical_position.y as f32,
                );
                let minimap_viewport_size = Vec2::new(
                    viewport.physical_size.x as f32 / 1.5,
                    viewport.physical_size.y as f32 / 1.5,
                );

                let cursor_relative_to_minimap = cursor_position - minimap_viewport_position;

                if cursor_relative_to_minimap.x >= 0.0
                    && cursor_relative_to_minimap.x <= minimap_viewport_size.x
                    && cursor_relative_to_minimap.y >= 0.0
                    && cursor_relative_to_minimap.y <= minimap_viewport_size.y
                {
                    camera_follow.follow_robot = false; // Imposta il flag su false

                    let click_proportions = cursor_relative_to_minimap / minimap_viewport_size;
                    let world_pos_x = minimap_transform.translation.x
                        + (click_proportions.x - 0.5) * (WORLD_SIZE as f32 * TILE_SIZE);
                    let world_pos_y = minimap_transform.translation.y
                        + (0.5 - click_proportions.y) * (WORLD_SIZE as f32 * TILE_SIZE);

                    for mut transform in main_camera_query.iter_mut() {
                        transform.translation.x = world_pos_x;
                        transform.translation.y = world_pos_y;
                    }
                } else {
                    camera_follow.follow_robot = true; // Imposta il flag su true
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

            //dimensione visibile
            let camera_scale = main_camera_transform.scale.x;

            //dimensione visibile basata sulle dimensioni del viewport e sulla scala della maincamera
            let visible_width = viewport_width * camera_scale / 1.5;
            let visible_height = viewport_height * camera_scale / 1.5;

            //dimensioni del rettangolo sulla minimappa
            let outline_size = Vec2::new(visible_width, visible_height);

            for (mut sprite, mut transform) in minimap_outline_query.iter_mut() {

                sprite.custom_size = Some(outline_size);

                //aggiorna la posizione del rettangolo sulla minimappa che corrisponde a quella della maincamera
                transform.translation.x = main_camera_transform.translation.x;
                transform.translation.y = main_camera_transform.translation.y;
                transform.translation.z = 999.0;
            }
        }
    }
}


//SETUP VIEWPORT(COME SCHERMO CONDIVISO)
/* fn set_camera_viewports(
    windows: Query<&Window>,
    mut resize_events: EventReader<WindowResized>,
    mut minimappa_camera: Query<&mut Camera, (With<MyMinimapCamera>, Without<MainCamera>)>,
    mut main_camera: Query<&mut Camera, With<MainCamera>>,
) {
    //sleep(std::time::Duration::from_secs(1));
    for resize_event in resize_events.read() {
        //parte sinistra (MINIMAPPA)
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

        //parte destra (MAINCAMERA)
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

    println!("VIEWPORT");
} */

/* fn set_camera_viewports(
    windows: Query<&Window>,
    mut resize_events: EventReader<WindowResized>,
    mut minimappa_camera: Query<&mut Camera, (With<MyMinimapCamera>, Without<MainCamera>)>,
    mut main_camera: Query<&mut Camera, (With<MainCamera>, Without<MyMinimapCamera>)>,
) {
    for resize_event in resize_events.read() {
        if let Ok(window) = windows.get(resize_event.window) {
            // Handle the minimap camera
            match minimappa_camera.get_single_mut() {
                Ok(mut camera) => {
                    camera.viewport = Some(Viewport {
                        physical_position: UVec2::new(0, 0),
                        physical_size: UVec2::new(
                            window.resolution.physical_width() / 6,
                            window.resolution.physical_height() / 4,
                        ),
                        
                        ..default()
                    });
                    println!("viewport1");
                }
                Err(_) => {
                    eprintln!("Error: Minimap camera not found.");
                }
            }

            // Handle the main camera
            match main_camera.get_single_mut() {
                Ok(mut camera) => {
                    camera.viewport = Some(Viewport {
                        physical_position: UVec2::new(0, 0),
                        physical_size: UVec2::new(
                            window.resolution.physical_width(),
                            window.resolution.physical_height(),
                        ),
                        ..default()
                    });
                    println!("viewport2");
                }
                
                Err(_) => {
                    eprintln!("Error: Main camera not found.");
                }
            }
        } else {
            eprintln!("Error: Window not found.");
        }
    }

    println!("VIEWPORT");
} */

fn set_camera_viewports(
    windows: Query<&Window>,
    mut minimappa_camera: Query<&mut Camera, (With<MyMinimapCamera>, Without<MainCamera>)>,
    mut main_camera: Query<&mut Camera, With<MainCamera>>,
) {
    if let Some(window) = windows.iter().next() {
        // Handle the minimap camera
        match minimappa_camera.get_single_mut() {
            Ok(mut camera) => {
                camera.viewport = Some(Viewport {
                    physical_position: UVec2::new(0, 0),
                    physical_size: UVec2::new(
                        window.resolution.physical_width() / 6,
                        window.resolution.physical_height() / 4,
                    ),
                    ..default()
                });
                println!("viewport1 updated");
            }
            Err(_) => {
                eprintln!("Error: Minimap camera not found.");
            }
        }

        // Handle the main camera
        match main_camera.get_single_mut() {
            Ok(mut camera) => {
                camera.viewport = Some(Viewport {
                    physical_position: UVec2::new(0, 0),
                    physical_size: UVec2::new(
                        window.resolution.physical_width(),
                        window.resolution.physical_height(),
                    ),
                    ..default()
                });
                println!("viewport2 updated");
            }
            Err(_) => {
                eprintln!("Error: Main camera not found.");
            }
        }
    } else {
        eprintln!("Error: Window not found.");
    }
}


//SPAWN DEI TILE
/*     fn update_show_tiles(
        world: &Vec<Vec<Option<Tile>>>,
        commands: &mut Commands,
        mut old_world: Query<&mut OldMapResource>,
        tile_icons: Res<TileIcons>,
        content_icons: Res<ContentIcons>,
        robot_position: Res<RobotPosition>, 
    ) {

        if let Ok(mut old_map_res) = old_world.get_single_mut() {
            let old_world = &mut old_map_res.world;

        for (x, row) in world.iter().enumerate() {
            for (y, tile) in row.iter().enumerate() {
                let old_tile = &old_world[x][y];
            
                if let Some(tile) = tile {
                    if old_tile.is_none() || old_tile.as_ref().unwrap().content != tile.content {
                        let tile_color = get_tile_icons(tile, &tile_icons);
                        let content_color = get_content_icons(tile, &content_icons);
                        
                        let z_value_content = if tile.content != Content::None { 5 } else { 0 };
                        let z_value_tile = 2;
                
                    if tile.content != Content::None {
                        commands
                            .spawn(ImageBundle {
                                image: content_color.unwrap().into(),
                                style: Style {
                                    width: Val::Px(TILE_SIZE),
                                    height: Val::Px(TILE_SIZE),
                                    left: Val::Px(x as f32 * TILE_SIZE), // Imposta la posizione orizzontale in base alla coordinata x del tile
                                    top: Val::Px(y as f32 * TILE_SIZE), // Imposta la posizione verticale in base alla coordinata y del tile
                                    ..Default::default()
                                },
                                z_index: ZIndex::Global(z_value_content),

                                ..Default::default()
                            })
                            .insert(RenderLayers::layer(0));
                            

                    
                    commands
                    .spawn(ImageBundle {
                        image: tile_color.into(),
                        style: Style {
                            width: Val::Px(TILE_SIZE),
                            height: Val::Px(TILE_SIZE),
                            left: Val::Px(x as f32 * TILE_SIZE), // Imposta la posizione orizzontale in base alla coordinata x del tile
                            top: Val::Px(y as f32 * TILE_SIZE),  // Imposta la posizione verticale in base alla coordinata y del tile
                            ..Default::default()
                        },
                        z_index: ZIndex::Global(z_value_tile),
                        ..Default::default()
                    })
                    .insert(RenderLayers::layer(0));
                }
            }
        }
    }
}
        *old_world = world.clone();
        }
    } */

//VECCHIO NON OTTIMIZZATO
 /*    fn update_show_tiles(
        world: &Vec<Vec<Option<Tile>>>,
        commands: &mut Commands,
        old_world: &mut Vec<Vec<Option<Tile>>>,
        tile_icons: Res<TileIcons>,
        content_icons: Res<ContentIcons>,
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
                    let tile_color = get_tile_icons(&tile, &tile_icons);
                    let content_color = get_content_icons(&tile, &content_icons);
                    let mut z_value = 10.0;
                    // Optionally spawn an additional sprite for the content if it's not None
                    if tile.content != Content::None {
                        commands
                            .spawn(SpriteBundle {
                                sprite: Sprite {
                                    color: Color::WHITE, // Use the content color
                                    custom_size: Some(Vec2::new(TILE_SIZE / 1.5, TILE_SIZE / 1.5)), // Smaller than the tile for distinction
                                    ..Default::default()
                                },
                                texture: content_color.unwrap().clone(),
                                transform: Transform::from_xyz(
                                    x as f32 * TILE_SIZE, // Centered on the tile
                                    y as f32 * TILE_SIZE, // Centered on the tile
                                    z_value,              // Slightly above the tile layer
                                ),
                                ..Default::default()
                            })
                            .insert(RenderLayers::layer(3))
                            .insert(Explode);
                        
                        z_value = 5.0;
                    }
    
                    // Create a base sprite for the tile
                    commands
                        .spawn(SpriteBundle {
                            sprite: Sprite {
                                color: Color::WHITE, // Use the tile color
                                custom_size: Some(Vec2::new(TILE_SIZE, TILE_SIZE)),
                                ..Default::default()
                            },
                            texture: tile_color.clone(),
                            transform: Transform::from_xyz(
                                x as f32 * TILE_SIZE, // X position with an offset
                                y as f32 * TILE_SIZE, // Y position with an offset
                                z_value,
                            ),
                            ..Default::default()
                        })
                        .insert(RenderLayers::layer(3))
                        .insert(Explode);
                }
            }
        }
        *old_world = world.clone();
    }
 */
use std::time::{Duration, Instant};

#[derive(Resource)]
struct LastUpdate(Instant);

#[derive(Component, Debug)]
struct TilePosition {
    x: usize,
    y: usize,
}


 //OTTIMIZATO, AGGIORNAMENTO SOLO VICINO AL ROBOT E DESPAWNA CONTENT
 fn update_show_tiles(
    world: &Vec<Vec<Option<Tile>>>,
    mut commands: Commands,
    old_world: &mut Vec<Vec<Option<Tile>>>,
    tile_icons: Res<TileIcons>,
    content_icons: Res<ContentIcons>,
    robot_position: Res<RobotPosition>,
    mut last_update: ResMut<LastUpdate>,
    mut query: Query<(Entity, &TilePosition), With<Sprite>>,
    mut content_counter: ResMut<ContentCounter>,
    discovered_signal: Res<DiscoveredSignal>,
) {
    
    if !discovered_signal.0.load(Ordering::SeqCst){
        let update_radius = 4;
        let mut count = 0;
        let mut despawn_count = 0;
        let player_x = robot_position.x as usize / TILE_SIZE as usize;
        let player_y = robot_position.y as usize / TILE_SIZE as usize;

        let start_x = player_x.saturating_sub(update_radius);
        let end_x = (player_x + update_radius).min(world.len() - 1);
        let start_y = player_y.saturating_sub(update_radius);
        let end_y = (player_y + update_radius).min(world[0].len() - 1);

        for x in start_x..=end_x {
            for y in start_y..=end_y {
                // Potenziale inversione degli indici qui se necessario
                let current_tile = &world[y][x];
                let old_content = old_world[y][x].as_ref().map(|t| t.content.clone());


                    if let Some(new_tile) = current_tile {
                        // Controllo se il tile attuale è una strada o se ci sono cambiamenti nei contenuti.
                        let is_street = matches!(new_tile.tile_type, TileType::Street);
                        let content_changed = old_world[y][x].is_none() || old_content != Some(new_tile.content.clone());
                    
                        if is_street || content_changed {
                            count += 1;
                            // Despawna il tile esistente prima di rispawnarlo, se è una strada
                            if is_street {
                                for (entity, pos) in query.iter_mut() {
                                    if pos.x == x && pos.y == y {
                                        commands.entity(entity).despawn_recursive();
                                    //  println!("despawned: {:?} pos {:?}", entity, pos);
                                        break; // Despawna solo l'entità corrispondente a quella posizione
                                    }
                                }
                            }
                            // Risppawna il tile
                            spawn_tile(new_tile, x, y, &mut commands, &tile_icons, &content_icons, &mut content_counter);
                            old_world[y][x] = Some(new_tile.clone());
                        }
                    }

                

                if let Some(old_content_unwrapped) = old_content {
                    if current_tile.is_none() || current_tile.as_ref().unwrap().content == Content::None {
                        if old_content_unwrapped != Content::None {
                            for (entity, pos) in query.iter_mut() {
                                if pos.x == x && pos.y == y {
                                    commands.entity(entity).despawn_recursive();
                                    despawn_count += 1;
                                }
                            }
                            old_world[y][x] = current_tile.clone();
                        }
                    }
                }
            }
        }
    } else{
        for (x, row) in world.iter().enumerate() {
            for (y, tile) in row.iter().enumerate() {
                let old_tile = &old_world[y][x];
                // Se il nuovo tile non e' None e il vecchio tile e' None, spawnalo
                if tile.is_some()
                    && (old_tile.is_none()
                        || old_tile.clone().unwrap().content != tile.clone().unwrap().content)
                    {
                        spawn_tile(&tile.clone().unwrap(), y, x, &mut commands, &tile_icons, &content_icons, &mut content_counter);
                }
            }
        }
        discovered_signal.0.store(false, Ordering::SeqCst);
    }
}

/* fn despawn_tiles(
    commands: &mut Commands,
    query: &mut Query<(Entity, &TilePosition), With<Sprite>>,
    x: usize,
    y: usize,
    
) {
    for (entity, pos) in query.iter_mut() {
        if pos.x == x && pos.y == y {
            commands.entity(entity).despawn_recursive();
        }
    }
} */

//************************************************INSERIRE DESPAWN******************************************** */
fn update_show_tiles_maze(
    world: &Vec<Vec<Option<Tile>>>,
    mut commands: &mut Commands,
    old_world: &mut Vec<Vec<Option<Tile>>>,
    tile_icons: &Res<TileIcons>,
    content_icons: &Res<ContentIcons>,
    robot_position: &Res<RobotPosition>,
    mut last_update: ResMut<LastUpdate>,
    mut query: Query<(Entity, &TilePosition), With<Sprite>>,
    mut content_counter: ResMut<ContentCounter>,
) {
    if last_update.0.elapsed() >= Duration::new(5, 0) {
        last_update.0 = Instant::now();  // Resetta il timer
        let mut count = 0;
        for (x, row) in world.iter().enumerate() {
            for (y, tile) in row.iter().enumerate() {
                let old_tile = &old_world[y][x];
                if tile.is_some() && (old_tile.is_none() || old_tile.as_ref().unwrap().content != tile.as_ref().unwrap().content) {
                    count += 1;
                    spawn_tile(tile.as_ref().unwrap(), y, x, commands, &tile_icons, &content_icons, &mut content_counter);
                    // Aggiorna il vecchio mondo con il nuovo tile
                    old_world[y][x] = Some(tile.clone().unwrap());
                }
            }
        }
      //  println!("I TILE AGGIORNATI SONO: {}", count);
    } else {

        let update_radius = 2;
        let mut count = 0;
        let player_x = robot_position.x as usize / TILE_SIZE as usize;
        let player_y = robot_position.y as usize / TILE_SIZE as usize;

        // Calcola gli indici di inizio e fine per x e y
        let start_x = player_x.saturating_sub(update_radius);
        let end_x = (player_x + update_radius).min(world.len() - 1);
        let start_y = player_y.saturating_sub(update_radius);
        let end_y = (player_y + update_radius).min(world[0].len() - 1);

        // Itera solo sui tile vicini al robot
        for x in start_x..=end_x {
            for y in start_y..=end_y {
                // Potenziale inversione degli indici qui se necessario
                let current_tile = &world[y][x];
                let old_content = old_world[y][x].as_ref().map(|t| t.content.clone());
    
    
                    if let Some(new_tile) = current_tile {
                        // Controllo se il tile attuale è una strada o se ci sono cambiamenti nei contenuti.
                        let is_street = matches!(new_tile.tile_type, TileType::Street);
                        let content_changed = old_world[y][x].is_none() || old_content != Some(new_tile.content.clone());
                    
                        if is_street || content_changed {
                            count += 1;
                            // Despawna il tile esistente prima di rispawnarlo, se è una strada
                            if is_street {
                                for (entity, pos) in query.iter_mut() {
                                    if pos.x == x && pos.y == y {
                                        commands.entity(entity).despawn_recursive();
                                     //  println!("despawned: {:?} pos {:?}", entity, pos);
                                        break; // Despawna solo l'entità corrispondente a quella posizione
                                    }
                                }
                            }
                            // Risppawna il tile
                            spawn_tile(new_tile, x, y, &mut commands, &tile_icons, &content_icons, &mut content_counter);
                            old_world[y][x] = Some(new_tile.clone());
                        }
                    }
    
                  
    
                if let Some(old_content_unwrapped) = old_content {
                    if current_tile.is_none() || current_tile.as_ref().unwrap().content == Content::None {
                        if old_content_unwrapped != Content::None {
                            for (entity, pos) in query.iter_mut() {
                                if pos.x == x && pos.y == y {
                                    commands.entity(entity).despawn_recursive();
                                   // despawn_count += 1;
                                }
                            }
                            old_world[y][x] = current_tile.clone();
                        }
                    }
                }
            }
        }
    }
}


    fn spawn_tile(
        tile: &Tile,
        x: usize,
        y: usize,
        commands: &mut Commands,
        tile_icons: &Res<TileIcons>,
        content_icons: &Res<ContentIcons>,
        mut content_counter: &mut ResMut<ContentCounter>,
    ) {
        let tile_color = get_tile_icons(tile, tile_icons);
        let content_color = get_content_icons(tile, content_icons);
        //let mut count = 0;

        // Determine the Z level based on the tile type
            let z_level = match tile.tile_type {
                TileType::Street => 4.0,  // Higher Z level for street tiles
                _ => 3.0,                 // Default Z level for all other tiles
            };

        // Spawn base tile sprite
        commands.spawn(SpriteBundle {
            sprite: Sprite {
                color: Color::WHITE,
                custom_size: Some(Vec2::new(TILE_SIZE, TILE_SIZE)),
                ..Default::default()
            },
            texture: tile_color.clone(),
            transform: Transform::from_xyz(
                x as f32 * TILE_SIZE,
                y as f32 * TILE_SIZE,
                z_level, // Base layer
            ),
            ..Default::default()
        }).insert(TilePosition { x, y })
        .insert(RenderLayers::layer(3)) // Assicurati che anche i contenuti siano visibili su entrambi i layer
        .insert(Explode);
    
        commands.spawn(SpriteBundle {
            sprite: Sprite {
                color: Color::WHITE,
                custom_size: Some(Vec2::new(TILE_SIZE, TILE_SIZE)),
                ..Default::default()
            },
            texture: tile_color.clone(),
            transform: Transform::from_xyz(
                x as f32 * TILE_SIZE,
                y as f32 * TILE_SIZE,
                3.0, // Base layer
            ),
            ..Default::default()
        }).insert(TilePosition { x, y })
        .insert(RenderLayers::layer(9)) // Assicurati che anche i contenuti siano visibili su entrambi i layer
        .insert(Explode);

        // Optionally spawn an additional sprite for the content if it's not None and the handle is valid
        if tile.content != Content::None {
            content_counter.count += 1;
            if let Some(content_texture) = content_color {
                commands.spawn(SpriteBundle {
                    sprite: Sprite {
                        color: Color::WHITE,
                        custom_size: Some(Vec2::new(TILE_SIZE / 1.5, TILE_SIZE / 1.5)), // Typically smaller than the base tile for visibility
                        ..Default::default()
                    },
                    texture: content_texture,
                    transform: Transform::from_xyz(
                        x as f32 * TILE_SIZE, // Centered on the tile
                        y as f32 * TILE_SIZE, // Centered on the tile
                        5.0,                 // Above the base tile layer
                    ),
                    ..Default::default()
                }).insert(TilePosition { x, y })
                .insert(RenderLayers::layer(8))
                .insert(Explode);
            }

            // println!("I CONTENT SONO: {}", content_counter.count);
        } 
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
#[derive(Component)]
struct HoverableButton {
    timer: Timer,
}

/* fn button_system_backpack(
    total_content: Res<ContentTracker>,
    mut interaction_query: Query<
        (
            &Interaction,
            &TagItem,
        ),
        (Changed<Interaction>, With<Button>),
    >,
    mut popup_query: Query<(&mut Style), With<PopupLabel>>,
    mut popup_text_query: Query<&mut Text, With<PopupLabelText>>,
) {
    // println!("Checking interactions...");  // Debug: verifica se il sistema viene chiamato

    for (interaction, tag_item) in interaction_query.iter_mut() {
        println!("Interaction: {:?}", interaction);  // Debug: stampa lo stato dell'interazione

        if let Ok((mut popup_style)) = popup_query.get_single_mut() {
            if let Ok(mut popup_text) = popup_text_query.get_single_mut() {
                match *interaction {
                    Interaction::Hovered => {
                        popup_style.display = Display::Flex; // Abilita la visualizzazione del popup
                        popup_style.position_type = PositionType::Absolute;
                        popup_style.left = Val::Px(10.0);  // Sinistra dello schermo
                        popup_style.top = Val::Percent(25.0);   // Alto dello schermo

                        // Imposta il testo in base al tipo di contenuto
                        popup_text.sections[0].value = match tag_item.item_type {
                            Content::Rock(_) => "La roccia più dura del mondo ciao ciao ciao ciao ciao ciao ciao ciao v ciao vciao  v ciao ciao ciao ciao ciao ciao".to_string(),
                            Content::Tree(_) => "Un grande albero".to_string(),
                            Content::Garbage(_) => "Un mucchio di rifiuti".to_string(),
                            Content::Fire => "Un fuoco ardente".to_string(),
                            // Altri casi
                            _ => "Elemento sconosciuto".to_string(),
                        };
                    }
                    Interaction::None => {
                        popup_style.display = Display::None; // Nasconde il popup
                    }
                    _ => {}
                }
            } else {
                println!("Failed to get popup entity.");  // Debug: fallimento nell'ottenere l'entità del popup
            }
        }
    }
} */



fn button_system_backpack(
    total_content: Res<ContentTracker>, // Accesso alla risorsa che tiene traccia del conteggio totale dei contenuti
    mut interaction_query: Query<
        (
            &Interaction,
            &TagItem,
        ),
        (Changed<Interaction>, With<Button>),
    >,
    mut popup_query: Query<(&mut Style), With<PopupLabel>>,
    mut popup_text_query: Query<&mut Text, With<PopupLabelText>>,
) {
    for (interaction, tag_item) in interaction_query.iter_mut() {
        // println!("Interaction: {:?}", interaction);  // Debug: stampa lo stato dell'interazione

        if let Ok((mut popup_style)) = popup_query.get_single_mut() {
            if let Ok(mut popup_text) = popup_text_query.get_single_mut() {
                match *interaction {
                    Interaction::Hovered => {
                        popup_style.display = Display::Flex; // Abilita la visualizzazione del popup
                        popup_style.position_type = PositionType::Absolute;
                        popup_style.left = Val::Px(10.0);  // Sinistra dello schermo
                        popup_style.top = Val::Percent(25.0);   // Alto dello schermo

                        // Recupera il conteggio totale e il conteggio dei depositi dal ContentTracker
                        let content_count = total_content.counts.get(&tag_item.item_type).unwrap_or(&0);
                        let deposited_count = total_content.deposited_counts.get(&tag_item.item_type).unwrap_or(&0);

                        // Imposta il testo in base al tipo di contenuto, includendo il conteggio totale e quello depositato
                        popup_text.sections[0].value = match tag_item.item_type {
                            Content::Rock(_) => format!("A SIMPLE ROCK:\n Gathered {} times.\n Used {} times.\n\n Often overlooked but essential for crafting tools.", content_count, deposited_count),
                            Content::Tree(_) => format!("ANCIENT TREE:\n Harvested {} times.\n Deposited {} times.\n\n Its wood serves as the backbone of homes and weaponry.", content_count, deposited_count),
                            Content::Garbage(_) => format!("MYSTERIOUS GARBAGE:\n Collected {} times.\n Crafted {} times.\n\n Sometimes trash, sometimes treasure.", content_count, deposited_count),
                            Content::Fire => format!("WILD FIRE:\n Observed {} times.\n Used {} times.\n\n Its warmth can comfort or consume.", content_count, deposited_count),
                            Content::Coin(_) => format!("SHINY COIN:\n Used {} times.\n Deposited {} times.\n\n A currency that fuels trade and wealth accumulation.", content_count, deposited_count),
                            Content::Water(_) => format!("FLOWING WATER:\n Utilized {} times.\n Drank {} times.\n\n Essential for life and various uses.", content_count, deposited_count),
                            Content::Bin(_) => format!("RECYCLING BIN:\n A bin doesn't fit in your backpack.\n\n Promotes environmental sustainability through recycling."),
                            Content::Crate(_) => format!("STORAGE CRATE:\n Stop stealing crates! RobotPolice will come after you.\n\n A crucial element for organizing and storing items."),
                            Content::Bank(_) => format!("LOCAL BANK:\n Only in your desires you can take a whole bank.\n\n A secure place for financial dealings and savings."),
                            Content::Market(_) => format!("TOWN MARKET:\n  You can't take a Market, maybe just buy something\n\n A hub for buying and selling goods."),
                            Content::Fish(_) => format!("FRESH FISH:\n Caught {} times.\n Ate {} times.\n\n A source of nourishment and trade.", content_count, deposited_count),
                            Content::Building => format!("STURDY BUILDING:\n You can't destroy this building terminator.\n\n Provides shelter and space for various activities."),
                            Content::Bush(_) => format!("GREEN BUSH:\n Pruned {} times.\n Used {} times.\n\n Adds beauty and structure to landscapes.", content_count, deposited_count),
                            Content::JollyBlock(_) => format!("JOLLY BLOCK:\n Played {} times.\n Deposited {} times.\n\n A source of joy and recreation.", content_count, deposited_count),
                            Content::Scarecrow => format!("GUARDIAN SCARECROW:\n Positioned {} times.\n Deposited {} times.\n\n Protects crops from birds and pests.", content_count, deposited_count),
                            _ => format!("UNKNOWN ELEMENT:\n Interacted {} times.\n Deposited {} times.\n\n A mystery to all who encounter it.", content_count, deposited_count),
                        };
                    }
                    Interaction::None => {
                        popup_style.display = Display::None; // Nasconde il popup
                    }
                    _ => {}
                }
            } else {
                println!("Failed to get popup entity.");  // Debug: fallimento nell'ottenere l'entità del popup
            }
        }
    }
}


//BOTTONI
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
            Option<&PauseButton>, 
            Option<&IncreaseSpeed>,
            Option<&DecreaseSpeed>,
            Option<&ActivityButton>,
            Option<&TeleportButton>,
            Option<&mut UiImage>,
        ),
        (Changed<Interaction>, With<Button>),
    >,
   // mut text_query: Query<&mut Text>,
    mut camera_query: Query<(&mut Transform, &Camera), With<MainCamera>>,
    mut label_query: Query<&mut Style, (With<Label>, Without<LabelBackPack>)>,
    mut label_backpack_query: Query<&mut Style, (With<LabelBackPack>, Without<Label>)>,
    robot_position: Res<RobotPosition>,
   // mut exit: EventWriter<AppExit>,
   // mut menu_state: ResMut<NextState<MenuState>>,
    mut game_state: ResMut<NextState<GameState>>,
    mut ai1_state: ResMut<NextState<Ai1_State>>,
    mut ai2_state: ResMut<NextState<Ai2_State>>,
    mut ai3_state: ResMut<NextState<Ai3_State>>,
    mut uberai_state: ResMut<NextState<UberAi_State>>,
    paused_signal: Res<PausedSignal>,
    mut speed_sleep: ResMut<SleepTime>,
    mut activity_signal: Res<ActivitySignal>,
    mut camera_control: ResMut<CameraControl>,
    mut teleport_signal: ResMut<TeleportSignal>,
    button_icons: Res<ButtonIcons>,
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
        pause_button,
        increase_speed,
        decrease_speed,
        activity_button,
        teleport_button,
        ui_image,
    ) in &mut interaction_query
    {
        ;
        match *interaction {
            Interaction::Pressed => {;

                if zoomin.is_some() && !camera_control.is_zooming {
                    camera_control.is_zooming = true;
                    adjust_camera_zoom_and_position(0.03, &mut camera_query, &robot_position);
                    camera_control.is_zooming = false;
                }

                if zoomout.is_some() && !camera_control.is_zooming {
                    camera_control.is_zooming = true;
                    adjust_camera_zoom_and_position(-0.03, &mut camera_query, &robot_position);
                    camera_control.is_zooming = false;

                } else if dropdown.is_some() {
                    for mut node_style in label_query.iter_mut() {
                        if node_style.display == Display::None {
                            node_style.display = Display::Flex; 
                        } else {
                            node_style.display = Display::None; 
                        }
                    }
                    //PING
                } else if closeapp.is_some() {
                    game_state.set(GameState::InMenu);
                    ai1_state.set(Ai1_State::Out);
                    ai2_state.set(Ai2_State::Out);
                    ai3_state.set(Ai3_State::Out);
                    uberai_state.set(UberAi_State::Out);

                    //label
                } else if dropdownback.is_some() {
                    for mut node_style in label_backpack_query.iter_mut() {
                        if node_style.display == Display::None {
                            node_style.display = Display::Flex;
                        } else {
                            node_style.display = Display::None;
                        }
                    }

                    //pausa
                } else if pause_button.is_some() {
                    let current_state = paused_signal.0.load(Ordering::SeqCst);
                    paused_signal.0.store(!current_state, Ordering::SeqCst);
                    println!("Stato di pausa cambiato: {}", !current_state);
                    if current_state {
                        ui_image.unwrap().texture = button_icons.play.clone();
                    } else {
                        ui_image.unwrap().texture = button_icons.pause.clone();

                    }
                    
                } else if increase_speed.is_some() {
                    // Ottieni il valore corrente del tempo di sleep
                    let current_sleep_time = speed_sleep.millis.load(Ordering::SeqCst);
                    
                    // Calcola il nuovo valore del tempo di sleep senza superare i 1000 millisecondi
                    let new_sleep_time = (current_sleep_time + 150).min(1000); // Aumenta di 250 senza superare 1000
                    
                    // Aggiorna il valore del tempo di sleep solo se non supera 1000
                    if current_sleep_time < 1000 {
                        speed_sleep.millis.store(new_sleep_time, Ordering::SeqCst);
                        println!("Tempo di sleep aumentato a: {}", new_sleep_time);
                    }
                    
                } else if decrease_speed.is_some() {
                    // Ottieni il valore corrente del tempo di sleep
                    let current_sleep_time = speed_sleep.millis.load(Ordering::SeqCst);
                    
                    // Calcola il nuovo valore del tempo di sleep senza scendere sotto i 30 millisecondi
                    let new_sleep_time = current_sleep_time.saturating_sub(150).max(30); // Diminuisce di 250 senza scendere sotto 30
                    
                    // Aggiorna il valore del tempo di sleep solo se è maggiore di 30
                    if current_sleep_time > 30 {
                        speed_sleep.millis.store(new_sleep_time, Ordering::SeqCst);
                        println!("Tempo di sleep diminuito a: {}", new_sleep_time);
                    }
                }
                else if activity_button.is_some() {
                    let current_state = activity_signal.0.load(Ordering::SeqCst);
                    activity_signal.0.store(!current_state, Ordering::SeqCst);
                    if !current_state{
                        ui_image.unwrap().texture = button_icons.actionin.clone();
                    }else{
                        ui_image.unwrap().texture = button_icons.actionout.clone();
                    }
                    println!("Stato di attivita cambiato: {}", !current_state);
                }
                else if teleport_button.is_some() {
                    let current_state = teleport_signal.0.load(Ordering::SeqCst);
                    teleport_signal.0.store(!current_state, Ordering::SeqCst);
                    if !current_state{
                        ui_image.unwrap().texture = button_icons.teleportactive.clone();
                    } else {
                        ui_image.unwrap().texture = button_icons.teleport.clone();
                    }
                    println!("Stato di attivita cambiato: {}", !current_state);
                }

                *color = Color::BLUE.into();
                border_color.0 = Color::RED;
            }
            Interaction::Hovered => {
                if dropdownback.is_some() {
                    *color = Color::BLACK.into();
                    border_color.0 = Color::GREEN;
                } else if closeapp.is_some() {
                   *color = Color::BLACK.into();
                   border_color.0 = Color::GREEN;
                } else if dropdown.is_some() {
                    *color = Color::BLACK.into();
                    border_color.0 = Color::GREEN;
                } else {
                    *color = Color::BLACK.into();
                    border_color.0 = Color::GREEN;
                }
            }
            
            
            Interaction::None => {
                if dropdownback.is_some() {
                    *color = Color::DARK_GRAY.into();
                    border_color.0 = Color::BLACK;
                } else if closeapp.is_some() {
                    *color = Color::DARK_GRAY.into();
                    border_color.0 = Color::BLACK;
                } else if dropdown.is_some() {
                    *color = Color::DARK_GRAY.into();
                    border_color.0 = Color::BLACK;
                } else{
                    *color = Color::WHITE.into();
                    border_color.0 = Color::BLACK;
                }
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

           
            let max_scale = max_scale_width.min(max_scale_height);

         
            let max_zoom = MAX_ZOOM.min(max_scale);

            //controlla che lo zoom sia nello scale
            let new_scale = (transform.scale.x + zoom_change).clamp(MIN_ZOOM, max_zoom);
            transform.scale.x = new_scale;
            transform.scale.y = new_scale;

            //controlla che la vista della camera non sia mai più grande del mondo di gioco
            let camera_half_width = ((viewport.physical_size.x as f32 / new_scale) / 2.0)
                .min(WORLD_SIZE as f32 * TILE_SIZE / 2.0);
            let camera_half_height = ((viewport.physical_size.y as f32 / new_scale) / 2.0)
                .min(WORLD_SIZE as f32 * TILE_SIZE / 2.0);

            
            let world_min_x = camera_half_width;
            let world_max_x = WORLD_SIZE as f32 * TILE_SIZE - camera_half_width;
            let world_min_y = camera_half_height;
            let world_max_y = WORLD_SIZE as f32 * TILE_SIZE - camera_half_height;

            
            if world_min_x > world_max_x || world_min_y > world_max_y {
                eprintln!("Il mondo di gioco è troppo piccolo per il livello di zoom attuale!");
                return;
            }

            
            transform.translation.x = robot_position.x;
            transform.translation.y = robot_position.y;
        }
    }
}

const NORMAL_BUTTON: Color = Color::rgb(0.15, 0.15, 0.15);
const NORMAL_BUTTON2: Color = Color::rgb(0.83, 0.83, 0.83);
const HOVERED_BUTTON: Color = Color::rgb(0.25, 0.25, 0.25);
const HOVERED_PRESSED_BUTTON: Color = Color::rgb(0.25, 0.65, 0.25);
const PRESSED_BUTTON: Color = Color::rgb(0.35, 0.75, 0.35);

//extra
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

fn icons_upgrade(
    mut tracker: ResMut<ContentTracker>,
 //  mut old_value: ResMut<OldValueBackPack>,
    weather_icons: Option<Res<WeatherIcons>>,
    robot_resource: Res<RobotResource>,
    mut energy_query: Query<
        &mut Text,
        (
            With<TagEnergy>,
            Without<Roboto>,
            Without<TagTime>,
            Without<TagBackPack>,
            Without<TagCoordinate>,
        ),
    >,
    mut coordinate_quert: Query<
        &mut Text,
        (
            With<TagCoordinate>,
            Without<Roboto>,
            Without<TagTime>,
            Without<TagBackPack>,
            Without<TagEnergy>,
        )
    >,
    mut time_query: Query<
        &mut Text,
        (
            With<TagTime>,
            Without<TagEnergy>,
            Without<Roboto>,
            Without<TagBackPack>,
            Without<TagCoordinate>,
        ),
    >,
    mut backpack_query: Query<
        ( &mut Text,
           &TagItem),       
            (
            With<TagBackPack>,
            Without<TagEnergy>,
            Without<Roboto>,
            Without<TagTime>,
            Without<TagCoordinate>,
        ),
    >,
    battery_query: Query<(&mut Style, &mut BackgroundColor), With<EnergyBar>>,
    sun_query: Query<&mut Sprite, With<SunTime>>,
    weather_image_query: Query<&mut UiImage, With<WeatherIcon>>,
    font_res: Res<FontResource>,
){
    if let Some(weather_icons) = weather_icons {

        let resource = robot_resource.0.lock().unwrap();
        let resource_copy = resource.clone();
        drop(resource);

        update_infos(
            resource_copy.clone(),
            weather_icons,
            tracker,
           // old_value,
            energy_query,
            coordinate_quert,
            time_query,
            backpack_query,
            battery_query,
            sun_query,
            weather_image_query,
            font_res,
        );
    }
}



//movimento del robot in base alla grandezza di una tile
//UPDATES
fn robot_movement_system_maze(
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
    tile_size: Res<TileSize>, 
    robot_resource: Res<RobotResource>,
    world: Res<MapResource>,
    
    tile_icons: Res<TileIcons>,
    content_icons: Res<ContentIcons>,
    mut old_world_query: Query<&mut OldMapResource>,
    robot_position: Res<RobotPosition>,
    last_update: ResMut<LastUpdate>, 
    content_counter: ResMut<ContentCounter>,
    mut query_sprite: Query<(Entity, &TilePosition), With<Sprite>>,
    
) {
    let world = world.0.lock().unwrap();
    if let Ok(mut old_world_res) = old_world_query.get_single_mut() {
        let old_world = &mut old_world_res.world;
        update_show_tiles_maze(&world, &mut commands, old_world, &tile_icons, &content_icons, &robot_position, last_update, query_sprite, content_counter); // Passa direttamente old_world
    }
    drop(world);
    let resource = robot_resource.0.lock().unwrap();
    let tile_step = tile_size.tile_size; 
    let resource_copy = resource.clone();
    drop(resource);
   
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
        transform.translation.x = tile_step * resource_copy.coordinate_column as f32;
        transform.translation.y = tile_step * resource_copy.coordinate_row as f32;
    }
}


fn robot_movement_system(
    commands: Commands,
    mut transform_query: Query<
        &mut Transform,
        (
            With<Roboto>,
            Without<TagEnergy>,
            Without<TagTime>,
            Without<TagBackPack>,
            Without<DirectionalLight>,
        ),
    >,
    tile_size: Res<TileSize>, 
    robot_resource: Res<RobotResource>,
    world: Res<MapResource>,
    tile_icons: Res<TileIcons>,
    content_icons: Res<ContentIcons>,
    mut old_world_query: Query<&mut OldMapResource>,
    robot_position: Res<RobotPosition>,
    mut last_update: ResMut<LastUpdate>,
    mut tile_position_query: Query<(Entity, &TilePosition), With<Sprite>>,
    mut content_counter: ResMut<ContentCounter>,
    discovered_signal: Res<DiscoveredSignal>,
) {
    let world = world.0.lock().unwrap();
    if let Ok(mut old_world_res) = old_world_query.get_single_mut() {
        let old_world = &mut old_world_res.world;
        // Corretto il passaggio di parametri a update_show_tiles
        update_show_tiles(&world, commands, old_world, tile_icons, content_icons, robot_position, last_update, tile_position_query, content_counter, discovered_signal);
    }
    drop(world);
    let resource = robot_resource.0.lock().unwrap();
    let tile_step = tile_size.tile_size; 
    let resource_copy = resource.clone();
    drop(resource);
    
    for mut transform in transform_query.iter_mut() {
        transform.translation.x = tile_step * resource_copy.coordinate_column as f32;
        transform.translation.y = tile_step * resource_copy.coordinate_row as f32;
    }
}
//serve per avere la posizione del puntino rosso ad ogni movimento
fn update_robot_position(
    mut robot_position: ResMut<RobotPosition>,
    robot_query: Query<&Transform, With<Roboto>>,
    mut robot_sprite_query: Query<&mut Sprite, With<Roboto>>,
    mut previous_x: ResMut<PreviousX>,
) {
    if let Ok(robot_transform) = robot_query.get_single() {
        robot_position.x = robot_transform.translation.x;
        robot_position.y = robot_transform.translation.y;

        if let Ok(mut robot_sprite) = robot_sprite_query.get_single_mut() {
            if robot_position.x < previous_x.0 {
                robot_sprite.flip_x = true;
            } else if robot_position.x > previous_x.0 {
                robot_sprite.flip_x = false;
            }
            // Update previous_x to current x
            previous_x.0 = robot_transform.translation.x;
        }
    }
}


//CAMERA CHE FOLLOWA IL ROBOT, NUOVO
fn follow_robot_system(
    robot_position: Res<RobotPosition>,
    mut camera_query: Query<(&mut Transform, &Camera), With<MainCamera>>,
    camera_follow: Res<CameraFollow>, // Aggiungi questo
    camera_control: ResMut<CameraControl>,
) {
    if camera_follow.follow_robot && !camera_control.is_zooming { // Controlla il flag prima di aggiornare la posizione
        if let Ok((mut camera_transform, camera)) = camera_query.get_single_mut() {
            if let Some(viewport) = &camera.viewport {
                let camera_scale = camera_transform.scale;
                let camera_half_width = (viewport.physical_size.x as f32 * camera_scale.x) / 3.3;
                let camera_half_height = (viewport.physical_size.y as f32 * camera_scale.y) / 3.3;
        
                let world_min_x = camera_half_width;
                let world_max_x = WORLD_SIZE as f32 * TILE_SIZE - camera_half_width;
                let world_min_y = camera_half_height;
                let world_max_y = WORLD_SIZE as f32 * TILE_SIZE - camera_half_height;

                let new_camera_x = robot_position.x.clamp(world_min_x, world_max_x);
                let new_camera_y = robot_position.y.clamp(world_min_y, world_max_y);

                camera_transform.translation.x = new_camera_x;
                camera_transform.translation.y = new_camera_y;
                camera_transform.translation.z = 50.0; 
            }
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

enum AiLogic {
    Falegname,
    Asfaltatore,
    Ricercatore,
    Completo,
}

fn moviment(robot_data: Arc<Mutex<RobotInfo>>, map: Arc<Mutex<Vec<Vec<Option<Tile>>>>>, ai_logic: AiLogic,  shutdown_signal: Arc<AtomicBool>, paused_signal: Arc<AtomicBool>, sleep_time: Arc<AtomicU64>, activity_signal: Arc<AtomicBool>, teleport_signal: Arc<AtomicBool>, discovered_signal: Arc<AtomicBool>, firstcall_signal: Arc<AtomicBool>) {
    let audio = get_audio_manager();
    let background_music = OxAgSoundConfig::new_looped_with_volume("assets/audio/background.ogg", 1.0);

    let mut robot = Robottino {
        shared_map: map,
        shared_robot: robot_data,
        robot: Robot::new(),
        audio: audio,
        weather_tool: WeatherPredictionTool::new(),
        ai_logic: ai_logic,
        maze_discovered: None,
        circumnavigate_maze: false,
        tp_maze: None,
        activity_signal: activity_signal,
        teleport_signal: teleport_signal,
        sleep_time_signal: sleep_time.clone(),
        discover_signal: discovered_signal,
        firstcall_signal: firstcall_signal,
        maze_corners: vec![vec![None; 2]; 2],
        tiles_percentage: HashMap::new().into(),
    };


    // world generator initialization

    // Runnable creation and start

    // println!("Generating runnable (world + robot)...");
    //  match robot.audio.play_audio(&background_music) {
    //      Ok(_) => {},
    //      Err(e) => {
    //          eprintln!("Failed to play audio: {}", e);
    //          std::process::exit(1);
    //      }
    //  }
    let mut world_gen =
        ghost_amazeing_island::world_generator::WorldGenerator::new(WORLD_SIZE, false, 1, 1.1);
    let mut runner = Runner::new(Box::new(robot), &mut world_gen);
    println!("Runnable succesfully generated");

     //MOVIMENTO ROBOT
    while !shutdown_signal.load(Ordering::SeqCst) {
        if !paused_signal.load(Ordering::SeqCst) {
            let current_sleep_time = sleep_time.load(Ordering::SeqCst);
            std::thread::sleep(Duration::from_millis(current_sleep_time));
            // Esegui la logica di movimento solo se il robot non è in pausa
            let rtn = runner.as_mut().unwrap().game_tick();
            //sleep(std::time::Duration::from_secs(1));
        } else {
            // Opzionalmente, inserisci qui una pausa per ridurre l'utilizzo della CPU quando in pausa
            std::thread::sleep(std::time::Duration::from_millis(100));
        }

    }


}


#[derive(Clone)]
struct RobotResource(Arc<Mutex<RobotInfo>>);
struct MapResource(Arc<Mutex<Vec<Vec<Option<Tile>>>>>);

#[derive(Component, Clone)]
struct OldMapResource {
    world: Vec<Vec<Option<Tile>>>,
}

impl bevy::prelude::Resource for RobotResource {}
impl bevy::prelude::Resource for MapResource {}

use std::sync::{Arc, Mutex};
use std::thread;



#[derive(Debug)]
struct Position {
    x: i32,
    y: i32,
}

#[derive(Clone, Debug)]
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

//**************************** */
//MENU CODE
/**************************** */

/* fn setup_menu_camera(mut commands: Commands, query: Query<Entity, With<OnMainMenuCamera>>) {
    // Verifica se esiste già una camera con il componente OnMainMenuCamera
    let camera_exists = query.iter().next().is_some();

    if !camera_exists {
        commands.spawn(Camera2dBundle {
            camera: Camera {
                order: 0,
                ..default()
            },
            ..default()
        })
        .insert(OnMainMenuCamera);
    }
} */

fn update_camera_visibility_menu(
    game_state: Res<State<GameState>>,
    mut query: Query<(&mut Visibility, &OnMainMenuCamera)>,
) {
    let is_menu_active = matches!(game_state.get(), GameState::InMenu);

    for (mut visibility, _) in query.iter_mut() {
        *visibility = if is_menu_active {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
    }
}

fn update_camera_visibility_game(
    game_state: Res<State<GameState>>,
    mut query: Query<(&mut Visibility, (&MyMinimapCamera, &MainCamera))>,
) {
    let is_menu_active = !matches!(game_state.get(), GameState::InMenu);

    for (mut visibility, _) in query.iter_mut() {
        *visibility = if is_menu_active {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
    }
}
// Enum that will be used as a global state for the game
#[derive(Clone, Copy, Default, Eq, PartialEq, Debug, Hash, States)]
enum GameState {
    #[default]
    InMenu,
    InAi1,
    InAi2,
    InAi3,
    InUberAi
}

// Tag component used to mark which setting is currently selected
#[derive(Component)]
struct SelectedOption;

#[derive(Component)]
struct PauseButton;

#[derive(Component)]
struct IncreaseSpeed;


#[derive(Component)]
struct DecreaseSpeed;

#[derive(Component)]
struct ActivityButton;

#[derive(Component)]
struct TeleportButton;

#[derive(Resource, Debug, Default)] 
struct ShutdownSignal(Arc<AtomicBool>);

#[derive(Resource, Debug, Default)] 
struct PausedSignal(Arc<AtomicBool>);

#[derive(Resource, Debug, Default)] 
struct DiscoveredSignal(Arc<AtomicBool>);

#[derive(Resource, Debug, Default)] 
struct TeleportSignal(Arc<AtomicBool>);


#[derive(Resource, Debug, Default)] 
struct FirstCallSignal(Arc<AtomicBool>);

#[derive(Resource, Debug)] 
struct SleepTime {
    millis: Arc<AtomicU64>,
}

#[derive(Resource, Debug, Default)] 
struct ActivitySignal(Arc<AtomicBool>);

//BOTTONI DEL MAIN MENU
#[derive(Clone, Copy, Default, Eq, PartialEq, Debug, Hash, States)]
enum MenuState {
    Main,
    #[default]
    Disabled,
    Ai1,
    Ai2,
    Ai3,
    UberAi,
}

//BOTTONI DEL MAIN MENU
#[derive(Component)]
enum MenuButtonAction {
    AI1,
    AI2,
    AI3,
    UberAI,
    Exit,
}

fn menu_setup(mut menu_state: ResMut<NextState<MenuState>>) {
    sleep(std::time::Duration::from_secs(1));
    menu_state.set(MenuState::Main);
}
fn start_ai1(mut menu_state: ResMut<NextState<MenuState>>) {
    sleep(std::time::Duration::from_secs(1));
    menu_state.set(MenuState::Ai1);
}
fn start_ai2(mut menu_state: ResMut<NextState<MenuState>>) {
    sleep(std::time::Duration::from_secs(1));
    menu_state.set(MenuState::Ai2);
}
fn start_ai3(mut menu_state: ResMut<NextState<MenuState>>) {
    sleep(std::time::Duration::from_secs(1));
    menu_state.set(MenuState::Ai3);
}
fn start_uberai(mut menu_state: ResMut<NextState<MenuState>>) {
    sleep(std::time::Duration::from_secs(1));
    menu_state.set(MenuState::UberAi);
}
fn start_in_ai1(mut menu_state: ResMut<NextState<Ai1_State>>) {
    sleep(std::time::Duration::from_secs(1));
    menu_state.set(Ai1_State::In);
}
fn start_in_ai2(mut menu_state: ResMut<NextState<Ai2_State>>) {
    sleep(std::time::Duration::from_secs(1));
    menu_state.set(Ai2_State::In);
}
fn start_in_ai3(mut menu_state: ResMut<NextState<Ai3_State>>) {
    sleep(std::time::Duration::from_secs(1));
    menu_state.set(Ai3_State::In);
}
fn start_in_uberai(mut menu_state: ResMut<NextState<UberAi_State>>) {
    sleep(std::time::Duration::from_secs(1));
    menu_state.set(UberAi_State::In);
}
fn start_update_ai1(mut menu_state: ResMut<NextState<Ai1_State>>) {
    sleep(std::time::Duration::from_secs(1));
    menu_state.set(Ai1_State::Run);
}
fn start_update_ai2(mut menu_state: ResMut<NextState<Ai2_State>>) {
    sleep(std::time::Duration::from_secs(1));
    menu_state.set(Ai2_State::Run);
}
fn start_update_ai3(mut menu_state: ResMut<NextState<Ai3_State>>) {
    sleep(std::time::Duration::from_secs(1));
    menu_state.set(Ai3_State::Run);
}
fn start_update_uberai(mut menu_state: ResMut<NextState<UberAi_State>>) {
    sleep(std::time::Duration::from_secs(1));
    menu_state.set(UberAi_State::Run);
}

// Generic system that takes a component as a parameter, and will despawn all entities with that component
fn despawn_screen<T: Component + std::fmt::Debug>(to_despawn: Query<Entity, With<T>>, mut commands: Commands) {
    for entity in &to_despawn {
      //  log::info!("Despawning entity with component: {:?}", entity);
        commands.entity(entity).despawn_recursive();
    }
}

// Generic system that takes a component as a parameter, and will despawn all entities with that component
fn despawn_screentry<T: Component + std::fmt::Debug>(to_despawn: Query<Entity, With<T>>, mut commands: Commands) {
    for entity in &to_despawn {
       log::info!("Despawning entity with component: {:?}", entity);
        commands.entity(entity).despawn_recursive();
    }
}

//PING
fn menu_action(
    interaction_query: Query<
        (&Interaction, &MenuButtonAction),
        (Changed<Interaction>, With<Button>),
    >,
    mut app_exit_events: EventWriter<AppExit>,
    mut menu_state: ResMut<NextState<MenuState>>,
    mut game_state: ResMut<NextState<GameState>>,
) {
    for (interaction, menu_button_action) in &interaction_query {
        if *interaction == Interaction::Pressed {
            match menu_button_action {
                MenuButtonAction::Exit => app_exit_events.send(AppExit),
                MenuButtonAction::AI1 => {
                    game_state.set(GameState::InAi1);
                    menu_state.set(MenuState::Disabled);
                }
                MenuButtonAction::AI2 => {
                    game_state.set(GameState::InAi2);
                    menu_state.set(MenuState::Disabled);
                }
                MenuButtonAction::AI3 => {
                    game_state.set(GameState::InAi3);
                    menu_state.set(MenuState::Disabled);
                }
                MenuButtonAction::UberAI => {
                    game_state.set(GameState::InUberAi);
                    menu_state.set(MenuState::Disabled);
                }
            }
        }
    }
}

        // This system handles changing all buttons color based on mouse interaction
        fn button_system_menu(
            mut interaction_query: Query<
                (&Interaction, &mut BackgroundColor, Option<&SelectedOption>),
                (Changed<Interaction>, With<Button>),
            >,
        ) {
            for (interaction, mut color, selected) in &mut interaction_query {
                *color = match (*interaction, selected) {
                    (Interaction::Pressed, _) | (Interaction::None, Some(_)) => PRESSED_BUTTON.into(),
                    (Interaction::Hovered, Some(_)) => HOVERED_PRESSED_BUTTON.into(),
                    (Interaction::Hovered, None) => HOVERED_BUTTON.into(),
                    (Interaction::None, None) => NORMAL_BUTTON.into(),
                }
            }
        }



    //PLUGIN MAIN MENU
    pub struct MenuPlugin;

    impl Plugin for MenuPlugin {
        fn build(&self, app: &mut App) {

            println!("entrato in menu");
            app
                // At start, the menu is not enabled. This will be changed in `menu_setup` when
                // entering the `GameState::Menu` state.
                // Current screen in the menu is handled by an independent state from `GameState`
                .add_state::<MenuState>()
                .add_systems(OnEnter(GameState::InMenu), menu_setup)
                // Systems to handle the main menu screen
                .add_systems(OnEnter(MenuState::Main), (initial_menu_setup))
                .add_systems(OnExit(MenuState::Main), (despawn_screen::<OnMainMenuScreen>))
                
                // Common systems to all screens that handles buttons behavior
                .add_systems(
                    Update,
                    (menu_action, button_system_menu).run_if(in_state(GameState::InMenu)),
                );
        }
    }

    fn stop_ai_thread(
        shutdown_signal: Res<ShutdownSignal>,
    ) {
        // Attiva il segnale di shutdown
        shutdown_signal.0.store(true, Ordering::SeqCst);
    
        // Qui puoi gestire altre operazioni di pulizia se necessario
        // Ad esempio, attendere che il thread termini, se hai conservato il suo handle
    }

    fn break_thread(
        paused_signal: Res<PausedSignal>,
    ) {
        
        paused_signal.0.store(true, Ordering::SeqCst);
    
    }

    fn run_thread(
        paused_signal: Res<PausedSignal>,
    ) {
        
        paused_signal.0.store(false, Ordering::SeqCst);
    
    }

    fn initialize_and_start_thread_ai1(
        mut commands: Commands,
        // altri parametri se necessari...
    ) {
        println!("Inizializzazione del thread AI...");
    
        // Creazione e inizializzazione di RobotInfo
        let robot_info = RobotInfo {
            energy_level: 1000,
            coordinate_row: 0,
            coordinate_column: 0,
            bp_size: 10,
            bp_contents: HashMap::new(),
            current_weather: None,
            next_weather: None,
            ticks_until_change: 0,
            time: "00:00".to_string(),
        };
    
        println!("RobotInfo inizializzato con: {:?}", robot_info);
    
        // Creazione di Arc<Mutex<>> per robot_data e map
        let robot_data = Arc::new(Mutex::new(robot_info));
        let map = Arc::new(Mutex::new(vec![vec![None; WORLD_SIZE as usize]; WORLD_SIZE as usize]));
    
        // Clonazione delle risorse condivise
        let robot_data_clone = robot_data.clone();
        let map_clone = map.clone();
    
        println!("Risorse condivise (robot_data e map) create e clonate");
        
        // Inserimento delle risorse nel sistema
        commands.insert_resource(RobotResource(robot_data_clone));
        commands.insert_resource(MapResource(map_clone));
    
        println!("Risorse inserite nel sistema di Bevy");

        let sleep_time_arc = Arc::new(AtomicU64::new(300));
        commands.insert_resource(SleepTime{ millis: sleep_time_arc.clone() });

        let activity_signal = Arc::new(AtomicBool::new(false)); // false significa che il robot non fa attivita' principale
        commands.insert_resource(ActivitySignal(activity_signal.clone()));

        let discovered_signal = Arc::new(AtomicBool::new(false)); // false significa che il robot non fa attivita' principale
        commands.insert_resource(DiscoveredSignal(discovered_signal.clone()));

        let teleport_signal = Arc::new(AtomicBool::new(false)); // false significa che il robot non fa attivita' principale
        commands.insert_resource(TeleportSignal(teleport_signal.clone()));

        let firstcall_signal = Arc::new(AtomicBool::new(false)); // false significa che il robot non fa attivita' principale
        commands.insert_resource(FirstCallSignal(firstcall_signal.clone()));
        
        // Creazione del segnale di shutdown
        let shutdown_signal = Arc::new(AtomicBool::new(false));
        let paused_signal = Arc::new(AtomicBool::new(false)); // false significa che il robot non è in pausa

        commands.insert_resource(ShutdownSignal(shutdown_signal.clone()));
        commands.insert_resource(PausedSignal(paused_signal.clone()));

    
        println!("Segnale di shutdown creato");
    
        // Avvio del thread
        let thread_handle = thread::spawn(move || {
            println!("Thread AI avviato");
    
            match std::panic::catch_unwind(|| {
                moviment(robot_data, map, AiLogic::Falegname, shutdown_signal.clone(), paused_signal.clone(), sleep_time_arc.clone(), activity_signal.clone(), teleport_signal.clone(), discovered_signal.clone(), firstcall_signal.clone());
            }) {
                Ok(_) => println!("Thread AI completato con successo"),
                Err(_) => println!("Thread AI terminato a causa di un panic"),
            }
        });
    
        // Opzionale: attendere la fine del thread se necessario
        // thread_handle.join().unwrap();
        // println!("Thread AI terminato e unito correttamente");
    }

    fn initialize_and_start_thread_ai2(
        mut commands: Commands,
        // altri parametri se necessari...
    ) {
        // Creazione e inizializzazione di RobotInfo
        let robot_info = RobotInfo {
            energy_level: 1000,
            coordinate_row: 0,
            coordinate_column: 0,
            bp_size: 10,
            bp_contents: HashMap::new(),
            current_weather: None,
            next_weather: None,
            ticks_until_change: 0,
            time: "00:00".to_string(),
        };
    
        // Creazione di Arc<Mutex<>> per robot_data e map
        let robot_data = Arc::new(Mutex::new(robot_info));
        let map = Arc::new(Mutex::new(vec![vec![None; WORLD_SIZE as usize]; WORLD_SIZE as usize]));
    
        // Clonazione delle risorse condivise
        let robot_data_clone = robot_data.clone();
        let map_clone = map.clone();
    
        // Inserimento delle risorse nel sistema
        commands.insert_resource(RobotResource(robot_data_clone));
        commands.insert_resource(MapResource(map_clone));

        //sleep(std::time::Duration::from_secs(3));
        let sleep_time_arc = Arc::new(AtomicU64::new(300));
        commands.insert_resource(SleepTime{ millis: sleep_time_arc.clone() });

          // Creazione del segnale di shutdown
        let shutdown_signal = Arc::new(AtomicBool::new(false));
        let paused_signal = Arc::new(AtomicBool::new(false)); // false significa che il robot non è in pausa

        commands.insert_resource(ShutdownSignal(shutdown_signal.clone()));
        commands.insert_resource(PausedSignal(paused_signal.clone()));

        let activity_signal = Arc::new(AtomicBool::new(false)); // false significa che il robot non fa attivita' principale
        commands.insert_resource(ActivitySignal(activity_signal.clone()));

        let discovered_signal = Arc::new(AtomicBool::new(false)); // false significa che il robot non fa attivita' principale
        commands.insert_resource(DiscoveredSignal(discovered_signal.clone()));

        let teleport_signal = Arc::new(AtomicBool::new(false)); // false significa che il robot non fa attivita' principale
        commands.insert_resource(TeleportSignal(teleport_signal.clone()));

        let firstcall_signal = Arc::new(AtomicBool::new(false)); // false significa che il robot non fa attivita' principale
        commands.insert_resource(FirstCallSignal(firstcall_signal.clone()));

        
    
        // Avvio del thread
        let thread_handle = thread::spawn(move || {
            //thread::sleep(std::time::Duration::from_secs(10));
            println!("Thread started");
            match std::panic::catch_unwind(|| {
                moviment(robot_data, map, AiLogic::Asfaltatore, shutdown_signal.clone(), paused_signal.clone(), sleep_time_arc.clone(), activity_signal.clone(), teleport_signal.clone(), discovered_signal.clone(), firstcall_signal.clone());
            }) {
                Ok(_) => println!("Thread completed successfully"),
                Err(_) => println!("Thread terminated due to panic"),
            }
        });

      

        //sleep(std::time::Duration::from_secs(3));

        //println!("PRIMA");
       //thread_handle.join().unwrap();
      // println!("JOINNNNNNNN");
    
    }


    fn initialize_and_start_thread_ai3(
        mut commands: Commands,
        // altri parametri se necessari...
    ) {
        // Creazione e inizializzazione di RobotInfo
        let robot_info = RobotInfo {
            energy_level: 1000,
            coordinate_row: 0,
            coordinate_column: 0,
            bp_size: 10,
            bp_contents: HashMap::new(),
            current_weather: None,
            next_weather: None,
            ticks_until_change: 0,
            time: "00:00".to_string(),
        };
    
        // Creazione di Arc<Mutex<>> per robot_data e map
        let robot_data = Arc::new(Mutex::new(robot_info));
        let map = Arc::new(Mutex::new(vec![vec![None; WORLD_SIZE as usize]; WORLD_SIZE as usize]));
    
        // Clonazione delle risorse condivise
        let robot_data_clone = robot_data.clone();
        let map_clone = map.clone();
    
        // Inserimento delle risorse nel sistema
        commands.insert_resource(RobotResource(robot_data_clone));
        commands.insert_resource(MapResource(map_clone));

        //sleep(std::time::Duration::from_secs(3));
        let sleep_time_arc = Arc::new(AtomicU64::new(300));
        commands.insert_resource(SleepTime{ millis: sleep_time_arc.clone() });

          // Creazione del segnale di shutdown
        let shutdown_signal = Arc::new(AtomicBool::new(false));
        let paused_signal = Arc::new(AtomicBool::new(false)); // false significa che il robot non è in pausa

        commands.insert_resource(ShutdownSignal(shutdown_signal.clone()));
        commands.insert_resource(PausedSignal(paused_signal.clone()));

        let activity_signal = Arc::new(AtomicBool::new(false)); // false significa che il robot non fa attivita' principale
        commands.insert_resource(ActivitySignal(activity_signal.clone()));

        let discovered_signal = Arc::new(AtomicBool::new(false)); // false significa che il robot non fa attivita' principale
        commands.insert_resource(DiscoveredSignal(discovered_signal.clone()));

        let teleport_signal = Arc::new(AtomicBool::new(false)); // false significa che il robot non fa attivita' principale
        commands.insert_resource(TeleportSignal(teleport_signal.clone()));

        let firstcall_signal = Arc::new(AtomicBool::new(false)); // false significa che il robot non fa attivita' principale
        commands.insert_resource(FirstCallSignal(firstcall_signal.clone()));

       
    
        // Avvio del thread
        let thread_handle = thread::spawn(move || {
            //thread::sleep(std::time::Duration::from_secs(10));
            println!("Thread started");
            match std::panic::catch_unwind(|| {
                moviment(robot_data, map, AiLogic::Ricercatore, shutdown_signal.clone(), paused_signal.clone(), sleep_time_arc.clone(), activity_signal.clone(), teleport_signal.clone(), discovered_signal.clone(), firstcall_signal.clone());
            }) {
                Ok(_) => println!("Thread completed successfully"),
                Err(_) => println!("Thread terminated due to panic"),
            }
        });

      

        //sleep(std::time::Duration::from_secs(3));

        //println!("PRIMA");
       //thread_handle.join().unwrap();
      // println!("JOINNNNNNNN");
    
    }

    fn initialize_and_start_thread_uberAi(
        mut commands: Commands,
        // altri parametri se necessari...
    ) {
        // Creazione e inizializzazione di RobotInfo
        let robot_info = RobotInfo {
            energy_level: 1000,
            coordinate_row: 0,
            coordinate_column: 0,
            bp_size: 10,
            bp_contents: HashMap::new(),
            current_weather: None,
            next_weather: None,
            ticks_until_change: 0,
            time: "00:00".to_string(),
        };
    
        // Creazione di Arc<Mutex<>> per robot_data e map
        let robot_data = Arc::new(Mutex::new(robot_info));
        let map = Arc::new(Mutex::new(vec![vec![None; WORLD_SIZE as usize]; WORLD_SIZE as usize]));
    
        // Clonazione delle risorse condivise
        let robot_data_clone = robot_data.clone();
        let map_clone = map.clone();
    
        // Inserimento delle risorse nel sistema
        commands.insert_resource(RobotResource(robot_data_clone));
        commands.insert_resource(MapResource(map_clone));

        //sleep(std::time::Duration::from_secs(3));
        let sleep_time_arc = Arc::new(AtomicU64::new(300));
        commands.insert_resource(SleepTime{ millis: sleep_time_arc.clone() });


          // Creazione del segnale di shutdown
        let shutdown_signal = Arc::new(AtomicBool::new(false));
        let paused_signal = Arc::new(AtomicBool::new(false)); // false significa che il robot non è in pausa

        commands.insert_resource(ShutdownSignal(shutdown_signal.clone()));
        commands.insert_resource(PausedSignal(paused_signal.clone()));
        let activity_signal = Arc::new(AtomicBool::new(false)); // false significa che il robot non fa attivita' principale
        commands.insert_resource(ActivitySignal(activity_signal.clone()));

        let discovered_signal = Arc::new(AtomicBool::new(false)); // false significa che il robot non fa attivita' principale
        commands.insert_resource(DiscoveredSignal(discovered_signal.clone()));

        let teleport_signal = Arc::new(AtomicBool::new(false)); // false significa che il robot non fa attivita' principale
        commands.insert_resource(TeleportSignal(teleport_signal.clone()));

        let firstcall_signal = Arc::new(AtomicBool::new(false)); // false significa che il robot non fa attivita' principale
        commands.insert_resource(FirstCallSignal(firstcall_signal.clone()));

    
        // Avvio del thread
        let thread_handle = thread::spawn(move || {
            //thread::sleep(std::time::Duration::from_secs(10));
            println!("Thread started");
            match std::panic::catch_unwind(|| {
                moviment(robot_data, map, AiLogic::Completo, shutdown_signal.clone(), paused_signal.clone(), sleep_time_arc.clone(), activity_signal.clone(), teleport_signal.clone(), discovered_signal.clone(), firstcall_signal.clone());
            }) {
                Ok(_) => println!("Thread completed successfully"),
                Err(_) => println!("Thread terminated due to panic"),
            }
        });

      

        //sleep(std::time::Duration::from_secs(3));

        //println!("PRIMA");
       //thread_handle.join().unwrap();
      // println!("JOINNNNNNNN");
    
    }



    


    //PLUGIN AI1
    pub struct Ai1Plugin;

    impl Plugin for Ai1Plugin {
        fn build(&self, app: &mut App) {

            println!("entrato in ai1");

        app
        
        .add_state::<Ai1_State>()
        .add_systems(OnEnter(GameState::InAi1),(initialize_and_start_thread_ai1, start_ai1))
        .add_systems(OnEnter(MenuState::Ai1), (setup, start_in_ai1))
        .add_systems(OnEnter(Ai1_State::In), (set_camera_viewports, start_update_ai1))
        .add_systems(OnExit(MenuState::Ai1),(stop_ai_thread, despawn_screentry::<Explodetry>, despawn_screen::<Explode>))
        .add_systems(Update, (icons_upgrade, cursor_events, robot_movement_system, update_robot_position, follow_robot_system, button_system, button_system_backpack, update_minimap_outline,)
        .run_if(in_state(Ai1_State::Run)));


        }
    }


     //PLUGIN AI2
     pub struct Ai2Plugin;

     impl Plugin for Ai2Plugin {
         fn build(&self, app: &mut App) {
 
             println!("entrato in ai2");
             // Dati condivisi tra thread
        /*  let robot_info= RobotInfo{
             energy_level: 1000,
             coordinate_row: 0,
             coordinate_column: 0,
             bp_size: 10,
             bp_contents: HashMap::new(),
             current_weather: None,
             next_weather: None,
             ticks_until_change: 0,
             time: "00:00".to_string()
         };
         
         let robot_data = Arc::new(Mutex::new(robot_info));
         let robot_data_clone = robot_data.clone();
 
         let map: Arc<Mutex<Vec<Vec<Option<Tile>>>>> = Arc::new(Mutex::new(vec![vec![None; WORLD_SIZE as usize]; WORLD_SIZE as usize]));
         let map_clone = map.clone();
 
 
         let robot_resource = RobotResource(robot_data_clone);
         let map_resource = MapResource(map_clone);
 
        thread::spawn(move || {
             moviment(robot_data, map);
         });  */
 
 
         app
         //.init_resource::<RobotPosition>()
         //.insert_resource(TileSize { tile_size: 3.0 })
         .add_state::<Ai2_State>()
         //.insert_resource(robot_resource)
         //.insert_resource(map_resource)
         .add_systems(OnEnter(GameState::InAi2),(initialize_and_start_thread_ai2, start_ai2))
         .add_systems(OnEnter(MenuState::Ai2), (setup, start_in_ai2))
         .add_systems(OnEnter(Ai2_State::In), (set_camera_viewports, start_update_ai2))
         .add_systems(OnExit(MenuState::Ai2),(stop_ai_thread, despawn_screen::<Explode>, despawn_screentry::<Explodetry>))
         .add_systems(Update, (icons_upgrade, cursor_events, robot_movement_system, update_robot_position, follow_robot_system, button_system, button_system_backpack, update_minimap_outline,).run_if(in_state(Ai2_State::Run)));
 
 
             //PROBLEMA
             //moviment.join().unwrap();
         }
     }


     //PLUGIN AI3
     pub struct Ai3Plugin;

     impl Plugin for Ai3Plugin {
         fn build(&self, app: &mut App) {
 
             println!("entrato in ai3");
             // Dati condivisi tra thread
        /*  let robot_info= RobotInfo{
             energy_level: 1000,
             coordinate_row: 0,
             coordinate_column: 0,
             bp_size: 10,
             bp_contents: HashMap::new(),
             current_weather: None,
             next_weather: None,
             ticks_until_change: 0,
             time: "00:00".to_string()
         };
         
         let robot_data = Arc::new(Mutex::new(robot_info));
         let robot_data_clone = robot_data.clone();
 
         let map: Arc<Mutex<Vec<Vec<Option<Tile>>>>> = Arc::new(Mutex::new(vec![vec![None; WORLD_SIZE as usize]; WORLD_SIZE as usize]));
         let map_clone = map.clone();
 
 
         let robot_resource = RobotResource(robot_data_clone);
         let map_resource = MapResource(map_clone);
 
        thread::spawn(move || {
             moviment(robot_data, map);
         });  */
 
 
         app
         //.init_resource::<RobotPosition>()
         //.insert_resource(TileSize { tile_size: 3.0 })
         .add_state::<Ai3_State>()
         //.insert_resource(robot_resource)
         //.insert_resource(map_resource)
         .add_systems(OnEnter(GameState::InAi3),(initialize_and_start_thread_ai3, start_ai3))
         .add_systems(OnEnter(MenuState::Ai3), (setup, start_in_ai3))
         .add_systems(OnEnter(Ai3_State::In), (set_camera_viewports, start_update_ai3))
         .add_systems(OnExit(MenuState::Ai3),(stop_ai_thread, despawn_screen::<Explode>, despawn_screentry::<Explodetry>))
         .add_systems(Update, (icons_upgrade, cursor_events, robot_movement_system, update_robot_position, follow_robot_system, button_system, button_system_backpack, update_minimap_outline,)
         .run_if(in_state(Ai3_State::Run)));
 
 
             //PROBLEMA
             //moviment.join().unwrap();
         }
     }


        //PLUGIN AI3
        pub struct UberAiPlugin;

        impl Plugin for UberAiPlugin {
            fn build(&self, app: &mut App) {
    
                println!("entrato in ai4");
                // Dati condivisi tra thread
           /*  let robot_info= RobotInfo{
                energy_level: 1000,
                coordinate_row: 0,
                coordinate_column: 0,
                bp_size: 10,
                bp_contents: HashMap::new(),
                current_weather: None,
                next_weather: None,
                ticks_until_change: 0,
                time: "00:00".to_string()
            };
            
            let robot_data = Arc::new(Mutex::new(robot_info));
            let robot_data_clone = robot_data.clone();
    
            let map: Arc<Mutex<Vec<Vec<Option<Tile>>>>> = Arc::new(Mutex::new(vec![vec![None; WORLD_SIZE as usize]; WORLD_SIZE as usize]));
            let map_clone = map.clone();
    
    
            let robot_resource = RobotResource(robot_data_clone);
            let map_resource = MapResource(map_clone);
    
           thread::spawn(move || {
                moviment(robot_data, map);
            });  */
    
    
            app
            //.init_resource::<RobotPosition>()
            //.insert_resource(TileSize { tile_size: 3.0 })
            .add_state::<UberAi_State>()
            //.insert_resource(robot_resource)
            //.insert_resource(map_resource)
            .add_systems(OnEnter(GameState::InUberAi),(initialize_and_start_thread_uberAi, start_uberai))
            .add_systems(OnEnter(MenuState::UberAi), (setup, start_in_uberai))
            .add_systems(OnEnter(UberAi_State::In), (set_camera_viewports, start_update_uberai))
            .add_systems(OnExit(MenuState::UberAi),(stop_ai_thread, despawn_screen::<Explode>, despawn_screentry::<Explodetry>))
            .add_systems(Update, (icons_upgrade, cursor_events, robot_movement_system, update_robot_position, follow_robot_system, button_system, button_system_backpack, update_minimap_outline,)
            .run_if(in_state(UberAi_State::Run)));
    
    
                //PROBLEMA
                //moviment.join().unwrap();
            }
        }

#[derive(Clone, Copy, Default, Eq, PartialEq, Debug, Hash, States)]
  enum Ai1_State{
    In, 
    #[default]
    Out, 
    Run
  }

  #[derive(Clone, Copy, Default, Eq, PartialEq, Debug, Hash, States)]
  enum Ai2_State{
    In, 
    #[default]
    Out, 
    Run
  }

  #[derive(Clone, Copy, Default, Eq, PartialEq, Debug, Hash, States)]
  enum Ai3_State{
    In, 
    #[default]
    Out, 
    Run
  }

  #[derive(Clone, Copy, Default, Eq, PartialEq, Debug, Hash, States)]
  enum UberAi_State{
    In, 
    #[default]
    Out, 
    Run
  }




 // Tag component used to tag entities added on the main menu screen
    #[derive(Component, Debug)]
    struct OnMainMenuScreen;

    #[derive(Component, Debug)]
    struct OnMainMenuCamera;


fn main() {
    
    /* // Dati condivisi tra thread
    let robot_info= RobotInfo{
        energy_level: 1000,
        coordinate_row: 0,
        coordinate_column: 0,
        bp_size: 10,
        bp_contents: HashMap::new(),
        current_weather: None,
        next_weather: None,
        ticks_until_change: 0,
        time: "00:00".to_string()
    };
    
    let robot_data = Arc::new(Mutex::new(robot_info));

    let map: Arc<Mutex<Vec<Vec<Option<Tile>>>>> = Arc::new(Mutex::new(vec![vec![None; WORLD_SIZE as usize]; WORLD_SIZE as usize]));

    let moviment = thread::spawn(move || {
        moviment(robot_data, map);
    }); */

    println!("entrato in main");

    

    App::new()
    .add_plugins(DefaultPlugins.set(WindowPlugin{
        primary_window: Some(Window{
            mode: WindowMode::BorderlessFullscreen,
            ..default()
        }),
        ..Default::default()
    }))
    .init_resource::<RobotPosition>()
    .insert_resource(TileSize { tile_size: 3.0 })
    .add_state::<GameState>()
    .add_plugins((MenuPlugin, Ai1Plugin, Ai2Plugin, Ai3Plugin, UberAiPlugin))
    .run();

    

  

    
}


struct Robottino {
    shared_robot: Arc<Mutex<RobotInfo>>,
    shared_map: Arc<Mutex<Vec<Vec<Option<Tile>>>>>,
    robot: Robot,
    audio: OxAgAudioTool,
    weather_tool: WeatherPredictionTool,
    ai_logic: AiLogic,
    maze_discovered: Option<(usize, usize)>,
    circumnavigate_maze: bool,
    maze_corners: Vec<Vec<Option<(usize, usize)>>>,
    tp_maze: Option<(usize, usize)>,
    activity_signal: Arc<AtomicBool>,
    teleport_signal: Arc<AtomicBool>,
    sleep_time_signal: Arc<AtomicU64>,
    discover_signal: Arc<AtomicBool>,
    firstcall_signal: Arc<AtomicBool>,
    tiles_percentage: Arc<HashMap<TileType, f64>>,
}

fn solve_labirint(
    robot: &mut Robottino,
    world: &mut robotics_lib::world::World,
    mut last_direction: Direction,
) { 
    robot.robot.energy = rust_and_furious_dynamo::dynamo::Dynamo::update_energy();
    let mut last_positions: Vec<Direction> = Vec::new();
    let mut stack: Vec<Direction> = Vec::new();

    loop {
        let tiles_option = cheapest_border(world, robot);
        if let Some(tiles) = tiles_option {
            //manage the return stat of move to cheapest border
            let result = move_to_cheapest_border(world, robot, tiles);}
        sleep(std::time::Duration::from_millis(300));
        robot_view(robot, world);
        update_map(robot, world);
    }
}

fn circumnavigate_maze(
    robot: &mut Robottino,
    world: &mut robotics_lib::world::World) {
    

    let view = robot_view(robot, world);
    //count number of wall
    let (count, count_down, count_right, count_up, count_left) = get_wall_count_view(&view);


    if  (count == 1 && (view[0][1].as_ref().unwrap().tile_type == TileType::Wall || view[0][2].as_ref().unwrap().tile_type == TileType::Wall)) || count_down > 1 {
        // destra
        if count == 1 && view[0][2].as_ref().unwrap().tile_type == TileType::Wall {
            robot.maze_corners[0][1] = Some((robot.robot.coordinate.get_row(), robot.robot.coordinate.get_col()));
        }
        let _ = go(robot, world, Direction::Right);
    }
    else if (count == 1 && (view[1][0].as_ref().unwrap().tile_type == TileType::Wall || view[0][0].as_ref().unwrap().tile_type == TileType::Wall)) || count_left > 1 {
        // giu
        if count == 1 && view[0][0].as_ref().unwrap().tile_type == TileType::Wall {
            robot.maze_corners[0][0] = Some((robot.robot.coordinate.get_row(), robot.robot.coordinate.get_col()));
        }
        let _ = go(robot, world, Direction::Up);
    }
    else if (count == 1 && (view[2][1].as_ref().unwrap().tile_type == TileType::Wall || view[2][0].as_ref().unwrap().tile_type == TileType::Wall)) || count_up > 1 {
        // sinistra
        if count == 1 && view[2][0].as_ref().unwrap().tile_type == TileType::Wall {
            robot.maze_corners[1][0] = Some((robot.robot.coordinate.get_row(), robot.robot.coordinate.get_col()));
        }
        let _ = go(robot, world, Direction::Left);
    }
    else if (count == 1 && (view[1][2].as_ref().unwrap().tile_type == TileType::Wall || view[2][2].as_ref().unwrap().tile_type == TileType::Wall)) || count_right > 1 {
        // su
        if count == 1 && view[2][2].as_ref().unwrap().tile_type == TileType::Wall {
            robot.maze_corners[1][1] = Some((robot.robot.coordinate.get_row(), robot.robot.coordinate.get_col()));
        }
        let _ = go(robot, world, Direction::Down);
    }   
        
}

fn get_wall_count_view(view: &Vec<Vec<Option<Tile>>>) -> (i32, i32, i32, i32, i32) {
    let mut count = 0;
    let mut count_down = 0;
    let mut count_right = 0;
    let mut count_up = 0;
    let mut count_left = 0;
            
    if let Some(tile) = view[0][0].as_ref() {
        if tile.tile_type == TileType::Wall {
            count += 1;
            count_down += 1;
            count_left += 1;
        }
    }

    if let Some(tile) = view[0][1].as_ref() {
        if tile.tile_type == TileType::Wall {
            count += 1;
            count_down += 1;
        }
    }
            
    if let Some(tile) = view[0][2].as_ref() {
        if tile.tile_type == TileType::Wall {
            count += 1;
            count_down += 1;
            count_right += 1;
        }
    }
            
    if let Some(tile) = view[1][0].as_ref() {
        if tile.tile_type == TileType::Wall {
            count += 1;
            count_left += 1;
        }
    }
            
    if let Some(tile) = view[1][2].as_ref() {
        if tile.tile_type == TileType::Wall {
            count += 1;
            count_right += 1;
        }
    }
            
    if let Some(tile) = view[2][0].as_ref() {
        if tile.tile_type == TileType::Wall {
            count += 1;
            count_up += 1;
            count_left += 1;
        }
    }
            
    if let Some(tile) = view[2][1].as_ref() {
        if tile.tile_type == TileType::Wall {
            count += 1;
            count_up += 1;
        }
    }
            
    if let Some(tile) = view[2][2].as_ref() {
        if tile.tile_type == TileType::Wall {
            count += 1;
            count_up += 1;
            count_right += 1;
        }
    }
    (count, count_down, count_right, count_up, count_left)
}

fn go_to_maze(robot: &mut Robottino, world: &mut robotics_lib::world::World, maze: (usize, usize)) {
    
    if let Some(directions) = nearest_tile_type(robot, world, TileType::Wall, false) {
        let view: Vec<Vec<Option<Tile>>> = robot_view(robot, world);
        for row in &view {
            for tile_option in row {
                if let Some(tile) = tile_option {
                    if tile.tile_type == TileType::Wall {
                        println!("Il robot ha raggiunto il labirinto.");
                        robot.circumnavigate_maze = true;
                    }
                }
            }
        }

        for direction in directions {
            let _ = go(robot, world, direction);
            break;
        }
            //  println!("Il robot ha raggiunto la destinazione o il teleport.");
    }
}

fn set_maze_location(robot: &mut Robottino, row: usize, column: usize) {
    robot.maze_discovered = Some((row, column));
}

fn ai_labirint(robot: &mut Robottino, world: &mut robotics_lib::world::World) {
    //maze are 18*18 so we check every 9 tiles
    //if robotmap some save it
    let search_teleport = robot.teleport_signal.load(Ordering::SeqCst);
    if search_teleport == true {
        println!("Il robot sta cercando il teleport...");
        if let Some(directions) = nearest_teleport(robot, world) {
            for direction in directions {
                    let _ = go(robot, world, direction);
                    break;
                }
                println!("Il robot ha raggiunto la destinazione o il teleport.");
            } else {
                    println!("Nessun percorso trovato al teleport.");
                }
    }else{
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
                                    set_maze_location(robot, row, col);
                                }
                            }
                            if let Some(tile) = &tiles[&(row - 1, col)] {
                                if tile.tile_type == TileType::Wall {
                                    set_maze_location(robot, row - 1, col);
                                }
                            }
                            if let Some(tile) = &tiles[&(row, col - 1)] {
                                if tile.tile_type == TileType::Wall {
                                    set_maze_location(robot, row, col - 1);
                                }
                            }
                            if let Some(tile) = &tiles[&(row - 1, col - 1)] {
                                if tile.tile_type == TileType::Wall {
                                    set_maze_location(robot, row - 1, col - 1);
                                }
                            }
                        }
                    }
                }
            }
        
        }
        
        if !robot.firstcall_signal.load(Ordering::SeqCst) {
            // println!("ENTRATOOOOOOOOOOOOOOO");
            robot.discover_signal.store(true, Ordering::SeqCst);
        }
        
        if robot.robot.energy.get_energy_level() < 300 {
            robot.robot.energy = rust_and_furious_dynamo::dynamo::Dynamo::update_energy();
        }
        robot_view(robot, world);
        
        // let tiles_option = cheapest_border(world, robot);
        //                 if let Some(tiles) = tiles_option {
        //                      let result = move_to_cheapest_border(world, robot, tiles);
        //                 }
        if !robot.firstcall_signal.load(Ordering::SeqCst) {
            sleep(Duration::from_millis(500));
            robot.firstcall_signal.store(true, Ordering::SeqCst);
        }
        

        if robot.maze_corners[0][0].is_some() && robot.maze_corners[0][1].is_some() && robot.maze_corners[1][0].is_some() && robot.maze_corners[1][1].is_some() {
            //summ all the corners row and column
            let mut row = 0;
            let mut col = 0;
            for i in 0..2 {
                for j in 0..2 {
                    row += robot.maze_corners[i][j].unwrap().0;
                    col += robot.maze_corners[i][j].unwrap().1;
                }
            }
            row /= 4;
            col /= 4;
            if let Some(directions) = go_to_coordinate(robot, world, Some((row, col)), false) {
                if directions.len() > 1 {
                    let mut length = directions.len();
                    for direction in directions {
                        length = length - 1;
                        if length == 0 {
                            break;
                        }
                        let _ = go(robot, world, direction);
                        break;
                    }
                } 
            }
        }
        else if robot.circumnavigate_maze {
            // println!("circumnavigooooo");
            circumnavigate_maze(robot, world);
        }
        else if let Some((row, col)) = robot.maze_discovered {
            go_to_maze(robot, world, (row, col));
        }
    }
}
fn ai_taglialegna(robot: &mut Robottino, world: &mut robotics_lib::world::World) {
    
    //se l'energia e' sotto il 300, la ricarico
    if robot.robot.energy.get_energy_level() < 300 {
        robot.robot.energy = rust_and_furious_dynamo::dynamo::Dynamo::update_energy();
    }

    let search_teleport = robot.teleport_signal.load(Ordering::SeqCst);
    if search_teleport == true {
        println!("Il robot sta cercando il teleport...");
        if let Some(directions) = nearest_teleport(robot, world) {
            for direction in directions {
                    let _ = go(robot, world, direction);
                    break;
                }
                println!("Il robot ha raggiunto la destinazione o il teleport.");
            } else {
                    println!("Nessun percorso trovato al teleport.");
                }
    }else{
        let v = robot_view(robot, world);
        let attivita = robot.activity_signal.load(Ordering::SeqCst);
        let a = robot.get_backpack().get_size();
        let b = robot.get_backpack().get_contents().values().sum::<usize>();
        if (a - 5) > b {
            if attivita == true{   
                // NEAREST TELEPORT (QUASI FUNZIONANTE)
                // if let Some(directions) = nearest_teleport(robot, world) {
                //         for direction in directions {
                //                 let _ = go(robot, world, direction);
                //                 break;
                //             }
                //             println!("Il robot ha raggiunto la destinazione o il teleport.");
                //         } else {
                //                 println!("Nessun percorso trovato al teleport.");
                //             }
                let mut shopping_list = ShoppingList {
                    list: vec![(
                        (Content::Tree(0), Some(OpActionInput::Destroy()))
                    )],
                };
    
                match get_best_action_to_element(robot, world, &mut shopping_list) {
                    None => {
                        let tiles_option = cheapest_border(world, robot);
                        if let Some(tiles) = tiles_option {
                             let result = move_to_cheapest_border(world, robot, tiles);             
                            if attivita == true{   
                                DestroyZone.execute(world, robot, Content::Tree(0));
                            }
                             let num_trees = robot.get_backpack().get_contents().get(&Content::Tree(0)).unwrap_or(&0);
                        }
                    }
                    Some(next_action) => {
                        // println!("{:?}", &rand);
                        println!("trovato albero?");
                        println!("{:?}", next_action);
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
            } else {
                let tiles_option = cheapest_border(world, robot);
                        if let Some(tiles) = tiles_option {
                             let result = move_to_cheapest_border(world, robot, tiles);             
                            if attivita == true{   
                                DestroyZone.execute(world, robot, Content::Tree(0));
                            }
                             let num_trees = robot.get_backpack().get_contents().get(&Content::Tree(0)).unwrap_or(&0);
                        }
            }
        } else {
            let mut shopping_list = ShoppingList {
                list: vec![(
                    Content::Crate(Range::default()),
                    Some(OpActionInput::Put(Content::Tree(0), a-5)),
                )],
            };
            match get_best_action_to_element(robot, world, &mut shopping_list) {
                None => {
                    let tiles_option = cheapest_border(world, robot);
                    if let Some(tiles) = tiles_option {
                        //manage the return stat of move to cheapest border
                        let result = move_to_cheapest_border(world, robot, tiles);
                        
                        // Debug print prima della distruzione/raccolta
                        // println!("Tentativo di raccogliere albero alla posizione corrente...");
                        if attivita == true{   
                            DestroyZone.execute(world, robot, Content::Tree(0));
                        }
            
                        // Debug print dopo la distruzione/raccolta
                    //  println!("Albero raccolto con successo!");
            
                        // Stampa opzionale per confermare il contenuto dello zaino
                        let num_trees = robot.get_backpack().get_contents().get(&Content::Tree(0)).unwrap_or(&0);
                    //  println!("Numero di alberi nello zaino: {}", num_trees);
                    }
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

}

fn ai_asfaltatore(robot: &mut Robottino, world: &mut robotics_lib::world::World) {
    if robot.robot.energy.get_energy_level() < 200 {
        robot.robot.energy = rust_and_furious_dynamo::dynamo::Dynamo::update_energy();
    }
    let search_teleport = robot.teleport_signal.load(Ordering::SeqCst);
    if search_teleport == true {
        println!("Il robot sta cercando il teleport...");
        if let Some(directions) = nearest_teleport(robot, world) {
            for direction in directions {
                    let _ = go(robot, world, direction);
                    break;
                }
                println!("Il robot ha raggiunto la destinazione o il teleport.");
            } else {
                    println!("Nessun percorso trovato al teleport.");
                }
    }else{
        let a = robot_view(robot, world);
    // println!("{:?}", a);
        let attivita = robot.activity_signal.load(Ordering::SeqCst);
    // println!("{:?}", attivita);
        let a = robot.get_backpack().get_size();
        let b = robot.get_backpack().get_contents().values().sum::<usize>();
        if b < 5 {
            if attivita == true{   
                let mut shopping_list = ShoppingList {
                    list: vec![(
                        (Content::Rock(0), Some(OpActionInput::Destroy()))
                    )],
                };

                match get_best_action_to_element(robot, world, &mut shopping_list) {
                    None => {
                        let tiles_option = cheapest_border(world, robot);
                        if let Some(tiles) = tiles_option {
                            let result = move_to_cheapest_border(world, robot, tiles);             
                            if attivita == true{   
                                DestroyZone.execute(world, robot, Content::Rock(0));
                            }
                        }
                    }
                    Some(next_action) => {
                    //   println!("trovata roccia?");
                    //  println!("{:?}", next_action);
                        match next_action {
                            OpActionOutput::Move(dir) => {
                                go(robot, world, dir);
                            }
                            OpActionOutput::Destroy(dir) => {
                                // println!("Destroy");
                                destroy(robot, world, dir);
                            }
                            OpActionOutput::Put(c, u, d) => {
                            // print!("depositandoooooooooooo");
                                //print c u d
                            //    println!("{:?} {:?} {:?}", c, u, d);
                                put(robot, world, c, u, d);
                            }
                        }
                    }
                }
            } else {
                let tiles_option = cheapest_border(world, robot);
                        if let Some(tiles) = tiles_option {
                            let result = move_to_cheapest_border(world, robot, tiles);             
                            if attivita == true{   
                                DestroyZone.execute(world, robot, Content::Rock(0));
                            }
                        }
            }
        } else {
            if let Some(directions) = nearest_tile_type(robot, world, TileType::ShallowWater, true) {
                println!("{:?}", directions);
                if directions.len() > 1 {
                    let mut length = directions.len();
                    for direction in directions {
                        length = length - 1;
                        if length == 0 {
                            break;
                        }
                        let _ = go(robot, world, direction);
                        let current_sleep_time = robot.sleep_time_signal.load(Ordering::SeqCst);
                        std::thread::sleep(Duration::from_millis(current_sleep_time));
                    }
                    //  println!("Il robot ha raggiunto la destinazione o il teleport.");
                    } 
                else if directions.len() == 1 {
                    for direction in directions {
                        let v = bessie::bessie::road_paving_machine(
                                    robot,
                                    world,
                                    direction,
                                    bessie::bessie::State::MakeRoad,
                                );
                            }
                } else{
                    
                    let tiles_option = cheapest_border(world, robot);
                        if let Some(tiles) = tiles_option {
                            let result = move_to_cheapest_border(world, robot, tiles);             
                            if attivita == true{   
                                DestroyZone.execute(world, robot, Content::Rock(0));
                            }
                        }
                } 
            }
        // nearest_tile_type(robot, world, Content::Water(0));
        //     sleep(std::time::Duration::from_millis(300));
        //     let v = bessie::bessie::road_paving_machine(
        //         robot,
        //         world,
        //         Direction::Up,
        //         bessie::bessie::State::MakeRoad,
        //     );
        //     //if err
        //     if v.is_err() {
        //         //random da 0 a 3
        //         let rand = rand::thread_rng().gen_range(0..4);
        //         match rand {
        //             0 => go(robot, world, Direction::Up),
        //             1 => go(robot, world, Direction::Down),
        //             2 => go(robot, world, Direction::Left),
        //             _ => go(robot, world, Direction::Right),
        //         };
        //     }
        }
    }
}
fn ai_completo_con_tool (robot: &mut Robottino, world: &mut robotics_lib::world::World) {
    //durata sleep in millisecondi per velocità robot
    let sleep_time_milly: u64 = 50;

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

        let new_weather = look_at_sky(world).get_weather_condition();
        let tiles_stats = discovered_tiles_stats(&robot_map(world));
        self.tiles_percentage = Arc::new(tiles_stats);

        /* let sleep_time_milly: u64 = 300;
        sleep(std::time::Duration::from_millis(sleep_time_milly)); */
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
        if self.robot.energy.get_energy_level() < 300 {
            self.robot.energy = rust_and_furious_dynamo::dynamo::Dynamo::update_energy();
        }
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

fn clear_map(map: &mut Vec<Vec<Option<Tile>>>) {
    for row in map.iter_mut() {
        for tile in row.iter_mut() {
            *tile = None;
        }
    }
}

fn update_map(robot: &mut Robottino, world: &mut robotics_lib::world::World) {
    let mut shared_map = robot.shared_map.lock().unwrap();

    if let Some(new_map) = robot_map(world) {
        *shared_map = new_map;
        
        // Utilizza filter_map per evitare di fare unwrap su None e colleziona solo i content di Tile
        let content_map = shared_map.iter()
            .map(|row| {
                row.iter()
                    .filter_map(|tile_option| tile_option.as_ref().map(|tile| tile.content.clone()))
                    .collect::<Vec<_>>() // Colleziona i content validi in un Vec
            })
            .collect::<Vec<_>>(); // Colleziona le righe in un Vec

    //   println!("mappa: {:?}", content_map);
    

    // Explicitly drop the lock
    drop(shared_map);
}
    let mut shared_robot = robot.shared_robot.lock().unwrap();
    let enviroment = look_at_sky(&world);

  //  println!("colonna: {:?}", shared_robot.coordinate_column);
  //  println!("riga: {:?}", shared_robot.coordinate_row);
  //  println!("contenuto {:?}", shared_robot.bp_contents);

    shared_robot.time = enviroment.get_time_of_day_string();
    shared_robot.current_weather = Some(enviroment.get_weather_condition());
    if let Some((prediction, ticks)) = weather_check(robot) {
        shared_robot.next_weather = Some(prediction);
        shared_robot.ticks_until_change = ticks;
    }
    drop(shared_robot);
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
    let background_music = OxAgSoundConfig::new_looped_with_volume("audio/background.ogg", 1.0);
    

    let mut events = HashMap::new();
    // events.insert(Event::Ready, OxAgSoundConfig::new("assets/default/event/event_ready.ogg"));
    //events.insert(Event::Terminated, OxAgSoundConfig::new("assets/default/event/event_terminated.ogg"));
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
    //weather.insert(WeatherType::Rainy, OxAgSoundConfig::new("audio/rainy.ogg"));
    //weather.insert(WeatherType::Foggy, OxAgSoundConfig::new("audio/foggy.ogg"));
    //weather.insert(WeatherType::Sunny, OxAgSoundConfig::new("audio/sunny.ogg"));
    //weather.insert(WeatherType::TrentinoSnow, OxAgSoundConfig::new("audio/trentino_snow.ogg"));
    //weather.insert(WeatherType::TropicalMonsoon, OxAgSoundConfig::new("audio/tropical_monsoon.ogg")); 

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
