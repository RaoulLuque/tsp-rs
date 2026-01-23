use tsp_core::instance::{InstanceMetadata, distance::Distance};

use crate::FileContent;

mod square_matrix;
mod triangular_matrix;

pub trait ParseFromTSPLib {
    fn from_node_coord_section<PointType: Sync + Send>(
        node_data: &Vec<PointType>,
        metadata: &InstanceMetadata,
        distance_function: impl Fn(&PointType, &PointType) -> Distance + Sync + Send + Copy,
    ) -> Self;

    fn from_explicit_full_matrix_section(
        data: &FileContent,
        index_in_map: &mut usize,
        metadata: &InstanceMetadata,
    ) -> Self;
}

fn find_row_column_from_lower_triangle_index(index: usize) -> (usize, usize) {
    let row = (-0.5 + ((0.25 + 2.0 * index as f64).sqrt())).floor() as usize;
    let column = index - (row * (row + 1)) / 2;
    (row, column)
}
