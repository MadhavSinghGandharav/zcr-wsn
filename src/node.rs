use rand::Rng;
use glam::Vec2;
use crate::config::{INITIAL_NODE_ENERGY_J, BASE_STATION_POSITION};

/// Represents a single sensor node in the Wireless Sensor Network (WSN) simulation.
///
/// This struct holds only the **state** of the node.
/// All protocol-specific logic (LEACH phases, cluster formation, data transmission, etc.)
/// is handled externally.
#[derive(Debug, Clone)]
pub struct Node {
    /// Unique identifier of the node (usually 0..TOTAL_SENSOR_NODES-1)
    pub id: usize,

    /// Physical location of the node in the deployment area (meters)
    pub position: Vec2,

    /// Current remaining energy of the node (Joules)
    pub remaining_energy_j: f32,

    /// Whether the node still has usable energy (> 0)
    pub is_alive: bool,

    /// Whether this node is currently acting as a Cluster Head in the current round
    pub is_cluster_head: bool,

    /// Whether this node is allowed/eligible to become a Cluster Head
    /// in the current round (based on LEACH probability and rotation rules)
    pub is_eligible_for_ch: bool,

    /// Precomputed Euclidean distance from this node to the base station (meters)
    pub distance_to_base_station_m: f32,

    /// ID of the Cluster Head this node is assigned to
    /// - `None`          → this node is a Cluster Head itself
    /// - `Some(ch_id)`   → this node is a normal member of cluster with head `ch_id`
    pub cluster_head_id: Option<usize>,

    /// List of member node IDs (only meaningful/used when this node is a Cluster Head)
    pub cluster_member_ids: Vec<usize>,
}

impl Node {
    /// Creates a new sensor node with randomized position and precomputed distance to sink.
    ///
    /// # Arguments
    /// * `id`       - Unique identifier for the node
    /// * `position` - (x, y) coordinates in the deployment area (meters)
    ///
    /// # Behavior
    /// - Sets energy to `INITIAL_NODE_ENERGY_J`
    /// - Node starts alive
    /// - Starts as non-Cluster Head
    /// - Eligible to become CH in round 1
    /// - Distance to base station is precomputed once
    pub fn new(id: usize, position: Vec2) -> Self {
        // Precompute distance (length is sqrt(x² + y²))
        let diff = BASE_STATION_POSITION - position;
        let distance_to_base_station_m = diff.length();

        Self {
            id,
            position,
            remaining_energy_j: INITIAL_NODE_ENERGY_J,
            is_alive: true,
            is_cluster_head: false,
            is_eligible_for_ch: true,
            distance_to_base_station_m,
            cluster_head_id: None,
            cluster_member_ids: Vec::new(),
        }
    }

    /// Creates a complete Wireless Sensor Network with `n_nodes` randomly placed nodes.
    ///
    /// # Arguments
    /// * `width`    - Width of the deployment area (meters)
    /// * `height`   - Height of the deployment area (meters)
    /// * `n_nodes`  - Number of sensor nodes to create
    ///
    /// # Returns
    /// Vector of `Node`s with random positions inside (1..width, 1..height)
    pub fn create_wsn(width: f32, height: f32, n_nodes: usize) -> Vec<Node> {
        let mut rng = rand::rng();

        (0..n_nodes)
            .map(|id| {
                let x = rng.random_range(1.0..width);
                let y = rng.random_range(1.0..height);
                let position = Vec2::new(x, y);
                Node::new(id, position)
            })
            .collect()
    }
}
