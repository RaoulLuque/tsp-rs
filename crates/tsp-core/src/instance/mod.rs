use crate::{
    instance::distances::{Asymmetric, Distances, Symmetric},
    tsp_lib_spec::{
        DisplayDataType, EdgeDataFormat, EdgeWeightFormat, EdgeWeightType, NodeCoordType,
        ProblemType,
    },
};

mod distances;

pub struct TSPInstance<T> {
    metadata: InstanceMetadata,
    /// Flattened distance matrix
    ///
    /// Row major order, i.e. distance from node i to node j is at index (i * num_nodes + j).
    /// Node indexing starts at 0.
    distances: Distances<T>,
}

impl TSPInstance<Symmetric> {
    pub fn new_symmetric_from_distances(metadata: InstanceMetadata, distances: Vec<u32>) -> Self {
        let dimension = metadata.dimension;
        Self {
            metadata,
            distances: Distances::new_symmetric_from_data(distances, dimension),
        }
    }
}

impl TSPInstance<Asymmetric> {
    pub fn new_asymmetric_from_distances(metadata: InstanceMetadata, distances: Vec<u32>) -> Self {
        let dimension = metadata.dimension;
        Self {
            metadata,
            distances: Distances::new_asymmetric_from_data(distances, dimension),
        }
    }
}

impl<T> TSPInstance<T> {
    pub fn metadata(&self) -> &InstanceMetadata {
        &self.metadata
    }

    pub fn raw_distances(&self) -> &[u32] {
        &self.distances.data
    }
}

pub struct InstanceMetadata {
    pub name: String,
    pub problem_type: ProblemType,
    pub comment: Option<String>,
    pub dimension: usize,
    pub capacity: Option<usize>,
    pub edge_weight_type: EdgeWeightType,
    pub edge_weight_format: Option<EdgeWeightFormat>,
    pub edge_data_format: Option<EdgeDataFormat>,
    /// Defaults to NO_COORDS
    pub node_coord_type: NodeCoordType,
    pub display_data_type: Option<DisplayDataType>,
}
