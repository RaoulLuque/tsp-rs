use std::cell::UnsafeCell;

use tsp_core::instance::{InstanceMetadata, distance::Distance, matrix::Matrix};

use super::ParseFromTSPLib;
use crate::distance_container::find_row_column_from_lower_triangle_index;

// TODO: Add more fine grained benchmarks to determine optimal parallelism bound
const PARALLELISM_BOUND: usize = 100_000;

impl ParseFromTSPLib for Matrix<Distance> {
    fn from_node_coord_section<PointType: Sync + Send>(
        node_data: &Vec<PointType>,
        metadata: &InstanceMetadata,
        distance_function: impl Fn(&PointType, &PointType) -> Distance + Sync + Send + Copy,
    ) -> Self {
        compute_dists_from_node_coords(&node_data, metadata.dimension, distance_function)
    }
}

/// TODO: Add documentation
fn compute_dists_from_node_coords<PointType: Send + Sync>(
    point_data: &[PointType],
    dimension: usize,
    distance_function: impl Fn(&PointType, &PointType) -> Distance + Sync + Send + Copy,
) -> Matrix<Distance> {
    let total_size = dimension * dimension;
    let number_of_entries = (dimension * (dimension + 1)) / 2;

    let mut distance_data = vec![Distance(0); total_size];

    if total_size < PARALLELISM_BOUND {
        compute_dists_from_node_coords_chunk(
            &mut distance_data,
            point_data,
            0,
            0,
            number_of_entries,
            distance_function,
            dimension,
        );
    } else {
        let nthreads = std::thread::available_parallelism().unwrap();
        let entries_per_chunk = number_of_entries.div_ceil(nthreads.get());

        std::thread::scope(|scope| {
            // We keep track of two main indices:
            // - current_chunk_start: The index in (the original) distance_data where the current
            //   chunk starts
            // - current_first_entry_index: The index, if one was to only consider the
            //   lower-triangular part of the matrix, of the first entry in the current chunk
            //
            // We distinguish these two "realms" by including chunk / entry in the variable names,
            // respectively.
            let mut current_chunk_start = 0;
            let mut current_first_entry_index = 0;
            let mut rest_distances = distance_data.as_mut_slice();
            while current_first_entry_index < number_of_entries {
                let (current_chunk_size, number_of_entries_current_chunk) = {
                    // We compute the last entry index (in lower-triangular indexing) for this chunk
                    // to determine how many entries this chunk will have, and where it ends in
                    // distance_data.
                    let last_entry_this_chunk = (number_of_entries - 1)
                        .min(current_first_entry_index + entries_per_chunk - 1);
                    let (end_row, end_column) =
                        find_row_column_from_lower_triangle_index(last_entry_this_chunk);

                    (
                        end_row * dimension + end_column + 1 - current_chunk_start,
                        last_entry_this_chunk - current_first_entry_index + 1,
                    )
                };
                let (distances_chunk, rest_distances_tmp) =
                    rest_distances.split_at_mut(current_chunk_size);
                rest_distances = rest_distances_tmp;

                println!(
                    "Current chunk start: {}, first entry index: {}, chunk size: {}, number of \
                     entries in chunk: {}",
                    current_chunk_start,
                    current_first_entry_index,
                    current_chunk_size,
                    number_of_entries_current_chunk,
                );

                scope.spawn(move || {
                    compute_dists_from_node_coords_chunk(
                        distances_chunk,
                        point_data,
                        current_chunk_start,
                        current_first_entry_index,
                        number_of_entries_current_chunk,
                        distance_function,
                        dimension,
                    )
                });

                current_first_entry_index += entries_per_chunk;
                current_chunk_start += current_chunk_size;
            }
        });
    }

    for row in 0..dimension {
        for column in 0..row {
            let value = distance_data[row * dimension + column];
            distance_data[column * dimension + row] = value;
        }
    }

    Matrix::new(distance_data, dimension)
}

#[inline(always)]
fn compute_dists_from_node_coords_chunk<PointType>(
    chunk: &mut [Distance],
    point_data: &[PointType],
    chunk_start_index: usize,
    first_entry_index: usize,
    number_of_entries_in_chunk: usize,
    distance_function: impl Fn(&PointType, &PointType) -> Distance + Copy,
    dimension: usize,
) {
    let (start_row, start_column) = find_row_column_from_lower_triangle_index(first_entry_index);
    let (end_row, end_column) = find_row_column_from_lower_triangle_index(
        first_entry_index + number_of_entries_in_chunk - 1,
    );
    println!(
        "Start row: {}, start column: {}, end row: {}, end column: {}",
        start_row, start_column, end_row, end_column
    );
    let start_row_point_data = &point_data[start_row];
    // We take() to only iterate up to start_row - 1, that is, we can omit the column = start_row
    // case, as it is always zero distance
    // We also skip() to start from start_column
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
            dimension,
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
                dimension,
            );
        }
    }

    let end_row_point_data = &point_data[end_row];
    // We take() to only iterate up to end_column, since it's one indexed, we add 1
    for (column, column_point_data) in point_data.iter().enumerate().take(end_column + 1) {
        compute_and_set_distance(
            chunk,
            end_row,
            column,
            chunk_start_index,
            end_row_point_data,
            column_point_data,
            distance_function,
            dimension,
        );
    }
}

#[inline(always)]
fn compute_and_set_distance<PointType>(
    chunk: &mut [Distance],
    row: usize,
    column: usize,
    chunk_start_index: usize,
    row_point_data: &PointType,
    column_point_data: &PointType,
    distance_function: impl Fn(&PointType, &PointType) -> Distance,
    dimension: usize,
) {
    let distance = distance_function(row_point_data, column_point_data);

    set_distance(chunk, distance, row, column, chunk_start_index, dimension);
}

#[inline(always)]
fn set_distance(
    chunk: &mut [Distance],
    distance: Distance,
    row: usize,
    column: usize,
    chunk_start_index: usize,
    dimension: usize,
) {
    let index_in_chunk = row * dimension + column - chunk_start_index;

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
