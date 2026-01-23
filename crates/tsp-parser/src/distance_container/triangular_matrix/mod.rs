use node_coord::compute_dists_from_node_coords;
use tsp_core::instance::{InstanceMetadata, distance::Distance, matrix::TriangularMatrix};

use super::ParseFromTSPLib;
mod node_coord;

impl ParseFromTSPLib for TriangularMatrix<Distance> {
    fn from_node_coord_section<PointType: Sync + Send>(
        node_data: &Vec<PointType>,
        metadata: &InstanceMetadata,
        distance_function: impl Fn(&PointType, &PointType) -> Distance + Sync + Send + Copy,
    ) -> Self {
        compute_dists_from_node_coords(&node_data, metadata.dimension, distance_function)
    }
}
