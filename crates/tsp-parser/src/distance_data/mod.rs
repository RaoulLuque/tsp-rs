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
        distances::{DistancesSymmetric, get_lower_triangle_matrix_entry},
    },
    tsp_lib_spec::TSPDataKeyword,
};

pub fn parse_data_sections(
    mmap: &Mmap,
    index_in_map: &mut usize,
    data_keyword: TSPDataKeyword,
    metadata: &InstanceMetadata,
) -> DistancesSymmetric {
    match data_keyword {
        TSPDataKeyword::NODE_COORD_SECTION => {
            retrieve_distance_data_from_node_coord_section(mmap, index_in_map, metadata)
        }
        _ => todo!("Other data sections are not yet implemented"),
    }
}

fn retrieve_distance_data_from_node_coord_section(
    mmap: &Mmap,
    index_in_map: &mut usize,
    metadata: &InstanceMetadata,
) -> DistancesSymmetric {
    let node_data = retrieve_node_data_from_node_coord_section(mmap, index_in_map, metadata);
    match metadata.edge_weight_type {
        tsp_core::tsp_lib_spec::EdgeWeightType::EUC_2D => {
            compute_distances_euclidean(node_data, metadata.dimension)
        }
        _ => unimplemented!(
            "Edge weight type {:?} is not yet implemented",
            metadata.edge_weight_type
        ),
    }
}

fn retrieve_node_data_from_node_coord_section(
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
        // TODO: Handwrite parsing to parse as unsigned integer first and if there's a decimal point, switch to float parsing using std
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

fn compute_distances_euclidean(
    point_data: Vec<(f64, f64)>,
    dimension: usize,
) -> DistancesSymmetric {
    let mut distance_data = vec![0; dimension * (dimension + 1) / 2];

    // for i in 0..dimension {
    //     for j in i..dimension {
    //         let index = get_lower_triangle_matrix_entry(i, j);
    //         let distance = compute_euclidean_distance(&point_data[i], &point_data[j]);
    //         debug_assert!(
    //             distance_data.len() > index,
    //             "Computed index {} for i: {}, j: {} is out of bounds for distance data of length {}",
    //             index,
    //             i,
    //             j,
    //             distance_data.len()
    //         );
    //         // Safety: Index is computed to be within bounds of distance_data
    //         unsafe { *distance_data.get_unchecked_mut(index) = distance };
    //     }
    // }

    DistancesSymmetric::new_from_data(distance_data, dimension)
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
    // TODO: Check if round_ties_even would behave the same and possibly achieve better performance
    (x + 0.5) as u32
}
