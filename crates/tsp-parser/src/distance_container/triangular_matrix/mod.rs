use explicit::parse_dists_from_lower_row;
use node_coord::compute_dists_from_node_coords;
use tsp_core::instance::{InstanceMetadata, distance::Distance, matrix::TriangularMatrix};

use super::ParseFromTSPLib;
use crate::{
    FileContent,
    distance_container::triangular_matrix::explicit::{
        parse_dists_from_full_matrix, parse_dists_from_upper_row,
    },
};
mod explicit;
mod node_coord;

impl ParseFromTSPLib for TriangularMatrix<Distance> {
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
        parse_dists_from_full_matrix(data, index_in_map, metadata.dimension)
    }

    fn from_explicit_upper_row_section(
        data: &FileContent,
        index_in_map: &mut usize,
        metadata: &InstanceMetadata,
    ) -> Self {
        parse_dists_from_upper_row(data, index_in_map, metadata.dimension, false)
    }

    fn from_explicit_lower_row_section(
        data: &FileContent,
        index_in_map: &mut usize,
        metadata: &InstanceMetadata,
    ) -> Self {
        parse_dists_from_lower_row(data, index_in_map, metadata.dimension, false)
    }

    fn from_explicit_upper_diag_row_section(
        data: &FileContent,
        index_in_map: &mut usize,
        metadata: &InstanceMetadata,
    ) -> Self {
        parse_dists_from_upper_row(data, index_in_map, metadata.dimension, true)
    }

    fn from_explicit_lower_diag_row_section(
        data: &FileContent,
        index_in_map: &mut usize,
        metadata: &InstanceMetadata,
    ) -> Self {
        parse_dists_from_lower_row(data, index_in_map, metadata.dimension, true)
    }
}
