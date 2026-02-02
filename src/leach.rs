
use crate::config::DATA_PACKET_SIZE_BITS;
use crate::simulator::{Protocol, Simulator};
use crate::utils::*;
use rand::Rng;
use crate::node::Node;

/// Implementation of the LEACH (Low-Energy Adaptive Clustering Hierarchy) protocol.
///
/// This is a simplified version commonly used in simulations:
/// - Cluster heads are selected probabilistically with rotation.
/// - Non-CH nodes join the nearest CH and deduct transmission energy to it.
/// - CHs deduct energy for receiving from members, aggregating data,
///   and transmitting one aggregated packet to the base station.
pub struct Leach {
    /// Current election threshold T(n) — updated each round
    election_threshold: f32,

    /// Desired cluster head probability 'p'
    cluster_head_probability: f32,

    /// Length of one full rotation cycle (1/p rounds on average)
    cycle_length_rounds: usize,
}

impl Leach {
    /// Creates a new LEACH instance with the given cluster head probability.
    pub fn new(cluster_head_probability: f32) -> Self {
        Self {
            election_threshold: 0.0, // will be updated in first round
            cluster_head_probability,
            cycle_length_rounds: (1.0 / cluster_head_probability) as usize,
        }
    }

    /// Updates the cluster head election threshold T(n) for the current round
    /// according to the standard LEACH formula.
    fn update_election_threshold(&mut self, current_round: usize) {
        let r_mod = (current_round % self.cycle_length_rounds) as f32;
        let denom = 1.0 - self.cluster_head_probability * r_mod;
        // Avoid division by zero / negative (though denom should stay > 0)
        self.election_threshold = (self.cluster_head_probability / denom).min(1.0);
    }

    /// Assigns alive non-CH nodes to the nearest cluster head,
    /// deducts transmission energy from members (to their CH),
    /// and registers members on the CH nodes.
    ///
    /// This represents the phase where nodes send data to their CH
    /// (join cost is often considered negligible or merged here).
    fn form_clusters(nodes: &mut Vec<Node>, cluster_head_ids: &Vec<usize>) {
        for node_id in 0..nodes.len() {
            
            if nodes[node_id].is_alive && !nodes[node_id].is_cluster_head {
                let mut min_distance_m = f32::INFINITY;
                let mut nearest_ch_id: Option<usize> = None;

                for &ch_id in cluster_head_ids {
                    let distance = (nodes[node_id].position - nodes[ch_id].position).length();
                    if distance < min_distance_m {
                        min_distance_m = distance;
                        nearest_ch_id = Some(ch_id);
                    }
                }

                if let Some(ch_id) = nearest_ch_id {
                    nodes[node_id].cluster_head_id = Some(ch_id);
                    nodes[ch_id].cluster_member_ids.push(node_id);
                    nodes[node_id].remaining_energy_j -=
                        calculate_transmit_energy(DATA_PACKET_SIZE_BITS, min_distance_m);
                }
            }
        }
    }
}

impl Protocol for Leach {
    fn name(&self) -> &'static str {
        "LEACH"
    }

    /// Executes one full round of the LEACH protocol.
    fn run_round(&mut self, simulator: &mut Simulator) {
        self.update_election_threshold(simulator.current_round);

        let mut rng = rand::rng();
        let mut selected_cluster_head_ids: Vec<usize> = Vec::new();

        // Phase 1: Reset state, handle dead nodes, elect cluster heads
        for node_id in 0..simulator.nodes.len() {
            let node = &mut simulator.nodes[node_id];

            reset_node_for_new_round(node);

            // Reset eligibility at the start of each new cycle
            if simulator.current_round % self.cycle_length_rounds == 0 {
                node.is_eligible_for_ch = true;
            }

            // Mark node dead if energy depleted
            if node.is_alive && node.remaining_energy_j <= 0.0 {
                node.is_alive = false;
                simulator.alive_node_count -= 1;
                continue;
            }

            // Probabilistic cluster head election
            if rng.random::<f32>() < self.election_threshold
                && node.is_alive
                && node.is_eligible_for_ch
            {
                node.is_cluster_head = true;
                node.is_eligible_for_ch = false;
                selected_cluster_head_ids.push(node_id);
            }
        }

        // Phase 2: Cluster assignment + member → CH data transmission energy
        Leach::form_clusters(&mut simulator.nodes, &selected_cluster_head_ids);

        // Phase 3: Cluster head energy costs (receive + aggregate + transmit to BS)
        for &ch_id in selected_cluster_head_ids.iter() {
            let ch_node = &mut simulator.nodes[ch_id];

            if !ch_node.is_alive {
                continue;
            }

            let member_count = ch_node.cluster_member_ids.len() as f32;

            // Receive + aggregate data from all members
            ch_node.remaining_energy_j -=
                (calculate_receive_energy(DATA_PACKET_SIZE_BITS)
                    + calculate_aggregation_energy(DATA_PACKET_SIZE_BITS))
                    * member_count;

            // Transmit one aggregated packet to the base station
            ch_node.remaining_energy_j -= calculate_transmit_energy(
                DATA_PACKET_SIZE_BITS,
                ch_node.distance_to_base_station_m,
            );
        }
    }
}
