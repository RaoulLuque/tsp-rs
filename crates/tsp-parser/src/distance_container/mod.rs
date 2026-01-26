use tsp_core::instance::{InstanceMetadata, distance::Distance};

use crate::FileContent;

mod square_matrix;
mod triangular_matrix;

pub trait ParseFromTSPLib {
    /// Parse distance container from NODE_COORD_SECTION
    fn from_node_coord_section<PointType: Sync + Send>(
        node_data: &Vec<PointType>,
        metadata: &InstanceMetadata,
        distance_function: impl Fn(&PointType, &PointType) -> Distance + Sync + Send + Copy,
    ) -> Self;

    /// Parse distance container from EDGE_WEIGHT_TYPE::EXPLICIT, EDGE_WEIGHT_FORMAT::FULL_MATRIX
    /// section
    fn from_explicit_full_matrix_section(
        data: &FileContent,
        index_in_map: &mut usize,
        metadata: &InstanceMetadata,
    ) -> Self;

    /// Parse distance container from EDGE_WEIGHT_TYPE::EXPLICIT, EDGE_WEIGHT_FORMAT::UPPER_ROW
    /// section
    fn from_explicit_upper_row_section(
        data: &FileContent,
        index_in_map: &mut usize,
        metadata: &InstanceMetadata,
    ) -> Self;

    /// Parse distance container from EDGE_WEIGHT_TYPE::EXPLICIT, EDGE_WEIGHT_FORMAT::LOWER_ROW
    /// section
    fn from_explicit_lower_row_section(
        data: &FileContent,
        index_in_map: &mut usize,
        metadata: &InstanceMetadata,
    ) -> Self;

    /// Parse distance container from EDGE_WEIGHT_TYPE::EXPLICIT, EDGE_WEIGHT_FORMAT::UPPER_DIAG_ROW
    /// section
    fn from_explicit_upper_diag_row_section(
        data: &FileContent,
        index_in_map: &mut usize,
        metadata: &InstanceMetadata,
    ) -> Self;

    /// Parse distance container from EDGE_WEIGHT_TYPE::EXPLICIT, EDGE_WEIGHT_FORMAT::LOWER_DIAG_ROW
    /// section
    fn from_explicit_lower_diag_row_section(
        data: &FileContent,
        index_in_map: &mut usize,
        metadata: &InstanceMetadata,
    ) -> Self;

    // TODO: Add missing EDGE_WEIGHT_FORMATs ( UPPER_COL, LOWER_COL, UPPER_DIAG_COL, LOWER_DIAG_COL
    // )
}

fn find_row_column_from_lower_triangle_index(index: usize) -> (usize, usize) {
    let row = (-0.5 + ((0.25 + 2.0 * index as f64).sqrt())).floor() as usize;
    let column = index - (row * (row + 1)) / 2;
    (row, column)
}
