use crate::node::Node;
use glam::Vec2;
use rand::seq::index::sample;

/// Maximum number of iterations allowed for the K-Means algorithm convergence.
const MAX_ITER: usize = 100;

/// Convergence threshold: maximum allowed centroid movement (in meters) to stop early.
const EPS: f32 = 1e-4;

/// Simple K-Means clustering implementation for grouping sensor nodes by their 2D positions.
///
/// Used to create spatial clusters (e.g. for zone-based or balanced cluster-head selection).
///
/// This is a basic Lloyd's algorithm with random centroid initialization from existing points.
pub(crate) struct KMeans {
    /// Number of clusters to form (k).
    n_clusters: usize,

    /// Current positions of the k cluster centroids.
    centroids: Vec<Vec2>,

    /// Cluster assignment for each node: index i → cluster number (0..n_clusters-1).
    clusters: Vec<usize>,
}

impl KMeans {
    /// Creates a new (unfitted) KMeans instance ready to cluster `n_clusters` groups.
    pub fn new(n_clusters: usize) -> Self {
        Self {
            n_clusters,
            centroids: Vec::new(),
            clusters: Vec::new(),
        }
    }

    /// Returns a reference to the current centroid positions.
    /// (Only meaningful after calling `fit()`)
    pub fn centroids(&self) -> &Vec<Vec2> {
        &self.centroids
    }

    /// Returns a reference to the cluster assignment vector.
    /// Length = number of nodes, value = assigned cluster index (0..n_clusters-1)
    pub fn clusters(&self) -> &Vec<usize> {
        &self.clusters
    }

    /// Recomputes each centroid as the mean position of all nodes assigned to it.
    ///
    /// Returns the previous centroid positions (used to check convergence).
    fn update_centroids(&mut self, wsn: &[Node]) -> Vec<Vec2> {
        let mut accum = vec![(Vec2::new(0.0, 0.0), 0usize); self.n_clusters];
        let previous_centroids = self.centroids.clone();

        // Accumulate position sums and point counts per cluster
        for (i, &c_id) in self.clusters.iter().enumerate() {
            accum[c_id].0 += wsn[i].position;
            accum[c_id].1 += 1;
        }

        // Update centroids to mean position (skip empty clusters)
        for (i, (sum, count)) in accum.into_iter().enumerate() {
            if count > 0 {
                self.centroids[i] = sum / count as f32;
            }
        }

        previous_centroids
    }

    /// Fits the K-Means model to the given list of sensor nodes.
    ///
    /// Steps:
    /// 1. Randomly selects `n_clusters` distinct nodes as initial centroids.
    /// 2. Iteratively:
    ///    - Assigns each node to the nearest centroid
    ///    - Updates centroids to mean of assigned nodes
    /// 3. Stops after `MAX_ITER` iterations or when maximum centroid movement < `EPS`.
    pub fn fit(&mut self, wsn: &Vec<Node>) {
        let mut rng = rand::rng();

        // Initialize centroids by randomly sampling n_clusters distinct node positions
        self.centroids = sample(&mut rng, wsn.len(), self.n_clusters)
            .into_iter()
            .map(|x| wsn[x].position)
            .collect();

        // Initialize cluster assignments (will be updated in loop)
        self.clusters = vec![0; wsn.len()];

        for _ in 0..MAX_ITER {
            // Assignment step: find nearest centroid for each node
            for i in 0..wsn.len() {
                let mut min_dist = f32::INFINITY;

                for (index, &centroid) in self.centroids.iter().enumerate() {
                    let dist = (wsn[i].position - centroid).length();
                    if dist < min_dist {
                        min_dist = dist;
                        self.clusters[i] = index;
                    }
                }
            }

            // Update step: recompute centroids and get previous positions
            let prev_centroids = self.update_centroids(&wsn);

            // Convergence check: maximum distance any centroid moved
            let mut max_shift: f32 = 0.0;
            for (a, b) in prev_centroids.iter().zip(self.centroids.iter()) {
                max_shift = max_shift.max((a - b).length());
            }

            if max_shift < EPS {
                break;
            }
        }
    }
}
