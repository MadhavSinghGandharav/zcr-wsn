use crate::node::Node;
use crate::config::{
    ENERGY_AGGREGATION_J,
    ENERGY_PER_BIT_ELECTRONICS_J,
    ENERGY_FREE_SPACE_AMP_J,
    ENERGY_MULTIPATH_AMP_J,
    FS_MULTIPATH_THRESHOLD_DISTANCE_M,
};

/// Calculates the energy consumed when transmitting data over a given distance
/// using the first-order radio model.
///
/// Includes:
/// - Electronics energy for TX
/// - Amplifier energy (free-space if distance ≤ threshold, multipath otherwise)
///
/// Returns energy in Joules.
pub(crate) fn calculate_transmit_energy(data_size_bits: f32, distance_m: f32) -> f32 {
    let mut transmit_energy_j = data_size_bits * ENERGY_PER_BIT_ELECTRONICS_J;

    if distance_m <= FS_MULTIPATH_THRESHOLD_DISTANCE_M {
        transmit_energy_j += data_size_bits * ENERGY_FREE_SPACE_AMP_J * distance_m.powi(2);
    } else {
        transmit_energy_j += data_size_bits * ENERGY_MULTIPATH_AMP_J * distance_m.powi(4);
    }

    transmit_energy_j
}

/// Resets a node's protocol-specific state at the start of a new round.
///
/// Clears cluster head status, assigned cluster head, and member list.
pub(crate) fn reset_node_for_new_round(node: &mut Node) {
    node.is_cluster_head = false;
    node.cluster_head_id = None;
    node.cluster_member_ids.clear();
}

/// Calculates the energy consumed when receiving data (electronics only).
///
/// Returns energy in Joules.
pub(crate) fn calculate_receive_energy(data_size_bits: f32) -> f32 {
    data_size_bits * ENERGY_PER_BIT_ELECTRONICS_J
}

/// Calculates the energy consumed by a cluster head when aggregating data
/// from its members (per bit per signal).
///
/// Returns energy in Joules.
pub(crate) fn calculate_aggregation_energy(data_size_bits: f32) -> f32 {
    data_size_bits * ENERGY_AGGREGATION_J
}
