/// Module for parsing distance data sections from TSP instance files.
///
/// According to TSPLIB95, distance data can be represented in various formats,
/// such as explicit distance matrices or coordinate-based representations.
///
/// Distance values are required to be non-negative integers. Computations are expected to be
/// carried out in double precision arithmetic, i.e. `f64` in Rust.
use memchr::memchr;
use memmap2::Mmap;
use tsp_core::{
    instance::{
        InstanceMetadata,
        distances::{DistanceMatrixSymmetric, get_lower_triangle_matrix_entry_row_bigger},
    },
    tsp_lib_spec::TSPDataKeyword,
};

// TODO: Add more fine grained benchmarks to determine optimal parallelism bound
const PARALLELISM_BOUND: usize = 300_000;

pub fn parse_data_sections(
    mmap: &Mmap,
    index_in_map: &mut usize,
    data_keyword: TSPDataKeyword,
    metadata: &InstanceMetadata,
) -> DistanceMatrixSymmetric {
    match data_keyword {
        TSPDataKeyword::NODE_COORD_SECTION => {
            parse_dist_from_node_coord_section(mmap, index_in_map, metadata)
        }
        _ => todo!("Other data sections are not yet implemented"),
    }
}

fn parse_dist_from_node_coord_section(
    mmap: &Mmap,
    index_in_map: &mut usize,
    metadata: &InstanceMetadata,
) -> DistanceMatrixSymmetric {
    let node_data = parse_node_coord_section(mmap, index_in_map, metadata);
    match metadata.edge_weight_type {
        tsp_core::tsp_lib_spec::EdgeWeightType::EUC_2D => {
            distances_euclidean(&node_data, metadata.dimension)
        }
        _ => unimplemented!(
            "Edge weight type {:?} is not yet implemented",
            metadata.edge_weight_type
        ),
    }
}

fn parse_node_coord_section(
    mmap: &Mmap,
    index_in_map: &mut usize,
    metadata: &InstanceMetadata,
) -> Vec<(f64, f64)> {
    let mut point_data: Vec<(f64, f64)> = Vec::with_capacity(metadata.dimension);

    // Read a line to test if the point data is floating point or integer
    let is_float_data = {
        let index_newline =
            memchr(b'\n', &mmap[*index_in_map..]).expect("The data section should not be empty");
        let line = &mmap[*index_in_map..*index_in_map + index_newline];

        // SAFETY: The TSP instance file is expected to be valid UTF-8
        let line_str = unsafe { std::str::from_utf8_unchecked(line) };

        // We assume the input to be split by ascii whitespace
        let mut parts = line_str.split_ascii_whitespace();
        let _node_index = parts.next();

        let y_str = parts.next().expect("Missing y coordinate");
        y_str.contains('.')
    };

    while let Some(index_newline) = memchr(b'\n', &mmap[*index_in_map..]) {
        let line = &mmap[*index_in_map..*index_in_map + index_newline];
        // SAFETY: The TSP instance file is expected to be valid UTF-8
        let line_str = unsafe { std::str::from_utf8_unchecked(line) };

        // Move the index to the start of the next line (+1 for the newline character)
        *index_in_map += index_newline + 1;

        // Check if end of file is reached
        if line_str == "EOF" {
            break;
        }

        // We assume the input to be split by ascii whitespace
        let mut parts = line_str.split_ascii_whitespace();
        let _node_index = parts.next();

        let x_str = parts.next().expect("Missing x coordinate");
        let y_str = parts.next().expect("Missing y coordinate");
        let (x, y) = if is_float_data {
            (
                x_str
                    .parse::<f64>()
                    .expect("x coordinate should always be a valid f64 floating point number"),
                y_str
                    .parse::<f64>()
                    .expect("y coordinate should always be a valid f64 floating point number"),
            )
        } else {
            (
                x_str
                    .parse::<u64>()
                    .expect("x coordinate should be a valid u64 integer by sampling first line")
                    as f64,
                y_str
                    .parse::<u64>()
                    .expect("y coordinate should be a valid u64 integer by sampling first line")
                    as f64,
            )
        };

        point_data.push((x, y));
    }

    point_data
}

fn distances_euclidean(point_data: &[(f64, f64)], dimension: usize) -> DistanceMatrixSymmetric {
    let total_size = dimension * (dimension + 1) / 2;

    let mut distance_data = vec![0; total_size];

    if total_size < PARALLELISM_BOUND {
        distances_euclidean_chunk(&mut distance_data, point_data, 0);
    } else {
        let nthreads = std::thread::available_parallelism().unwrap();
        let chunk_size = total_size.div_ceil(nthreads.get());

        std::thread::scope(|scope| {
            let mut current_chunk_start = 0;

            for chunk in distance_data.chunks_mut(chunk_size) {
                scope.spawn(move || {
                    distances_euclidean_chunk(chunk, point_data, current_chunk_start)
                });

                current_chunk_start += chunk_size;
            }
        });
    }

    DistanceMatrixSymmetric::new_from_data(distance_data, dimension)
}

#[inline(always)]
fn distances_euclidean_chunk(
    chunk: &mut [u32],
    point_data: &[(f64, f64)],
    chunk_start_index: usize,
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
        );
    }
}

#[inline(always)]
fn compute_and_set_distance(
    chunk: &mut [u32],
    row: usize,
    column: usize,
    chunk_start_index: usize,
    row_point_data: &(f64, f64),
    column_point_data: &(f64, f64),
) {
    let distance = compute_euclidean_distance(row_point_data, column_point_data);

    set_distance(chunk, distance, row, column, chunk_start_index);
}

#[inline(always)]
fn set_distance(
    chunk: &mut [u32],
    distance: u32,
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

/// Computes the Euclidean distance between two points as defined in TSPLIB95.
#[inline(always)]
fn compute_euclidean_distance(point_a: &(f64, f64), point_b: &(f64, f64)) -> u32 {
    nint(((point_a.0 - point_b.0).powi(2) + (point_a.1 - point_b.1).powi(2)).sqrt())
}

/// Nearest integer function as defined in TSPLIB95.
///
/// Expects a non-negative float input.
#[inline(always)]
fn nint(x: f64) -> u32 {
    (x + 0.5) as u32
}
