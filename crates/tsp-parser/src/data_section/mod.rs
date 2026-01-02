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
    instance::InstanceMetadata,
    tsp_lib_spec::{EdgeWeightType, TSPDataKeyword},
};

pub(crate) mod distance_function;

use crate::{
    data_section::distance_function::{
        euclidean_distance_2d, geographical_distance, get_2d_distance_function,
    },
    distance_container::ParseFromTSPLib,
};

/// A point in 2D space.
#[derive(Debug, Clone, Copy)]
pub struct Point2D {
    pub x: f64,
    pub y: f64,
}

/// A point in 3D space.
#[derive(Debug, Clone, Copy)]
pub struct Point3D {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

/// A point in geographical latitude and longitude radiance coordinates.
#[derive(Debug, Clone, Copy)]
pub struct GeoPoint {
    pub latitude: f64,
    pub longitude: f64,
}

pub fn parse_data_sections<DistanceContainer: ParseFromTSPLib>(
    mmap: &Mmap,
    index_in_map: &mut usize,
    data_keyword: TSPDataKeyword,
    metadata: &InstanceMetadata,
) -> DistanceContainer {
    match data_keyword {
        TSPDataKeyword::NODE_COORD_SECTION => match metadata.edge_weight_type {
            // The distance function is not chosen via a match statement here because the compiler
            // does not seem to inline the distance function properly in that case
            // (leading to a big performance hit)
            EdgeWeightType::EUC_2D => {
                let distance_function = euclidean_distance_2d;
                let node_data = parse_2d_node_coord_section(mmap, index_in_map, metadata);
                DistanceContainer::from_2d_node_coord_section(
                    &node_data,
                    metadata,
                    distance_function,
                )
            }
            EdgeWeightType::MAX_2D => {
                let distance_function = distance_function::max_distance_2d;
                let node_data = parse_2d_node_coord_section(mmap, index_in_map, metadata);
                DistanceContainer::from_2d_node_coord_section(
                    &node_data,
                    metadata,
                    distance_function,
                )
            }
            EdgeWeightType::MAN_2D => {
                let distance_function = distance_function::manhattan_distance_2d;
                let node_data = parse_2d_node_coord_section(mmap, index_in_map, metadata);
                DistanceContainer::from_2d_node_coord_section(
                    &node_data,
                    metadata,
                    distance_function,
                )
            }
            EdgeWeightType::CEIL_2D => {
                let distance_function = distance_function::ceil_distance_2d;
                let node_data = parse_2d_node_coord_section(mmap, index_in_map, metadata);
                DistanceContainer::from_2d_node_coord_section(
                    &node_data,
                    metadata,
                    distance_function,
                )
            }
            EdgeWeightType::ATT => {
                let distance_function = distance_function::att_distance_2d;
                let node_data = parse_2d_node_coord_section(mmap, index_in_map, metadata);
                DistanceContainer::from_2d_node_coord_section(
                    &node_data,
                    metadata,
                    distance_function,
                )
            }
            EdgeWeightType::GEO => {
                let distance_function = geographical_distance;
                // TODO(perf): Possibly parallelize the conversion to geo coordinates
                let node_data = parse_2d_node_coord_section(mmap, index_in_map, metadata)
                    .into_iter()
                    .map(|point| distance_function::convert_to_geo_coordinates(&point))
                    .collect::<Vec<GeoPoint>>();
                DistanceContainer::from_geo_node_coord_section(
                    &node_data,
                    metadata,
                    distance_function,
                )
            }
            EdgeWeightType::EUC_3D | EdgeWeightType::MAX_3D | EdgeWeightType::MAN_3D => {
                unimplemented!("3D node coordinate parsing is not yet implemented");
            }
            EdgeWeightType::EXPLICIT => {
                panic!("NODE_COORD_SECTION should not be used with EXPLICIT edge weight type")
            }
            _ => unimplemented!(
                "Node coordinate type {:?} is not yet implemented",
                metadata.node_coord_type
            ),
        },
        _ => todo!("Other data sections are not yet implemented"),
    }
}

fn parse_2d_node_coord_section(
    mmap: &Mmap,
    index_in_map: &mut usize,
    metadata: &InstanceMetadata,
) -> Vec<Point2D> {
    let mut point_data: Vec<Point2D> = Vec::with_capacity(metadata.dimension);

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

        point_data.push(Point2D { x, y });
    }

    point_data
}
