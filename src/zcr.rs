
use crate::config::INITIAL_ENERGY;
use crate::simulator::{Protocol, SIMULATOR};
use crate::node::Node;
use crate::clustering::KMeans;

pub struct ZCR {
    n_ch: usize,
    ch_probability: f32,
    cluster_heads: Vec<Vec<usize>>,
}

impl ZCR {
    pub fn new(probability: f32) -> Self {
        Self {
            n_ch: 0, // computed every round
            ch_probability: probability,
            cluster_heads: Vec::new()
        }
    }

}

impl Protocol for ZCR {
    fn name(&self) -> &'static str {
        "ZCR"
    }

    fn run_round(&mut self, sim: &mut SIMULATOR) {
        // Simulator-level exit assumed, but protocol stays safe
        if sim.alive_count == 0 {
            return;
        }

        // Number of clusters / CHs
        self.n_ch = (self.ch_probability * sim.alive_count as f32).ceil() as usize;

        if self.n_ch == 0 {
            return;
        }

        // --- KMeans clustering ---
        let mut kmeans = KMeans::new(self.n_ch);
        kmeans.fit(&sim.wsn);

        // One CH per cluster (Option handles empty clusters)
        let mut cluster_heads: Vec<Option<usize>> = vec![None; self.n_ch];
        let mut scores: Vec<f32> = vec![f32::NEG_INFINITY; self.n_ch];

        // --- Select CH per cluster ---
        for (node_id, &cluster_idx) in kmeans.clusters().iter().enumerate() {
            let node = &sim.wsn[node_id];

            // Skip dead nodes (extra safety)
            if !node.is_alive {
                continue;
            }

            let energy_score = node.res_energy / INITIAL_ENERGY;
            let distance_score =
                (node.position - kmeans.centroids()[cluster_idx]).length() / 707.0;

            // Higher energy good, larger distance bad
            let score = energy_score - distance_score;

            if score > scores[cluster_idx] {
                scores[cluster_idx] = score;
                cluster_heads[cluster_idx] = Some(node_id);
            }
        }

        
    }
}

