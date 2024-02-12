
use robotics_lib::interface::Direction::Up;
use robotics_lib::interface::{ go, one_direction_view,  robot_view, Direction};
use robotics_lib::{
    energy::Energy,
    runner::{backpack::BackPack, Robot, Runnable, Runner},
    world::coordinates::Coordinate

};
fn main() {
    let robot = Robottino::new();

    // world generator initialization
    let mut world_gen =
        ghost_amazeing_island::world_generator::WorldGenerator::new(244, false, 1, 1.1);
    // Runnable creation and start
    println!("Generating runnable (world + robot)...");
    let mut runner = Runner::new(Box::new(robot), &mut world_gen);
    println!("Runnable succesfully generated");

    for _i in 0..15 {
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
        let view = robot_view(self, world);
        if let Some(oggetto) = &view[0][0]{

            let mut tree_direction: Option<Direction> = None;
            println!("{:?}", oggetto)
        };

        let actual_energy = self.get_energy().get_energy_level();
        println!("{:?}", actual_energy);
        let _ = go(self, world, Direction::Down);
        let perceived_world = robot_view(self, world);
        let one_view = one_direction_view(self, world, Up, 5);
        let actual_energy = self.get_energy().get_energy_level();
        println!("{:?}", actual_energy);
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
