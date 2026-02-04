use crate::{
    config::{SENSOR_VISUAL_RADIUS_PX, METERS_TO_PIXELS},
    node::Node,
};
use macroquad::prelude::*;

/// Common trait for different WSN protocols (currently mainly LEACH).
pub trait Protocol {
    /// Execute one full round of the protocol on the given simulator state.
    fn run_round(&mut self, simulator: &mut Simulator);

    /// Human-readable name of the protocol (used in logs/UI/etc.).
    fn name(&self) -> &'static str;

}

/// Central simulation state — holds the network and current round information.
pub struct Simulator {
    /// All sensor nodes in the network
    pub nodes: Vec<Node>,

    /// Current simulation round number (starts at 0)
    pub current_round: usize,

    /// How many nodes still have energy > 0
    pub alive_node_count: usize,
}

impl Simulator {
    /// Creates a new simulator with randomly placed nodes.
    pub fn new(width: f32, height: f32, node_count: usize) -> Self {
        let nodes = Node::create_wsn(width, height, node_count);

        Self {
            nodes,
            current_round: 0,
            alive_node_count: node_count,
        }
    }

    /// Draws all nodes on screen using Macroquad.
    /// Colors indicate status: dead (dark red), cluster head (green), normal (light yellow).
    pub fn render(&self) {
        for node in self.nodes.iter() {
            let color = if !node.is_alive {
                Color::from_rgba(180, 60, 60, 255)   // dead - dark red
            } else if node.is_cluster_head {
                Color::from_rgba(89, 172, 119, 255)  // cluster head - green
            } else {
                Color::from_rgba(245, 235, 200, 255) // normal alive node - light yellow
            };

            let screen_position = node.position * METERS_TO_PIXELS;

            draw_circle(
                screen_position.x,
                screen_position.y,
                SENSOR_VISUAL_RADIUS_PX,
                color,
            );
        }
    }

    /// Advances simulation by one round and lets the protocol do its work.
    pub fn update<P: Protocol>(&mut self, protocol: &mut P) {
        self.current_round += 1;
        protocol.run_round(self);
    }
}
