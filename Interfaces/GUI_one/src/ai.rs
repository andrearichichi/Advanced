use std::{collections::HashMap, sync::{Arc, Mutex}, thread::sleep};

use arrusticini_destroy_zone::DestroyZone;
use crab_rave_explorer::algorithm::{cheapest_border, move_to_cheapest_border};
use ohcrab_weather::weather_tool::WeatherPredictionTool;
use oxagaudiotool::{sound_config::OxAgSoundConfig, OxAgAudioTool};
use robotics_lib::{energy::Energy, interface::{discover_tiles, look_at_sky, robot_map, robot_view}, runner::{backpack::BackPack, Robot, Runnable, Runner}, world::{coordinates::Coordinate, environmental_conditions::WeatherType, tile::{Content, Tile}}};

use crate::{RobotInfo, WORLD_SIZE};

impl Runnable for Robottino {
    fn process_tick(&mut self, world: &mut robotics_lib::world::World) {
        // in base alla logica scelta, esegue la funzione corrispondente
        match self.ai_logic {
            AiLogic::Falegname => ai_taglialegna(self, world),
            AiLogic::Asfaltatore => ai_asfaltatore(self, world),
            AiLogic::Ricercatore => ai_labirint(self, world),
            AiLogic::Completo => ai_completo_con_tool(self, world),
        }

        
        //update map
        let mut shared_map = self.shared_map.lock().unwrap();
        if let Some(new_map) = robot_map(world) {
            *shared_map = new_map;
        }
        let mut shared_robot = self.shared_robot.lock().unwrap();
        let enviroment = look_at_sky(&world);

        shared_robot.time = enviroment.get_time_of_day_string();
        shared_robot.current_weather = Some(enviroment.get_weather_condition());
        if let Some((prediction, ticks)) = weather_check(self) {
            shared_robot.next_weather = Some(prediction);
            shared_robot.ticks_until_change = ticks; 
        }
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

struct Robottino {
    shared_robot: Arc<Mutex<RobotInfo>>,
    shared_map: Arc<Mutex<Vec<Vec<Option<Tile>>>>>,
    robot: Robot,
    audio: OxAgAudioTool,
    weather_tool: WeatherPredictionTool,
    ai_logic: AiLogic
}

fn ai_labirint(robot: &mut Robottino, world: &mut robotics_lib::world::World){
    //maze are 18*18 so we check every 9 tiles
    //if robotmap some save it
    
    if let Some(map) = robot_map(world) {
    
        //quanto e' grande la mappa
        let map_size = map.len();
        let times_to_discover_map_for_side = map_size/9+1;
        for i in 1..times_to_discover_map_for_side {
            for j in 1..times_to_discover_map_for_side {
                if robot.robot.energy.get_energy_level() < 300 {
                    robot.robot.energy = rust_and_furious_dynamo::dynamo::Dynamo::update_energy();
                }
                let row = i*9;
                let col = j*9;
                println!("{:?}",discover_tiles(robot, world, &[(row-1, col), (row, col),(row-1, col-1),(row, col-1)]));
            }
        }
    }
}

fn ai_taglialegna(robot: &mut Robottino, world: &mut robotics_lib::world::World){

}
fn ai_asfaltatore(robot: &mut Robottino, world: &mut robotics_lib::world::World){}
fn ai_completo_con_tool(robot: &mut Robottino, world: &mut robotics_lib::world::World){
    //durata sleep in millisecondi per velocità robot
    let sleep_time_milly: u64 = 30;
        
    sleep(std::time::Duration::from_millis(sleep_time_milly));
    //se l'energia e' sotto il 300, la ricarico
    
    // weather_check(self);
    
    // sleep(std::time::Duration::from_millis(300));
    // bessie::bessie::road_paving_machine(self, world, Direction::Up, State::MakeRoad);
    DestroyZone.execute(world, robot, Content::Tree(0));
    let a = robot.get_backpack();
    print!("{:?}", a);
    
    //print coordinate
    let coordinates: &Coordinate = robot.get_coordinate();
    println!("{:?}", coordinates);
    robot_view(robot, world);
    let tiles_option = cheapest_border(world, robot);
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
        let result = move_to_cheapest_border(world, robot, tiles);
        if let Err((_tiles, error)) = result {
            println!("The robot cannot move due to a {:?}", error);
        }
    }
    //print coordinate

    let actual_energy = robot.get_energy().get_energy_level();
    println!("{:?}", actual_energy);
    let coordinates = robot.get_coordinate();
    println!("{:?}", coordinates);
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

pub fn moviment(robot_data: Arc<Mutex<RobotInfo>>, map: Arc<Mutex<Vec<Vec<Option<Tile>>>>>){
    println!("Hello, world!");
    let audio = get_audio_manager();
    //let background_music = OxAgSoundConfig::new_looped_with_volume("assets/audio/background.ogg", 2.0);

    let mut robot = Robottino {
        shared_map: map,
        shared_robot: robot_data,
        robot: Robot::new(),
        audio: audio,
        weather_tool: WeatherPredictionTool::new(),
        ai_logic: AiLogic::Completo
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

//enum per ai_logic (4 stringhe)
enum AiLogic {
    Falegname,
    Asfaltatore,
    Ricercatore,
    Completo,
}

