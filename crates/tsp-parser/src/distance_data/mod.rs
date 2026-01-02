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
        distance::Distance,
        matrix::{MatrixSym, get_lower_triangle_matrix_entry_row_bigger},
    },
    tsp_lib_spec::TSPDataKeyword,
};

pub(crate) mod distance_function;

use crate::{
    distance_container::ParseFromTSPLib, distance_data::distance_function::get_distance_function,
};

pub fn parse_data_sections<DistanceContainer: ParseFromTSPLib>(
    mmap: &Mmap,
    index_in_map: &mut usize,
    data_keyword: TSPDataKeyword,
    metadata: &InstanceMetadata,
) -> DistanceContainer {
    match data_keyword {
        TSPDataKeyword::NODE_COORD_SECTION => {
            let distance_function = get_distance_function(&metadata.edge_weight_type);
            let node_data = parse_node_coord_section(mmap, index_in_map, metadata);
            DistanceContainer::from_node_coord_section(&node_data, metadata, distance_function)
        }
        _ => todo!("Other data sections are not yet implemented"),
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
