use node_coord::compute_dists_from_node_coords;
use tsp_core::instance::{InstanceMetadata, distance::Distance, matrix::SquareMatrix};

use super::ParseFromTSPLib;
use crate::FileContent;
mod node_coord;

impl ParseFromTSPLib for SquareMatrix<Distance> {
    fn from_node_coord_section<PointType: Sync + Send>(
        node_data: &Vec<PointType>,
        metadata: &InstanceMetadata,
        distance_function: impl Fn(&PointType, &PointType) -> Distance + Sync + Send + Copy,
    ) -> Self {
        compute_dists_from_node_coords(&node_data, metadata.dimension, distance_function)
    }

    fn from_explicit_full_matrix_section(
        data: &FileContent,
        index_in_map: &mut usize,
        metadata: &InstanceMetadata,
    ) -> Self {
        todo!("Explicit edge weight format FULL_MATRIX is not supported yet")
    }

    fn from_explicit_upper_row_section(
        data: &FileContent,
        index_in_map: &mut usize,
        metadata: &InstanceMetadata,
    ) -> Self {
        todo!("Explicit edge weight format UPPER_ROW is not supported yet")
    }

    fn from_explicit_lower_row_section(
        data: &FileContent,
        index_in_map: &mut usize,
        metadata: &InstanceMetadata,
    ) -> Self {
        todo!("Explicit edge weight format LOWER_ROW is not supported yet")
    }

    fn from_explicit_upper_diag_row_section(
        data: &FileContent,
        index_in_map: &mut usize,
        metadata: &InstanceMetadata,
    ) -> Self {
        todo!("Explicit edge weight format UPPER_DIAG_ROW is not supported yet")
    }

    fn from_explicit_lower_diag_row_section(
        data: &FileContent,
        index_in_map: &mut usize,
        metadata: &InstanceMetadata,
    ) -> Self {
        todo!("Explicit edge weight format LOWER_DIAG_ROW is not supported yet")
    }
}
