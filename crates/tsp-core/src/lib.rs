struct TSPInstance {
    metadata: InstanceMetadata,
    /// Flattened distance matrix
    ///
    /// Row major order, i.e. distance from node i to node j is at index (i * num_nodes + j).
    /// Node indexing starts at 0.
    distances: Vec<f32>,
}

struct InstanceMetadata {
    name: String,
    num_nodes: usize,
}
