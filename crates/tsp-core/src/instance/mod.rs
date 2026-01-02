use crate::{
    instance::{
        distance::Distance,
        edge::UnEdge,
        matrix::{Matrix, MatrixSym},
    },
    tsp_lib_spec::{
        DisplayDataType, EdgeDataFormat, EdgeWeightFormat, EdgeWeightType, NodeCoordType,
        ProblemType,
    },
};

pub mod distance;
pub mod edge;
pub mod matrix;
pub mod node;

#[derive(Debug, Clone)]
pub struct TSPSymInstance<DistanceContainer> {
    metadata: InstanceMetadata,
    /// Flattened distance matrix
    ///
    /// Row major order, i.e. distance from node i to node j is at index (i * num_nodes + j).
    /// Node indexing starts at 0.
    distances: DistanceContainer,
}

impl<DistanceContainer> TSPSymInstance<DistanceContainer> {
    pub fn new(distance_container: DistanceContainer, metadata: InstanceMetadata) -> Self {
        let dimension = metadata.dimension;
        Self {
            metadata,
            distances: distance_container,
        }
    }

    pub fn metadata(&self) -> &InstanceMetadata {
        &self.metadata
    }
}

impl TSPSymInstance<MatrixSym<Distance>> {
    pub fn raw_distances(&self) -> &[Distance] {
        self.distances.data()
    }

    pub fn distance_matrix(&self) -> &MatrixSym<Distance> {
        &self.distances
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

#[derive(Debug, Clone)]
pub struct UnTour {
    pub edges: Vec<UnEdge>,
    pub cost: Distance,
}

impl PartialEq for UnTour {
    fn eq(&self, other: &Self) -> bool {
        self.cost == other.cost && self.edges.len() == other.edges.len() && {
            let mut res = true;
            for edge in &self.edges {
                if !other.edges.contains(edge) {
                    res = false;
                    break;
                }
            }
            res
        }
    }
}

impl Eq for UnTour {}
