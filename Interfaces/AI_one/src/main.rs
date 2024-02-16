
use std::collections::HashMap;
use std::thread::sleep;

use bessie::bessie::State;
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

    let audio = get_audio_manager();

    let mut robot = Robottino {
        robot: Robot::new(),
        audio: audio,
        weather_tool: WeatherPredictionTool::new()
    };

    // world generator initialization
    let mut world_gen =
        ghost_amazeing_island::world_generator::WorldGenerator::new(505, false, 1, 1.1);
    // Runnable creation and start
    println!("Generating runnable (world + robot)...");
    // match robot.audio.play_audio(&background_music) {
    //     Ok(_) => {},
    //     Err(e) => {
    //         eprintln!("Failed to play audio: {}", e);
    //         std::process::exit(1);
    //     }
    // }
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
        weather_check(self);
        
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


fn weather_check(robot: &Robottino) {
    let ticks_until_weather = match robot.weather_tool.ticks_until_weather_change(100000000000) {
        Ok(ticks) => ticks,
        Err(e) => {
            eprintln!("Failed to get ticks until weather change: {:?}", e);
            return;
        }
    };
    let predict = match robot.weather_tool.predict(ticks_until_weather) {
        Ok(prediction) => prediction,
        Err(e) => {
            eprintln!("Failed to predict weather: {:?}", e);
            return;
        }
    };
    println!("Ticks until change: {:?}", ticks_until_weather);
    println!("into: {:?}", predict);
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