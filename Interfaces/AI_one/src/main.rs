
use std::collections::HashMap;
use std::thread::sleep;

use crab_rave_explorer::algorithm::{cheapest_border, move_to_cheapest_border};
use oxagaudiotool::sound_config::{self, OxAgSoundConfig};
use robotics_lib::event::events::Event;
use robotics_lib::interface::Direction::Up;
use ohcrab_weather::weather_tool::WeatherPredictionTool;
use arrusticini_destroy_zone::DestroyZone;
use oxagaudiotool::OxAgAudioTool;
use robotics_lib::interface::{ go, one_direction_view, robot_map, robot_view, Direction};
use robotics_lib::world;
use robotics_lib::world::environmental_conditions::WeatherType;
use robotics_lib::world::tile::{Content, TileType};
use robotics_lib::{
    energy::Energy,
    runner::{backpack::BackPack, Robot, Runnable, Runner},
    world::coordinates::Coordinate
};
fn main() {
    // Setup the sound configs
    // We suggest you to use small files as if you use too many big audio files the startup times may increase
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

    let mut robot = Robottino {
        robot: Robot::new(),
        audio: audio,
        weather_tool: WeatherPredictionTool::new()
    };

    // world generator initialization
    let mut world_gen =
        ghost_amazeing_island::world_generator::WorldGenerator::new(22, false, 1, 1.1);
    // Runnable creation and start
    println!("Generating runnable (world + robot)...");
    match robot.audio.play_audio(&background_music) {
        Ok(_) => {},
        Err(e) => {
            eprintln!("Failed to play audio: {}", e);
            std::process::exit(1);
        }
    }
    let mut runner = Runner::new(Box::new(robot), &mut world_gen);
    println!("Runnable succesfully generated");

    for _i in 0..10000 {
        let rtn = runner.as_mut().unwrap().game_tick();
    }
}

struct Robottino {
    robot: Robot,
    audio: OxAgAudioTool,
    weather_tool: WeatherPredictionTool
}


impl Runnable for Robottino {
    fn process_tick(&mut self, world: &mut robotics_lib::world::World) {
        //se l'energia e' sotto il 300, la ricarico
        if self.robot.energy.get_energy_level() < 300 {
            self.robot.energy = rust_and_furious_dynamo::dynamo::Dynamo::update_energy();
        }
        let ticks_until_weather = self.weather_tool.ticks_until_weather(robotics_lib::world::environmental_conditions::WeatherType::Rainy, 100000000000);
        println!("Ticks until foggy: {:?}", ticks_until_weather);
        // let audio1 = OxAgSoundConfig::new("src/audio/pluck_002.ogg");
        // let audio2 = OxAgSoundConfig::new("src/audio/pluck_002.ogg");
        // let audio3 = OxAgSoundConfig::new("src/audio/pluck_002.ogg");
        // let audio4 = OxAgSoundConfig::new("src/audio/pluck_002.ogg");

        // let mut my_map: HashMap<Event, OxAgSoundConfig> = HashMap::new();
        // my_map.insert(Event::Ready, audio1);

        // let mut my_tile_map: HashMap<TileType, OxAgSoundConfig> = HashMap::new();
        // my_tile_map.insert(TileType::Grass, audio2);

        // let mut my_weather_map: HashMap<WeatherType, OxAgSoundConfig> = HashMap::new();
        // my_weather_map.insert(WeatherType::Sunny, audio3); // Removed .clone() here
        
        // let audio_manager = OxAgAudioTool::new(my_map, my_tile_map, my_weather_map);

        // match audio_manager {
        //     Ok(mut audio_manager) => {
        //         audio_manager.play_audio(&audio4);
        //     },
        //     Err(error) => {
        //         println!("Failed to create audio manager: {}", error);
        //     },
        // }
        //sleep
        sleep(std::time::Duration::from_millis(30000));
        DestroyZone.execute(world, self, Content::Tree(1));
        WeatherPredictionTool::new();
        // WeatherPredictionTool::ticks_until_weather(&WeatherPredictionTool::new(), required_weather, 1);

        sleep(std::time::Duration::from_millis(300));
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

    }

    fn handle_event(&mut self, event: robotics_lib::event::events::Event) {
        self.weather_tool.process_event(&event);
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
