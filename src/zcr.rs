use core::f32;
use crate::clustering::KMeans;
use crate::config::{
    INITIAL_NODE_ENERGY_J,
    DATA_PACKET_SIZE_BITS,
    FS_MULTIPATH_THRESHOLD_DISTANCE_M,
};
use crate::node::Node;
use crate::simulator::{Protocol, Simulator};
use crate::utils::{
    calculate_aggregation_energy,
    calculate_receive_energy,
    reset_node_for_new_round,
    calculate_transmit_energy,
};

/// ZCR: Zone-based Cluster Routing (proposed variant)
/// - Uses K-Means to partition nodes into spatial clusters
/// - Selects one "best" cluster head per cluster using energy + distance-to-centroid score
/// - Divides cluster heads into two zones: near (< threshold) and far
/// - Far-zone CHs may relay through the nearest near-zone CH if closer than direct to sink
pub struct Zcr {
    /// Number of cluster heads selected for the current round
    num_cluster_heads: usize,

    /// Desired probability used to compute expected number of CHs
    cluster_head_probability: f32,

    /// Zone separation of cluster heads:
    /// [0] = far-zone CHs (distance to sink > threshold)
    /// [1] = near-zone CHs  (distance to sink ≤ threshold)
    zone_cluster_heads: Vec<Vec<usize>>,
}

impl Zcr {
    pub fn new(cluster_head_probability: f32) -> Self {
        Self {
            num_cluster_heads: 0,
            cluster_head_probability,
            zone_cluster_heads: vec![Vec::new(), Vec::new()],
        }
    }

    /// Assigns selected cluster heads to near or far zone based on distance to base station.
    /// Also marks them as cluster heads.
    fn assign_zones(
        &mut self,
        selected_cluster_head_ids: &[Option<usize>],
        nodes: &mut Vec<Node>,
    ) {
        // Clear previous assignments
        self.zone_cluster_heads[0].clear();
        self.zone_cluster_heads[1].clear();

        for opt_ch_id in selected_cluster_head_ids.iter().flatten() {
            let ch_id = *opt_ch_id;
            let distance_to_bs = nodes[ch_id].distance_to_base_station_m;

            if distance_to_bs <= FS_MULTIPATH_THRESHOLD_DISTANCE_M {
                self.zone_cluster_heads[1].push(ch_id); // near zone
            } else {
                self.zone_cluster_heads[0].push(ch_id); // far zone
            }

            nodes[ch_id].is_cluster_head = true;
        }
    }

    /// Performs cluster formation:
    /// - Assigns alive non-CH nodes to their cluster's selected CH
    /// - Deducts transmission energy from member nodes to their CH
    fn form_clusters(
        &mut self,
        selected_cluster_head_ids: &[Option<usize>],
        nodes: &mut Vec<Node>,
        cluster_assignments: &[usize],
    ) {
        for (node_id, &cluster_idx) in cluster_assignments.iter().enumerate() {

            if !nodes[node_id].is_alive || nodes[node_id].is_cluster_head {
                continue;
            }

            if let Some(ch_id) = selected_cluster_head_ids[cluster_idx] {
                nodes[node_id].cluster_head_id = Some(ch_id);
                nodes[ch_id].cluster_member_ids.push(node_id);

                let distance_to_ch = (nodes[node_id].position - nodes[ch_id].position).length();
                nodes[node_id].remaining_energy_j -=
                    calculate_transmit_energy(DATA_PACKET_SIZE_BITS, distance_to_ch);
            }
        }
    }

