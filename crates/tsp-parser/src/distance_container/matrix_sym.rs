use tsp_core::instance::{
    InstanceMetadata,
    distance::Distance,
    matrix::{MatrixSym, get_lower_triangle_matrix_entry_row_bigger},
};

use super::ParseFromTSPLib;
use crate::data_section::{GeoPoint, Point2D};

// TODO: Add more fine grained benchmarks to determine optimal parallelism bound
const PARALLELISM_BOUND: usize = 300_000;

impl ParseFromTSPLib for MatrixSym<Distance> {
    fn from_2d_node_coord_section(
        node_data: &Vec<Point2D>,
        metadata: &InstanceMetadata,
        distance_function: impl Fn(&Point2D, &Point2D) -> Distance + Sync + Send + Copy,
    ) -> Self {
        compute_dists_from_node_coords::<Point2D>(&node_data, metadata.dimension, distance_function)
    }

    fn from_geo_node_coord_section(
        node_data: &Vec<GeoPoint>,
        metadata: &InstanceMetadata,
        distance_function: impl Fn(&GeoPoint, &GeoPoint) -> Distance + Sync + Send + Copy,
    ) -> Self {
        compute_dists_from_node_coords::<GeoPoint>(
            &node_data,
            metadata.dimension,
            distance_function,
        )
    }
}

fn compute_dists_from_node_coords<PointType: Send + Sync + Copy>(
    point_data: &[PointType],
    dimension: usize,
    distance_function: impl Fn(&PointType, &PointType) -> Distance + Sync + Send + Copy,
) -> MatrixSym<Distance> {
    let total_size = dimension * (dimension + 1) / 2;

    let mut distance_data = vec![Distance(0); total_size];

    if total_size < PARALLELISM_BOUND {
        compute_dists_from_node_coords_chunk(&mut distance_data, point_data, 0, distance_function);
    } else {
        let nthreads = std::thread::available_parallelism().unwrap();
        let chunk_size = total_size.div_ceil(nthreads.get());

        std::thread::scope(|scope| {
            let mut current_chunk_start = 0;

            for chunk in distance_data.chunks_mut(chunk_size) {
                scope.spawn(move || {
                    compute_dists_from_node_coords_chunk(
                        chunk,
                        point_data,
                        current_chunk_start,
                        distance_function,
                    )
                });

                current_chunk_start += chunk_size;
            }
        });
    }

    MatrixSym::new(distance_data, dimension)
}

#[inline(always)]
fn compute_dists_from_node_coords_chunk<PointType: Copy>(
    chunk: &mut [Distance],
    point_data: &[PointType],
    chunk_start_index: usize,
    distance_function: impl Fn(&PointType, &PointType) -> Distance + Copy,
) {
    let (start_row, start_column) = {
        // We solve for row such that (row * (row + 1)) / 2 <= chunk_start_index is tight (i.e. row
        // + 1 would exceed)
        let row = (-0.5 + ((0.25 + 2.0 * chunk_start_index as f64).sqrt())).floor() as usize;
        let column = chunk_start_index - (row * (row + 1)) / 2;
        (row, column)
    };

    let (end_row, end_column) = {
        let chunk_end_index = chunk_start_index + chunk.len() - 1;
        // We solve for row such that (row * (row + 1)) / 2 <= chunk_end_index is tight (i.e. row
        // + 1 would exceed)
        let row = (-0.5 + ((0.25 + 2.0 * chunk_end_index as f64).sqrt())).floor() as usize;
        let column = chunk_end_index - (row * (row + 1)) / 2;
        (row, column)
    };

    let start_row_point_data = &point_data[start_row];
    // We can omit the column = start_row case, as it is always zero distance
    for (column, column_point_data) in point_data
        .iter()
        .enumerate()
        .take(start_row)
        .skip(start_column)
    {
        compute_and_set_distance(
            chunk,
            start_row,
            column,
            chunk_start_index,
            start_row_point_data,
            column_point_data,
            distance_function,
        );
    }

    for row in (start_row + 1)..end_row {
        let row_point_data = &point_data[row];
        // We can omit the column = start_row case, as it is always zero distance
        for (column, column_point_data) in point_data.iter().enumerate().take(row) {
            compute_and_set_distance(
                chunk,
                row,
                column,
                chunk_start_index,
                row_point_data,
                column_point_data,
                distance_function,
            );
        }
    }

    let end_row_point_data = &point_data[end_row];
    // We can omit the column = start_row case, as it is always zero distance
    for (column, column_point_data) in point_data.iter().enumerate().take(end_column) {
        compute_and_set_distance(
            chunk,
            end_row,
            column,
            chunk_start_index,
            end_row_point_data,
            column_point_data,
            distance_function,
        );
    }
}

#[inline(always)]
fn compute_and_set_distance<PointType: Copy>(
    chunk: &mut [Distance],
    row: usize,
    column: usize,
    chunk_start_index: usize,
    row_point_data: &PointType,
    column_point_data: &PointType,
    distance_function: impl Fn(&PointType, &PointType) -> Distance,
) {
    let distance = distance_function(row_point_data, column_point_data);

    set_distance(chunk, distance, row, column, chunk_start_index);
}

#[inline(always)]
fn set_distance(
    chunk: &mut [Distance],
    distance: Distance,
    row: usize,
    column: usize,
    chunk_start_index: usize,
) {
    let index_in_chunk =
        get_lower_triangle_matrix_entry_row_bigger(row, column) - chunk_start_index;

    debug_assert!(
        chunk.len() > index_in_chunk,
        "Computed index {} for i: {}, j: {} is out of bounds for distance data of length {}",
        index_in_chunk,
        row,
        column,
        chunk.len()
    );
    // Safety: Index is computed to be within bounds of distance_data
    unsafe { *chunk.get_unchecked_mut(index_in_chunk) = distance };
}
