
use std::thread::sleep;

use crab_rave_explorer::algorithm::{cheapest_border, move_to_cheapest_border};
use robotics_lib::interface::Direction::Up;
use robotics_lib::interface::{ go, one_direction_view, robot_map, robot_view, Direction};
use robotics_lib::{
    energy::Energy,
    runner::{backpack::BackPack, Robot, Runnable, Runner},
    world::coordinates::Coordinate
};
fn main() {
    let robot = Robottino::new();

    // world generator initialization
    let mut world_gen =
        ghost_amazeing_island::world_generator::WorldGenerator::new(22, false, 1, 1.1);
    // Runnable creation and start
    println!("Generating runnable (world + robot)...");
    let mut runner = Runner::new(Box::new(robot), &mut world_gen);
    println!("Runnable succesfully generated");

    for _i in 0..10000 {
        let rtn = runner.as_mut().unwrap().game_tick();
    }
}

struct Robottino {
    robot: Robot,
}

impl Robottino {
    fn new() -> Self {
        Robottino {
            robot: Robot::new(),
        }
    }
}

impl Runnable for Robottino {
    fn process_tick(&mut self, world: &mut robotics_lib::world::World) {
        //se l'energia e' sotto il 300, la ricarico
        if self.robot.energy.get_energy_level() < 300 {
            self.robot.energy = rust_and_furious_dynamo::dynamo::Dynamo::update_energy();
        }
        // ohcrab_weather::weather_tool::WeatherPredictionTool da testare il predict
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

    fn handle_event(&mut self, event: robotics_lib::event::events::Event) {}

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
