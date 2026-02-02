use glam::Vec2;

// =============================================================================
// Simulation Area & Visualization
// =============================================================================
/// Width of the deployment area (in meters).
pub const DEPLOYMENT_AREA_WIDTH_M: f32 = 500.0;

/// Height of the deployment area (in meters).
pub const DEPLOYMENT_AREA_HEIGHT_M: f32 = 500.0;

/// Visualization window width in pixels.
pub const VISUALIZATION_WIDTH_PX: f32 = 1200.0;

/// Visualization window height in pixels.
pub const VISUALIZATION_HEIGHT_PX: f32 = 720.0;

/// Scaling factor to convert simulation meters → screen pixels (x and y components).
pub const METERS_TO_PIXELS: Vec2 = Vec2::new(
    VISUALIZATION_WIDTH_PX / DEPLOYMENT_AREA_WIDTH_M,
    VISUALIZATION_HEIGHT_PX / DEPLOYMENT_AREA_HEIGHT_M,
);

/// Target frames per second for visualization.
pub const TARGET_FPS: u32 = 10;

/// Visualization radius of a sensor node (pixels).
pub const SENSOR_VISUAL_RADIUS_PX: f32 = 10.0;

// =============================================================================
// LEACH Protocol Parameters
// =============================================================================
/// Total number of sensor nodes in the network.
pub const TOTAL_SENSOR_NODES: usize = 100;

/// Desired probability that a node becomes a cluster head in any given round.
pub const CLUSTER_HEAD_PROBABILITY: f32 = 0.1;

/// Expected (rounded up) number of cluster heads per round.
pub const EXPECTED_NUM_CLUSTER_HEADS: usize =
    (TOTAL_SENSOR_NODES as f32 * CLUSTER_HEAD_PROBABILITY).ceil() as usize;

/// Number of rounds after which — on average — every node should have been
/// a cluster head once.
pub const CLUSTER_HEAD_CYCLE_LENGTH_ROUNDS: usize =
    (1.0 / CLUSTER_HEAD_PROBABILITY) as usize;

// =============================================================================
// First-Order Radio Energy Model Parameters
// =============================================================================
/// Initial energy available to each sensor node (Joules).
pub const INITIAL_NODE_ENERGY_J: f32 = 2.0;

/// Energy consumed by radio electronics — both transmit and receive (J/bit).
pub const ENERGY_PER_BIT_ELECTRONICS_J: f32 = 5e-8;

/// Free-space amplifier energy per bit per square meter (J/bit/m²).
pub const ENERGY_FREE_SPACE_AMP_J: f32 = 1e-11;

/// Multipath amplifier energy per bit per fourth power of distance (J/bit/m⁴).
pub const ENERGY_MULTIPATH_AMP_J: f32 = 1.3e-15;

/// Energy cost of data aggregation per bit per signal (J/bit/signal).
pub const ENERGY_AGGREGATION_J: f32 = 5e-9;

/// Size of a data packet (bits).
pub const DATA_PACKET_SIZE_BITS: f32 = 4000.0;

// =============================================================================
// Radio Propagation & Threshold
// =============================================================================
/// Distance threshold between free-space and multipath models (meters).
/// Computed as sqrt( E_FREE_SPACE_AMP / E_MULTIPATH_AMP ).
pub const FS_MULTIPATH_THRESHOLD_DISTANCE_M: f32 = 87.7;

// =============================================================================
// Base Station (Sink)
// =============================================================================
/// Fixed location of the base station / sink node (center of area).
pub const BASE_STATION_POSITION: Vec2 =
    Vec2::new(DEPLOYMENT_AREA_WIDTH_M / 2.0, DEPLOYMENT_AREA_HEIGHT_M / 2.0);

// =============================================================================
// Simulation Control
// =============================================================================
/// Maximum number of rounds to run in the simulation.
/// May terminate earlier if all nodes deplete their energy.
pub const MAX_SIMULATION_ROUNDS: usize = 2000;
