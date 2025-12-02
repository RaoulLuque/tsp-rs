use typed_builder::TypedBuilder;

use crate::tsp_lib_spec::{
    DisplayDataType, EdgeDataFormat, EdgeWeightFormat, EdgeWeightType, NodeCoordType, ProblemType,
};

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

#[derive(TypedBuilder)]
pub struct InstanceMetadata {
    pub name: String,
    pub problem_type: ProblemType,
    #[builder(default)]
    pub comment: Option<String>,
    pub dimension: u32,
    #[builder(default)]
    pub capacity: Option<u32>,
    pub edge_weight_type: EdgeWeightType,
    #[builder(default)]
    pub edge_weight_format: Option<EdgeWeightFormat>,
    #[builder(default)]
    pub edge_data_format: Option<EdgeDataFormat>,
    #[builder(default=NodeCoordType::NO_COORDS)]
    pub node_coord_type: NodeCoordType,
    #[builder(default)]
    pub display_data_type: Option<DisplayDataType>,
}
