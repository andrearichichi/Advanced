use robotics_lib::{energy::Energy, runner::{backpack::BackPack, Robot, Runnable, Runner}, world::{coordinates::Coordinate, world_generator::Generator}};

fn main() {
    let robot: Box<dyn Runnable> = Box::new(Robottino::new());

    // world generator initialization
    let mut world_gen = ghost_amazeing_island::world_generator::WorldGenerator::new(10, false, 0, 0.0);
    let mut interface = world_gen.gen();
    println!("{:?}", interface);
    // Runnable creation and start
    println!("Generating runnable (world + robot)...");
    let mut run = Runner::new(robot, &mut world_gen);
    println!("Runnable succesfully generated");

    // for _i in 0..n_iterations {
    //     let rtn = run.as_mut().unwrap().game_tick();

}


struct Robottino {
    robot: Robot,
}

impl Robottino{
    fn new()-> Self{
        Robottino { robot: Robot::new() }
    }
}

impl Runnable for Robottino{
    fn process_tick(&mut self, world: &mut robotics_lib::world::World) {
        todo!()
    }

    fn handle_event(&mut self, event: robotics_lib::event::events::Event) {
        
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