    /// Applies energy dissipation for all cluster heads:
    /// - Far-zone CHs: either direct to BS or relay via nearest near-zone CH
    /// - Near-zone CHs: always direct to BS
    /// - All CHs deduct RX + aggregation for their members
    fn dissipate_cluster_head_energy(&self, nodes: &mut Vec<Node>) {
        // Far-zone CHs (may relay)
        for &far_ch_id in &self.zone_cluster_heads[0] {

            let mut min_relay_distance = f32::INFINITY;
            let mut best_near_ch_id: Option<usize> = None;

            // Find nearest near-zone CH
            for &near_ch_id in &self.zone_cluster_heads[1] {
                let distance = (nodes[far_ch_id].position - nodes[near_ch_id].position).length();
                if distance < min_relay_distance {
                    min_relay_distance = distance;
                    best_near_ch_id = Some(near_ch_id);
                }
            }

            let member_count = nodes[far_ch_id].cluster_member_ids.len() as f32;

            // RX + aggregation from members (always)
            nodes[far_ch_id].remaining_energy_j -=
                (calculate_receive_energy(DATA_PACKET_SIZE_BITS)
                    + calculate_aggregation_energy(DATA_PACKET_SIZE_BITS))
                    * member_count;

            // Transmission to BS or relay
            if let Some(near_ch_id) = best_near_ch_id {
                if min_relay_distance < nodes[far_ch_id].distance_to_base_station_m {
                    // Relay via near CH
                    nodes[far_ch_id].remaining_energy_j -=
                        calculate_transmit_energy(DATA_PACKET_SIZE_BITS, min_relay_distance);

                    // Near CH receives the relayed packet
                    nodes[near_ch_id].remaining_energy_j -=
                        calculate_receive_energy(DATA_PACKET_SIZE_BITS)
                            + calculate_aggregation_energy(DATA_PACKET_SIZE_BITS);
                } else {
                    // Direct to BS is cheaper
                    nodes[far_ch_id].remaining_energy_j -= calculate_transmit_energy(
                        DATA_PACKET_SIZE_BITS,
                        nodes[far_ch_id].distance_to_base_station_m,
                    );
                }
            } else {
                // No near CH available → direct
                nodes[far_ch_id].remaining_energy_j -= calculate_transmit_energy(
                    DATA_PACKET_SIZE_BITS,
                    nodes[far_ch_id].distance_to_base_station_m,
                );
            }
        }

        // Near-zone CHs: always direct to BS + RX/agg from members
        for &near_ch_id in &self.zone_cluster_heads[1] {
            let member_count = nodes[near_ch_id].cluster_member_ids.len() as f32;

            nodes[near_ch_id].remaining_energy_j -=
                member_count
                    * (calculate_receive_energy(DATA_PACKET_SIZE_BITS)
                        + calculate_aggregation_energy(DATA_PACKET_SIZE_BITS));

            nodes[near_ch_id].remaining_energy_j -= calculate_transmit_energy(
                DATA_PACKET_SIZE_BITS,
                nodes[near_ch_id].distance_to_base_station_m,
            );
        }
    }
}

impl Protocol for Zcr {
    fn name(&self) -> &'static str {
        "ZCR"
    }

    fn run_round(&mut self, simulator: &mut Simulator) {
        if simulator.alive_node_count == 0 {
            return;
        }

        // Determine number of desired cluster heads this round
        self.num_cluster_heads =
            (self.cluster_head_probability * simulator.alive_node_count as f32).ceil() as usize;

        if self.num_cluster_heads == 0 {
            return;
        }

        // Spatial clustering with K-Means
        let mut kmeans = KMeans::new(self.num_cluster_heads);
        kmeans.fit(&simulator.nodes);

        // Select best CH candidate per cluster using energy + centrality score
        let mut selected_cluster_head_ids: Vec<Option<usize>> =
            vec![None; self.num_cluster_heads];
        let mut best_scores: Vec<f32> = vec![f32::NEG_INFINITY; self.num_cluster_heads];

        for (node_id, &cluster_idx) in kmeans.clusters().iter().enumerate() {
            let node = &mut simulator.nodes[node_id];

            reset_node_for_new_round(node);

            // Check for energy depletion
            if node.is_alive && node.remaining_energy_j <= 0.0 {
                node.is_alive = false;
                simulator.alive_node_count -= 1;
                continue;
            }

            if !node.is_alive {
                continue;
            }

            // Score = normalized remaining energy - normalized distance to centroid
            let energy_score = node.remaining_energy_j / INITIAL_NODE_ENERGY_J;
            let distance_to_centroid = (node.position - kmeans.centroids()[cluster_idx]).length();
            let distance_penalty = distance_to_centroid / 707.0; // ≈ sqrt(500² + 500²)

            let score = energy_score - distance_penalty;

            if score > best_scores[cluster_idx] {
                best_scores[cluster_idx] = score;
                selected_cluster_head_ids[cluster_idx] = Some(node_id);
            }
        }

        // Zone assignment + mark CHs
        self.assign_zones(&selected_cluster_head_ids, &mut simulator.nodes);

        // Member assignment + member → CH energy cost
        self.form_clusters(
            &selected_cluster_head_ids,
            &mut simulator.nodes,
            kmeans.clusters(),
        );

        // All CH energy costs (RX/agg + TX direct or relayed)
        self.dissipate_cluster_head_energy(&mut simulator.nodes);
    }
}
