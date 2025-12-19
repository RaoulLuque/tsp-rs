use crate::{
    instance::distances::DistanceMatrixSymmetric,
    tsp_lib_spec::{
        DisplayDataType, EdgeDataFormat, EdgeWeightFormat, EdgeWeightType, NodeCoordType,
        ProblemType,
    },
};

pub mod distances;

#[derive(Debug, Clone)]
pub struct TSPSymInstance {
    metadata: InstanceMetadata,
    /// Flattened distance matrix
    ///
    /// Row major order, i.e. distance from node i to node j is at index (i * num_nodes + j).
    /// Node indexing starts at 0.
    distances: DistanceMatrixSymmetric,
}

impl TSPSymInstance {
    pub fn new_from_raw_data(distance_data: Vec<u32>, metadata: InstanceMetadata) -> Self {
        let dimension = metadata.dimension;
        Self {
            metadata,
            distances: DistanceMatrixSymmetric::new_from_data(distance_data, dimension),
        }
    }

    pub fn new_from_distances_sym(
        distances: DistanceMatrixSymmetric,
        metadata: InstanceMetadata,
    ) -> Self {
        Self {
            metadata,
            distances,
        }
    }

    pub fn metadata(&self) -> &InstanceMetadata {
        &self.metadata
    }

    pub fn raw_distances(&self) -> &[u32] {
        &self.distances.data
    }
}

#[derive(Debug, Clone)]
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
