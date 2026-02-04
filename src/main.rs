use std::io::{BufWriter,Write};
use std::fs::File;

use zcr_wsn::config::{
    DEPLOYMENT_AREA_WIDTH_M,
    DEPLOYMENT_AREA_HEIGHT_M,
    CLUSTER_HEAD_PROBABILITY,
    TOTAL_SENSOR_NODES,
};
use zcr_wsn::leach::Leach;
use zcr_wsn::simulator::Simulator;
use macroquad::prelude::*;

/// Target simulation speed: how many simulation rounds per real second
const TARGET_SIMULATION_STEPS_PER_SECOND: f32 = 10.0;

fn window_conf() -> Conf {
    Conf {
        window_title: "Wireless Sensor Network Simulator".to_owned(),
        window_width: 1200,
        window_height: 720,
        fullscreen: false,
        ..Default::default()
    }
}

#[macroquad::main(window_conf)]
async fn main() {
    // Initialize simulation with configured area size and node count
    let mut simulator = Simulator::new(
        DEPLOYMENT_AREA_WIDTH_M,
        DEPLOYMENT_AREA_HEIGHT_M,
        TOTAL_SENSOR_NODES,
    );

    // Create LEACH protocol instance with desired CH probability
    let mut protocol = Leach::new(CLUSTER_HEAD_PROBABILITY);

    // File creation 
    let file: File = std::fs::File::create("../leach.csv").unwrap();
    let mut writer: BufWriter<File> = BufWriter::new(file);
    
    // Accumulator for fixed-time-step simulation loop
    let mut time_accumulator = 0.0;

    // Fixed simulation timestep (seconds per round)
    let fixed_timestep = 1.0 / TARGET_SIMULATION_STEPS_PER_SECOND;

    loop {
        let delta_time = get_frame_time();
        time_accumulator += delta_time;

        // Catch up simulation with fixed timestep (multiple updates possible per frame)
        while time_accumulator >= fixed_timestep {
            simulator.update(&mut protocol);
            for node in simulator.nodes.iter(){
                writeln!(writer,"{},{},{}",simulator.current_round,simulator.alive_node_count,node.remaining_energy_j).unwrap();
            }

            time_accumulator -= fixed_timestep;
        }

        // Rendering
        clear_background(Color::from_rgba(29, 32, 33, 255));
        simulator.render();

        next_frame().await;
    }
}
