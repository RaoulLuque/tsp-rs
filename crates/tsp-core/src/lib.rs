pub struct TSPInstance {
    metadata: InstanceMetadata,
    /// Flattened distance matrix
    ///
    /// Row major order, i.e. distance from node i to node j is at index (i * num_nodes + j).
    /// Node indexing starts at 0.
    distances: Vec<f32>,
}

impl TSPInstance {
    pub fn new_from_distances(metadata: InstanceMetadata, distances: Vec<f32>) -> Self {
        Self {
            metadata,
            distances,
        }
    }
}

pub struct InstanceMetadata {
    pub name: String,
    pub num_nodes: usize,
}
