
use crate::config::{INITIAL_ENERGY, PACKET_SIZE};
use crate::simulator::{Protocol, SIMULATOR};
use crate::node::Node;
use crate::clustering::KMeans;
use crate::utils::{transmission_energy,reset_node};

pub struct ZCR {
    n_ch: usize,
    ch_probability: f32,

    // [0] = far zone CHs, [1] = near zone CHs
    zone_cluster_heads: Vec<Vec<usize>>,
}

impl ZCR {
    pub fn new(probability: f32) -> Self {
        Self {
            n_ch: 0,
            ch_probability: probability,
            zone_cluster_heads: vec![Vec::new(), Vec::new()],
        }
    }

    fn assign_zones(
        &mut self,
        selected_ch: &[Option<usize>],
        wsn: &mut Vec<Node>,
    ) {
        // clear previous round zones
        self.zone_cluster_heads[0].clear();
        self.zone_cluster_heads[1].clear();

        for ch_id in selected_ch.iter().flatten() {
            if wsn[*ch_id].distance_to_sink < 87.0 {
                self.zone_cluster_heads[1].push(*ch_id); // near
            } else {
                self.zone_cluster_heads[0].push(*ch_id); // far
            }

            wsn[*ch_id].is_cluster_head = true;
        }
    }

    fn cluster_formation(&mut self, selected_ch: Vec<Option<usize>>, wsn: &mut Vec<Node>,clusters: &[usize]){
        // ---- cluster formation + normal node TX energy ----
        for (node_id, &cluster_idx) in clusters.iter().enumerate() {

            if !wsn[node_id].is_alive || wsn[node_id].is_cluster_head {
                continue;
            }

            if let Some(ch_id) = selected_ch[cluster_idx] {
                wsn[node_id].cluster_head_id = Some(ch_id);
                wsn[ch_id].cluster_members.push(node_id);

                let d = (wsn[node_id].position - wsn[ch_id].position).length();

                wsn[node_id].res_energy -= transmission_energy(PACKET_SIZE, d);
            }
        }
    }
}

impl Protocol for ZCR {
    fn name(&self) -> &'static str {
        "ZCR"
    }

    fn run_round(&mut self, sim: &mut SIMULATOR) {
        if sim.alive_count == 0 {
            return;
        }

        // ---- compute number of CHs ----
        self.n_ch = (self.ch_probability * sim.alive_count as f32).ceil() as usize;

        if self.n_ch == 0 {
            return;
        }

        // ---- KMeans clustering ----
        let mut kmeans = KMeans::new(self.n_ch);
        kmeans.fit(&sim.wsn);

        // ---- select best CH per cluster ----
        let mut selected_ch: Vec<Option<usize>> = vec![None; self.n_ch];
        let mut scores: Vec<f32> = vec![f32::NEG_INFINITY; self.n_ch];

        for (node_id, &cluster_idx) in kmeans.clusters().iter().enumerate() {
            reset_node(&mut sim.wsn[node_id]);
            if !sim.wsn[node_id].is_alive {
                continue;
            }

            let energy_score = sim.wsn[node_id].res_energy / INITIAL_ENERGY;
            let distance_penalty =
                (sim.wsn[node_id].position - kmeans.centroids()[cluster_idx]).length() / 707.0;

            let score = energy_score - distance_penalty;

            if score > scores[cluster_idx] {
                scores[cluster_idx] = score;
                selected_ch[cluster_idx] = Some(node_id);
            }
        }

        // ---- ZCR zone assignment ----
        self.assign_zones(&selected_ch, &mut sim.wsn);
        self.cluster_formation(selected_ch, &mut sim.wsn, kmeans.clusters());

    }
}

